// Copyright 2026 KONGSBERG
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

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use ic_emit::File;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    AliasTy, Ann, BitmaskTy, ConstTy, Def, DefFlags, DefId, DefKind, EnumTy, ExceptTy, InterfaceTy,
    Numeric, ParamKind, ProtoTy, StructTy, TyKind, UnionTy, ValueTy,
};
use ic_vfs::{FileId, SourceMap};

use crate::types::{default_value, py_type};
use crate::writer::PyWriter;
use crate::{PythonOptions, py};

type FileKey = (Vec<String>, String);

#[derive(Default)]
struct Stdlib {
    dataclasses: bool,
    enum_: bool,
    typing: bool,
    abc: bool,
    decimal: bool,
}

#[derive(Default)]
struct Imports {
    stdlib: Stdlib,
    types: BTreeMap<String, BTreeSet<String>>,
}

impl Imports {
    fn emit(&self, w: &mut PyWriter) {
        let mut has_imports = false;

        if self.stdlib.abc {
            py!(w, "import abc as _abc_\n");
            has_imports = true;
        }

        if self.stdlib.dataclasses {
            py!(w, "import dataclasses as _dataclasses_\n");
            has_imports = true;
        }

        if self.stdlib.decimal {
            py!(w, "import decimal as _decimal_\n");
            has_imports = true;
        }

        if self.stdlib.enum_ {
            py!(w, "import enum as _enum_\n");
            has_imports = true;
        }

        if self.stdlib.typing {
            py!(w, "import typing as _typing_\n");
            has_imports = true;
        }

        for (module, names) in &self.types {
            let names_str = names.iter().cloned().collect::<Vec<_>>().join(", ");
            py!(w, "from ", module, " import ", names_str, "\n");
            has_imports = true;
        }

        if has_imports {
            py!(w, "\n");
        }
    }
}

pub struct PyGen<'a> {
    hir: &'a ResolvedGraph,
    source_map: &'a SourceMap,
    _options: PythonOptions,
}

impl<'a> PyGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, source_map: &'a SourceMap, options: PythonOptions) -> Self {
        Self {
            hir,
            source_map,
            _options: options,
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut files = vec![];
        let groups = self.group_definitions_by_file();

        for ((module_path, filename), defs) in &groups {
            files.push(self.emit_file(module_path, filename, defs));
        }

        let init_files = self.generate_init_files(&groups);
        files.extend(init_files);
        files
    }

    fn generate_init_files(&self, groups: &BTreeMap<FileKey, Vec<DefId>>) -> Vec<File> {
        use std::collections::BTreeSet;

        let mut module_exports: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();

        for ((module_path, filename), defs) in groups {
            module_exports
                .entry(module_path.clone())
                .or_default()
                .insert(filename.clone());

            for &def_id in defs {
                let def = self.hir.context.definitions.get(def_id);
                if !matches!(def.kind, DefKind::Module(_)) {
                    module_exports
                        .entry(module_path.clone())
                        .or_default()
                        .insert(def.ident.name.clone());
                }
            }
        }

        let mut all_paths: BTreeSet<Vec<String>> = BTreeSet::new();
        for path in module_exports.keys() {
            let mut current = vec![];
            all_paths.insert(current.clone());
            for segment in path {
                current.push(segment.clone());
                all_paths.insert(current.clone());
            }
        }

        let mut files = vec![];

        for module_path in all_paths {
            let mut w = PyWriter::new();
            Self::emit_header(&mut w);

            let filenames: BTreeSet<_> = groups
                .iter()
                .filter(|((mp, _), _)| *mp == module_path)
                .map(|((_, f), _)| f.clone())
                .collect();

            for filename in &filenames {
                py!(w, "from ._", filename, " import *\n");
            }

            let child_modules: BTreeSet<&String> = groups
                .keys()
                .filter(|(mp, _)| mp.len() == module_path.len() + 1 && mp.starts_with(&module_path))
                .map(|(mp, _)| &mp[module_path.len()])
                .collect();

            for child in child_modules {
                py!(w, "from . import ", child, "\n");
            }

            let mut path: PathBuf = module_path.iter().collect();
            path.push("__init__.py");

            files.push(File::Generated {
                path,
                source: w.finish(),
            });
        }

        files
    }

    fn emit_definition(&self, w: &mut PyWriter, def_id: DefId) {
        let def = self.hir.context.definitions.get(def_id);

        match &def.kind {
            DefKind::Struct(struct_ty) => self.emit_struct(w, def, struct_ty),
            DefKind::Enum(enum_ty) => self.emit_enum(w, def, enum_ty),
            DefKind::Bitmask(bitmask_ty) => self.emit_bitmask(w, def, bitmask_ty),
            DefKind::Union(union_ty) => self.emit_union(w, def, union_ty),
            DefKind::Alias(alias) => self.emit_alias(w, def, alias),
            DefKind::Except(except_ty) => self.emit_exception(w, def, except_ty),
            DefKind::Interface(interface_ty) => self.emit_interface(w, def, interface_ty),
            DefKind::Valuetype(value_ty) => self.emit_valuetype(w, def, value_ty),
            DefKind::Const(const_ty) => {
                let is_enum_member = matches!(
                    def.parent
                        .map(|p| &self.hir.context.definitions.get(p).kind),
                    Some(DefKind::Enum(_) | DefKind::Bitmask(_))
                );
                if !is_enum_member {
                    self.emit_const(w, def, const_ty);
                }
            }
            DefKind::Module(_) | DefKind::Bitset(_) | DefKind::Annotation(_) | DefKind::Decl(_) => {
            }
        }
    }

    fn source_filename(&self, def_id: DefId) -> Option<String> {
        let def = self.hir.context.definitions.get(def_id);
        let span = def.ident.span;
        let file_id = span.start.file_id;
        if file_id == FileId::_do_not_use() {
            return None;
        }
        let file_path = self.source_map.name(file_id);
        let stem = file_path.file_stem()?.to_str()?;
        Some(stem.to_string())
    }

    fn module_path(&self, def_id: DefId) -> Vec<String> {
        let mut path = vec![];
        let mut current = Some(def_id);

        while let Some(id) = current {
            let def = self.hir.context.definitions.get(id);
            if matches!(def.kind, DefKind::Module(_)) {
                path.push(def.ident.name.clone());
            }
            current = def.parent;
        }

        path.reverse();
        path
    }

    fn group_definitions_by_file(&self) -> BTreeMap<FileKey, Vec<DefId>> {
        let mut groups: BTreeMap<FileKey, Vec<DefId>> = BTreeMap::new();

        for &def_id in &self.hir.order {
            self.collect_definitions(def_id, &mut groups);
        }

        groups
    }

    fn collect_definitions(&self, def_id: DefId, groups: &mut BTreeMap<FileKey, Vec<DefId>>) {
        let def = self.hir.context.definitions.get(def_id);

        match &def.kind {
            DefKind::Module(module_ty) => {
                for &child_id in &module_ty.definitions {
                    self.collect_definitions(child_id, groups);
                }
            }
            _ => {
                self.add_to_groups(def_id, groups);
            }
        }
    }

    fn add_to_groups(&self, def_id: DefId, groups: &mut BTreeMap<FileKey, Vec<DefId>>) {
        let def = self.hir.context.definitions.get(def_id);

        let Some(filename) = self.source_filename(def_id) else {
            return;
        };

        let module_path = if let Some(parent) = def.parent {
            self.module_path(parent)
        } else {
            vec![]
        };

        let key = (module_path, filename);
        groups.entry(key).or_default().push(def_id);
    }

    fn is_optional(&self, annotations: &[Ann]) -> bool {
        annotations.iter().any(|ann| {
            if let Some(def_id) = ann.def_id {
                let def = self.hir.context.type_of(def_id);
                def.flags.contains(DefFlags::IS_BUILTIN) && def.ident.name == "optional"
            } else {
                false
            }
        })
    }

    fn emit_struct(&self, w: &mut PyWriter, def: &Def, struct_ty: &StructTy) {
        py!(w, "@_dataclasses_.dataclass(slots=True, order=True)\n");
        py!(w, "class ", def);
        if let Some(parent) = struct_ty.parent {
            let parent_def = self.hir.context.definitions.get(parent);
            py!(w, parent_def);
        }
        py!(w, ":\n");

        w.indent();

        if struct_ty.members.is_empty() {
            py!(w, "pass\n");
        } else {
            for member in &struct_ty.members {
                let is_optional = self.is_optional(&member.annotations);
                let ty_str = if is_optional {
                    format!("{} | None", py_type(self.hir, &member.ty, def.id))
                } else {
                    py_type(self.hir, &member.ty, def.id)
                };

                let default = if is_optional {
                    "None".to_string()
                } else {
                    default_value(self.hir, &member.ty)
                };

                py!(w, member.ident.name, ": ", ty_str, " = ", default, "\n");
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn format_numeric(value: &Numeric) -> String {
        match value {
            Numeric::Bool(b) => if *b { "True" } else { "False" }.to_string(),
            Numeric::Char(c) => format!("\"{}\"", c.escape_default()),
            Numeric::Int8(v) => v.to_string(),
            Numeric::UInt8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::UInt32(v) => v.to_string(),
            Numeric::Int64(v) => v.to_string(),
            Numeric::UInt64(v) => v.to_string(),
            Numeric::Float(v) => v.to_string(),
            Numeric::Double(v) => v.to_string(),
            Numeric::String(s) => format!("\"{}\"", s.escape_default()),
            _ => "0".to_string(),
        }
    }

    fn emit_enum(&self, w: &mut PyWriter, def: &Def, enum_ty: &EnumTy) {
        py!(w, "class ", def, "(_enum_.Enum):\n");
        w.indent();

        for &member_id in &enum_ty.fields {
            let member_def = self.hir.context.definitions.get(member_id);
            if let DefKind::Const(const_ty) = &member_def.kind {
                let val = Self::format_numeric(&const_ty.value);
                py!(w, member_def, " = ", val, "\n");
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_bitmask(&self, w: &mut PyWriter, def: &Def, bitmask_ty: &BitmaskTy) {
        py!(w, "class ", def, "(_enum_.Flag):\n");
        w.indent();

        for &member_id in &bitmask_ty.flags {
            let member_def = self.hir.context.definitions.get(member_id);
            if let DefKind::Const(const_ty) = &member_def.kind {
                let val = Self::format_numeric(&const_ty.value);
                py!(w, member_def, " = ", val, "\n");
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_union(&self, w: &mut PyWriter, def: &Def, union_ty: &UnionTy) {
        let disc_type = py_type(self.hir, &union_ty.disc.ty, def.id);

        let value_types: Vec<String> = union_ty
            .variants
            .iter()
            .filter(|v| !matches!(v.ty.kind, TyKind::Null))
            .map(|v| py_type(self.hir, &v.ty, def.id))
            .collect();
        let value_union = if value_types.is_empty() {
            "None".to_string()
        } else {
            value_types.join(" | ")
        };

        py!(w, "@_dataclasses_.dataclass(slots=True, order=True)\n");
        py!(w, "class ", def, ":\n");
        w.indent();
        py!(w, "_discriminator: ", disc_type, " = 0\n");
        py!(w, "_value: ", value_union, " | None = None\n");
        w.dedent();
        py!(w, "\n");

        w.indent();
        py!(w, "@property\n");
        py!(w, "def discriminator(self) -> ", disc_type, ":\n");
        w.indent();
        py!(w, "return self._discriminator\n");
        w.dedent();
        w.dedent();
        py!(w, "\n");

        for variant in &union_ty.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                continue;
            }

            let variant_type = py_type(self.hir, &variant.ty, def.id);

            w.indent();
            py!(w, "@property\n");
            py!(w, "def ", variant.ident.name, "(self) -> ", variant_type, ":\n");
            w.indent();

            if variant.is_default {
                py!(w, "return self._value\n");
            } else if variant.labels.len() == 1 {
                let label = Self::format_numeric(&variant.labels[0].value);
                py!(w, "if self._discriminator != ", label, ":\n");
                w.indent();
                py!(w, "raise ValueError(\"", variant.ident.name, " not selected\")\n");
                w.dedent();
                py!(w, "return self._value\n");
            } else {
                let labels: Vec<String> = variant
                    .labels
                    .iter()
                    .map(|l| Self::format_numeric(&l.value))
                    .collect();
                py!(w, "if self._discriminator not in (", labels.join(", "), "):\n");
                w.indent();
                py!(w, "raise ValueError(\"", variant.ident.name, " not selected\")\n");
                w.dedent();
                py!(w, "return self._value\n");
            }
            w.dedent();
            w.dedent();
            py!(w, "\n");

            w.indent();
            py!(w, "@", variant.ident.name, ".setter\n");
            py!(w, "def ", variant.ident.name, "(self, val: ", variant_type, ") -> None:\n");
            w.indent();
            if !variant.labels.is_empty() {
                let first_label = Self::format_numeric(&variant.labels[0].value);
                py!(w, "self._discriminator = ", first_label, "\n");
            }
            py!(w, "self._value = val\n");
            w.dedent();
            w.dedent();
            py!(w, "\n");
        }

        w.indent();
        py!(w, "def default(self) -> None:\n");
        w.indent();
        py!(w, "self._discriminator = 0\n");
        py!(w, "self._value = None\n");
        w.dedent();
        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_alias(&self, w: &mut PyWriter, def: &Def, alias: &AliasTy) {
        let ty_str = py_type(self.hir, &alias.ty, def.id);
        py!(w, def, ": _typing_.TypeAlias = ", ty_str, "\n\n");
    }

    fn emit_const(&self, w: &mut PyWriter, def: &Def, const_ty: &ConstTy) {
        let ty_str = py_type(self.hir, &const_ty.ty, def.id);
        let value_str = Self::format_numeric(&const_ty.value);
        py!(w, def, ": ", ty_str, " = ", value_str, "\n\n");
    }

    fn emit_exception(&self, w: &mut PyWriter, def: &Def, except_ty: &ExceptTy) {
        py!(w, "@_dataclasses_.dataclass(slots=True, order=True)\n");
        py!(w, "class ", def, "(Exception):\n");
        w.indent();

        if except_ty.members.is_empty() {
            py!(w, "pass\n");
        } else {
            for member in &except_ty.members {
                let ty_str = py_type(self.hir, &member.ty, def.id);
                let default = default_value(self.hir, &member.ty);
                py!(w, member.ident.name, ": ", ty_str, " = ", default, "\n");
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_interface(&self, w: &mut PyWriter, def: &Def, interface_ty: &InterfaceTy) {
        let bases: Vec<_> = if interface_ty.parents.is_empty() {
            vec!["_abc_.ABC".to_string()]
        } else {
            interface_ty
                .parents
                .iter()
                .map(|&p| self.hir.context.definitions.get(p).ident.name.clone())
                .collect()
        };

        py!(w, "class ", def, "(", bases.join(", "), "):\n");
        w.indent();

        if interface_ty.attributes.is_empty()
            && interface_ty.prototypes.is_empty()
            && interface_ty.definitions.is_empty()
        {
            py!(w, "pass\n");
        } else {
            for &nested_id in &interface_ty.definitions {
                self.emit_definition(w, nested_id);
            }

            for attr in &interface_ty.attributes {
                let ty_str = py_type(self.hir, &attr.ty, def.id);
                py!(w, "@property\n");
                py!(w, "@_abc_.abstractmethod\n");
                py!(w, "def ", attr.ident.name, "(self) -> ", ty_str, ": ...\n");
                py!(w, "\n");
            }

            for proto in &interface_ty.prototypes {
                self.emit_prototype(w, def, proto);
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_prototype(&self, w: &mut PyWriter, def: &Def, proto: &ProtoTy) {
        let ret_ty = py_type(self.hir, &proto.ty, def.id);

        let params: Vec<String> = proto
            .params
            .iter()
            .filter(|p| p.kind == ParamKind::In || p.kind == ParamKind::Inout)
            .map(|p| format!("{}: {}", p.ident.name, py_type(self.hir, &p.ty, def.id)))
            .collect();

        let params_str = if params.is_empty() {
            "self".to_string()
        } else {
            format!("self, {}", params.join(", "))
        };

        py!(w, "@_abc_.abstractmethod\n");
        py!(w, "def ", proto.ident.name, "(", params_str, ") -> ", ret_ty, ": ...\n");
        py!(w, "\n");
    }

    fn emit_valuetype(&self, w: &mut PyWriter, def: &Def, value_ty: &ValueTy) {
        let mut bases: Vec<String> = vec![];

        if let Some(parent) = value_ty.parent {
            bases.push(self.hir.context.definitions.get(parent).ident.name.clone());
        }

        if let Some(supports) = value_ty.supports {
            bases.push(
                self.hir
                    .context
                    .definitions
                    .get(supports)
                    .ident
                    .name
                    .clone(),
            );
        }

        if bases.is_empty() && !value_ty.prototypes.is_empty() {
            bases.push("_abc_.ABC".to_string());
        }

        py!(w, "@_dataclasses_.dataclass(slots=True, order=True)\n");
        if bases.is_empty() {
            py!(w, "class ", def, ":\n");
        } else {
            py!(w, "class ", def, "(", bases.join(", "), "):\n");
        }
        w.indent();

        if value_ty.members.is_empty()
            && value_ty.attributes.is_empty()
            && value_ty.prototypes.is_empty()
            && value_ty.definitions.is_empty()
        {
            py!(w, "pass\n");
        } else {
            for &nested_id in &value_ty.definitions {
                self.emit_definition(w, nested_id);
            }

            for member in &value_ty.members {
                let ty_str = py_type(self.hir, &member.ty, def.id);
                let default = default_value(self.hir, &member.ty);
                py!(w, member.ident.name, ": ", ty_str, " = ", default, "\n");
            }

            for attr in &value_ty.attributes {
                let ty_str = py_type(self.hir, &attr.ty, def.id);
                py!(w, "@property\n");
                py!(w, "@_abc_.abstractmethod\n");
                py!(w, "def ", attr.ident.name, "(self) -> ", ty_str, ": ...\n");
                py!(w, "\n");
            }

            for proto in &value_ty.prototypes {
                self.emit_prototype(w, def, proto);
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_header(w: &mut PyWriter) {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");
        py!(w, "# @generated by ic-idl ", IC_VERSION, "\n\n");
    }

    fn collect_imports(&self, defs: &[DefId], current_module: &[String]) -> Imports {
        let mut imports = Imports::default();

        for &def_id in defs {
            self.collect_stdlib_imports(def_id, &mut imports);
            self.collect_type_imports(def_id, current_module, &mut imports);
        }

        imports
    }

    fn collect_stdlib_imports(&self, def_id: DefId, imports: &mut Imports) {
        let def = self.hir.context.definitions.get(def_id);

        match &def.kind {
            DefKind::Struct(_) | DefKind::Union(_) | DefKind::Except(_) => {
                imports.stdlib.dataclasses = true;
            }
            DefKind::Enum(_) | DefKind::Bitmask(_) => {
                imports.stdlib.enum_ = true;
            }
            DefKind::Alias(_) => {
                imports.stdlib.typing = true;
            }
            DefKind::Interface(interface_ty) => {
                imports.stdlib.abc = true;
                for &nested_id in &interface_ty.definitions {
                    self.collect_stdlib_imports(nested_id, imports);
                }
            }
            DefKind::Valuetype(value_ty) => {
                imports.stdlib.dataclasses = true;
                if value_ty.parent.is_none()
                    && value_ty.supports.is_none()
                    && !value_ty.prototypes.is_empty()
                {
                    imports.stdlib.abc = true;
                }
                if !value_ty.prototypes.is_empty() || !value_ty.attributes.is_empty() {
                    imports.stdlib.abc = true;
                }
                for &nested_id in &value_ty.definitions {
                    self.collect_stdlib_imports(nested_id, imports);
                }
            }
            _ => {}
        }
    }

    fn collect_type_imports(
        &self,
        def_id: DefId,
        current_module: &[String],
        imports: &mut Imports,
    ) {
        for dep_id in self.hir.context.deps(def_id) {
            let dep_def = self.hir.context.definitions.get(dep_id);

            if dep_def.flags.contains(DefFlags::IS_BUILTIN) {
                continue;
            }

            if matches!(
                dep_def.kind,
                DefKind::Module(_) | DefKind::Annotation(_) | DefKind::Const(_)
            ) {
                continue;
            }

            let dep_module = self.module_path(dep_id);
            if dep_module != current_module {
                let module_str = if dep_module.is_empty() {
                    ".".to_string()
                } else {
                    format!(".{}", dep_module.join("."))
                };
                imports
                    .types
                    .entry(module_str)
                    .or_default()
                    .insert(dep_def.ident.name.clone());
            }
        }
    }

    fn emit_file(&self, module_path: &[String], filename: &str, defs: &[DefId]) -> File {
        let imports = self.collect_imports(defs, module_path);

        let mut w = PyWriter::new();
        Self::emit_header(&mut w);
        imports.emit(&mut w);

        for &def_id in defs {
            self.emit_definition(&mut w, def_id);
        }

        let mut path: PathBuf = module_path.iter().collect();
        path.push(format!("_{filename}.py"));

        File::Generated {
            path,
            source: w.finish(),
        }
    }
}
