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

use crate::codegen::{CppGen, cpp_primitive, emit_escaped_string};

impl CppGen<'_> {
    pub fn emit_typedef(&self, decl_w: &mut Twine, def: &Def, alias_ty: &ic_hir::hir::AliasTy) {
        let alias_name = &def.ident.name;
        let ty_str = self.cpp_type(&alias_ty.ty, def.id);
        w!(decl_w, "using ", alias_name, " = ", ty_str, ";\n\n");
    }

    pub fn emit_enum(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
    ) {
        let enum_name = &def.ident.name;

        if self.options.scoped_enums {
            w!(decl_w, "enum class ", enum_name, " : int32_t {\n");
        } else {
            w!(decl_w, "enum ", enum_name, " : int32_t {\n");
        }

        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            let field_name = &field_def.ident.name;

            w!(decl_w, field_name);

            if field_def
                .flags
                .contains(ic_hir::hir::DefFlags::IS_ENUMERATED)
            {
                if let ic_hir::hir::DefKind::Const(const_ty) = &field_def.kind {
                    w!(decl_w, " = ");
                    self.emit_numeric_value(decl_w, &const_ty.value, def.id);
                }
            }

            if i < enum_ty.fields.len() - 1 {
                w!(decl_w, ",\n");
            } else {
                w!(decl_w, "\n");
            }
        }

        w!(decl_w, "};\n\n");

        self.emit_type_traits(impl_w, def);
        self.emit_enum_serializer(impl_w, def);
    }

    fn emit_enum_serializer(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);

        w!(w, "template <class Archive>\n");
        w!(w, "struct ::ic_cts::Serializer<Archive, ", qualified_name, "> {\n");
        w!(w, "void operator()(Archive& a_archive, ", qualified_name, "& a_value, const ::ic_cts::TypeInfo* a_info) {\n");
        w!(w, "auto integer_value = static_cast<int32_t>(a_value);\n");
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
        bitmask_ty: &ic_hir::hir::BitmaskTy,
    ) {
        let bitmask_name = &def.ident.name;
        let underlying_type = cpp_primitive(bitmask_ty.ty);

        w!(decl_w, "enum ", bitmask_name, "Bits : ", underlying_type, " {\n");

        for (i, &flag_id) in bitmask_ty.flags.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);
            let flag_name = &flag_def.ident.name;

            w!(decl_w, flag_name, " = ");

            if let ic_hir::hir::DefKind::Const(const_ty) = &flag_def.kind {
                self.emit_numeric_value(decl_w, &const_ty.value, def.id);
            }

            if i < bitmask_ty.flags.len() - 1 {
                w!(decl_w, ",\n");
            } else {
                w!(decl_w, "\n");
            }
        }

        w!(decl_w, "};\n\n");
        w!(decl_w, "using ", bitmask_name, " = ", underlying_type, ";\n\n");

        self.emit_type_traits(impl_w, def);
        self.emit_bitmask_serializer(impl_w, def, bitmask_ty);
    }

    fn emit_bitmask_serializer(
        &self,
        w: &mut Twine,
        def: &Def,
        bitmask_ty: &ic_hir::hir::BitmaskTy,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        let bitmask_name = &def.ident.name;
        let underlying_type = cpp_primitive(bitmask_ty.ty);

        w!(w, "template <class Archive>\n");
        w!(w, "struct ::ic_cts::Serializer<Archive, ", qualified_name, "> {\n");
        w!(w, "void operator()(Archive& a_archive, ", bitmask_name, "& a_value, const ::ic_cts::TypeInfo* a_info) {\n");
        w!(w, "auto integer_value = static_cast<", underlying_type, ">(a_value);\n");
        w!(w, "a_archive.primitive_io(integer_value, a_info ? a_info : &::ic_cts::TypeTraits<", qualified_name, ">::type_info);\n");
        w!(w, "a_value = static_cast<", bitmask_name, ">(integer_value);\n");
        w!(w, "}\n");
        w!(w, "};\n\n");
    }

    pub fn emit_const(&self, decl_w: &mut Twine, def: &Def, const_ty: &ic_hir::hir::ConstTy) {
        let const_name = &def.ident.name;
        let constness = if self.is_constexpr_type(&const_ty.ty) {
            "constexpr"
        } else {
            "const"
        };

        match &const_ty.value {
            ic_hir::hir::Numeric::String(s) => {
                w!(decl_w, "inline constexpr const char* ", const_name, " = \"");
                emit_escaped_string(decl_w, s);
                w!(decl_w, "\";\n\n");
            }
            ic_hir::hir::Numeric::Const(const_def_id) => {
                let referenced_const_def = self.hir.context.definitions.get(*const_def_id);
                let scoped_name = self.scoped_name(*const_def_id, def.id);

                let ty_str =
                    if let ic_hir::hir::DefKind::Const(ref_const_ty) = &referenced_const_def.kind {
                        if matches!(ref_const_ty.value, ic_hir::hir::Numeric::String(_)) {
                            "const char*".to_string()
                        } else {
                            self.cpp_type(&const_ty.ty, def.id)
                        }
                    } else {
                        self.cpp_type(&const_ty.ty, def.id)
                    };

                w!(decl_w, "inline ", constness, " ", ty_str, " ", const_name, " = ");
                w!(decl_w, scoped_name, ";\n\n");
            }
            _ => {
                let ty_str = self.cpp_type(&const_ty.ty, def.id);
                w!(decl_w, "inline ", constness, " ", ty_str, " ", const_name, " = ");
                self.emit_numeric_value(decl_w, &const_ty.value, def.id);
                w!(decl_w, ";\n\n");
            }
        }
    }

    fn is_constexpr_type(&self, ty: &ic_hir::hir::Ty) -> bool {
        match &ty.kind {
            ic_hir::hir::TyKind::Primitive(_) => true,
            ic_hir::hir::TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                matches!(def.kind, ic_hir::hir::DefKind::Enum(_))
            }
            _ => false,
        }
    }
}
