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

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefFlags, DefId, DefKind, PrimitiveTy, Ty, TyKind};

use crate::group::{
    collect_struct_members, collect_type_dependencies, group_types_by_scc, is_proto_type,
    resolve_typedef,
};

type Path = Vec<String>;

pub struct ProtoGen<'a> {
    hir: &'a ResolvedGraph,
}

impl<'a> ProtoGen<'a> {
    pub fn new(hir: &'a ResolvedGraph) -> Self {
        Self { hir }
    }

    fn package_path(&self, def_id: DefId) -> Path {
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

    fn is_proto_type(&self, def_id: DefId) -> bool {
        is_proto_type(self.hir, def_id)
    }

    fn resolve_typedef(&self, def_id: DefId) -> DefId {
        resolve_typedef(self.hir, def_id)
    }

    fn contains_sequence(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Array { .. } | TyKind::Sequence { .. } => true,
            TyKind::Adt(def_id) => {
                let resolved_id = self.resolve_typedef(*def_id);
                let resolved_def = self.hir.context.definitions.get(resolved_id);
                if let DefKind::Alias(alias_ty) = &resolved_def.kind {
                    self.contains_sequence(&alias_ty.ty)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn contains_map(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Map { .. } => true,
            TyKind::Adt(def_id) => {
                let resolved_id = self.resolve_typedef(*def_id);
                let resolved_def = self.hir.context.definitions.get(resolved_id);
                if let DefKind::Alias(alias_ty) = &resolved_def.kind {
                    self.contains_map(&alias_ty.ty)
                } else {
                    false
                }
            }
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
                } else if self.contains_sequence(elem_ty) || self.contains_map(elem_ty) {
                    // Nested sequence or map - need a wrapper message
                    let inner_type = self.proto_type(elem_ty, current_package, wrappers);
                    let wrapper_content = format!("{inner_type} value");

                    if let Some(existing_name) = wrappers.get(&wrapper_content) {
                        format!("repeated {existing_name}")
                    } else {
                        let wrapper_name = format!("Seq{}", wrappers.len());
                        wrappers.insert(wrapper_content, wrapper_name.clone());
                        format!("repeated {wrapper_name}")
                    }
                } else {
                    format!(
                        "repeated {}",
                        self.proto_type(elem_ty, current_package, wrappers)
                    )
                }
            }
            TyKind::Map { key, elem, .. } => {
                let key_type = self.proto_type(key, current_package, wrappers);

                // If the value is a map, we need to wrap it in a message
                if self.contains_map(elem) {
                    let inner_type = self.proto_type(elem, current_package, wrappers);
                    let wrapper_content = format!("{inner_type} value");

                    if let Some(existing_name) = wrappers.get(&wrapper_content) {
                        format!("map<{key_type}, {existing_name}>")
                    } else {
                        let wrapper_name = format!("MapValue{}", wrappers.len());
                        wrappers.insert(wrapper_content, wrapper_name.clone());
                        format!("map<{key_type}, {wrapper_name}>")
                    }
                } else {
                    let value_type = self.proto_type(elem, current_package, wrappers);
                    format!("map<{key_type}, {value_type}>")
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

    fn emit_message(&self, def_id: DefId) -> String {
        let mut w = Twine::new();
        let name = self.proto_name(def_id);
        let members = collect_struct_members(self.hir, def_id);
        let current_package = self.package_path(def_id);
        let mut wrappers = BTreeMap::new();

        w!(w, "message ", name, " ", "{\n");

        let mut field_id = 0;
        for (member_name, member_ty) in members {
            field_id += 1;
            let ty_str = self.proto_type(&member_ty, &current_package, &mut wrappers);
            w!(w, ty_str, " ", member_name, " = ", field_id, ";\n");
        }

        for (content, name) in wrappers {
            w!(w, "message ", name, " {\n", content, " = 1;\n}\n");
        }

        w!(w, "}\n");
        w.finish()
    }

    fn emit_enum(&self, def_id: DefId) -> String {
        let mut w = Twine::new();
        let def = self.hir.context.definitions.get(def_id);
        let name = self.proto_name(def_id);

        w!(w, "enum ", name, " ", "{\n");

        if let DefKind::Enum(enum_ty) = &def.kind {
            // Check if enum has a zero value; proto3 requires first value to be 0
            let zero_field = enum_ty.fields.iter().find(|&&field_id| {
                let field_def = self.hir.context.definitions.get(field_id);
                if let DefKind::Const(const_ty) = &field_def.kind {
                    self.hir.context.integer_value(&const_ty.value) == Some(0)
                } else {
                    false
                }
            });

            if let Some(&zero_id) = zero_field {
                let field_def = self.hir.context.definitions.get(zero_id);
                w!(w, field_def.ident.name, " = 0;\n");
            } else {
                w!(w, name, "_UNKNOWN = 0;\n");
            }

            for field_id in &enum_ty.fields {
                if Some(field_id) == zero_field {
                    continue;
                }

                let field_def = self.hir.context.definitions.get(*field_id);
                if let DefKind::Const(const_ty) = &field_def.kind {
                    let value = self.hir.context.integer_value(&const_ty.value).unwrap_or(0);
                    let field_name = &field_def.ident.name;
                    w!(w, field_name, " = ", value, ";\n");
                }
            }
        }

        w!(w, "}\n");
        w.finish()
    }

    fn emit_union(&self, def_id: DefId) -> String {
        let mut w = Twine::new();
        let def = self.hir.context.definitions.get(def_id);
        let name = self.proto_name(def_id);
        let current_package = self.package_path(def_id);
        let mut wrappers = BTreeMap::new();
        let mut variant_wrappers = vec![];

        w!(w, "message ", name, " {\n");
        w!(w, "oneof inner {\n");

        if let DefKind::Union(union_ty) = &def.kind {
            let mut field_id = 0;
            for variant in &union_ty.variants {
                if let TyKind::Null = variant.ty.kind {
                    continue;
                }

                field_id += 1;
                let variant_name = &variant.ident.name;

                // Check if this is a sequence/array/map type - if so, wrap it
                if self.contains_sequence(&variant.ty) || self.contains_map(&variant.ty) {
                    let wrapper_name = format!("{variant_name}_wrapper");
                    let inner_type = self.proto_type(&variant.ty, &current_package, &mut wrappers);
                    let wrapper_def =
                        format!("message {wrapper_name} {{\n{inner_type} value = 1;\n}}\n");
                    variant_wrappers.push(wrapper_def);
                    w!(w, wrapper_name, " ", variant_name, " = ", field_id, ";\n");
                } else {
                    let ty_str = self.proto_type(&variant.ty, &current_package, &mut wrappers);
                    w!(w, ty_str, " ", variant_name, " = ", field_id, ";\n");
                }
            }
        }

        w!(w, "}\n");

        for wrapper_def in variant_wrappers {
            w!(w, wrapper_def);
        }

        for (content, name) in wrappers {
            w!(w, "message ", name, " {\n", content, " = 1;\n}\n");
        }

        w!(w, "}\n");
        w.finish()
    }

    fn emit_definition(&self, def_id: DefId) -> String {
        let def = self.hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Struct(_) | DefKind::Except(_) | DefKind::Valuetype(_) => {
                self.emit_message(def_id)
            }
            DefKind::Union(_) => self.emit_union(def_id),
            DefKind::Enum(_) => self.emit_enum(def_id),
            _ => String::new(),
        }
    }

    fn emit_prelude(&self, def_id: DefId, def_to_file: &HashMap<DefId, String>) -> String {
        let mut w = Twine::new();
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

    fn collect_types(&self, def_id: DefId, types: &mut Vec<DefId>) {
        let def = self.hir.context.definitions.get(def_id);
        if def.flags.contains(DefFlags::IS_BUILTIN) {
            return;
        }

        if self.is_proto_type(def_id) {
            types.push(def_id);
        }

        if let DefKind::Module(module_ty) = &def.kind {
            for &nested_id in &module_ty.definitions {
                self.collect_types(nested_id, types);
            }
        }
    }

    fn emit_prelude_multi(
        &self,
        def_ids: &[DefId],
        def_to_file: &HashMap<DefId, String>,
    ) -> String {
        let mut w = Twine::new();
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
        for &def_id in def_ids {
            out.push_str(&self.emit_definition(def_id));
        }
        out
    }

    pub fn generate(&self) -> Vec<File> {
        let mut types = vec![];
        for &def_id in &self.hir.order {
            self.collect_types(def_id, &mut types);
        }

        let groups = group_types_by_scc(self.hir, &types);

        let mut def_to_file: HashMap<DefId, String> = HashMap::new();
        let mut file_specs = vec![];

        for group in groups {
            let scc = group.types;
            if scc.is_empty() {
                continue;
            }

            let file_name = if scc.len() == 1 {
                self.file_name(scc[0])
            } else {
                let path = self.package_path(scc[0]);
                let mut names: Vec<_> = scc.iter().map(|&id| self.proto_name(id)).collect();
                names.sort_unstable();
                let combined_name = names.join("_");
                let mut file = PathBuf::from_iter(path);
                file.push(combined_name);
                file.set_extension("proto");
                file.to_string_lossy().replace('\\', "/")
            };

            for &def_id in &scc {
                def_to_file.insert(def_id, file_name.clone());
            }

            file_specs.push((scc, file_name));
        }

        let mut files = vec![];
        for (scc, file_name) in file_specs {
            let content = self.emit_file_multi(&scc, &def_to_file);
            files.push(File::Generated {
                path: PathBuf::from(file_name),
                source: content,
            });
        }

        files
    }
}
