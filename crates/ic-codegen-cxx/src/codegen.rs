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
use ic_hir::hir::{
    Decl, Def, DefId, DefKind, InterfaceTy, Member, ModuleTy, Numeric, ParamKind, PrimitiveTy,
    ProtoTy, Ty, TyKind, UnionTy,
};
use ic_vfs::{FileId, SourceMap};

use crate::CppOptions;
use crate::deps::collect_def_dependencies;

type Path = Vec<String>;

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

    fn cpp_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    fn get_scope(&self, def_id: DefId) -> Option<DefId> {
        let def = self.hir.context.definitions.get(def_id);
        let mut current = def.parent?;

        loop {
            let def = self.hir.context.definitions.get(current);
            if matches!(def.kind, DefKind::Module(_)) {
                return Some(current);
            }
            if self.options.scoped_enums && matches!(def.kind, DefKind::Enum(_)) {
                return Some(current);
            }
            current = def.parent?;
        }
    }

    fn common_scope(&self, def_id1: DefId, def_id2: DefId) -> Option<DefId> {
        let mut scope1 = self.get_scope(def_id1)?;
        let mut scope2 = self.get_scope(def_id2)?;

        let mut ancestors1 = Vec::new();
        loop {
            ancestors1.push(scope1);
            scope1 = match self.get_scope(scope1) {
                Some(s) => s,
                None => break,
            };
        }

        loop {
            if ancestors1.contains(&scope2) {
                return Some(scope2);
            }
            scope2 = self.get_scope(scope2)?;
        }
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

            current = match self.get_scope(current) {
                Some(scope) => scope,
                None => break,
            };
        }

        path.reverse();
        path
    }

    pub fn scoped_name(
        &self,
        target_def_id: DefId,
        relative_to_def_id: impl Into<Option<DefId>>,
    ) -> String {
        let type_name = self.cpp_name(target_def_id).to_string();
        let relative_to_def_id = relative_to_def_id.into();
        let target_scope = self.get_scope(target_def_id);

        match (target_scope, relative_to_def_id) {
            (None, _) => type_name,
            (Some(target_scope), None) => {
                let full_path = self.build_path_from(target_scope, None);
                let pkg_name = full_path.join("::");
                if pkg_name.is_empty() {
                    type_name
                } else {
                    format!("{pkg_name}::{type_name}")
                }
            }
            (Some(target_scope), Some(relative_to_def_id)) => {
                let current_scope = self.get_scope(relative_to_def_id);

                match current_scope {
                    None => {
                        let full_path = self.build_path_from(target_scope, None);
                        let pkg_name = full_path.join("::");
                        if pkg_name.is_empty() {
                            type_name
                        } else {
                            format!("{pkg_name}::{type_name}")
                        }
                    }
                    Some(current_scope) => {
                        let target_scope_id = self
                            .hir
                            .context
                            .scopes
                            .find_scope_containing_def(target_def_id);

                        let current_scope_id = self
                            .hir
                            .context
                            .scopes
                            .find_scope_containing_def(relative_to_def_id);

                        if target_scope_id == current_scope_id && target_scope_id.is_some() {
                            return type_name;
                        }

                        let common = self.common_scope(target_def_id, relative_to_def_id);
                        if common == Some(target_scope) || common == Some(current_scope) {
                            let relative_path = self.build_path_from(target_scope, common);
                            let pkg_name = relative_path.join("::");
                            if pkg_name.is_empty() {
                                type_name
                            } else {
                                format!("{pkg_name}::{type_name}")
                            }
                        } else {
                            let full_path = self.build_path_from(target_scope, None);
                            let pkg_name = full_path.join("::");
                            if pkg_name.is_empty() {
                                type_name
                            } else {
                                format!("{pkg_name}::{type_name}")
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn has_default_value(&self, ty: &Ty) -> bool {
        let resolved_ty = match &ty.kind {
            TyKind::Adt(def_id) => self.hir.context.base_type_of(*def_id),
            _ => ty.clone(),
        };

        match &resolved_ty.kind {
            TyKind::Primitive(prim) => matches!(
                prim,
                PrimitiveTy::Int8
                    | PrimitiveTy::Int16
                    | PrimitiveTy::Int32
                    | PrimitiveTy::Int64
                    | PrimitiveTy::UInt8
                    | PrimitiveTy::UInt16
                    | PrimitiveTy::UInt32
                    | PrimitiveTy::UInt64
                    | PrimitiveTy::Bool
                    | PrimitiveTy::Float32
                    | PrimitiveTy::Float64
                    | PrimitiveTy::Float128
                    | PrimitiveTy::Char
                    | PrimitiveTy::WChar
            ),
            TyKind::Array { .. } => true,
            _ => false,
        }
    }

    pub fn cpp_type(&self, ty: &Ty, relative_def: impl Into<Option<DefId>>) -> String {
        let relative_def_opt = relative_def.into();
        match &ty.kind {
            TyKind::Primitive(prim) => cpp_primitive(*prim).to_string(),
            TyKind::String { wide, .. } => {
                if *wide {
                    "::std::wstring".to_string()
                } else {
                    "::std::string".to_string()
                }
            }
            TyKind::Adt(def_id) => self.scoped_name(*def_id, relative_def_opt),
            TyKind::Sequence { ty, .. } => {
                let inner = self.cpp_type(ty, relative_def_opt);
                format!("::std::vector<{inner}>")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.cpp_type(key, relative_def_opt);
                let elem_ty = self.cpp_type(elem, relative_def_opt);
                format!("::std::map<{key_ty}, {elem_ty}>")
            }
            TyKind::Array { ty, len, .. } => {
                let inner = self.cpp_type(ty, relative_def_opt);
                format!("::std::array<{inner}, {len}>")
            }
            TyKind::Any | TyKind::Fixed | TyKind::Null => "void".to_string(),
        }
    }

    #[allow(clippy::unused_self)]
    pub fn should_use_move(&self, ty: &Ty) -> bool {
        !matches!(&ty.kind, TyKind::Primitive(_))
    }

    pub fn emit_numeric_value(
        &self,
        w: &mut Twine,
        value: &Numeric,
        relative_def: impl Into<Option<DefId>>,
    ) {
        let relative_def_opt = relative_def.into();
        match value {
            Numeric::Null => w!(w, "nullptr"),
            Numeric::Bool(v) => w!(w, if *v { "true" } else { "false" }),
            Numeric::Char(v) => w!(w, "'", v.to_string(), "'"),
            Numeric::Int8(v) => w!(w, v.to_string()),
            Numeric::UInt8(v) => w!(w, v.to_string(), "U"),
            Numeric::Int16(v) => w!(w, v.to_string()),
            Numeric::UInt16(v) => w!(w, v.to_string(), "U"),
            Numeric::Int32(v) => w!(w, v.to_string()),
            Numeric::UInt32(v) => w!(w, v.to_string(), "U"),
            Numeric::Int64(v) => w!(w, v.to_string(), "LL"),
            Numeric::UInt64(v) => w!(w, v.to_string(), "ULL"),
            Numeric::Float(v) => w!(w, format!("{:.7}", v), "f"),
            Numeric::Double(v) => w!(w, format!("{:.16}", v)),
            Numeric::String(s) => {
                w!(w, "\"");
                emit_escaped_string(w, s);
                w!(w, "\"");
            }
            Numeric::Const(const_def_id) => {
                let name = self.scoped_name(*const_def_id, relative_def_opt);
                w!(w, name);
            }
            Numeric::Sequence { values, .. } | Numeric::Array { values, .. } => {
                w!(w, "{");
                for (i, elem) in values.iter().enumerate() {
                    self.emit_numeric_value(w, elem, relative_def_opt);
                    if i < values.len() - 1 {
                        w!(w, ", ");
                    }
                }
                w!(w, "}");
            }
            Numeric::Struct { fields, .. } => {
                w!(w, "{");
                for (i, (_ident, value)) in fields.iter().enumerate() {
                    self.emit_numeric_value(w, value, relative_def_opt);
                    if i < fields.len() - 1 {
                        w!(w, ", ");
                    }
                }
                w!(w, "}");
            }
            Numeric::Map { entries, .. } => {
                w!(w, "{");
                for (i, (key, value)) in entries.iter().enumerate() {
                    w!(w, "{ ");
                    self.emit_numeric_value(w, key, relative_def_opt);
                    w!(w, ", ");
                    self.emit_numeric_value(w, value, relative_def_opt);
                    w!(w, " }");
                    if i < entries.len() - 1 {
                        w!(w, ", ");
                    }
                }
                w!(w, "}");
            }
            Numeric::Union { discriminant, .. } => {
                w!(w, "{ ");
                self.emit_numeric_value(w, discriminant, relative_def_opt);
                w!(w, " }");
            }
        }
    }

    pub fn emit_default_initializer(&self, w: &mut Twine, ty: &Ty) {
        match &ty.kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => w!(w, "false"),
                PrimitiveTy::Int8
                | PrimitiveTy::Int16
                | PrimitiveTy::Int32
                | PrimitiveTy::Int64 => w!(w, "0"),
                PrimitiveTy::UInt8 | PrimitiveTy::UInt16 | PrimitiveTy::UInt32 => w!(w, "0U"),
                PrimitiveTy::UInt64 => w!(w, "0ULL"),
                PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128 => w!(w, "0.0"),
                PrimitiveTy::Char => w!(w, "'\\0'"),
                PrimitiveTy::WChar => w!(w, "L'\\0'"),
                PrimitiveTy::Void => {}
            },
            TyKind::Array { .. } => w!(w, "{}"),
            _ => {}
        }
    }

    pub fn emit_type_traits(&self, w: &mut Twine, def: &Def) {
        self.emit_type_traits_with_suffix(w, def, "");
    }

    pub fn emit_type_traits_with_suffix(&self, w: &mut Twine, def: &Def, suffix: &str) {
        let qualified_name = self.scoped_name(def.id, None);
        let struct_name = &def.ident.name;
        let full_qualified_name = format!("{}{}", qualified_name, suffix);
        let full_struct_name = format!("{}{}", struct_name, suffix);

        w!(w, "template <>\n");
        w!(w, "struct ::ic_cts::TypeTraits<", full_qualified_name, "> {\n");
        w!(w, "using value_type = ", full_qualified_name, ";\n");
        w!(w, "using in_type = const ", full_qualified_name, "&;\n");
        w!(w, "using out_type = ", full_qualified_name, "&;\n");
        w!(w, "using inout_type = ", full_qualified_name, "&;\n");
        w!(w, "using ref_type = std::shared_ptr<", full_qualified_name, ">;\n");
        w!(w, "using weak_ref_type = std::weak_ptr<", full_qualified_name, ">;\n");

        if let DefKind::Struct(_) | DefKind::Union(_) = &def.kind {
            w!(w, "using sequence_type = ", full_struct_name, "Seq;\n");
        }

        w!(w, "static const ::ic_cts::TypeInfo type_info;\n");

        match &def.kind {
            DefKind::Struct(_) => w!(w, "static const bool is_struct = true;\n"),
            DefKind::Union(_) => w!(w, "static const bool is_union = true;\n"),
            DefKind::Enum(_) => w!(w, "static const bool is_enum = true;\n"),
            DefKind::Bitmask(_) => w!(w, "static const bool is_bitmask = true;\n"),
            _ => {}
        }
        w!(w, "};\n\n");
    }

    pub fn emit_typedef_sequence(&self, w: &mut Twine, type_name: &str) {
        w!(w, "using ", type_name, "Seq = ::std::vector<", type_name, ">;\n\n");
    }

    pub fn emit_hash_specialization(&self, w: &mut Twine, def: &Def) {
        let qualified_name = self.scoped_name(def.id, None);

        w!(w, "template<>\n");
        w!(w, "struct ::std::hash<", qualified_name, "> {\n");
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

    fn emit_hash_struct_members(&self, w: &mut Twine, def: &Def, members: &[Member]) {
        // Check if this struct has a parent
        if let DefKind::Struct(struct_ty) = &def.kind {
            if let Some(parent_id) = struct_ty.parent {
                let parent_name = self.scoped_name(parent_id, None);
                w!(w, "h ^= ::std::hash<", parent_name, ">()(s);\n");
            }
        }

        // Hash own members
        for member in members {
            let member_name = format!("s.{}", member.ident.name);
            self.emit_hash_member(w, &member_name, &member.ty, 0);
        }
    }

    fn emit_hash_union(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        w!(w, "h ^= ::std::hash<");
        match &union_ty.disc.ty.kind {
            TyKind::Adt(disc_def_id) => {
                let qualified_disc_name = self.scoped_name(*disc_def_id, None);
                w!(w, qualified_disc_name);
            }
            _ => {
                w!(w, self.cpp_type(&union_ty.disc.ty, def.id));
            }
        }
        w!(w, ">()(s._d());\n");

        w!(w, "switch (s._d()) {\n");
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

            let member_name = format!("s.{}()", variant.ident.name);
            self.emit_hash_member(w, &member_name, &variant.ty, 0);
            w!(w, "break;\n");

            w.dedent();
        }

        w.indent();
        w!(w, "}\n");
    }

    fn emit_hash_member(&self, w: &mut Twine, name: &str, ty: &Ty, level: usize) {
        match &ty.kind {
            TyKind::Array { ty: inner_ty, .. } => {
                let mut current_name = name.to_string();
                w!(w, "for (auto& value_", level, " : ", current_name, ") {\n");
                current_name = format!("value_{level}");
                self.emit_hash_member(w, &current_name, inner_ty, level + 1);
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
                self.emit_hash_member(w, &new_name, inner_ty, level + 1);
                w!(w, "}\n");
            }
            TyKind::Map { key, elem, .. } => {
                let new_name = format!("value_{level}");
                let by_ref = if self.should_use_move(elem) { "&" } else { "" };
                w!(w, "for (auto", by_ref, " ", new_name, " : ", name, ") {\n");

                let key_name = format!("{new_name}.first");
                self.emit_hash_member(w, &key_name, key, level + 1);

                let elem_name = format!("{new_name}.second");
                self.emit_hash_member(w, &elem_name, elem, level + 1);

                w!(w, "}\n");
            }
            _ => {
                let type_str = self.cpp_type(ty, None);
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

#[allow(clippy::unused_self)]
impl CppGen<'_> {
    fn emit_module(&self, decl_w: &mut Twine, impl_w: &mut Twine, def: &Def, module: &ModuleTy) {
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
        interface_ty: &InterfaceTy,
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

    fn emit_prototype(&self, w: &mut Twine, interface_def: &Def, proto: &ProtoTy) {
        let method_name = &proto.ident.name;

        let return_ty_str = match &proto.ty.kind {
            TyKind::Primitive(PrimitiveTy::Void) => "void".to_string(),
            _ => self.cpp_type(&proto.ty, interface_def.id),
        };

        w!(w, "virtual ", return_ty_str, " ", method_name, "(\n");

        for (i, param) in proto.params.iter().enumerate() {
            let ty_str = self.cpp_type(&param.ty, interface_def.id);
            let param_name = &param.ident.name;

            let param_mode = match param.kind {
                ParamKind::In => "",
                ParamKind::Out | ParamKind::Inout => "&",
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

    fn emit_forward_decl(&self, w: &mut Twine, def: &Def, decl: Decl) {
        match decl {
            Decl::Struct | Decl::Union => {
                w!(w, "struct ", def.ident.name, ";\n");
            }
            Decl::Interface | Decl::Valuetype => {
                w!(w, "class ", def.ident.name, ";\n");
            }
            Decl::Native => {}
        }
    }

    fn emit_definition(&self, decl_w: &mut Twine, impl_w: &mut Twine, def_id: DefId) {
        let def = self.hir.context.definitions.get(def_id);

        match &def.kind {
            DefKind::Module(module) => self.emit_module(decl_w, impl_w, def, module),
            DefKind::Struct(struct_ty) => self.emit_struct(decl_w, impl_w, def, struct_ty),
            DefKind::Except(except_ty) => self.emit_exception(decl_w, impl_w, def, except_ty),
            DefKind::Union(union_ty) => self.emit_union(decl_w, impl_w, def, union_ty),
            DefKind::Enum(enum_ty) => self.emit_enum(decl_w, impl_w, def, enum_ty),
            DefKind::Bitmask(bitmask_ty) => self.emit_bitmask(decl_w, impl_w, def, bitmask_ty),
            DefKind::Const(const_ty) => self.emit_const(decl_w, def, const_ty),
            DefKind::Alias(alias_ty) => self.emit_typedef(decl_w, def, alias_ty),
            DefKind::Interface(interface_ty) => {
                self.emit_interface(decl_w, impl_w, def, interface_ty);
            }
            DefKind::Decl(decl) => self.emit_forward_decl(decl_w, def, *decl),
            _ => {}
        }
    }

    fn output_extension(&self) -> &str {
        self.options.header_ext.as_deref().unwrap_or("h")
    }

    fn output_filename(&self, file_id: FileId) -> String {
        let source_path = self.source_map.name(file_id);
        let output_file = source_path.with_extension(self.output_extension());
        output_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string()
    }

    fn build_path(&self, file_name: &str) -> String {
        if let Some(subfolder) = &self.options.header_subdir {
            format!("{}/{}", subfolder, file_name).replace('\\', "/")
        } else {
            file_name.replace('\\', "/")
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
            let mut header = Twine::new();
            let mut decls = Twine::new();
            let mut impls = Twine::new();

            w!(header, "#pragma once\n\n");
            w!(header, "#include <array>\n");
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
                let dep_file_name = self.output_filename(dep_file_id);
                let include_path = self.build_path(&dep_file_name);
                w!(header, "\n#include \"", include_path, "\"");
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

            let output_file_name = self.output_filename(file_id);
            let output_path = self.build_path(&output_file_name);

            result.push(File::Generated {
                path: PathBuf::from(output_path),
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
