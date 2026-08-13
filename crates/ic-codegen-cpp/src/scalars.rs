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
use ic_hir::hir::{
    AliasTy, BitmaskTy, ConstTy, Def, DefFlags, DefKind, EnumTy, Numeric, Ty, TyKind,
};

use crate::codegen::{CppGen, cpp_primitive};

impl CppGen<'_> {
    pub fn emit_typedef(&self, decl_w: &mut Twine, def: &Def, alias_ty: &AliasTy) {
        let alias_name = &def.ident.name;
        let ty_str = self.cpp_type(&alias_ty.ty, def.id);
        w!(decl_w, "using ", alias_name, " = ", ty_str, ";\n\n");
    }

    pub fn emit_enum(&self, decl_w: &mut Twine, impl_w: &mut Twine, def: &Def, enum_ty: &EnumTy) {
        let enum_name = &def.ident.name;
        let underlying_type = cpp_primitive(enum_ty.ty);

        if self.options.unscoped_enums {
            w!(decl_w, "enum ", enum_name, " : ", underlying_type, " {\n");
        } else {
            w!(decl_w, "enum class ", enum_name, " : ", underlying_type, " {\n");
        }

        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            let field_name = &field_def.ident.name;

            w!(decl_w, field_name);

            if field_def.flags.contains(DefFlags::IS_ENUMERATED)
                && let DefKind::Const(const_ty) = &field_def.kind
            {
                w!(decl_w, " = ");
                self.emit_numeric_value(decl_w, &const_ty.value, def.id);
            }

            if i < enum_ty.fields.len() - 1 {
                w!(decl_w, ",\n");
            } else {
                w!(decl_w, "\n");
            }
        }

        w!(decl_w, "};\n\n");

        self.emit_type_traits(impl_w, def);
        self.emit_enum_serializer(impl_w, def, enum_ty);
        self.emit_formatter_specialization(impl_w, def);
    }

    fn emit_enum_serializer(&self, w: &mut Twine, def: &Def, enum_ty: &EnumTy) {
        let qualified_name = self.scoped_name(def.id, None);

        w!(w, "template <class Archive>\n");
        w!(w, "struct ic_cts::Serializer<Archive, ", qualified_name, "> {\n");
        w!(w, "void operator()(Archive& a_archive, ", qualified_name, "& a_value, const ::ic_cts::TypeInfo* a_info) {\n");
        w!(w, "auto integer_value = static_cast<", cpp_primitive(enum_ty.ty), ">(a_value);\n");
        w!(w, "a_archive.primitive_io(integer_value, a_info ? a_info : &::ic_cts::TypeTraits<", qualified_name, ">::type_info);\n");
        w!(w, "a_value = static_cast<", qualified_name, ">(integer_value);\n");
        w!(w, "}\n");
        w!(w, "};\n\n");
    }

    pub fn emit_bitmask(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        bitmask_ty: &BitmaskTy,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        let bitmask_name = &def.ident.name;
        let underlying_type = cpp_primitive(bitmask_ty.ty);

        w!(decl_w, "struct ", bitmask_name, " {\n");

        w!(decl_w, "enum : ", underlying_type, " {\n");
        for (i, &flag_id) in bitmask_ty.flags.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);
            let flag_name = &flag_def.ident.name;

            w!(decl_w, flag_name, " = ");

            if let DefKind::Const(const_ty) = &flag_def.kind {
                self.emit_numeric_value(decl_w, &const_ty.value, def.id);
            }

            if i < bitmask_ty.flags.len() - 1 {
                w!(decl_w, ",\n");
            } else {
                w!(decl_w, "\n");
            }
        }
        w!(decl_w, "};\n\n");

        w!(decl_w, bitmask_name, "() = default;\n");

        w!(decl_w, bitmask_name, "(", underlying_type, " v);\n\n");
        w!(impl_w, "inline ", qualified_name, "::", bitmask_name, "(", underlying_type, " v) : _value(v) {}\n");

        w!(decl_w, bitmask_name, "& operator|=(",  underlying_type, " rhs);\n");
        w!(impl_w, "inline ", qualified_name, "& ", qualified_name, "::operator|=(",  underlying_type, " rhs) {\n");
        w!(impl_w, "_value |= rhs;\n");
        w!(impl_w, "return *this;\n");
        w!(impl_w, "}\n\n");

        w!(decl_w, bitmask_name, "& operator&=(",  underlying_type, " rhs);\n");
        w!(impl_w, "inline ", qualified_name, "& ", qualified_name, "::operator&=(",  underlying_type, " rhs) {\n");
        w!(impl_w, "_value &= rhs;\n");
        w!(impl_w, "return *this;\n");
        w!(impl_w, "}\n\n");

        w!(decl_w, bitmask_name, "& operator^=(",  underlying_type, " rhs);\n");
        w!(impl_w, "inline ", qualified_name, "& ", qualified_name, "::operator^=(",  underlying_type, " rhs) {\n");
        w!(impl_w, "_value ^= rhs;\n");
        w!(impl_w, "return *this;\n");
        w!(impl_w, "}\n\n");

        w!(decl_w, "operator ", underlying_type, "() const;\n\n");
        w!(impl_w, "inline ", qualified_name, "::operator ", underlying_type, "() const {\nreturn _value;\n}\n\n");

        w!(decl_w, "private:\n");
        w!(decl_w, underlying_type, " _value{0};\n");

        w!(decl_w, "};\n\n");

        self.emit_type_traits_with_suffix(impl_w, def, "");
        self.emit_bitmask_serializer(impl_w, def, bitmask_ty);
        self.emit_hash_declaration(impl_w, def);
        self.emit_formatter_specialization(impl_w, def);
    }

    fn emit_bitmask_serializer(&self, w: &mut Twine, def: &Def, bitmask_ty: &BitmaskTy) {
        let qualified_name = self.scoped_name(def.id, None);
        let underlying_type = cpp_primitive(bitmask_ty.ty);

        w!(w, "template <class Archive>\n");
        w!(w, "struct ic_cts::Serializer<Archive, ", qualified_name, "> {\n");
        w!(w, "void operator()(Archive& a_archive, ", qualified_name, "& a_value, const ::ic_cts::TypeInfo* a_info) {\n");
        w!(w, "auto integer_value = static_cast<", underlying_type, ">(a_value);\n");
        w!(w, "a_archive.primitive_io(integer_value, a_info ? a_info : &::ic_cts::TypeTraits<", qualified_name, ">::type_info);\n");
        w!(w, "a_value = static_cast<", qualified_name, ">(integer_value);\n");
        w!(w, "}\n");
        w!(w, "};\n\n");
    }

    fn needs_static_keyword(&self, def: &Def) -> bool {
        if let Some(parent_id) = def.parent {
            let parent_def = self.hir.context.definitions.get(parent_id);
            matches!(
                parent_def.kind,
                DefKind::Valuetype(_) | DefKind::Interface(_)
            )
        } else {
            false
        }
    }

    fn string_literal_ty(&self, const_ty: &ConstTy) -> &str {
        let resolved_ty = self.hir.context.resolve_ty(&const_ty.ty);
        let wide = matches!(resolved_ty.kind, TyKind::String { wide: true, .. });
        if self.options.char_ptr_constants {
            if wide {
                "const char16_t*"
            } else {
                "const char*"
            }
        } else if wide {
            "::std::u16string_view"
        } else {
            "::std::string_view"
        }
    }

    pub fn emit_const(&self, decl_w: &mut Twine, def: &Def, const_ty: &ConstTy) {
        let const_name = &def.ident.name;
        let constness = if self.is_constexpr_type(&const_ty.ty) {
            "constexpr"
        } else {
            "const"
        };

        let static_keyword = if self.needs_static_keyword(def) {
            "static "
        } else {
            ""
        };

        match &const_ty.value {
            Numeric::String(_) | Numeric::WString(_) => {
                let string_ty = self.string_literal_ty(const_ty);
                w!(decl_w, "inline ", static_keyword, "constexpr ", string_ty, " ", const_name, " = ");
                self.emit_numeric_value_with_ty(
                    decl_w,
                    &const_ty.value,
                    &const_ty.ty,
                    def.id,
                    false,
                );
                w!(decl_w, ";\n\n");
            }
            Numeric::Const(const_def_id) => {
                let const_def_ty = self
                    .hir
                    .context
                    .resolve_ty(&self.hir.context.base_type_of(*const_def_id));
                let scoped_name = self.scoped_name(*const_def_id, def.id);

                let ty_str = if matches!(const_def_ty.kind, TyKind::String { .. }) {
                    self.string_literal_ty(const_ty).into()
                } else {
                    self.cpp_type(&const_ty.ty, def.id)
                };

                w!(decl_w, "inline ", static_keyword, constness, " ", ty_str, " ", const_name, "{");
                w!(decl_w, scoped_name, "};\n\n");
            }
            _ => {
                let ty_str = self.cpp_type(&const_ty.ty, def.id);
                let is_array = matches!(const_ty.value, Numeric::Array { .. });
                if is_array {
                    w!(decl_w, "inline ", static_keyword, constness, " ", ty_str, " ", const_name);
                    self.emit_numeric_value_with_ty(
                        decl_w,
                        &const_ty.value,
                        &const_ty.ty,
                        def.id,
                        false,
                    );
                    w!(decl_w, ";\n\n");
                } else {
                    w!(decl_w, "inline ", static_keyword, constness, " ", ty_str, " ", const_name, "{");
                    self.emit_numeric_value_with_ty(
                        decl_w,
                        &const_ty.value,
                        &const_ty.ty,
                        def.id,
                        false,
                    );
                    w!(decl_w, "};\n\n");
                }
            }
        }
    }

    fn is_constexpr_type(&self, ty: &Ty) -> bool {
        let resolved_ty = match &ty.kind {
            TyKind::Adt(def_id) => self.hir.context.base_type_of(*def_id),
            _ => ty.clone(),
        };

        match &resolved_ty.kind {
            TyKind::Primitive(_) => true,
            TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                matches!(def.kind, DefKind::Enum(_))
            }
            _ => false,
        }
    }
}
