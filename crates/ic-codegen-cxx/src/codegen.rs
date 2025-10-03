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

use std::collections::HashMap;
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefId, DefKind, StructTy, Ty, TyKind};
use ic_vfs::{FileId, SourceMap};

use crate::CppOptions;
use crate::deps::collect_def_dependencies;
use crate::helpers::cpp_primitive;

type Path = Vec<String>;

pub struct CppGen<'a> {
    hir: &'a ResolvedGraph,
    source_map: &'a SourceMap,
    options: CppOptions,
}

impl<'a> CppGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, source_map: &'a SourceMap, options: CppOptions) -> Self {
        Self {
            hir,
            source_map,
            options,
        }
    }

    pub fn cpp_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    pub fn get_enclosing_interface(&self, def_id: DefId) -> Option<DefId> {
        let def = self.hir.context.definitions.get(def_id);
        let mut current = def.parent?;

        loop {
            let current_def = self.hir.context.definitions.get(current);
            if matches!(current_def.kind, DefKind::Interface(_)) {
                return Some(current);
            }
            current = current_def.parent?;
        }
    }

    pub fn qualified_struct_name(&self, def_id: DefId) -> String {
        let struct_name = self.cpp_name(def_id);
        if let Some(interface_id) = self.get_enclosing_interface(def_id) {
            let interface_name = self.cpp_name(interface_id);
            format!("{interface_name}::{struct_name}")
        } else {
            struct_name.to_string()
        }
    }

    pub fn get_scope(&self, def_id: DefId) -> Option<DefId> {
        let def = self.hir.context.definitions.get(def_id);
        let mut current = def.parent?;

        loop {
            let def = self.hir.context.definitions.get(current);
            if matches!(def.kind, DefKind::Module(_)) {
                return Some(current);
            }
            current = def.parent?;
        }
    }

    fn common_scope(&self, def_id1: DefId, def_id2: DefId) -> Option<DefId> {
        let scope1 = self.get_scope(def_id1)?;
        let scope2 = self.get_scope(def_id2)?;

        let mut ancestors1 = Vec::new();
        let mut current = scope1;
        loop {
            ancestors1.push(current);
            let def = self.hir.context.definitions.get(current);
            match def.parent {
                Some(parent_id) => {
                    let parent_def = self.hir.context.definitions.get(parent_id);
                    if matches!(parent_def.kind, DefKind::Module(_)) {
                        current = parent_id;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        let mut current = scope2;
        loop {
            if ancestors1.contains(&current) {
                return Some(current);
            }
            let def = self.hir.context.definitions.get(current);
            match def.parent {
                Some(parent_id) => {
                    let parent_def = self.hir.context.definitions.get(parent_id);
                    if matches!(parent_def.kind, DefKind::Module(_)) {
                        current = parent_id;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        None
    }

    fn build_path_from(&self, from_scope: DefId, to_scope: Option<DefId>) -> Path {
        let mut path = Vec::new();
        let mut current = from_scope;

        loop {
            if Some(current) == to_scope {
                break;
            }

            let def = self.hir.context.definitions.get(current);
            path.push(def.ident.name.clone());

            match def.parent {
                Some(parent_id) => {
                    let parent_def = self.hir.context.definitions.get(parent_id);
                    if matches!(parent_def.kind, DefKind::Module(_)) {
                        current = parent_id;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        path.reverse();
        path
    }

    pub fn scoped_name(&self, target_def_id: DefId, relative_to_def_id: DefId) -> String {
        let type_name = self.cpp_name(target_def_id);

        let target_scope = self.get_scope(target_def_id);
        let current_scope = self.get_scope(relative_to_def_id);

        match (target_scope, current_scope) {
            (None, _) => type_name.to_string(),
            (Some(target_scope), None) => {
                let full_path = self.build_path_from(target_scope, None);
                let pkg_name = full_path.join("::");
                if pkg_name.is_empty() {
                    type_name.to_string()
                } else {
                    format!("{pkg_name}::{type_name}")
                }
            }
            (Some(target_scope), Some(current_scope)) => {
                if target_scope == current_scope {
                    return type_name.to_string();
                }

                let common = self.common_scope(target_def_id, relative_to_def_id);
                if common == Some(current_scope) {
                    let relative_path = self.build_path_from(target_scope, common);
                    let pkg_name = relative_path.join("::");
                    format!("{pkg_name}::{type_name}")
                } else {
                    let full_path = self.build_path_from(target_scope, None);
                    let pkg_name = full_path.join("::");
                    if pkg_name.is_empty() {
                        type_name.to_string()
                    } else {
                        format!("{pkg_name}::{type_name}")
                    }
                }
            }
        }
    }

    pub fn cpp_type(&self, ty: &Ty, relative_def: DefId) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => cpp_primitive(*prim).to_string(),
            TyKind::String { wide, .. } => {
                if *wide {
                    "::std::wstring".to_string()
                } else {
                    "::std::string".to_string()
                }
            }
            TyKind::Adt(def_id) => self.scoped_name(*def_id, relative_def),
            TyKind::Sequence { ty, .. } => {
                let inner = self.cpp_type(ty, relative_def);
                format!("::std::vector<{inner}>")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.cpp_type(key, relative_def);
                let elem_ty = self.cpp_type(elem, relative_def);
                format!("::std::map<{key_ty}, {elem_ty}>")
            }
            TyKind::Array { ty, .. } => self.cpp_type(ty, relative_def),
            TyKind::Any | TyKind::Fixed | TyKind::Null => "void".to_string(),
        }
    }

    fn format_array_bounds(&self, ty: &Ty) -> String {
        let mut result = String::new();
        let mut current_ty = ty;

        while let TyKind::Array {
            ty: inner_ty, len, ..
        } = &current_ty.kind
        {
            result.push('[');
            result.push_str(&len.to_string());
            result.push(']');
            current_ty = inner_ty;
        }
        result
    }

    fn emit_members(&self, w: &mut Twine, def: &Def, members: &[ic_hir::hir::Member]) {
        for member in members {
            let ty_str = self.cpp_type(&member.ty, def.id);
            let array_bounds = self.format_array_bounds(&member.ty);
            w!(w, ty_str, " ", member.ident.name, array_bounds);

            if self.has_default_value(&member.ty) {
                w!(w, " { ");
                self.emit_default_initializer(w, &member.ty);
                w!(w, " }");
            }
            w!(w, ";\n");
        }
    }

    fn emit_struct(&self, decl_w: &mut Twine, impl_w: &mut Twine, def: &Def, struct_ty: &StructTy) {
        let struct_name = &def.ident.name;

        w!(decl_w, "struct ", struct_name);

        if let Some(parent) = struct_ty.parent {
            w!(decl_w, " : public ", self.scoped_name(parent, def.id));
        }

        w!(decl_w, " {\n");

        self.emit_struct_like_constructors(decl_w, def, &struct_ty.members);
        self.emit_struct_like_comparison_operators(decl_w, def, &struct_ty.members);

        w!(decl_w, "\n");

        self.emit_members(decl_w, def, &struct_ty.members);

        w!(decl_w, "};\n\n");

        self.emit_typedef_sequence(decl_w, struct_name);
        self.emit_hash_specialization(impl_w, def);
        if !struct_ty.members.is_empty() {
            self.emit_struct_like_constructor_impl(impl_w, def, &struct_ty.members);
        }
        self.emit_struct_like_comparison_impl(impl_w, def, &struct_ty.members);
    }

    fn emit_exception(
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
        self.emit_struct_like_comparison_operators(decl_w, def, &except_ty.members);

        w!(decl_w, "\n");

        self.emit_members(decl_w, def, &except_ty.members);

        w!(decl_w, "};\n\n");

        self.emit_hash_specialization(impl_w, def);
        self.emit_struct_like_comparison_impl(impl_w, def, &except_ty.members);
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

        w!(impl_w, "inline ", exception_name, "::", exception_name, "()  :\n");
        w!(impl_w, "runtime_error(\"", exception_name, "\") {}\n\n");

        if !members.is_empty() {
            let qualified_name = self.qualified_struct_name(def.id);
            w!(impl_w, "inline ", qualified_name, "::", exception_name, "(\n");
            for (i, member) in members.iter().enumerate() {
                let ty_str = self.cpp_type(&member.ty, def.id);
                w!(impl_w, ty_str, " a_", member.ident.name);
                if i < members.len() - 1 {
                    w!(impl_w, ",\n");
                }
            }
            w!(impl_w, ") :\n");
            w!(impl_w, "runtime_error(\"", exception_name, "\"),\n");

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

    fn emit_struct_like_constructors(
        &self,
        w: &mut Twine,
        def: &Def,
        members: &[ic_hir::hir::Member],
    ) {
        let struct_name = &def.ident.name;

        w!(w, struct_name, "() = default;\n");
        w!(w, struct_name, "(const ", struct_name, "&) = default;\n");
        w!(w, struct_name, "& operator=(const ", struct_name, "&) = default;\n");
        w!(w, struct_name, "(", struct_name, "&&) = default;\n");
        w!(w, struct_name, "& operator=(", struct_name, "&&) = default;\n");

        if !members.is_empty() {
            if members.len() == 1 {
                w!(w, "explicit ");
            }
            w!(w, struct_name, "(\n");
            for (i, member) in members.iter().enumerate() {
                let ty_str = self.cpp_type(&member.ty, def.id);
                w!(w, ty_str, " a_", member.ident.name);
                if i < members.len() - 1 {
                    w!(w, ",\n");
                }
            }
            w!(w, ");\n");
        }
    }

    fn emit_struct_like_comparison_operators(
        &self,
        w: &mut Twine,
        def: &Def,
        _members: &[ic_hir::hir::Member],
    ) {
        let struct_name = &def.ident.name;

        w!(w, "bool operator<(const ", struct_name, "& a_other) const;\n");
        w!(w, "bool operator==(const ", struct_name, "& a_other) const;\n");
        w!(w, "bool operator!=(const ", struct_name, "& a_other) const { return !(*this == a_other); }\n");
        w!(w, "bool operator>(const ", struct_name, "& a_other) const { return a_other < *this; }\n");
        w!(w, "bool operator<=(const ", struct_name, "& a_other) const { return !(a_other < *this); }\n");
        w!(w, "bool operator>=(const ", struct_name, "& a_other) const { return !(*this < a_other); }\n");
    }

    pub fn has_default_value(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Primitive(prim) => matches!(
                prim,
                ic_hir::hir::PrimitiveTy::Int8
                    | ic_hir::hir::PrimitiveTy::Int16
                    | ic_hir::hir::PrimitiveTy::Int32
                    | ic_hir::hir::PrimitiveTy::Int64
                    | ic_hir::hir::PrimitiveTy::UInt8
                    | ic_hir::hir::PrimitiveTy::UInt16
                    | ic_hir::hir::PrimitiveTy::UInt32
                    | ic_hir::hir::PrimitiveTy::UInt64
                    | ic_hir::hir::PrimitiveTy::Bool
                    | ic_hir::hir::PrimitiveTy::Float32
                    | ic_hir::hir::PrimitiveTy::Float64
                    | ic_hir::hir::PrimitiveTy::Float128
                    | ic_hir::hir::PrimitiveTy::Char
                    | ic_hir::hir::PrimitiveTy::WChar
            ),
            TyKind::Array { .. } => true,
            _ => false,
        }
    }

    pub fn should_use_move(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Primitive(_) => false,
            _ => true,
        }
    }

    pub fn emit_default_initializer(&self, w: &mut Twine, ty: &Ty) {
        match &ty.kind {
            TyKind::Primitive(prim) => match prim {
                ic_hir::hir::PrimitiveTy::Bool => w!(w, "false"),
                ic_hir::hir::PrimitiveTy::Int8
                | ic_hir::hir::PrimitiveTy::Int16
                | ic_hir::hir::PrimitiveTy::Int32
                | ic_hir::hir::PrimitiveTy::Int64 => w!(w, "0"),
                ic_hir::hir::PrimitiveTy::UInt8
                | ic_hir::hir::PrimitiveTy::UInt16
                | ic_hir::hir::PrimitiveTy::UInt32 => w!(w, "0U"),
                ic_hir::hir::PrimitiveTy::UInt64 => w!(w, "0ULL"),
                ic_hir::hir::PrimitiveTy::Float32
                | ic_hir::hir::PrimitiveTy::Float64
                | ic_hir::hir::PrimitiveTy::Float128 => w!(w, "0.0"),
                ic_hir::hir::PrimitiveTy::Char => w!(w, "'\\0'"),
                ic_hir::hir::PrimitiveTy::WChar => w!(w, "L'\\0'"),
                ic_hir::hir::PrimitiveTy::Void => {}
            },
            TyKind::Array { ty, .. } => self.emit_default_initializer(w, ty),
            _ => {}
        }
    }

    fn emit_typedef_sequence(&self, w: &mut Twine, struct_name: &str) {
        w!(w, "using ", struct_name, "Seq = ::std::vector<", struct_name, ">;\n");
    }

    pub fn emit_hash_specialization(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.qualified_struct_name(def.id);

        w!(w, "template<>\n");
        w!(w, "struct std::hash<", qualified_name, "> {\n");
        w!(w, "using argument_type = ", qualified_name, ";\n");
        w!(w, "using result_type = std::size_t;\n");
        w!(w, "result_type operator()(const argument_type&) const noexcept;\n");
        w!(w, "};\n");
    }

    fn emit_struct_like_constructor_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        members: &[ic_hir::hir::Member],
    ) {
        let qualified_name = self.qualified_struct_name(def.id);
        let struct_name = &def.ident.name;

        w!(w, "inline ", qualified_name, "::", struct_name, "(\n");
        for (i, member) in members.iter().enumerate() {
            let ty_str = self.cpp_type(&member.ty, def.id);
            w!(w, ty_str, " a_", member.ident.name);
            if i < members.len() - 1 {
                w!(w, ",\n");
            }
        }
        w!(w, ") :\n");

        for (i, member) in members.iter().enumerate() {
            if self.should_use_move(&member.ty) {
                w!(w, member.ident.name, "(std::move(a_", member.ident.name, "))");
            } else {
                w!(w, member.ident.name, "(a_", member.ident.name, ")");
            }
            if i < members.len() - 1 {
                w!(w, ",\n");
            }
        }
        w!(w, " {}\n\n");
    }

    fn emit_struct_like_comparison_impl(
        &self,
        w: &mut Twine,
        def: &Def,
        members: &[ic_hir::hir::Member],
    ) {
        let qualified_name = self.qualified_struct_name(def.id);
        let param = if members.is_empty() { "" } else { " a_other" };

        w!(w, "inline bool ", qualified_name, "::operator<(const ", qualified_name, "&", param, ") const {\n");
        if members.is_empty() {
            w!(w, "return false;\n");
        } else {
            for (i, member) in members.iter().enumerate() {
                let member_name = &member.ident.name;
                if i < members.len() - 1 {
                    w!(w, "if (this->", member_name, " < a_other.", member_name, ") { return true; }\n");
                    w!(w, "if (a_other.", member_name, " < this->", member_name, ") { return false; }\n");
                } else {
                    w!(w, "return this->", member_name, " < a_other.", member_name, ";\n");
                }
            }
        }
        w!(w, "}\n\n");

        w!(w, "inline bool ", qualified_name, "::operator==(const ", qualified_name, "&", param, ") const {\n");
        for member in members {
            let member_name = &member.ident.name;
            w!(w, "if (!(this->", member_name, " == a_other.", member_name, ")) { return false; }\n");
        }
        w!(w, "return true;\n");
        w!(w, "}\n\n");
    }

    fn emit_enum(&self, decl_w: &mut Twine, def: &Def, enum_ty: &ic_hir::hir::EnumTy) {
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
                    self.emit_numeric_value(decl_w, &const_ty.value);
                }
            }

            if i < enum_ty.fields.len() - 1 {
                w!(decl_w, ",\n");
            } else {
                w!(decl_w, "\n");
            }
        }

        w!(decl_w, "};\n\n");
    }

    fn emit_bitmask(&self, decl_w: &mut Twine, def: &Def, bitmask_ty: &ic_hir::hir::BitmaskTy) {
        let bitmask_name = &def.ident.name;
        let underlying_type = cpp_primitive(bitmask_ty.ty);

        w!(decl_w, "enum ", bitmask_name, "Bits : ", underlying_type, " {\n");

        for (i, &flag_id) in bitmask_ty.flags.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);
            let flag_name = &flag_def.ident.name;

            w!(decl_w, flag_name, " = ");

            if let ic_hir::hir::DefKind::Const(const_ty) = &flag_def.kind {
                self.emit_numeric_value(decl_w, &const_ty.value);
            }

            if i < bitmask_ty.flags.len() - 1 {
                w!(decl_w, ",\n");
            } else {
                w!(decl_w, "\n");
            }
        }

        w!(decl_w, "};\n\n");
        w!(decl_w, "using ", bitmask_name, " = ", underlying_type, ";\n\n");
    }

    fn emit_const(&self, decl_w: &mut Twine, def: &Def, const_ty: &ic_hir::hir::ConstTy) {
        let const_name = &def.ident.name;

        match &const_ty.value {
            ic_hir::hir::Numeric::String(s) => {
                w!(decl_w, "inline constexpr const char* ", const_name, " = \"");
                self.emit_escaped_string(decl_w, s);
                w!(decl_w, "\";\n");
            }
            ic_hir::hir::Numeric::Const(const_def_id) => {
                let referenced_const_def = self.hir.context.definitions.get(*const_def_id);
                let referenced_const_scoped_name = self.scoped_name(*const_def_id, def.id);

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

                w!(decl_w, "inline constexpr ", ty_str, " ", const_name, " = ", referenced_const_scoped_name, ";\n");
            }
            _ => {
                let ty_str = self.cpp_type(&const_ty.ty, def.id);
                w!(decl_w, "inline constexpr ", ty_str, " ", const_name, " = ");
                self.emit_numeric_value(decl_w, &const_ty.value);
                w!(decl_w, ";\n");
            }
        }
    }

    fn emit_escaped_string(&self, w: &mut Twine, s: &str) {
        for ch in s.chars() {
            match ch {
                '"' => w!(w, "\\\""),
                '\\' => w!(w, "\\\\"),
                '\n' => w!(w, "\\n"),
                '\r' => w!(w, "\\r"),
                '\t' => w!(w, "\\t"),
                _ => w!(w, ch.to_string()),
            }
        }
    }

    pub fn emit_numeric_value(&self, w: &mut Twine, value: &ic_hir::hir::Numeric) {
        match value {
            ic_hir::hir::Numeric::Int8(v) => w!(w, v.to_string()),
            ic_hir::hir::Numeric::UInt8(v) => w!(w, v.to_string(), "U"),
            ic_hir::hir::Numeric::Int16(v) => w!(w, v.to_string()),
            ic_hir::hir::Numeric::UInt16(v) => w!(w, v.to_string(), "U"),
            ic_hir::hir::Numeric::Int32(v) => w!(w, v.to_string()),
            ic_hir::hir::Numeric::UInt32(v) => w!(w, v.to_string(), "U"),
            ic_hir::hir::Numeric::Int64(v) => w!(w, v.to_string(), "LL"),
            ic_hir::hir::Numeric::UInt64(v) => w!(w, v.to_string(), "ULL"),
            _ => {}
        }
    }

    fn emit_module(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        module: &ic_hir::hir::ModuleTy,
    ) {
        w!(decl_w, "namespace ", def.ident.name, " {\n");
        decl_w.dedent();
        for &nested_id in &module.definitions {
            self.emit_definition(decl_w, impl_w, nested_id);
        }
        w!(decl_w, "} // namespace ", def.ident.name, "\n\n");
    }

    fn emit_typedef(&self, decl_w: &mut Twine, def: &Def, alias_ty: &ic_hir::hir::AliasTy) {
        let alias_name = &def.ident.name;
        let ty_str = self.cpp_type(&alias_ty.ty, def.id);
        w!(decl_w, "using ", alias_name, " = ", ty_str, ";\n");
    }

    fn emit_interface(
        &self,
        decl_w: &mut Twine,
        impl_w: &mut Twine,
        def: &Def,
        interface_ty: &ic_hir::hir::InterfaceTy,
    ) {
        let interface_name = &def.ident.name;

        w!(decl_w, "class ", interface_name, " {\n");
        w!(decl_w, "public:\n");

        w!(decl_w, "virtual ~", interface_name, "() = default;\n");

        for &nested_id in &interface_ty.definitions {
            self.emit_definition(decl_w, impl_w, nested_id);
        }

        for prototype in &interface_ty.prototypes {
            self.emit_prototype(decl_w, def, prototype);
        }

        w!(decl_w, "};\n\n");
    }

    fn emit_prototype(&self, w: &mut Twine, interface_def: &Def, proto: &ic_hir::hir::ProtoTy) {
        let method_name = &proto.ident.name;

        let return_ty_str = match &proto.ty.kind {
            TyKind::Primitive(ic_hir::hir::PrimitiveTy::Void) => "void".to_string(),
            _ => self.cpp_type(&proto.ty, interface_def.id),
        };

        w!(w, "virtual ", return_ty_str, " ", method_name, "(\n");

        for (i, param) in proto.params.iter().enumerate() {
            let ty_str = self.cpp_type(&param.ty, interface_def.id);
            let param_name = &param.ident.name;

            let param_mode = match param.kind {
                ic_hir::hir::ParamKind::In => "",
                ic_hir::hir::ParamKind::Out | ic_hir::hir::ParamKind::Inout => "&",
            };

            w!(w, ty_str, param_mode, " a_", param_name);

            if i < proto.params.len() - 1 {
                w!(w, ",\n");
            } else {
                w!(w, "\n");
            }
        }

        w!(w, ") = 0;\n\n");
    }

    fn emit_forward_decl(&self, w: &mut Twine, def: &Def, decl: &ic_hir::hir::Decl) {
        match decl {
            ic_hir::hir::Decl::Struct | ic_hir::hir::Decl::Union => {
                w!(w, "struct ", def.ident.name, ";\n");
            }
            ic_hir::hir::Decl::Interface | ic_hir::hir::Decl::Valuetype => {
                w!(w, "class ", def.ident.name, ";\n");
            }
            ic_hir::hir::Decl::Native => {}
        }
    }

    fn emit_definition(&self, decl_w: &mut Twine, impl_w: &mut Twine, def_id: DefId) {
        let def = self.hir.context.definitions.get(def_id);

        match &def.kind {
            DefKind::Module(module) => self.emit_module(decl_w, impl_w, def, module),
            DefKind::Struct(struct_ty) => self.emit_struct(decl_w, impl_w, def, struct_ty),
            DefKind::Except(except_ty) => self.emit_exception(decl_w, impl_w, def, except_ty),
            DefKind::Union(union_ty) => self.emit_union(decl_w, impl_w, def, union_ty),
            DefKind::Enum(enum_ty) => self.emit_enum(decl_w, def, enum_ty),
            DefKind::Bitmask(bitmask_ty) => self.emit_bitmask(decl_w, def, bitmask_ty),
            DefKind::Const(const_ty) => self.emit_const(decl_w, def, const_ty),
            DefKind::Alias(alias_ty) => self.emit_typedef(decl_w, def, alias_ty),
            DefKind::Interface(interface_ty) => {
                self.emit_interface(decl_w, impl_w, def, interface_ty);
            }
            DefKind::Decl(decl) => self.emit_forward_decl(decl_w, def, decl),
            _ => {}
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut files_map: HashMap<FileId, Vec<DefId>> = HashMap::new();

        for &def_id in &self.hir.order {
            let def = self.hir.context.definitions.get(def_id);
            let file_id = def.ident.span.start.file_id;
            files_map.entry(file_id).or_default().push(def_id);
        }

        let mut result = Vec::new();
        for (file_id, def_ids) in files_map {
            let file_name = self
                .source_map
                .name(file_id)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap();

            let mut header = Twine::new();
            let mut decls = Twine::new();
            let mut impls = Twine::new();

            w!(header, "#pragma once\n\n");
            w!(header, "#include <cstdint>\n");
            w!(header, "#include <string>\n");
            w!(header, "#include <vector>\n");
            w!(header, "#include <map>\n");

            let mut dependencies = std::collections::HashSet::new();
            for &def_id in &def_ids {
                collect_def_dependencies(self.hir, def_id, file_id, &mut dependencies);
            }

            let mut deps_vec: Vec<FileId> = dependencies.into_iter().collect();
            deps_vec.sort();

            for &dep_file_id in &deps_vec {
                let dep_file = self
                    .source_map
                    .name(dep_file_id)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap();

                let dep_file_h = dep_file.replace(".idl", ".h");
                w!(header, "\n#include \"", dep_file_h, "\"");
            }

            if !deps_vec.is_empty() {
                w!(header, "\n");
            }

            w!(header, "\n");

            for def_id in def_ids {
                self.emit_definition(&mut decls, &mut impls, def_id);
            }

            let mut content = header.finish();
            content.push_str(&decls.finish());
            content.push_str(&impls.finish());

            let output_file = file_name.replace(".idl", ".h");
            result.push(File::Generated {
                path: PathBuf::from(output_file),
                source: content,
            });
        }
        result
    }
}
