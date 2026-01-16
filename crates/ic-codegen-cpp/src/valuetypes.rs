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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
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
use ic_hir::hir::{Def, ValueTy};

use crate::codegen::CppGen;

impl CppGen<'_> {
    pub fn emit_valuetype(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        valuetype_ty: &ValueTy,
    ) {
        let valuetype_name = &def.ident.name;

        w!(decl_w, "struct ", valuetype_name);
        if let Some(parent) = valuetype_ty.parent {
            w!(decl_w, " : public ", self.scoped_name(parent, def.id));
        }

        w!(decl_w, " {\n");

        self.emit_valuetype_constructors(decl_w, def, valuetype_ty);
        Self::emit_valuetype_comparison_operators(decl_w, valuetype_name);

        for proto in &valuetype_ty.prototypes {
            self.emit_prototype_declaration(decl_w, def, proto);
        }

        if !valuetype_ty.prototypes.is_empty() {
            w!(decl_w, "\n");
        }

        for member in &valuetype_ty.members {
            self.emit_member(decl_w, member, def.id);
        }

        w!(decl_w, "};\n\n");

        self.emit_typedef_sequence(decl_w, def);
        self.emit_type_traits(impl_w, def);
        self.emit_hash_declaration(impl_w, def);
        self.emit_valuetype_serializer(impl_w, def);

        let all_members = self.collect_all_members(def.id);
        if !all_members.is_empty() {
            self.emit_valuetype_constructor_impl(impl_w, def, valuetype_ty);
        }
        self.emit_valuetype_comparison_impl(impl_w, def);

        for proto in &valuetype_ty.prototypes {
            self.emit_prototype_stub_implementation(impl_w, def, proto);
        }
    }

    fn emit_valuetype_constructors(&self, w: &mut Twine, def: &Def, _valuetype_ty: &ValueTy) {
        let valuetype_name = &def.ident.name;

        w!(w, valuetype_name, "() = default;\n");
        w!(w, valuetype_name, "(const ", valuetype_name, "&) = default;\n");
        w!(w, valuetype_name, "& operator=(const ", valuetype_name, "&) = default;\n");
        w!(w, valuetype_name, "(", valuetype_name, " &&) = default;\n");
        w!(w, valuetype_name, "& operator=(", valuetype_name, " &&) = default;\n");

        let all_members = self.collect_all_members(def.id);

        if !all_members.is_empty() {
            if all_members.len() == 1 {
                w!(w, "explicit ");
            }
            w!(w, valuetype_name, "(\n");
            for (i, member) in all_members.iter().enumerate() {
                let ty_str = self.cpp_type(&member.ty, def.id);
                w!(w, ty_str, " a_", member.ident.name);
                if i < all_members.len() - 1 {
                    w!(w, ",\n");
                }
            }
            w!(w, ");\n");
        }
    }

    fn emit_valuetype_comparison_operators(w: &mut Twine, valuetype_name: &str) {
        w!(w, "bool operator<(const ", valuetype_name, " & a_other) const;\n");
        w!(w, "bool operator==(const ", valuetype_name, " & a_other) const;\n");
        w!(w, "bool operator!=(const ", valuetype_name, " & a_other) const { return !(*this == a_other); }\n");
        w!(w, "bool operator>(const ", valuetype_name, " & a_other) const { return a_other < *this; }\n");
        w!(w, "bool operator<=(const ", valuetype_name, " & a_other) const { return !(a_other < *this); }\n");
        w!(w, "bool operator>=(const ", valuetype_name, " & a_other) const { return !(*this < a_other); }\n\n");
    }

    fn emit_prototype_declaration(&self, w: &mut Twine, def: &Def, proto: &ic_hir::hir::ProtoTy) {
        let return_type = self.cpp_type(&proto.ty, def.id);
        w!(w, return_type, " ", proto.ident.name, "(\n");

        for (i, param) in proto.params.iter().enumerate() {
            let param_type = self.cpp_type(&param.ty, def.id);
            w!(w, param_type, " a_", param.ident.name);
            if i < proto.params.len() - 1 {
                w!(w, ",\n");
            }
        }

        w!(w, "\n);\n");
    }

    fn emit_prototype_stub_implementation(
        &self,
        w: &mut Twine,
        def: &Def,
        proto: &ic_hir::hir::ProtoTy,
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        let return_type = self.cpp_type(&proto.ty, def.id);

        w!(w, "inline ", return_type, " ", qualified_name, "::", proto.ident.name, "(\n");

        for (i, param) in proto.params.iter().enumerate() {
            let param_type = self.cpp_type(&param.ty, def.id);
            w!(w, param_type, " a_", param.ident.name);
            if i < proto.params.len() - 1 {
                w!(w, ",\n");
            }
        }

        w!(w, "\n) {\n");

        if !matches!(
            &proto.ty.kind,
            ic_hir::hir::TyKind::Primitive(ic_hir::hir::PrimitiveTy::Void)
        ) {
            w!(w, "\treturn ", return_type, "{};\n");
        }

        w!(w, "}\n\n");
    }

    fn emit_valuetype_constructor_impl(&self, w: &mut Twine, def: &Def, valuetype_ty: &ValueTy) {
        let qualified_name = self.scoped_name(def.id, None);
        let valuetype_name = &def.ident.name;
        let all_members = self.collect_all_members(def.id);

        w!(w, "inline ", qualified_name, "::", valuetype_name, "(\n");
        for (i, member) in all_members.iter().enumerate() {
            let ty_str = self.cpp_type(&member.ty, def.id);
            w!(w, ty_str, " a_", member.ident.name);
            if i < all_members.len() - 1 {
                w!(w, ",\n");
            }
        }
        w!(w, "\n) : ");

        let mut has_parent = false;
        if let Some(parent_id) = valuetype_ty.parent {
            has_parent = true;
            let parent_name = self.scoped_name(parent_id, None);
            let parent_all_members = self.collect_all_members(parent_id);

            w!(w, parent_name, "(");
            for (i, member) in parent_all_members.iter().enumerate() {
                if self.should_use_move(&member.ty) {
                    w!(w, "std::move(a_", member.ident.name, ")");
                } else {
                    w!(w, "a_", member.ident.name);
                }
                if i < parent_all_members.len() - 1 {
                    w!(w, ", ");
                }
            }
            w!(w, ")");
        }

        for (i, member) in valuetype_ty.members.iter().enumerate() {
            if has_parent || i > 0 {
                w!(w, ",\n\t");
            }
            if self.should_use_move(&member.ty) {
                w!(w, member.ident.name, "(std::move(a_", member.ident.name, "))");
            } else {
                w!(w, member.ident.name, "(a_", member.ident.name, ")");
            }
        }

        w!(w, " {}\n\n");
    }

    fn emit_valuetype_comparison_impl(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);
        let all_members = self.collect_all_members(def.id);
        let param = if all_members.is_empty() {
            ""
        } else {
            " a_other"
        };

        w!(w, "inline bool ", qualified_name, "::operator<(const ", qualified_name, " &", param, ") const {\n");
        if all_members.is_empty() {
            w!(w, "return false;\n");
        } else {
            for (i, member) in all_members.iter().enumerate() {
                let member_name = &member.ident.name;
                if i < all_members.len() - 1 {
                    w!(w, "if (this->", member_name, " < a_other.", member_name, ") { return true; }\n");
                    w!(w, "if (a_other.", member_name, " < this->", member_name, ") { return false; }\n");
                } else {
                    w!(w, "return this->", member_name, " < a_other.", member_name, ";\n");
                }
            }
        }
        w!(w, "}\n\n");

        w!(w, "inline bool ", qualified_name, "::operator==(const ", qualified_name, " &", param, ") const {\n");
        for member in &all_members {
            let member_name = &member.ident.name;
            w!(w, "if (!(this->", member_name, " == a_other.", member_name, ")) { return false; }\n");
        }
        w!(w, "return true;\n");
        w!(w, "}\n\n");
    }

    fn emit_valuetype_serializer(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);
        let all_members = self.collect_all_members(def.id);
        let value_param = if all_members.is_empty() {
            ""
        } else {
            " a_value"
        };

        w!(w, "template <class Archive>\n");
        w!(w, "struct ic_cts::Serializer<Archive, ", qualified_name, "> {\n");
        w!(w, "void operator()(Archive& a_archive, ", qualified_name, "&", value_param, ", const ::ic_cts::TypeInfo*) {\n");
        w!(w, "auto a_info = &::ic_cts::TypeTraits<", qualified_name, ">::type_info;\n");
        w!(w, "typename Archive::StructValue serializer(a_archive, a_info);\n");

        for (i, member) in all_members.iter().enumerate() {
            let member_name = &member.ident.name;
            w!(w, "serializer.io(a_info->members[", i.to_string(), "], a_value.", member_name, ");\n");
        }

        w!(w, "}\n");
        w!(w, "};\n\n");
    }
}
