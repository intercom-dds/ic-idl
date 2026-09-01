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
use ic_hir::hir::{Def, DefId, DefKind, Label, TyKind, UnionTy, Variant};
use ic_hir_analysis::annotation::{MemberLike, default_value, is_external};
use ic_hir_analysis::enum_value::default_enumerator;
use ic_hir_analysis::union_case::{
    default_discriminator, default_union_case, unused_discriminator,
};

use crate::codegen::CppGen;

const UNION_DISC_FIELD: &str = "ic_discriminator_value_";

impl CppGen<'_> {
    pub fn emit_union(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
    ) {
        let disc_type = self.cpp_type(&union_ty.disc.ty, def.id);
        w!(decl_w, "class ", def, " {\n");
        w!(decl_w, "public:\n");

        // Constructor declarations
        w!(decl_w, def, "();\n");
        w!(decl_w, def, "(const ", def, "& a_other);\n");
        w!(decl_w, def, "& operator=(const ", def, "& a_other);\n");
        w!(decl_w, def, "(", def, "&& a_other) noexcept;\n");
        w!(decl_w, def, "& operator=(", def, "&& a_other) noexcept;\n");
        w!(decl_w, "~", def, "() noexcept;\n\n");

        // Comparison operators
        self.emit_union_comparison_operators(decl_w, def);

        // Discriminator accessors
        w!(decl_w, "[[nodiscard]]", disc_type, " _d() const { return ", UNION_DISC_FIELD, "; }\n");
        w!(decl_w, "void _d(", disc_type, " discriminator);\n\n");
        if has_implicit_default_variant(union_ty).is_some() {
            w!(decl_w, "void _default();\n\n");
        }

        // Member accessors
        for variant in &union_ty.variants {
            if !matches!(variant.ty.kind, TyKind::Null) {
                self.emit_union_member_accessors(decl_w, def, variant, &disc_type);
            }
        }

        // Private members
        w!(decl_w, "private:\n");
        w!(decl_w, "union ICUnionType_ {\n");
        w!(decl_w, "ICUnionType_() {}\n");
        w!(decl_w, "~ICUnionType_() {}\n");
        for variant in &union_ty.variants {
            if !matches!(variant.ty.kind, TyKind::Null) {
                let member_type = self.cpp_type_member(variant, def.id, false);
                w!(decl_w, member_type, " ", variant.ident.name, ";\n");
            }
        }
        w!(decl_w, "} ic_union_value_;\n");
        w!(decl_w, disc_type, " ", UNION_DISC_FIELD, ";\n");
        w!(decl_w, "void free_union_();\n");
        w!(decl_w, "};\n\n");

        self.emit_typedef_sequence(impl_w, def);
        self.emit_type_traits(impl_w, def);
        self.emit_hash_declaration(impl_w, def);
        self.emit_union_serializer(impl_w, def, union_ty);
        self.emit_formatter_specialization(impl_w, def);

        self.emit_union_impl(impl_w, def, union_ty, &disc_type);
    }

    pub fn emit_hash_union(&self, w: &mut Twine, union_ty: &UnionTy) {
        w!(w, "std::size_t seed = 0;\n");
        w!(w, "::ic_cts::hash_combine(seed, s._d());\n\n");

        self.emit_union_switch(w, union_ty, "s._d()", |w, variant| {
            if !matches!(variant.ty.kind, TyKind::Null) {
                let member_name = format!("s.{}()", variant.ident.name);
                w!(w, "::ic_cts::hash_combine(seed, ", member_name, ");\n");
            }
            true
        });

        w!(w, "return seed;\n");
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
        variant: &Variant,
        disc_type: &str,
    ) {
        let member_type = self.cpp_type_member(variant, def.id, false);
        let member_name = &variant.ident.name;
        let is_complex = self.should_use_move(&variant.ty, Some(variant));

        if is_complex {
            // Getter (reference)
            w!(w, member_type, "& ", member_name, "();\n");

            // Getter (const reference)
            w!(w, "[[nodiscard]] const ", member_type, "& ", member_name, "() const;\n");

            if !is_external(&self.hir.context, variant) {
                // Setter (const reference)
                w!(w, "void ", member_name, "(const ", member_type, "& a_value);\n");
            }

            // Move setter
            w!(w, "void ", member_name, "(", member_type, "&& a_value);\n");
        } else {
            // Primitive: getter (reference)
            w!(w, "[[nodiscard]] ", member_type, "& ", member_name, "();\n");

            // Primitive: getter (by value)
            w!(w, "[[nodiscard]] ", member_type, " ", member_name, "() const;\n");

            // Primitive: setter (by value)
            w!(w, "void ", member_name, "(", member_type, " a_value);\n");
        }

        // Default variant gets special discriminator setter
        if variant.is_default {
            w!(w, "void ", member_name, "(", member_type, " a_value, ", disc_type, " discriminator);\n");
        }

        w!(w, "\n");
    }

    fn emit_union_switch<F>(&self, w: &mut Twine, union_ty: &UnionTy, disc_var: &str, mut body: F)
    where
        F: FnMut(&mut Twine, &Variant) -> bool,
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

    fn should_emit_variant_check(&self, union_ty: &UnionTy, variant: &Variant) -> bool {
        !variant.is_default || union_ty.variants.len() > 1
    }

    fn emit_variant_check_condition(&self, w: &mut Twine, union_ty: &UnionTy, variant: &Variant) {
        if variant.is_default {
            let non_default_variants: Vec<_> =
                union_ty.variants.iter().filter(|v| !v.is_default).collect();

            if non_default_variants.is_empty() {
                w!(w, "false");
            } else {
                w!(w, UNION_DISC_FIELD, " == ");
                for (i, v) in non_default_variants.iter().enumerate() {
                    if i > 0 {
                        w!(w, " || ", UNION_DISC_FIELD, " == ");
                    }
                    if let Some(first_label) = v.labels.first() {
                        self.emit_numeric_value(w, &first_label.value, None);
                    }
                }
            }
        } else {
            for (i, label) in variant.labels.iter().enumerate() {
                if i > 0 {
                    w!(w, " && ", UNION_DISC_FIELD, " != ");
                } else {
                    w!(w, UNION_DISC_FIELD, " != ");
                }
                self.emit_numeric_value(w, &label.value, None);
            }
        }
    }

    fn emit_set_discriminator_to_variant(
        &self,
        w: &mut Twine,
        variant: &Variant,
        union_ty: &UnionTy,
        disc_label: Option<&Label>,
        def_id: DefId,
    ) {
        w!(w, UNION_DISC_FIELD, " = ");
        if let Some(first_label) = disc_label.or(variant.labels.first()) {
            self.emit_numeric_value(w, &first_label.value, def_id);
        } else if let Some(discriminator) = unused_discriminator(&self.hir.context, union_ty) {
            self.emit_numeric_value(w, &discriminator, def_id);
        } else {
            let default_value = self.get_default_value_expr(&union_ty.disc, def_id);
            w!(w, default_value);
        }
        w!(w, ";\n");
    }

    fn emit_variant_init(&self, w: &mut Twine, variant: &Variant, value_expr: &str) {
        if self.should_use_move(&variant.ty, Some(variant)) {
            if is_external(&self.hir.context, variant) {
                w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", new ", self.cpp_type(&variant.ty, None), "(", value_expr, "));\n");
            } else {
                w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", ", value_expr, ");\n");
            }
        } else {
            w!(w, "ic_union_value_.", variant.ident.name, " = ", value_expr, ";\n");
        }
    }

    fn get_variant_default_expr(&self, variant: &Variant, def_id: DefId) -> String {
        if let Some(default) = default_value(&self.hir.context, variant) {
            let mut w = ic_emit::printer::Twine::new();
            self.emit_numeric_value_with_ty(&mut w, default, &variant.ty, def_id, false);
            w.finish()
        } else {
            self.get_default_value_expr(variant, def_id)
        }
    }

    fn emit_default_discriminator_check(
        &self,
        w: &mut Twine,
        union_ty: &UnionTy,
        discriminator_var: &str,
    ) {
        let non_default_variants: Vec<_> =
            union_ty.variants.iter().filter(|v| !v.is_default).collect();

        if non_default_variants.is_empty() {
            return;
        }

        w!(w, "if (");
        let mut first = true;
        for v in &non_default_variants {
            for label in &v.labels {
                if !first {
                    w!(w, " || ");
                }
                first = false;
                w!(w, discriminator_var, " == ");
                self.emit_numeric_value(w, &label.value, None);
            }
        }
        w!(w, ") {\n");
    }

    fn emit_union_impl(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy, disc_type: &str) {
        self.emit_union_constructor(w, def, union_ty);
        self.emit_union_copy_constructor(w, def, union_ty, disc_type);
        self.emit_union_copy_assignment(w, def, union_ty, disc_type);
        self.emit_union_move_constructor(w, def, union_ty, disc_type);
        self.emit_union_move_assignment(w, def, union_ty, disc_type);
        self.emit_union_destructor(w, def);
        self.emit_union_comparison_impl(w, def, union_ty);
        self.emit_union_discriminator_setter(w, def, union_ty, disc_type);
        self.emit_union_member_impl(w, def, union_ty, disc_type);
        self.emit_union_free(w, def, union_ty);
    }

    fn emit_union_constructor(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::", def, "() {\n");

        let case = default_union_case(&self.hir.context, union_ty);
        let discriminator = default_discriminator(&self.hir.context, union_ty);
        w!(w, UNION_DISC_FIELD, " = ");
        self.emit_numeric_value(w, &discriminator, def.id);
        w!(w, ";\n");
        if !matches!(case.variant.ty.kind, TyKind::Null) {
            let default_value = self.get_variant_default_expr(case.variant, def.id);
            self.emit_variant_init(w, case.variant, &default_value);
        }
        w!(w, "}\n\n");
    }

    fn get_default_value_expr(&self, member_like: &impl MemberLike, relative_def: DefId) -> String {
        match &member_like.ty().kind {
            TyKind::String { .. } => "::std::string{}".to_string(),
            TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. } => {
                let type_name = self.cpp_type(member_like.ty(), relative_def);
                format!("{type_name}{{}}")
            }
            TyKind::Primitive(_) => format!(
                "{}()",
                self.cpp_type_member(member_like, relative_def, true)
            ),
            TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                match &def.kind {
                    DefKind::Struct(struct_ty) => {
                        let type_name = self.cpp_type(member_like.ty(), relative_def);
                        let mut result = format!("{type_name}{{");
                        for (i, member) in struct_ty.members.iter().enumerate() {
                            let field_default = self.get_default_value_expr(member, relative_def);
                            if i > 0 {
                                result.push_str(", ");
                            }
                            result.push_str(&field_default);
                        }
                        result.push('}');
                        result
                    }
                    DefKind::Valuetype(valuetype_ty) => {
                        let type_name = self.cpp_type(member_like.ty(), relative_def);
                        let mut result = format!("{type_name}{{");
                        for (i, member) in valuetype_ty.members.iter().enumerate() {
                            let field_default = self.get_default_value_expr(member, relative_def);
                            if i > 0 {
                                result.push_str(", ");
                            }
                            result.push_str(&field_default);
                        }
                        result.push('}');
                        result
                    }
                    DefKind::Enum(enum_ty) => {
                        let field_id = default_enumerator(&self.hir.context, enum_ty);
                        self.scoped_name(field_id, relative_def)
                    }
                    DefKind::Union(_) | DefKind::Alias(_) => {
                        let type_name = self.cpp_type(member_like.ty(), relative_def);
                        format!("{type_name}{{}}")
                    }
                    DefKind::Bitmask(bitmask_ty) => {
                        Self::primitive_default(bitmask_ty.ty).to_string()
                    }
                    _ => "0".to_string(),
                }
            }
            _ => "0".to_string(),
        }
    }

    fn emit_union_copy_constructor(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::", def, "(const ", qualified_name, "& a_other) {\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if !matches!(variant.ty.kind, TyKind::Null) {
                if self.should_use_move(&variant.ty, Some(variant)) {
                    if is_external(&self.hir.context, variant) {
                        w!(
                            w,
                            "::ic_cts::construct_at(&ic_union_value_.",
                            variant.ident.name,
                            ", new ", self.cpp_type(&variant.ty, None),"( *a_other.ic_union_value_.",
                            variant.ident.name,
                            "));\n",
                        );
                    } else {
                        w!(
                            w,
                            "::ic_cts::construct_at(&ic_union_value_.",
                            variant.ident.name,
                            ", a_other.ic_union_value_.",
                            variant.ident.name,
                            ");\n",
                        );
                    }
                } else {
                    w!(
                        w,
                        "ic_union_value_.",
                        variant.ident.name,
                        " = a_other.ic_union_value_.",
                        variant.ident.name,
                        ";\n",
                    );
                }
            }
            true
        });

        w!(w, "}\n\n");
    }

    fn emit_union_copy_assignment(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "& ", qualified_name, "::operator=(const ", qualified_name, "& a_other) {\n");
        w!(w, "if (this != &a_other) {\n");
        w!(w, "free_union_();\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if !matches!(variant.ty.kind, TyKind::Null) {
                if self.should_use_move(&variant.ty, Some(variant)) {
                    if is_external(&self.hir.context, variant) {
                        w!(
                            w,
                            "::ic_cts::construct_at(&ic_union_value_.",
                            variant.ident.name,
                            ", new ", self.cpp_type(&variant.ty, None),"( *a_other.ic_union_value_.",
                            variant.ident.name,
                            "));\n",
                        );
                    } else {
                        w!(
                            w,
                            "::ic_cts::construct_at(&ic_union_value_.",
                            variant.ident.name,
                            ", a_other.ic_union_value_.",
                            variant.ident.name,
                            ");\n",
                        );
                    }
                } else {
                    w!(
                        w,
                        "ic_union_value_.",
                        variant.ident.name,
                        " = a_other.ic_union_value_.",
                        variant.ident.name,
                        ";\n",
                    );
                }
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
        union_ty: &UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "::", def, "(", qualified_name, "&& a_other) noexcept {\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if !matches!(variant.ty.kind, TyKind::Null) {
                if self.should_use_move(&variant.ty, Some(variant)) {
                    w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", std::move(a_other.ic_union_value_.", variant.ident.name, "));\n");
                } else {
                    w!(w, "ic_union_value_.", variant.ident.name, " = a_other.ic_union_value_.", variant.ident.name, ";\n");
                }
            }
            true
        });

        w!(w, "}\n\n");
    }

    fn emit_union_move_assignment(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        _disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline ", qualified_name, "& ", qualified_name, "::operator=(", qualified_name, "&& a_other) noexcept {\n");
        w!(w, "if (this != &a_other) {\n");
        w!(w, "free_union_();\n");
        w!(w, UNION_DISC_FIELD, " = a_other.", UNION_DISC_FIELD, ";\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if !matches!(variant.ty.kind, TyKind::Null) {
                if self.should_use_move(&variant.ty, Some(variant)) {
                    w!(w, "::ic_cts::construct_at(&ic_union_value_.", variant.ident.name, ", std::move(a_other.ic_union_value_.", variant.ident.name, "));\n");
                } else {
                    w!(w, "ic_union_value_.", variant.ident.name, " = a_other.ic_union_value_.", variant.ident.name, ";\n");
                }
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

    fn emit_union_comparison_impl(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let qualified_name = self.scoped_name(def.id, None);

        // operator<
        w!(w, "inline bool ", qualified_name, "::operator<(const ", qualified_name, "& a_other) const {\n");
        w!(w, "if (_d() < a_other._d()) { return true; }\n");
        w!(w, "if (a_other._d() < _d()) { return false; }\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if matches!(variant.ty.kind, TyKind::Null) {
                w!(w, "return false;\n");
            } else {
                w!(w, "return this->", variant.ident.name, "() < a_other.", variant.ident.name, "();\n");
            }
            false
        });

        w!(w, "return false;\n");
        w!(w, "}\n\n");

        // operator==
        w!(w, "inline bool ", qualified_name, "::operator==(const ", qualified_name, "& a_other) const {\n");
        w!(w, "if (!(_d() == a_other._d())) return false;\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if matches!(variant.ty.kind, TyKind::Null) {
                w!(w, "return true;\n");
            } else {
                w!(w, "return this->", variant.ident.name, "() == a_other.", variant.ident.name, "();\n");
            }
            false
        });

        w!(w, "return true;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_discriminator_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        disc_type: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::_d(", disc_type, " discriminator) {\n");

        self.emit_union_switch(w, union_ty, "discriminator", |w, variant| {
            if variant.is_default {
                let has_non_default = union_ty.variants.iter().any(|v| !v.is_default);

                if has_non_default {
                    self.emit_default_discriminator_check(w, union_ty, UNION_DISC_FIELD);
                }

                w!(w, "free_union_();\n");

                if !matches!(variant.ty.kind, TyKind::Null) {
                    let default_val = self.get_variant_default_expr(variant, def.id);
                    self.emit_variant_init(w, variant, &default_val);
                }

                if has_non_default {
                    w!(w, "}\n");
                }
            } else if !matches!(variant.ty.kind, TyKind::Null) {
                w!(w, "if (");
                for (i, label) in variant.labels.iter().enumerate() {
                    if i > 0 {
                        w!(w, " && ", UNION_DISC_FIELD, " != ");
                    } else {
                        w!(w, UNION_DISC_FIELD, " != ");
                    }
                    self.emit_numeric_value(w, &label.value, None);
                }
                w!(w, ") {\n");
                w!(w, "free_union_();\n");

                let default_val = self.get_variant_default_expr(variant, def.id);
                self.emit_variant_init(w, variant, &default_val);
                w!(w, "}\n");
            }
            true
        });

        w!(w, UNION_DISC_FIELD, " = discriminator;\n");
        w!(w, "}\n\n");

        // IDL4-CPP 7.2.4.3.2 Unions
        if let Some(variant) = has_implicit_default_variant(union_ty) {
            w!(w, "inline void ", qualified_name, "::_default() {\n");
            if let Some(first_label) = variant.labels.first() {
                w!(w, "_d(");
                self.emit_numeric_value(w, &first_label.value, def.id);
                w!(w, ");\n");
            } else if let Some(discriminator) = unused_discriminator(&self.hir.context, union_ty) {
                w!(w, "_d(");
                self.emit_numeric_value(w, &discriminator, def.id);
                w!(w, ");\n");
            } else {
                w!(w, "_d(", self.get_default_value_expr(&union_ty.disc, def.id), ");\n");
            }
            w!(w, "}\n\n");
        }
    }

    fn emit_union_member_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        disc_type: &str,
    ) {
        for variant in &union_ty.variants {
            if !matches!(variant.ty.kind, TyKind::Null) {
                self.emit_variant_getters(w, def, union_ty, variant);
                self.emit_variant_setters(w, def, union_ty, variant, disc_type);
            }
        }
    }

    fn emit_variant_getters(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        variant: &Variant,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        let member_type = self.cpp_type_member(variant, None, false);
        let member_name = &variant.ident.name;

        // Reference getter
        w!(w, "inline ", member_type, "& ", qualified_name, "::", member_name, "() {\n");
        if self.should_emit_variant_check(union_ty, variant) {
            w!(w, "if (");
            self.emit_variant_check_condition(w, union_ty, variant);
            w!(w, ") {\n");
            w!(w, "throw std::logic_error(\"Union ", def, " not set to value ", member_name, "\");\n");
            w!(w, "}\n");
        }
        w!(w, "return ic_union_value_.", member_name, ";\n");
        w!(w, "}\n\n");

        // Const getter (by reference for complex, by value for primitive)
        if self.should_use_move(&variant.ty, Some(variant)) {
            w!(w, "inline const ", member_type, "& ", qualified_name, "::", member_name, "() const {\n");
        } else {
            w!(w, "inline ", member_type, " ", qualified_name, "::", member_name, "() const {\n");
        }
        if self.should_emit_variant_check(union_ty, variant) {
            w!(w, "if (");
            self.emit_variant_check_condition(w, union_ty, variant);
            w!(w, ") {\n");
            w!(w, "throw std::logic_error(\"Union ", def, " not set to value ", member_name, "\");\n");
            w!(w, "}\n");
        }
        w!(w, "return ic_union_value_.", member_name, ";\n");
        w!(w, "}\n\n");
    }

    fn emit_variant_setters(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        variant: &Variant,
        disc_type: &str,
    ) {
        let member_type = self.cpp_type_member(variant, None, false);
        let member_name = &variant.ident.name;

        if !is_external(&self.hir.context, variant) {
            self.emit_variant_copy_setter(w, def, union_ty, variant, &member_type, member_name);
        }

        if self.should_use_move(&variant.ty, Some(variant)) {
            self.emit_variant_move_setter(w, def, union_ty, variant, &member_type, member_name);
        }

        if variant.is_default {
            self.emit_variant_default_setter(
                w,
                def,
                union_ty,
                disc_type,
                variant,
                &member_type,
                member_name,
            );
        }
    }

    fn emit_variant_copy_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        variant: &Variant,
        member_type: &str,
        member_name: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        if self.should_use_move(&variant.ty, Some(variant)) {
            w!(w, "inline void ", qualified_name, "::", member_name, "(const ", member_type, "& a_value) {\n");
        } else {
            w!(w, "inline void ", qualified_name, "::", member_name, "(", member_type, " a_value) {\n");
        }
        w!(w, "if (");
        self.emit_variant_check_condition(w, union_ty, variant);
        w!(w, ") {\n");
        w!(w, "free_union_();\n");
        self.emit_set_discriminator_to_variant(w, variant, union_ty, None, def.id);

        if self.should_use_move(&variant.ty, Some(variant)) {
            w!(w, "::ic_cts::construct_at(&ic_union_value_.", member_name, ", a_value);\n");
            w!(w, "} else {\n");
            w!(w, "ic_union_value_.", member_name, " = a_value;\n");
        }
        w!(w, "}\n");
        if !self.should_use_move(&variant.ty, Some(variant)) {
            w!(w, "ic_union_value_.", member_name, " = a_value;\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_variant_move_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        variant: &Variant,
        member_type: &str,
        member_name: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::", member_name, "(", member_type, "&& a_value) {\n");
        w!(w, "if (");
        self.emit_variant_check_condition(w, union_ty, variant);
        w!(w, ") {\n");
        w!(w, "free_union_();\n");
        self.emit_set_discriminator_to_variant(w, variant, union_ty, None, def.id);
        w!(w, "::ic_cts::construct_at(&ic_union_value_.", member_name, ", std::move(a_value));\n");
        w!(w, "} else {\n");
        w!(w, "ic_union_value_.", member_name, " = std::move(a_value);\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_variant_default_setter(
        &self,
        w: &mut Twine,
        def: &Def,
        union_ty: &UnionTy,
        disc_type: &str,
        variant: &Variant,
        member_type: &str,
        member_name: &str,
    ) {
        let qualified_name = self.scoped_name(def.id, None);

        w!(w, "inline void ", qualified_name, "::", member_name, "(", member_type, " a_value, ", disc_type, " discriminator) {\n");

        let has_non_default = union_ty.variants.iter().any(|v| !v.is_default);

        if has_non_default {
            self.emit_default_discriminator_check(w, union_ty, "discriminator");
            w!(
                w,
                "throw std::logic_error(\"Illegal discriminator for member ",
                member_name, " of union ", def, "\");\n",
            );
            w!(w, "}\n");
        }

        w!(w, "free_union_();\n");
        if self.should_use_move(&variant.ty, Some(variant)) {
            w!(w, "::ic_cts::construct_at(&ic_union_value_.", member_name, ", std::move(a_value));\n");
        } else {
            w!(w, "ic_union_value_.", member_name, " = a_value;\n");
        }
        w!(w, UNION_DISC_FIELD, " = discriminator;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_free(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let qualified_name = self.scoped_name(def.id, None);
        w!(w, "inline void ", qualified_name, "::free_union_() {\n");

        self.emit_union_switch(w, union_ty, UNION_DISC_FIELD, |w, variant| {
            if !matches!(variant.ty.kind, TyKind::Null)
                && self.should_use_move(&variant.ty, Some(variant))
            {
                w!(w, "std::destroy_at(&ic_union_value_.", variant.ident.name, ");\n");
            }
            true
        });

        w!(w, "}\n\n");
    }

    fn emit_union_serializer(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let qualified_name = self.scoped_name(def.id, None);

        w!(w, "template <class Archive>\n");
        w!(w, "struct ic_cts::Serializer<Archive, ", qualified_name, "> {\n");
        w!(w, "void operator()(Archive& a_archive, ", qualified_name, "& a_value, const ::ic_cts::TypeInfo*) {\n");
        w!(w, "auto a_info = &::ic_cts::TypeTraits<", qualified_name, ">::type_info;\n");
        w!(w, "typename Archive::StructValue serializer(a_archive, a_info);\n");
        w!(w, "auto discr = a_value._d();\n");
        w!(w, "serializer.io(a_info->members[0], discr);\n");

        self.emit_union_switch(w, union_ty, "discr", |w, variant| {
            w!(w, "if (Archive::IS_READER) {\n");
            w!(w, "a_value._d(discr);\n");
            w!(w, "}\n");

            if !matches!(variant.ty.kind, TyKind::Null) {
                let member_idx = union_ty.variants.iter().position(|v| v.ident.name == variant.ident.name).unwrap() + 1;

                let ptr = if is_external(&self.hir.context, variant)  { "*" } else { "" };
                w!(w, "serializer.io(a_info->members[", member_idx.to_string(), "], ", ptr, "a_value.", variant.ident.name, "());\n");
            }

            true
        });

        w!(w, "}\n");
        w!(w, "};\n\n");
    }
}

fn has_implicit_default_variant(union_ty: &UnionTy) -> Option<&Variant> {
    union_ty
        .variants
        .iter()
        .find(|v| v.is_default && v.ty.kind == TyKind::Null)
}
