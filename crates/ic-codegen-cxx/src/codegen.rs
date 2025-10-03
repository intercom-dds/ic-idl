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
use ic_hir::hir::{Def, DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_vfs::{FileId, SourceMap};

use crate::CppOptions;
use crate::deps::collect_def_dependencies;

type Path = Vec<String>;

pub(crate) fn format_array_bounds(ty: &Ty) -> String {
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

pub(crate) fn has_default_value(ty: &Ty) -> bool {
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

pub struct CppGen<'a> {
    pub(crate) hir: &'a ResolvedGraph,
    pub(crate) options: CppOptions,
    source_map: &'a SourceMap,
}

impl<'a> CppGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, source_map: &'a SourceMap, options: CppOptions) -> Self {
        Self {
            hir,
            options,
            source_map,
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

        // Build the full scope path
        let mut scopes = Vec::new();
        let mut current = self.hir.context.definitions.get(def_id).parent;

        while let Some(parent_id) = current {
            let parent_def = self.hir.context.definitions.get(parent_id);
            match &parent_def.kind {
                DefKind::Module(_) => {
                    scopes.push(parent_def.ident.name.as_str());
                }
                DefKind::Interface(_) => {
                    scopes.push(parent_def.ident.name.as_str());
                }
                _ => {}
            }
            current = parent_def.parent;
        }

        if scopes.is_empty() {
            struct_name.to_string()
        } else {
            scopes.reverse();
            format!("{}::{}", scopes.join("::"), struct_name)
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

    #[allow(clippy::unused_self)]
    pub fn should_use_move(&self, ty: &Ty) -> bool {
        !matches!(&ty.kind, TyKind::Primitive(_))
    }

    #[allow(clippy::only_used_in_recursion)]
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

    pub fn emit_hash_specialization(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.qualified_struct_name(def.id);

        w!(w, "template<>\n");
        w!(w, "struct std::hash<", qualified_name, "> {\n");
        w!(w, "using argument_type = ", qualified_name, ";\n");
        w!(w, "using result_type = std::size_t;\n");
        w!(w, "result_type operator()(const argument_type& s) const noexcept {\n");
        w!(w, "result_type h = 0;\n");

        match &def.kind {
            DefKind::Struct(struct_ty) => {
                self.emit_hash_struct_members(w, def, &struct_ty.members);
            }
            DefKind::Union(union_ty) => {
                self.emit_hash_union(w, def, union_ty);
            }
            DefKind::Except(except_ty) => {
                self.emit_hash_struct_members(w, def, &except_ty.members);
            }
            _ => {}
        }

        w!(w, "return h;\n");
        w!(w, "}\n");
        w!(w, "};\n\n");
    }

    fn emit_hash_struct_members(&self, w: &mut Twine, def: &Def, members: &[ic_hir::hir::Member]) {
        // Check if this struct has a parent
        if let ic_hir::hir::DefKind::Struct(struct_ty) = &def.kind {
            if let Some(parent_id) = struct_ty.parent {
                let parent_name = self.qualified_struct_name(parent_id);
                w!(w, "h ^= std::hash<", parent_name, ">()(s);\n");
            }
        }

        // Hash own members
        for member in members {
            let member_name = format!("s.{}", member.ident.name);
            self.emit_hash_member(w, &member_name, &member.ty, def.id, 0);
        }
    }

    fn emit_hash_union(&self, w: &mut Twine, def: &Def, union_ty: &ic_hir::hir::UnionTy) {
        w!(w, "h ^= std::hash<");
        w!(w, self.cpp_type(&union_ty.disc.ty, def.id));
        w!(w, ">()(s._d());\n");

        w!(w, "switch (s._d()) {\n");
        w.dedent();

        for variant in &union_ty.variants {
            if variant.is_default {
                w!(w, "default:\n");
            } else {
                for label in &variant.labels {
                    w!(w, "case ");
                    emit_numeric_value(w, &label.value);
                    w!(w, ":\n");
                }
            }
            w.indent();

            let member_name = format!("s.{}()", variant.ident.name);
            self.emit_hash_member(w, &member_name, &variant.ty, def.id, 0);
            w!(w, "break;\n");

            w.dedent();
        }

        w.indent();
        w!(w, "}\n");
    }

    fn emit_hash_member(
        &self,
        w: &mut Twine,
        name: &str,
        ty: &Ty,
        relative_def: DefId,
        level: usize,
    ) {
        match &ty.kind {
            TyKind::Array { ty: inner_ty, .. } => {
                let mut current_name = name.to_string();
                w!(w, "for (auto& value_", level, " : ", current_name, ") {\n");
                current_name = format!("value_{level}");
                self.emit_hash_member(w, &current_name, inner_ty, relative_def, level + 1);
                w!(w, "}\n");
            }
            TyKind::Sequence { ty: inner_ty, .. } => {
                let new_name = format!("value_{level}");
                let by_ref = if self.should_use_move(inner_ty) {
                    "&"
                } else {
                    ""
                };
                w!(w, "for (auto", by_ref, " ", new_name, " : ", name, ") {\n");
                self.emit_hash_member(w, &new_name, inner_ty, relative_def, level + 1);
                w!(w, "}\n");
            }
            TyKind::Map { key, elem, .. } => {
                let new_name = format!("value_{level}");
                let by_ref = if self.should_use_move(elem) { "&" } else { "" };
                w!(w, "for (auto", by_ref, " ", new_name, " : ", name, ") {\n");

                let key_name = format!("{new_name}.first");
                self.emit_hash_member(w, &key_name, key, relative_def, level + 1);

                let elem_name = format!("{new_name}.second");
                self.emit_hash_member(w, &elem_name, elem, relative_def, level + 1);

                w!(w, "}\n");
            }
            _ => {
                let type_str = self.cpp_type(ty, relative_def);
                w!(w, "h ^= std::hash<", type_str, ">()(", name, ");\n");
            }
        }
    }
}

pub(crate) fn emit_escaped_string(w: &mut Twine, s: &str) {
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

pub(crate) fn emit_numeric_value(w: &mut Twine, value: &ic_hir::hir::Numeric) {
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

#[allow(clippy::unused_self)]
impl CppGen<'_> {
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

    fn emit_forward_decl(&self, w: &mut Twine, def: &Def, decl: ic_hir::hir::Decl) {
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
            DefKind::Decl(decl) => self.emit_forward_decl(decl_w, def, *decl),
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
                let dep_file_path = self.source_map.name(dep_file_id);
                let dep_file_with_h = dep_file_path.with_extension("h");
                let dep_file_h = dep_file_with_h
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap();

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

pub(crate) fn cpp_primitive(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Void => "void",
        PrimitiveTy::Bool => "bool",
        PrimitiveTy::Char => "char",
        PrimitiveTy::WChar => "char16_t",
        PrimitiveTy::Int8 => "int8_t",
        PrimitiveTy::UInt8 => "uint8_t",
        PrimitiveTy::Int16 => "int16_t",
        PrimitiveTy::UInt16 => "uint16_t",
        PrimitiveTy::Int32 => "int32_t",
        PrimitiveTy::UInt32 => "uint32_t",
        PrimitiveTy::Int64 => "int64_t",
        PrimitiveTy::UInt64 => "uint64_t",
        PrimitiveTy::Float32 => "float",
        PrimitiveTy::Float64 => "double",
        PrimitiveTy::Float128 => "long double",
    }
}
