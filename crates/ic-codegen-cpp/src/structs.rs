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
use ic_hir::hir::{Def, StructTy};

use crate::codegen::CppGen;

impl CppGen<'_> {
    pub fn emit_struct(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        struct_ty: &StructTy,
    ) {
        w!(decl_w, "struct ", def);
        if let Some(parent) = struct_ty.parent {
            w!(decl_w, " : public ", self.scoped_name(parent, def.id));
        }

        w!(decl_w, " {\n");

        self.emit_struct_like_constructors(decl_w, def);
        self.emit_struct_like_comparison_operators(decl_w, def);

        w!(decl_w, "\n");

        self.emit_members(decl_w, def, &struct_ty.members);

        w!(decl_w, "};\n\n");

        self.emit_typedef_sequence(decl_w, def);
        self.emit_type_traits(impl_w, def);
        self.emit_hash_declaration(impl_w, def);
        self.emit_serializer_specialization(impl_w, def);

        let all_members = self.collect_all_members(def.id);
        if !all_members.is_empty() {
            self.emit_struct_like_constructor_impl(impl_w, def);
        }
        self.emit_struct_like_comparison_impl(impl_w, def, &all_members);
    }

    pub fn emit_exception(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        except_ty: &ic_hir::hir::ExceptTy,
    ) {
        let exception_name = &def.ident.name;

        w!(decl_w, "struct ", exception_name, " : std::runtime_error\n");
        w!(decl_w, " {\n");

        self.emit_exception_constructors(decl_w, impl_w, def, &except_ty.members);
        self.emit_struct_like_comparison_operators(decl_w, def);

        w!(decl_w, "\n");

        self.emit_members(decl_w, def, &except_ty.members);

        w!(decl_w, "};\n\n");

        self.emit_hash_declaration(impl_w, def);
        self.emit_struct_like_comparison_impl(impl_w, def, &except_ty.members);
    }

    fn emit_members(&self, w: &mut Twine, def: &Def, members: &[ic_hir::hir::Member]) {
        for member in members {
            let ty_str = self.cpp_type(&member.ty, def.id);
            w!(w, ty_str, " ", member.ident.name);

            if self.has_default_value(&member.ty) {
                w!(w, "{");
                self.emit_default_initializer(w, &member.ty);
                w!(w, "}");
            }
            w!(w, ";\n");
        }
    }

    fn emit_exception_constructors(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        members: &[ic_hir::hir::Member],
    ) {
        let exception_name = &def.ident.name;

        w!(decl_w, exception_name, "();\n");
        w!(decl_w, exception_name, "(const ", exception_name, "&) = default;\n");
        w!(decl_w, exception_name, "& operator=(const ", exception_name, "&) = default;\n");
        w!(decl_w, exception_name, "(", exception_name, "&&) = default;\n");
        w!(decl_w, exception_name, "& operator=(", exception_name, "&&) = default;\n");

        if !members.is_empty() {
            if members.len() == 1 {
                w!(decl_w, "explicit ");
            }
            w!(decl_w, exception_name, "(\n");
            for (i, member) in members.iter().enumerate() {
                let ty_str = self.cpp_type(&member.ty, def.id);
                w!(decl_w, ty_str, " a_", member.ident.name);
                if i < members.len() - 1 {
                    w!(decl_w, ",\n");
                }
            }
            w!(decl_w, ");\n");
        }

        let qualified_name = self.scoped_name(def.id, None);
        w!(impl_w, "inline ", qualified_name, "::", exception_name, "()  :\n");
        w!(impl_w, "std::runtime_error(\"", exception_name, "\") {}\n\n");

        if !members.is_empty() {
            w!(impl_w, "inline ", qualified_name, "::", exception_name, "(\n");
            for (i, member) in members.iter().enumerate() {
                let ty_str = self.cpp_type(&member.ty, def.id);
                w!(impl_w, ty_str, " a_", member.ident.name);
                if i < members.len() - 1 {
                    w!(impl_w, ",\n");
                }
            }
            w!(impl_w, ") :\n");
            w!(impl_w, "std::runtime_error(\"", exception_name, "\"),\n");

            for (i, member) in members.iter().enumerate() {
                if self.should_use_move(&member.ty) {
                    w!(impl_w, member.ident.name, "(std::move(a_", member.ident.name, "))");
                } else {
                    w!(impl_w, member.ident.name, "(a_", member.ident.name, ")");
                }
                if i < members.len() - 1 {
                    w!(impl_w, ",\n");
                }
            }
            w!(impl_w, " {}\n\n");
        }
    }

    fn emit_struct_like_constructors(&self, w: &mut Twine, def: &Def) {
        let struct_name = &def.ident.name;

        w!(w, struct_name, "() = default;\n");
        w!(w, struct_name, "(const ", struct_name, "&) = default;\n");
        w!(w, struct_name, "& operator=(const ", struct_name, "&) = default;\n");
        w!(w, struct_name, "(", struct_name, "&&) = default;\n");
        w!(w, struct_name, "& operator=(", struct_name, "&&) = default;\n");

        let all_members = self.collect_all_members(def.id);

        if !all_members.is_empty() {
            if all_members.len() == 1 {
                w!(w, "explicit ");
            }
            w!(w, struct_name, "(\n");
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

    fn emit_struct_like_comparison_operators(&self, w: &mut Twine, def: &Def) {
        let struct_name = &def.ident.name;

        w!(w, "bool operator<(const ", struct_name, "& a_other) const;\n");
        w!(w, "bool operator==(const ", struct_name, "& a_other) const;\n");
        w!(w, "bool operator!=(const ", struct_name, "& a_other) const { return !(*this == a_other); }\n");
        w!(w, "bool operator>(const ", struct_name, "& a_other) const { return a_other < *this; }\n");
        w!(w, "bool operator<=(const ", struct_name, "& a_other) const { return !(a_other < *this); }\n");
        w!(w, "bool operator>=(const ", struct_name, "& a_other) const { return !(*this < a_other); }\n");
    }

    fn emit_struct_like_constructor_impl(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);
        let struct_name = &def.ident.name;
        let all_members = self.collect_all_members(def.id);

        w!(w, "inline ", qualified_name, "::", struct_name, "(\n");
        for (i, member) in all_members.iter().enumerate() {
            let ty_str = self.cpp_type(&member.ty, def.id);
            w!(w, ty_str, " a_", member.ident.name);
            if i < all_members.len() - 1 {
                w!(w, ",\n");
            }
        }
        w!(w, "\n) : ");

        // Check if there's a parent and emit parent constructor call
        let mut has_parent = false;
        if let ic_hir::hir::DefKind::Struct(struct_ty) = &def.kind
            && let Some(parent_id) = struct_ty.parent
        {
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

        // Initialize own members
        if let ic_hir::hir::DefKind::Struct(struct_ty) = &def.kind {
            for (i, member) in struct_ty.members.iter().enumerate() {
                if has_parent || i > 0 {
                    w!(w, ",\n\t");
                }
                if self.should_use_move(&member.ty) {
                    w!(w, member.ident.name, "(std::move(a_", member.ident.name, "))");
                } else {
                    w!(w, member.ident.name, "(a_", member.ident.name, ")");
                }
            }
        }

        w!(w, " {}\n\n");
    }

    fn emit_struct_like_comparison_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        all_members: &[ic_hir::hir::Member],
    ) {
        let qualified_name = self.scoped_name(def.id, None);
        let param = if all_members.is_empty() {
            ""
        } else {
            " a_other"
        };

        w!(w, "inline bool ", qualified_name, "::operator<(const ", qualified_name, "&", param, ") const {\n");
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

        w!(w, "inline bool ", qualified_name, "::operator==(const ", qualified_name, "&", param, ") const {\n");
        for member in all_members {
            let member_name = &member.ident.name;
            w!(w, "if (!(this->", member_name, " == a_other.", member_name, ")) { return false; }\n");
        }
        w!(w, "return true;\n");
        w!(w, "}\n\n");
    }

    fn emit_serializer_specialization(&self, w: &mut Twine, def: &Def) {
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
