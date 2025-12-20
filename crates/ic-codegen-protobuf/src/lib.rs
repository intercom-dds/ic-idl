// Copyright 2024 KONGSBERG
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

use std::collections::HashSet;
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefFlags, DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir_xform::Target;

type Path = Vec<String>;

const PROTO_KEYWORDS: &[&str] = &[
    "syntax",
    "map",
    "int32",
    "import",
    "extensions",
    "int64",
    "weak",
    "reserved",
    "uint32",
    "public",
    "rpc",
    "uint64",
    "package",
    "stream",
    "sint32",
    "option",
    "returns",
    "sint64",
    "inf",
    "to",
    "fixed32",
    "nan",
    "max",
    "fixed64",
    "message",
    "repeated",
    "sfixed32",
    "enum",
    "optional",
    "sfixed64",
    "service",
    "required",
    "bool",
    "extend",
    "string",
    "float",
    "group",
    "bytes",
    "double",
    "oneof",
];

struct ProtoGen<'a> {
    hir: &'a ResolvedGraph,
}

impl<'a> ProtoGen<'a> {
    fn new(hir: &'a ResolvedGraph) -> Self {
        Self { hir }
    }

    fn package_path(&self, def_id: DefId) -> Path {
        let mut path = Vec::new();
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
        let def = self.hir.context.definitions.get(def_id);
        matches!(
            def.kind,
            DefKind::Struct(_)
                | DefKind::Except(_)
                | DefKind::Union(_)
                | DefKind::Enum(_)
                | DefKind::Valuetype(_)
        )
    }

    fn resolve_typedef(&self, def_id: DefId) -> DefId {
        self.hir.context.base_id_of(def_id)
    }

    fn proto_type(&self, ty: &Ty, current_package: &Path) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => proto_primitive(*prim),
            TyKind::String { .. } => "string".to_string(),
            TyKind::Array { ty: elem_ty, .. } | TyKind::Sequence { ty: elem_ty, .. } => {
                if matches!(
                    elem_ty.kind,
                    TyKind::Primitive(PrimitiveTy::UInt8 | PrimitiveTy::Int8)
                ) {
                    "bytes".to_string()
                } else {
                    format!("repeated {}", self.proto_type(elem_ty, current_package))
                }
            }
            TyKind::Map { key, elem, .. } => {
                let key_type = self.proto_type(key, current_package);
                let value_type = self.proto_type(elem, current_package);
                format!("map<{key_type}, {value_type}>")
            }
            TyKind::Adt(def_id) => {
                let resolved_id = self.resolve_typedef(*def_id);
                let resolved_def = self.hir.context.definitions.get(resolved_id);

                match &resolved_def.kind {
                    DefKind::Alias(alias_ty) => self.proto_type(&alias_ty.ty, current_package),
                    DefKind::Bitmask(bitmask_ty) => proto_primitive(bitmask_ty.ty),
                    _ => self.scoped_name(resolved_id, current_package),
                }
            }
            _ => "bytes".to_string(),
        }
    }

    fn collect_struct_members(&self, def_id: DefId) -> Vec<(String, Ty)> {
        let def = self.hir.context.definitions.get(def_id);
        let mut members = Vec::new();

        match &def.kind {
            DefKind::Struct(struct_ty) => {
                if let Some(parent_id) = struct_ty.parent {
                    members.extend(self.collect_struct_members(parent_id));
                }

                for member in &struct_ty.members {
                    members.push((member.ident.name.clone(), member.ty.clone()));
                }
            }
            DefKind::Except(except_ty) => {
                for member in &except_ty.members {
                    members.push((member.ident.name.clone(), member.ty.clone()));
                }
            }
            DefKind::Valuetype(valuetype_ty) => {
                if let Some(parent_id) = valuetype_ty.parent {
                    members.extend(self.collect_struct_members(parent_id));
                }

                for member in &valuetype_ty.members {
                    members.push((member.ident.name.clone(), member.ty.clone()));
                }
            }
            DefKind::Union(union_ty) => {
                for variant in &union_ty.variants {
                    members.push((variant.ident.name.clone(), variant.ty.clone()));
                }
            }
            _ => {}
        }

        members
    }

    fn collect_dependencies(&self, def_id: DefId) -> HashSet<DefId> {
        let mut all_deps = HashSet::new();
        let mut current = Some(def_id);

        while let Some(id) = current {
            let def = self.hir.context.definitions.get(id);

            for dep_id in self.hir.context.deps(id) {
                let resolved_id = self.resolve_typedef(dep_id);
                if self.is_proto_type(resolved_id) {
                    all_deps.insert(resolved_id);
                }
            }

            current = match &def.kind {
                DefKind::Struct(s) => s.parent,
                DefKind::Valuetype(v) => v.parent,
                _ => None,
            };
        }

        all_deps.remove(&def_id);
        all_deps
    }

    fn emit_message(&self, def_id: DefId) -> String {
        let mut w = Twine::new();
        let name = self.proto_name(def_id);
        let members = self.collect_struct_members(def_id);
        let current_package = self.package_path(def_id);

        w!(w, "message ", name, " ", "{\n");

        let mut field_id = 0;
        for (member_name, member_ty) in members {
            field_id += 1;
            let ty_str = self.proto_type(&member_ty, &current_package);
            w!(w, ty_str, " ", member_name, " = ", field_id, ";\n");
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
            for field_id in &enum_ty.fields {
                let field_def = self.hir.context.definitions.get(*field_id);
                if let DefKind::Const(const_ty) = &field_def.kind {
                    let value = self
                        .hir
                        .context
                        .unsigned_value(&const_ty.value)
                        .unwrap_or(0);

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

        w!(w, "message ", name, " {\n");
        w!(w, "oneof inner {\n");

        if let DefKind::Union(union_ty) = &def.kind {
            let mut field_id = 0;
            for variant in &union_ty.variants {
                if let TyKind::Null = variant.ty.kind {
                    continue;
                }

                field_id += 1;
                let ty_str = self.proto_type(&variant.ty, &current_package);
                let variant_name = &variant.ident.name;
                w!(w, ty_str, " ", variant_name, " = ", field_id, ";\n");
            }
        }

        w!(w, "}\n");
        w!(w, "}\n");
        w.finish()
    }

    fn emit_definition(&self, def_id: DefId) -> String {
        let def = self.hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Struct(_) | DefKind::Except(_) => self.emit_message(def_id),
            DefKind::Union(_) => self.emit_union(def_id),
            DefKind::Enum(_) => self.emit_enum(def_id),
            _ => String::new(),
        }
    }

    fn emit_prelude(&self, def_id: DefId) -> String {
        let mut w = Twine::new();
        w!(w, "syntax = \"proto3\";\n");
        w.blank();

        let pkg_name = self.package_name(def_id);
        if !pkg_name.is_empty() {
            w!(w, "package ", pkg_name, ";\n");
            w.blank();
        }

        let mut imports: Vec<_> = self
            .collect_dependencies(def_id)
            .into_iter()
            .map(|dep| self.file_name(dep))
            .collect();

        imports.sort();

        if !imports.is_empty() {
            for import in imports {
                w!(w, "import \"", import, "\";\n");
            }
            w.blank();
        }

        w.finish()
    }

    fn emit_file(&self, def_id: DefId) -> String {
        let mut out = String::new();
        out.push_str(&self.emit_prelude(def_id));
        out.push_str(&self.emit_definition(def_id));
        out
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

    fn generate(&mut self) -> Vec<File> {
        let mut types = Vec::new();

        for &def_id in &self.hir.order {
            self.collect_types(def_id, &mut types);
        }

        let mut files = Vec::new();
        for def_id in types {
            let file_name = self.file_name(def_id);
            let content = self.emit_file(def_id);
            files.push(File::Generated {
                path: PathBuf::from(file_name),
                source: content,
            });
        }

        files
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

#[must_use]
pub fn codegen_proto(hir: &ic_hir::ResolvedGraph) -> Vec<File> {
    // Move nested types into modules
    let (hir, moved_defs) = ic_hir_xform::move_nested::transform(hir.clone());

    // Escape keywords
    let target = Target {
        keywords: PROTO_KEYWORDS.iter().copied().collect(),
        moved_defs,
        ..Target::default()
    };
    let hir = ic_hir_xform::rename::transform(hir, &target);

    let mut generator = ProtoGen::new(&hir);
    generator.generate()
}
