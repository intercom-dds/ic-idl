// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_emit::printer::{Twine, w};
use ic_hir::hir::Def;

use crate::codegen::{CppGen, has_default_value};

const UNION_DISC_FIELD: &str = "ic_discriminator_value_";

#[allow(clippy::unused_self)]
impl CppGen<'_> {
    pub fn emit_union(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
    ) {
        let disc_type = self.cpp_type(&union_ty.disc.ty, def.id);
        w!(decl_w, "struct ", def, " {\n");

        // Constructor declarations
        w!(decl_w, def, "();\n");
        w!(decl_w, def, "(const ", def, "& a_other);\n");
        w!(decl_w, def, "& operator=(const ", def, "& a_other);\n");
        w!(decl_w, def, "(", def, "&& a_other) noexcept;\n");
        w!(decl_w, def, "& operator=(", def, "&& a_other) noexcept;\n");
        w!(decl_w, "~", def, "() noexcept;\n\n");

        // Comparison operators
        self.emit_union_comparison_operators(decl_w, def);

        // Swap friend
        w!(decl_w, "friend void swap(", def, "& a_first, ", def, "& a_second) noexcept;\n\n");

        // Discriminator accessors
        w!(decl_w, disc_type, " _d() const { return ", UNION_DISC_FIELD, "; }\n");
        w!(decl_w, "void _d(", disc_type, " discriminator);\n\n");

        // Member accessors
        for variant in &union_ty.variants {
            self.emit_union_member_accessors(decl_w, def, variant, &disc_type);
        }

        // Private members
        w!(decl_w, "private:\n");
        w!(decl_w, "union ICUnionType_ {\n");
        w!(decl_w, "ICUnionType_() {}\n");
        w!(decl_w, "~ICUnionType_() {}\n");
        for variant in &union_ty.variants {
            let member_type = self.cpp_type(&variant.ty, def.id);
            w!(decl_w, member_type, " ", variant.ident.name, ";\n");
        }
        w!(decl_w, "} ic_union_value_;\n");
        w!(decl_w, disc_type, " ", UNION_DISC_FIELD, ";\n");
        w!(decl_w, "void free_union_();\n");
        w!(decl_w, "};\n\n");

        self.emit_hash_specialization(impl_w, def);

        self.emit_union_impl(impl_w, def, union_ty, &disc_type);
    }

    fn emit_union_comparison_operators(&self, w: &mut Twine, def: &Def) {
        w!(w, "bool operator<(const ", def, "& a_other) const;\n");
        w!(w, "bool operator==(const ", def, "& a_other) const;\n");
        w!(w, "bool operator!=(const ", def, "& a_other) const { return !(*this == a_other); }\n");
        w!(w, "bool operator>(const ", def, "& a_other) const { return a_other < *this; }\n");
        w!(w, "bool operator<=(const ", def, "& a_other) const { return !(a_other < *this); }\n");
        w!(w, "bool operator>=(const ", def, "& a_other) const { return !(*this < a_other); }\n\n");
    }

    fn emit_union_member_accessors(
        &self,
        w: &mut Twine,
        def: &Def,
        variant: &ic_hir::hir::Variant,
        disc_type: &str,
    ) {
        let member_type = self.cpp_type(&variant.ty, def.id);
        let member_name = &variant.ident.name;
        let is_complex = self.should_use_move(&variant.ty);

        if is_complex {
            // Getter (reference)
            w!(w, member_type, "& ", member_name, "();\n");

            // Getter (const reference)
            w!(w, "const ", member_type, "& ", member_name, "() const;\n");

            // Setter (const reference)
            w!(w, "void ", member_name, "(const ", member_type, "& a_value);\n");

            // Move setter
            w!(w, "void ", member_name, "(", member_type, "&& a_value);\n");
        } else {
            // Primitive: getter (reference)
            w!(w, member_type, "& ", member_name, "();\n");

            // Primitive: getter (by value)
            w!(w, member_type, " ", member_name, "() const;\n");

            // Primitive: setter (by value)
            w!(w, "void ", member_name, "(", member_type, " a_value);\n");
        }

        // Default variant gets special discriminator setter
        if variant.is_default {
            w!(w, "void ", member_name, "(", member_type, " a_value, ", disc_type, " discriminator);\n");
        }

        w!(w, "\n");
    }

    fn emit_union_switch<F>(
        &self,
        w: &mut Twine,
        union_ty: &ic_hir::hir::UnionTy,
        disc_var: &str,
        mut body: F,
    ) where
        F: FnMut(&mut Twine, &ic_hir::hir::Variant) -> bool,
    {
        w!(w, "switch (", disc_var, ") {\n");
        w.dedent();

        for variant in &union_ty.variants {
            if variant.is_default {
                w!(w, "default:\n");
            } else {
                for label in &variant.labels {
                    w!(w, "case ");
                    self.emit_numeric_value(w, &label.value, None);
                    w!(w, ":\n");
                }
            }
            w.indent();

            let emit_break = body(w, variant);

            if emit_break {
                w!(w, "break;\n");
            }
            w.dedent();
        }

        w.indent();
        w!(w, "}\n");
    }

    fn emit_variant_check_condition(
        &self,
        w: &mut Twine,
        union_ty: &ic_hir::hir::UnionTy,
        variant: &ic_hir::hir::Variant,
    ) {
        if variant.is_default {
            w!(w, UNION_DISC_FIELD, " == ");
            for (i, v) in union_ty
                .variants
                .iter()
                .filter(|v| !v.is_default)
                .enumerate()
            {
                if i > 0 {
                    w!(w, " || ", UNION_DISC_FIELD, " == ");
                }
                if let Some(first_label) = v.labels.first() {
                    self.emit_numeric_value(w, &first_label.value, None);
                }
            }
        } else {
            w!(w, UNION_DISC_FIELD, " != ");
            if let Some(first_label) = variant.labels.first() {
                self.emit_numeric_value(w, &first_label.value, None);
            }
        }
    }

    fn emit_set_discriminator_to_variant(&self, w: &mut Twine, variant: &ic_hir::hir::Variant) {
        w!(w, UNION_DISC_FIELD, " = ");
        if let Some(first_label) = variant.labels.first() {
            self.emit_numeric_value(w, &first_label.value, None);
        }
        w!(w, ";\n");
    }

    fn emit_variant_init(&self, w: &mut Twine, variant: &ic_hir::hir::Variant, value_expr: &str) {
        if self.should_use_move(&variant.ty) {
            w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", ", value_expr, ");\n");
        } else {
            w!(w, "ic_union_value_.", variant.ident.name, " = ", value_expr, ";\n");
        }
    }

    fn emit_default_discriminator_check(
        &self,
        w: &mut Twine,
        union_ty: &ic_hir::hir::UnionTy,
        discriminator_var: &str,
    ) {
        w!(w, "if (");
        for (i, v) in union_ty
            .variants
            .iter()
            .filter(|v| !v.is_default)
            .enumerate()
        {
            if i > 0 {
                w!(w, " || ");
            }
            w!(w, discriminator_var, " == ");
            if let Some(first_label) = v.labels.first() {
                self.emit_numeric_value(w, &first_label.value, None);
            }
        }
        w!(w, ") {\n");
    }

    fn emit_union_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        disc_type: &str,
    ) {
        self.emit_union_constructor(w, def, union_ty, disc_type);
        self.emit_union_copy_constructor(w, def, union_ty, disc_type);
        self.emit_union_copy_assignment(w, def, union_ty, disc_type);
        self.emit_union_move_constructor(w, def, union_ty, disc_type);
        self.emit_union_move_assignment(w, def, union_ty, disc_type);
        self.emit_union_destructor(w, def);
        self.emit_union_comparison_impl(w, def, union_ty);
        self.emit_union_swap(w, def);
        self.emit_union_discriminator_setter(w, def, union_ty, disc_type);
        self.emit_union_member_impl(w, def, union_ty, disc_type);
        self.emit_union_free(w, def, union_ty);
    }

    fn emit_union_constructor(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::", def, "() {\n");

        if let Some(first_variant) = union_ty.variants.first() {
            self.emit_set_discriminator_to_variant(w, first_variant);
            let default_val = self.get_default_value_expr(&first_variant.ty);
            self.emit_variant_init(w, first_variant, &default_val);
        }
        w!(w, "}\n\n");
    }

    fn get_default_value_expr(&self, ty: &ic_hir::hir::Ty) -> String {
        match &ty.kind {
            ic_hir::hir::TyKind::String { .. } => "std::string{}".to_string(),
            ic_hir::hir::TyKind::Array { .. } | ic_hir::hir::TyKind::Sequence { .. } => {
                "{}".to_string()
            }
            _ => "0".to_string(),
        }
    }

    fn emit_union_copy_constructor(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::", def, "(const ", qualified_name, "& a_other) {\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if self.should_use_move(&variant.ty) {
                w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", a_other.ic_union_value_.", variant.ident.name, ");\n");
            } else {
                w!(w, "ic_union_value_.", variant.ident.name, " = a_other.ic_union_value_.", variant.ident.name, ";\n");
            }
            true
        });

        w!(w, "}\n\n");
    }

    fn emit_union_copy_assignment(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "& ", qualified_name, "::operator=(const ", qualified_name, "& a_other) {\n");
        w!(w, "if (this != &a_other) {\n");
        w!(w, "free_union_();\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if self.should_use_move(&variant.ty) {
                w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", a_other.ic_union_value_.", variant.ident.name, ");\n");
            } else {
                w!(w, "ic_union_value_.", variant.ident.name, " = a_other.ic_union_value_.", variant.ident.name, ";\n");
            }
            true
        });

        w!(w, "}\n");
        w!(w, "return *this;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_move_constructor(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::", def, "(", qualified_name, "&& a_other) noexcept {\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if self.should_use_move(&variant.ty) {
                w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", std::move(a_other.ic_union_value_.", variant.ident.name, "));\n");
            } else {
                w!(w, "ic_union_value_.", variant.ident.name, " = a_other.ic_union_value_.", variant.ident.name, ";\n");
            }
            true
        });

        w!(w, "}\n\n");
    }

    fn emit_union_move_assignment(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "& ", qualified_name, "::operator=(", qualified_name, "&& a_other) noexcept {\n");
        w!(w, "if (this != &a_other) {\n");
        w!(w, "free_union_();\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if self.should_use_move(&variant.ty) {
                w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", std::move(a_other.ic_union_value_.", variant.ident.name, "));\n");
            } else {
                w!(w, "ic_union_value_.", variant.ident.name, " = a_other.ic_union_value_.", variant.ident.name, ";\n");
            }
            true
        });

        w!(w, "}\n");
        w!(w, "return *this;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_destructor(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::~", def, "() noexcept {\n");
        w!(w, "free_union_();\n");
        w!(w, "}\n\n");
    }

    fn emit_union_comparison_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
    ) {
        let qualified_name = self.scoped_name(def.id, None);

        // operator<
        w!(w, "inline bool ", qualified_name, "::operator<(const ", qualified_name, "& a_other) const {\n");
        w!(w, "if (_d() < a_other._d()) { return true; }\n");
        w!(w, "if (a_other._d() < _d()) { return false; }\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            w!(w, "return this->", variant.ident.name, "() < a_other.", variant.ident.name, "();\n");
            false
        });

        w!(w, "}\n\n");

        // operator==
        w!(w, "inline bool ", qualified_name, "::operator==(const ", qualified_name, "& a_other) const {\n");
        w!(w, "if (!(_d() == a_other._d())) return false;\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            w!(w, "return this->", variant.ident.name, "() == a_other.", variant.ident.name, "();\n");
            false
        });

        w!(w, "}\n\n");
    }

    fn emit_union_swap(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void swap(", qualified_name, "& a_first, ", qualified_name, "& a_second) noexcept {\n");
        w!(w, qualified_name, " a_first_tmp = std::move(a_first);\n");
        w!(w, "a_first = std::move(a_second);\n");
        w!(w, "a_second = std::move(a_first_tmp);\n");
        w!(w, "}\n\n");
    }

    fn emit_union_discriminator_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::_d(", disc_type, " discriminator) {\n");

        self.emit_union_switch(w, union_ty, "discriminator", |w, variant| {
            if variant.is_default {
                self.emit_default_discriminator_check(w, union_ty, UNION_DISC_FIELD);
                w!(w, "free_union_();\n");

                let default_val = if has_default_value(&variant.ty) {
                    let mut temp_w = Twine::new();
                    self.emit_default_initializer(&mut temp_w, &variant.ty);
                    temp_w.finish()
                } else {
                    format!(
                        "static_cast<{}>({:.7})",
                        self.cpp_type(&variant.ty, None),
                        0.0
                    )
                };

                w!(w, "ic_union_value_.", variant.ident.name, " = ", default_val, ";\n");
                w!(w, "}\n");
            } else {
                w!(w, "if (", UNION_DISC_FIELD, " != ");
                if let Some(first_label) = variant.labels.first() {
                    self.emit_numeric_value(w, &first_label.value, None);
                }
                w!(w, ") {\n");
                w!(w, "free_union_();\n");

                let default_val = self.get_default_value_expr(&variant.ty);
                self.emit_variant_init(w, variant, &default_val);
                w!(w, "}\n");
            }
            true
        });

        w!(w, UNION_DISC_FIELD, " = discriminator;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_member_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        disc_type: &str,
    ) {
        for variant in &union_ty.variants {
            self.emit_variant_getters(w, def, union_ty, variant);
            self.emit_variant_setters(w, def, union_ty, variant, disc_type);
        }
    }

    fn emit_variant_getters(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        variant: &ic_hir::hir::Variant,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        let member_type = self.cpp_type(&variant.ty, None);
        let member_name = &variant.ident.name;

        // Reference getter
        w!(w, "inline ", member_type, "& ", qualified_name, "::", member_name, "() {\n");
        w!(w, "if (");
        self.emit_variant_check_condition(w, union_ty, variant);
        w!(w, ") {\n");
        w!(w, "throw std::logic_error(\"Union ", def, " not set to value ", member_name, "\");\n");
        w!(w, "}\n");
        w!(w, "return ic_union_value_.", member_name, ";\n");
        w!(w, "}\n\n");

        // Const getter (by reference for complex, by value for primitive)
        if self.should_use_move(&variant.ty) {
            w!(w, "inline const ", member_type, "& ", qualified_name, "::", member_name, "() const {\n");
        } else {
            w!(w, "inline ", member_type, " ", qualified_name, "::", member_name, "() const {\n");
        }
        w!(w, "if (");
        self.emit_variant_check_condition(w, union_ty, variant);
        w!(w, ") {\n");
        w!(w, "throw std::logic_error(\"Union ", def, " not set to value ", member_name, "\");\n");
        w!(w, "}\n");
        w!(w, "return ic_union_value_.", member_name, ";\n");
        w!(w, "}\n\n");
    }

    fn emit_variant_setters(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        variant: &ic_hir::hir::Variant,
        disc_type: &str,
    ) {
        let member_type = self.cpp_type(&variant.ty, None);
        let member_name = &variant.ident.name;

        self.emit_variant_copy_setter(w, def, union_ty, variant, &member_type, member_name);

        if self.should_use_move(&variant.ty) {
            self.emit_variant_move_setter(w, def, union_ty, variant, &member_type, member_name);
        }

        if variant.is_default {
            self.emit_variant_default_setter(
                w,
                def,
                union_ty,
                disc_type,
                &member_type,
                member_name,
            );
        }
    }

    fn emit_variant_copy_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        variant: &ic_hir::hir::Variant,
        member_type: &str,
        member_name: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        if self.should_use_move(&variant.ty) {
            w!(w, "inline void ", qualified_name, "::", member_name, "(const ", member_type, "& a_value) {\n");
        } else {
            w!(w, "inline void ", qualified_name, "::", member_name, "(", member_type, " a_value) {\n");
        }
        w!(w, "if (");
        self.emit_variant_check_condition(w, union_ty, variant);
        w!(w, ") {\n");
        w!(w, "free_union_();\n");
        if !variant.is_default {
            self.emit_set_discriminator_to_variant(w, variant);
        }

        if self.should_use_move(&variant.ty) {
            w!(w, "::ic_cts::construct_at(&ic_union_value_.", member_name, ", a_value);\n");
            w!(w, "} else {\n");
            w!(w, "ic_union_value_.", member_name, " = a_value;\n");
        }
        w!(w, "}\n");
        if !self.should_use_move(&variant.ty) {
            w!(w, "ic_union_value_.", member_name, " = a_value;\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_variant_move_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        variant: &ic_hir::hir::Variant,
        member_type: &str,
        member_name: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::", member_name, "(", member_type, "&& a_value) {\n");
        w!(w, "if (");
        self.emit_variant_check_condition(w, union_ty, variant);
        w!(w, ") {\n");
        w!(w, "free_union_();\n");
        if !variant.is_default {
            self.emit_set_discriminator_to_variant(w, variant);
        }
        w!(w, "::ic_cts::construct_at(&ic_union_value_.", member_name, ", std::move(a_value));\n");
        w!(w, "} else {\n");
        w!(w, "ic_union_value_.", member_name, " = std::move(a_value);\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn emit_variant_default_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        disc_type: &str,
        member_type: &str,
        member_name: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::", member_name, "(", member_type, " a_value, ", disc_type, " discriminator) {\n");

        self.emit_default_discriminator_check(w, union_ty, "discriminator");
        w!(
            w,
            "throw std::logic_error(\"Illegal discriminator for member ",
            member_name, " of union ", def, "\");\n",
        );
        w!(w, "}\n");

        self.emit_default_discriminator_check(w, union_ty, UNION_DISC_FIELD);
        w!(w, "free_union_();\n");
        w!(w, "}\n");

        w!(w, "ic_union_value_.", member_name, " = a_value;\n");
        w!(w, UNION_DISC_FIELD, " = discriminator;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_free(&self, w: &mut Twine, def: &Def, union_ty: &ic_hir::hir::UnionTy) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::free_union_() {\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if self.should_use_move(&variant.ty) {
                w!(w, "std::destroy_at(&ic_union_value_.", variant.ident.name, ");\n");
            }
            true
        });

        w!(w, "}\n\n");
    }
}
