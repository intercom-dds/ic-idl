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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use ic_cli::color::Colorize;
use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefFlags, DefId, DefKind, EnumTy, PrimitiveTy, Ty, TyKind, UnionTy};

use crate::group::{
    collect_struct_members, collect_type_dependencies, group_types_by_scc, is_proto_type,
    resolve_typedef,
};
use crate::options::ProtoOptions;

type Path = Vec<String>;

pub struct ProtoGen<'a> {
    hir: &'a ResolvedGraph,
    _options: ProtoOptions,
    scc_map: HashMap<DefId, DefId>,
}

fn collect_all_proto_types(hir: &ResolvedGraph) -> Vec<DefId> {
    let mut types = vec![];
    for &def_id in &hir.order {
        collect_types_recursive(hir, def_id, &mut types);
    }
    types
}

fn collect_types_recursive(hir: &ResolvedGraph, def_id: DefId, types: &mut Vec<DefId>) {
    let def = hir.context.definitions.get(def_id);
    if def.flags.contains(DefFlags::IS_BUILTIN) {
        return;
    }

    if is_proto_type(hir, def_id) {
        types.push(def_id);
    }

    if let DefKind::Module(module_ty) = &def.kind {
        for &nested_id in &module_ty.definitions {
            collect_types_recursive(hir, nested_id, types);
        }
    }
}

fn get_containing_module(hir: &ResolvedGraph, def_id: DefId) -> Option<DefId> {
    let mut current = def_id;
    while let Some(parent_id) = hir.context.definitions.get(current).parent {
        let parent_def = hir.context.definitions.get(parent_id);
        if matches!(parent_def.kind, DefKind::Module(_)) {
            return Some(parent_id);
        }
        current = parent_id;
    }
    None
}

impl<'a> ProtoGen<'a> {
    #[allow(clippy::print_stderr)]
    pub fn new(hir: &'a ResolvedGraph, options: ProtoOptions) -> Self {
        let types = collect_all_proto_types(hir);
        let type_order: HashMap<DefId, usize> =
            types.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        let groups = group_types_by_scc(hir, &types);

        let mut scc_map = HashMap::new();

        for group in groups {
            let mut scc = group.types;
            if scc.is_empty() {
                continue;
            }

            scc.sort_by_key(|id| type_order.get(id).unwrap_or(&usize::MAX));
            let leader = scc[0];
            let leader_mod = get_containing_module(hir, leader);

            let mut foreign_types = Vec::new();

            for &id in &scc {
                let type_mod = get_containing_module(hir, id);
                if type_mod != leader_mod {
                    foreign_types.push(id);
                    continue;
                }
                scc_map.insert(id, leader);
            }

            if !foreign_types.is_empty() {
                let leader_name = &hir.context.definitions.get(leader).ident.name;
                let foreign_names: Vec<&str> = foreign_types
                    .iter()
                    .map(|&id| hir.context.definitions.get(id).ident.name.as_str())
                    .collect();

                eprintln!(
                    "{}: Type(s) {} are part of an SCC with '{}' but belong to different modules. \
                     Inter-module recursion is not supported in Protobuf generation.",
                    "warning".yellow().bold(),
                    foreign_names.join(", "),
                    leader_name
                );
            }
        }

        Self {
            hir,
            _options: options,
            scc_map,
        }
    }

    fn package_path(&self, def_id: DefId) -> Path {
        let effective_id = self.scc_map.get(&def_id).copied().unwrap_or(def_id);

        let mut path = vec![];
        let mut current = effective_id;

        while let Some(parent_id) = self.hir.context.definitions.get(current).parent {
            let parent_def = self.hir.context.definitions.get(parent_id);
            if matches!(parent_def.kind, DefKind::Module(_)) {
                path.push(parent_def.ident.name.clone());
            }
            current = parent_id;
        }

        path.reverse();
        path
    }

    fn package_name(&self, def_id: DefId) -> String {
        let path = self.package_path(def_id);
        path.join(".")
    }

    fn file_name(&self, def_id: DefId) -> String {
        let path = self.package_path(def_id);
        let name = self.proto_name(def_id);
        let mut file = PathBuf::from_iter(path);
        file.push(name);
        file.set_extension("proto");
        file.to_string_lossy().replace('\\', "/")
    }

    fn proto_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    fn scoped_name(&self, def_id: DefId, current_package: &Path) -> String {
        let target_path = self.package_path(def_id);
        let proto_name = self.proto_name(def_id);

        if &target_path == current_package {
            return proto_name.to_string();
        }

        if target_path.len() > current_package.len()
            && target_path[..current_package.len()] == *current_package
        {
            let relative_path = &target_path[current_package.len()..];
            let pkg_name = relative_path.join(".");
            format!("{pkg_name}.{proto_name}")
        } else {
            let pkg_name = target_path.join(".");
            if pkg_name.is_empty() {
                proto_name.to_string()
            } else {
                format!("{pkg_name}.{proto_name}")
            }
        }
    }

    fn resolve_typedef(&self, def_id: DefId) -> DefId {
        resolve_typedef(self.hir, def_id)
    }

    fn is_message_type(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Adt(def_id) => {
                let resolved_id = self.resolve_typedef(*def_id);
                let def = self.hir.context.definitions.get(resolved_id);
                match &def.kind {
                    DefKind::Struct(_)
                    | DefKind::Union(_)
                    | DefKind::Except(_)
                    | DefKind::Valuetype(_) => true,
                    DefKind::Alias(alias) => self.is_message_type(&alias.ty),
                    _ => false,
                }
            }
            // Wrapped repeated fields and maps are messages
            TyKind::Array { ty: elem_ty, .. } | TyKind::Sequence { ty: elem_ty, .. } => {
                // octet/int8 arrays map to 'bytes' scalar, not a message
                if matches!(
                    elem_ty.kind,
                    TyKind::Primitive(PrimitiveTy::UInt8 | PrimitiveTy::Int8)
                ) {
                    return false;
                }

                true
            }
            TyKind::Map { .. } => true,
            _ => false,
        }
    }

    fn proto_primitive(prim: PrimitiveTy) -> String {
        match prim {
            PrimitiveTy::Bool => "bool".to_string(),
            PrimitiveTy::Int8 | PrimitiveTy::Int16 | PrimitiveTy::Int32 => "int32".to_string(),
            PrimitiveTy::Char
            | PrimitiveTy::WChar
            | PrimitiveTy::UInt8
            | PrimitiveTy::UInt16
            | PrimitiveTy::UInt32 => "uint32".to_string(),
            PrimitiveTy::Int64 => "int64".to_string(),
            PrimitiveTy::UInt64 => "uint64".to_string(),
            PrimitiveTy::Float32 => "float".to_string(),
            PrimitiveTy::Float64 | PrimitiveTy::Float128 => "double".to_string(),
            PrimitiveTy::Void => "google.protobuf.Empty".to_string(),
        }
    }

    fn sanitize_proto_name(name: &str) -> String {
        name.replace('.', "_")
            .replace("::", "_")
            .replace(['<', '>', ',', ' '], "_")
    }

    fn proto_type(
        &self,
        ty: &Ty,
        current_package: &Path,
        wrappers: &mut BTreeMap<String, String>,
    ) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => Self::proto_primitive(*prim),
            TyKind::String { .. } => "string".to_string(),
            TyKind::Array { ty: elem_ty, .. } | TyKind::Sequence { ty: elem_ty, .. } => {
                if matches!(
                    elem_ty.kind,
                    TyKind::Primitive(PrimitiveTy::UInt8 | PrimitiveTy::Int8)
                ) {
                    "bytes".to_string()
                } else {
                    // Wrapped sequence or nested sequence or map - need a wrapper message
                    let inner_type = self.proto_type(elem_ty, current_package, wrappers);
                    let wrapper_content = format!("repeated {inner_type} value");

                    // Return the wrapper name directly (singular message field)
                    if let Some(existing_name) = wrappers.get(&wrapper_content) {
                        existing_name.clone()
                    } else {
                        let clean_inner = inner_type.trim_start_matches("repeated ").trim();
                        let safe_inner_type = Self::sanitize_proto_name(clean_inner);
                        let name = format!("Seq_{safe_inner_type}_");
                        wrappers.insert(wrapper_content, name.clone());
                        name
                    }
                }
            }
            TyKind::Map { key, elem, .. } => {
                let key_type = self.proto_type(key, current_package, wrappers);
                let value_type = self.proto_type(elem, current_package, wrappers);

                let wrapper_content = format!("map<{key_type}, {value_type}> value");

                if let Some(existing_name) = wrappers.get(&wrapper_content) {
                    existing_name.clone()
                } else {
                    let safe_key = Self::sanitize_proto_name(&key_type);
                    let safe_value = Self::sanitize_proto_name(&value_type);
                    let name = format!("Map_{safe_key}_{safe_value}_");
                    wrappers.insert(wrapper_content, name.clone());
                    name
                }
            }
            TyKind::Adt(def_id) => {
                let resolved_id = self.resolve_typedef(*def_id);
                let resolved_def = self.hir.context.definitions.get(resolved_id);

                match &resolved_def.kind {
                    DefKind::Alias(alias_ty) => {
                        self.proto_type(&alias_ty.ty, current_package, wrappers)
                    }
                    DefKind::Bitmask(bitmask_ty) => Self::proto_primitive(bitmask_ty.ty),
                    _ => self.scoped_name(resolved_id, current_package),
                }
            }
            _ => "bytes".to_string(),
        }
    }

    fn emit_message(&self, def: &Def) -> String {
        let mut w = Twine::new();
        let members = collect_struct_members(self.hir, def.id);
        let current_package = self.package_path(def.id);
        let mut wrappers = BTreeMap::new();

        w!(w, "message ", def, " ", "{\n");

        let mut field_id = 0;
        for member in members {
            field_id += 1;
            let ty_str = self.proto_type(&member.ty, &current_package, &mut wrappers);

            let is_message_type = self.is_message_type(&member.ty);
            let label_optional =
                !is_message_type && !ty_str.starts_with("repeated") && !ty_str.starts_with("map");

            if label_optional {
                w!(
                    w,
                    "optional ",
                    ty_str,
                    " ",
                    member.name,
                    " = ",
                    field_id,
                    ";\n"
                );
            } else {
                w!(w, ty_str, " ", member.name, " = ", field_id, ";\n");
            }
        }

        for (content, name) in wrappers {
            w.blank();
            w!(w, "message ", name, " {\n", content, " = 1;\n}\n");
        }

        w!(w, "}\n");
        w.finish()
    }

    fn emit_enum(&self, def: &Def, enum_ty: &EnumTy) -> String {
        let mut w = Twine::new();
        w!(w, "enum ", def, " ", "{\n");

        // Check if enum has a zero value; proto3 requires first value to be 0
        let zero_field = enum_ty.fields.iter().find(|&&field_id| {
            let field_def = self.hir.context.definitions.get(field_id);
            if let DefKind::Const(const_ty) = &field_def.kind {
                self.hir.context.integer_value(&const_ty.value) == 0
            } else {
                false
            }
        });

        if let Some(&zero_id) = zero_field {
            let field_def = self.hir.context.definitions.get(zero_id);
            w!(w, field_def.ident.name, " = 0;\n");
        } else {
            w!(w, def, "_UNKNOWN = 0;\n");
        }

        for field_id in &enum_ty.fields {
            if Some(field_id) == zero_field {
                continue;
            }

            let field_def = self.hir.context.definitions.get(*field_id);
            if let DefKind::Const(const_ty) = &field_def.kind {
                let value = self.hir.context.integer_value(&const_ty.value);
                let field_name = &field_def.ident.name;
                w!(w, field_name, " = ", value, ";\n");
            }
        }

        w!(w, "}\n");
        w.finish()
    }

    fn emit_union(&self, def: &Def, union_ty: &UnionTy) -> String {
        let mut w = Twine::new();
        let current_package = self.package_path(def.id);
        let mut wrappers = BTreeMap::new();
        w!(w, "message ", def, " {\n");
        w!(w, "oneof inner {\n");

        let mut field_id = 0;
        for variant in &union_ty.variants {
            if let TyKind::Null = variant.ty.kind {
                continue;
            }

            field_id += 1;
            let variant_name = &variant.ident.name;

            let ty_str = self.proto_type(&variant.ty, &current_package, &mut wrappers);
            w!(w, ty_str, " ", variant_name, " = ", field_id, ";\n");
        }

        w!(w, "}\n");

        for (content, name) in wrappers {
            w.blank();
            w!(w, "message ", name, " {\n", content, " = 1;\n}\n");
        }

        w!(w, "}\n");
        w.finish()
    }

    fn emit_definition(&self, def_id: DefId) -> String {
        let def = self.hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Struct(_) | DefKind::Except(_) | DefKind::Valuetype(_) => {
                self.emit_message(def)
            }
            DefKind::Union(union_ty) => self.emit_union(def, union_ty),
            DefKind::Enum(enum_ty) => self.emit_enum(def, enum_ty),
            _ => String::new(),
        }
    }

    fn emit_prelude(&self, def_id: DefId, def_to_file: &HashMap<DefId, String>) -> String {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");

        let mut w = Twine::new();
        w!(w, "// @generated by ic-idl ", IC_VERSION, "\n\n");
        w!(w, "syntax = \"proto3\";\n");
        w.blank();

        let pkg_name = self.package_name(def_id);
        if !pkg_name.is_empty() {
            w!(w, "package ", pkg_name, ";\n");
            w.blank();
        }

        let mut imports: Vec<_> = collect_type_dependencies(self.hir, def_id)
            .into_iter()
            .filter_map(|dep| def_to_file.get(&dep).cloned())
            .collect();

        imports.sort();
        imports.dedup();

        if !imports.is_empty() {
            for import in imports {
                w!(w, "import \"", import, "\";\n");
            }
            w.blank();
        }

        w.finish()
    }

    fn emit_prelude_multi(
        &self,
        def_ids: &[DefId],
        def_to_file: &HashMap<DefId, String>,
    ) -> String {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");

        let mut w = Twine::new();
        w!(w, "// @generated by ic-idl ", IC_VERSION, "\n\n");
        w!(w, "syntax = \"proto3\";\n");
        w.blank();

        let pkg_name = self.package_name(def_ids[0]);
        if !pkg_name.is_empty() {
            w!(w, "package ", pkg_name, ";\n");
            w.blank();
        }

        let ids_in_file: HashSet<DefId> = def_ids.iter().copied().collect();
        let mut all_deps = HashSet::new();

        for &def_id in def_ids {
            for dep in collect_type_dependencies(self.hir, def_id) {
                if !ids_in_file.contains(&dep) {
                    all_deps.insert(dep);
                }
            }
        }

        let mut imports: Vec<_> = all_deps
            .into_iter()
            .filter_map(|dep| def_to_file.get(&dep).cloned())
            .collect();

        imports.sort();
        imports.dedup();

        if !imports.is_empty() {
            for import in imports {
                w!(w, "import \"", import, "\";\n");
            }
            w.blank();
        }

        w.finish()
    }

    fn emit_file_multi(&self, def_ids: &[DefId], def_to_file: &HashMap<DefId, String>) -> String {
        let mut out = String::new();
        if def_ids.len() == 1 {
            out.push_str(&self.emit_prelude(def_ids[0], def_to_file));
        } else {
            out.push_str(&self.emit_prelude_multi(def_ids, def_to_file));
        }
        for (i, &def_id) in def_ids.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&self.emit_definition(def_id));
        }
        out
    }

    fn emit_facade(&self, def_id: DefId, target_file: &str) -> String {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");

        let mut w = Twine::new();
        w!(w, "// @generated by ic-idl ", IC_VERSION, "\n\n");
        w!(w, "syntax = \"proto3\";\n");
        w.blank();

        let mut path = vec![];
        let mut current = def_id;

        while let Some(parent_id) = self.hir.context.definitions.get(current).parent {
            let parent_def = self.hir.context.definitions.get(parent_id);
            if matches!(parent_def.kind, DefKind::Module(_)) {
                path.push(parent_def.ident.name.clone());
            }
            current = parent_id;
        }
        path.reverse();
        let pkg_name = path.join(".");

        if !pkg_name.is_empty() {
            w!(w, "package ", pkg_name, ";\n");
            w.blank();
        }

        w!(w, "import public \"", target_file, "\";\n");
        w.finish()
    }

    pub fn generate(&self) -> Vec<File> {
        enum FileKind {
            Definition(Vec<DefId>),
            Facade(DefId, String),
        }

        let types = collect_all_proto_types(self.hir);

        let mut groups: BTreeMap<DefId, Vec<DefId>> = BTreeMap::new();
        for def_id in types {
            let leader = self.scc_map.get(&def_id).copied().unwrap_or(def_id);
            groups.entry(leader).or_default().push(def_id);
        }

        let mut def_to_file: HashMap<DefId, String> = HashMap::new();
        let mut pending_files: Vec<(String, FileKind)> = vec![];

        for (leader_def, mut scc) in groups {
            let type_order: HashMap<DefId, usize> = self
                .hir
                .order
                .iter()
                .enumerate()
                .map(|(i, &id)| (id, i))
                .collect();
            scc.sort_by_key(|id| type_order.get(id).unwrap_or(&usize::MAX));

            let leader_file_name = self.file_name(leader_def);

            for &def_id in &scc {
                def_to_file.insert(def_id, leader_file_name.clone());
            }

            pending_files.push((leader_file_name.clone(), FileKind::Definition(scc.clone())));

            for &def_id in &scc {
                if def_id == leader_def {
                    continue;
                }

                let mut path = vec![];
                let mut current = def_id;
                while let Some(parent_id) = self.hir.context.definitions.get(current).parent {
                    let parent_def = self.hir.context.definitions.get(parent_id);
                    if matches!(parent_def.kind, DefKind::Module(_)) {
                        path.push(parent_def.ident.name.clone());
                    }
                    current = parent_id;
                }
                path.reverse();

                let name = self.proto_name(def_id);
                let mut file = PathBuf::from_iter(path);
                file.push(name);
                file.set_extension("proto");
                let facade_file_name = file.to_string_lossy().replace('\\', "/");

                pending_files.push((
                    facade_file_name,
                    FileKind::Facade(def_id, leader_file_name.clone()),
                ));
            }
        }

        let mut files = vec![];
        for (file_name, kind) in pending_files {
            let content = match kind {
                FileKind::Definition(ids) => self.emit_file_multi(&ids, &def_to_file),
                FileKind::Facade(def_id, target) => self.emit_facade(def_id, &target),
            };
            files.push(File::Generated {
                path: PathBuf::from(file_name),
                source: content,
            });
        }

        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_primitive() {
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Bool), "bool");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Int8), "int32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Int16), "int32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Int32), "int32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Int64), "int64");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Char), "uint32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::WChar), "uint32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::UInt8), "uint32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::UInt16), "uint32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::UInt32), "uint32");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::UInt64), "uint64");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Float32), "float");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Float64), "double");
        assert_eq!(ProtoGen::proto_primitive(PrimitiveTy::Float128), "double");
        assert_eq!(
            ProtoGen::proto_primitive(PrimitiveTy::Void),
            "google.protobuf.Empty"
        );
    }
}
