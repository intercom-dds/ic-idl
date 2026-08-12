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
use std::fmt::Write;
use std::path::PathBuf;

use ic_emit::File;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    AliasTy, Ann, BitmaskTy, ConstTy, Def, DefFlags, DefId, DefKind, EnumTy, ExceptTy, InterfaceTy,
    Numeric, ParamKind, ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy,
};
use ic_vfs::SourceMap;

use crate::imports::{ImportContext, collect_imports, is_exportable};
use crate::writer::PyWriter;
use crate::{PythonOptions, py};

const ORDER_IGNORE: &str = "  # ty: ignore[subclass-of-dataclass-with-order]";

type FileKey = (Vec<String>, String);

fn escape_python_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_ascii_graphic() || c == ' ' => result.push(c),
            c if c as u32 <= 0xFF => {
                _ = write!(result, "\\x{:02x}", c as u32);
            }
            c if c as u32 <= 0xFFFF => {
                _ = write!(result, "\\u{:04x}", c as u32);
            }
            c => {
                _ = write!(result, "\\U{:08x}", c as u32);
            }
        }
    }
    result
}

fn format_float(v: f64) -> String {
    if v.is_nan() {
        "float('nan')".to_string()
    } else if v.is_infinite() {
        if v.is_sign_positive() {
            "float('inf')".to_string()
        } else {
            "float('-inf')".to_string()
        }
    } else if v.fract() == 0.0 {
        format!("{v:.1}")
    } else {
        v.to_string()
    }
}

fn escape_python_char(c: char) -> String {
    match c {
        '\\' => "\\\\".to_string(),
        '"' => "\\\"".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        c if c.is_ascii_graphic() || c == ' ' => c.to_string(),
        c if c as u32 <= 0xFF => format!("\\x{:02x}", c as u32),
        c if c as u32 <= 0xFFFF => format!("\\u{:04x}", c as u32),
        c => format!("\\U{:08x}", c as u32),
    }
}

fn sanitize_module_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn collect_adt_refs(ty: &Ty, refs: &mut Vec<DefId>) {
    match &ty.kind {
        TyKind::Adt(def_id) => refs.push(*def_id),
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => collect_adt_refs(ty, refs),
        TyKind::Map { key, elem, .. } => {
            collect_adt_refs(key, refs);
            collect_adt_refs(elem, refs);
        }
        TyKind::Primitive(_)
        | TyKind::String { .. }
        | TyKind::Any
        | TyKind::Fixed
        | TyKind::Null => {}
    }
}

pub struct PyGen<'a> {
    pub hir: &'a ResolvedGraph,
    source_map: &'a SourceMap,
    options: PythonOptions,
}

impl<'a> PyGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, source_map: &'a SourceMap, options: PythonOptions) -> Self {
        Self {
            hir,
            source_map,
            options,
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut files = vec![];
        let groups = self.group_definitions_by_file();

        let file_deferred: BTreeMap<&FileKey, Vec<DefId>> = groups
            .iter()
            .map(|(key, defs)| (key, self.deferred_aliases(defs)))
            .collect();

        let deferred: BTreeSet<DefId> = file_deferred.values().flatten().copied().collect();

        for (key, defs) in &groups {
            let (module_path, filename) = key;
            files.push(self.emit_file(module_path, filename, defs, &deferred, &file_deferred[key]));
        }

        let init_files = self.generate_init_files(&groups);
        files.extend(init_files);

        if self.options.py_typed {
            let mut w = PyWriter::new(ImportContext::default());
            Self::emit_header(&mut w);
            files.push(File::Generated {
                path: "py.typed".into(),
                source: w.finish(),
            });
        }

        files
    }

    fn generate_init_files(&self, groups: &BTreeMap<FileKey, Vec<DefId>>) -> Vec<File> {
        let mut all_paths: BTreeSet<Vec<String>> = BTreeSet::new();
        for (module_path, _) in groups.keys() {
            let mut current = vec![];
            all_paths.insert(current.clone());
            for segment in module_path {
                current.push(segment.clone());
                all_paths.insert(current.clone());
            }
        }

        let mut files = vec![];

        for module_path in &all_paths {
            let mut w = PyWriter::new(ImportContext::default());
            Self::emit_header(&mut w);

            let file_defs: BTreeMap<_, _> = groups
                .iter()
                .filter(|((mp, _), _)| mp == module_path)
                .map(|((_, f), defs)| (f.clone(), defs))
                .collect();

            let mut all_names = vec![];

            let child_modules: BTreeSet<_> = all_paths
                .iter()
                .filter(|p| {
                    p.len() == module_path.len() + 1 && p.starts_with(module_path.as_slice())
                })
                .filter_map(|p| p.last().cloned())
                .collect();

            for submodule in &child_modules {
                py!(w, "from . import ", submodule, "\n");
                all_names.push(submodule.clone());
            }

            for (filename, defs) in &file_defs {
                let names: Vec<_> = defs
                    .iter()
                    .filter(|&&def_id| is_exportable(self.hir, def_id))
                    .map(|&def_id| self.hir.context.type_of(def_id).ident.name.clone())
                    .collect();

                if !names.is_empty() {
                    py!(w, "from ._", filename, " import (\n");
                    w.indent();
                    for name in &names {
                        py!(w, name, ",\n");
                    }
                    w.dedent();
                    py!(w, ")\n");
                    all_names.extend(names);
                }
            }

            if !all_names.is_empty() {
                py!(w, "\n__all__ = [\n");
                w.indent();
                for name in &all_names {
                    py!(w, "\"", name, "\",\n");
                }
                w.dedent();
                py!(w, "]\n");
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
        let def = self.hir.context.type_of(def_id);

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
                    def.parent.map(|p| &self.hir.context.type_of(p).kind),
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
        let def = self.hir.context.type_of(def_id);
        let span = def.ident.span;
        let file_path = self.source_map.name(span.start.file_id);
        let stem = file_path.file_stem()?.to_str()?;
        Some(sanitize_module_name(stem))
    }

    fn module_path(&self, def_id: DefId) -> Vec<String> {
        let mut path = vec![];
        let mut current = Some(def_id);

        while let Some(id) = current {
            let def = self.hir.context.type_of(id);
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
        let def = self.hir.context.type_of(def_id);

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
        let def = self.hir.context.type_of(def_id);
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

    fn index_definitions(
        &self,
        def_id: DefId,
        next: &mut usize,
        order: &mut BTreeMap<DefId, usize>,
        aliases: &mut Vec<(DefId, usize)>,
    ) {
        let def = self.hir.context.type_of(def_id);
        let index = *next;
        *next += 1;
        order.insert(def_id, index);

        match &def.kind {
            DefKind::Alias(_) => aliases.push((def_id, index)),
            DefKind::Interface(interface_ty) => {
                for &nested_id in &interface_ty.definitions {
                    self.index_definitions(nested_id, next, order, aliases);
                }
            }
            DefKind::Valuetype(value_ty) => {
                for &nested_id in &value_ty.definitions {
                    self.index_definitions(nested_id, next, order, aliases);
                }
            }
            _ => {}
        }
    }

    fn path_root(&self, def_id: DefId) -> DefId {
        let mut root = def_id;
        let mut current = self.hir.context.type_of(def_id).parent;

        while let Some(id) = current {
            let def = self.hir.context.type_of(id);
            if matches!(def.kind, DefKind::Module(_)) {
                break;
            }
            root = id;
            current = def.parent;
        }

        root
    }

    fn py_base(&self, w: &PyWriter, def_id: DefId) -> String {
        if self.is_imported(w, def_id) && !w.deferred_aliases.contains(&def_id) {
            return self.py_def(w, def_id);
        }

        self.py_def(w, self.hir.context.base_id_of(def_id))
    }

    fn valuetype_parent(&self, def: &Def) -> Option<DefId> {
        def.parent
            .filter(|&id| matches!(self.hir.context.type_of(id).kind, DefKind::Valuetype(_)))
    }

    fn deferred_aliases(&self, defs: &[DefId]) -> Vec<DefId> {
        let mut next = 0;
        let mut order = BTreeMap::new();
        let mut aliases = vec![];

        for &def_id in defs {
            self.index_definitions(def_id, &mut next, &mut order, &mut aliases);
        }

        let mut deferred = vec![];
        let mut deferred_set = BTreeSet::new();

        for (def_id, index) in aliases {
            let def = self.hir.context.type_of(def_id);
            let DefKind::Alias(alias) = &def.kind else {
                continue;
            };

            let mut refs = vec![];
            collect_adt_refs(&alias.ty, &mut refs);

            let root = self.path_root(def_id);
            let inside_class = root != def_id;

            let is_deferred = self.valuetype_parent(def).is_some()
                || refs.iter().any(|&id| {
                    deferred_set.contains(&id)
                        || order.get(&id).is_some_and(|&target| target > index)
                        || (inside_class && self.path_root(id) == root)
                });

            if is_deferred {
                deferred_set.insert(def_id);
                deferred.push(def_id);
            }
        }

        deferred
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
            let parent_name = self.py_base(w, parent.def_id);
            py!(w, "(", parent_name, "):", ORDER_IGNORE, "\n");
        } else {
            py!(w, ":\n");
        }

        w.indent();

        if struct_ty.members.is_empty() {
            py!(w, "pass\n");
        } else {
            for member in &struct_ty.members {
                let is_optional = self.is_optional(&member.annotations);
                let ty_str = if is_optional {
                    format!("{} | None", self.py_type(w, &member.ty))
                } else {
                    self.py_type(w, &member.ty)
                };

                let default = if is_optional {
                    "None".to_string()
                } else {
                    self.field_default(w, &member.ty)
                };

                py!(w, member.ident.name, ": ", ty_str, " = ", default, "\n");
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn format_numeric(&self, w: &PyWriter, value: &Numeric) -> String {
        match value {
            Numeric::Null => "None".to_string(),
            Numeric::Bool(b) => if *b { "True" } else { "False" }.to_string(),
            Numeric::Char(c) | Numeric::WChar(c) => format!("\"{}\"", escape_python_char(*c)),
            Numeric::Int8(v) => v.to_string(),
            Numeric::UInt8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::UInt32(v) => v.to_string(),
            Numeric::Int64(v) => v.to_string(),
            Numeric::UInt64(v) => v.to_string(),
            Numeric::Float(v) => format_float(f64::from(*v)),
            Numeric::Double(v) => format_float(*v),
            Numeric::String(s) | Numeric::WString(s) => format!("\"{}\"", escape_python_string(s)),
            Numeric::Const(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                if let Some(parent_id) = def.parent
                    && let parent_def = self.hir.context.type_of(parent_id)
                    && matches!(parent_def.kind, DefKind::Enum(_))
                {
                    let enum_name = self.py_def(w, parent_id);
                    return format!("{}.{}", enum_name, def.ident.name);
                }
                self.py_def(w, def.id)
            }
            Numeric::Array { values, .. } | Numeric::Sequence { values, .. } => {
                let items: Vec<_> = values.iter().map(|v| self.format_numeric(w, v)).collect();
                format!("[{}]", items.join(", "))
            }
            Numeric::Map { entries, .. } => {
                let items: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}: {}",
                            self.format_numeric(w, k),
                            self.format_numeric(w, v)
                        )
                    })
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Numeric::Struct { ty, fields } => {
                let struct_name = self.py_def(w, *ty);
                let items: Vec<_> = fields.iter().map(|v| self.format_numeric(w, v)).collect();
                format!("{}({})", struct_name, items.join(", "))
            }
            Numeric::Union { value, .. } => self.format_numeric(w, value),
        }
    }

    fn emit_enum(&self, w: &mut PyWriter, def: &Def, enum_ty: &EnumTy) {
        py!(w, "class ", def, "(_enum_.Enum):\n");
        w.indent();

        for &member_id in &enum_ty.fields {
            let member_def = self.hir.context.type_of(member_id);
            if let DefKind::Const(const_ty) = &member_def.kind {
                let val = self.format_numeric(w, &const_ty.value);
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
            let member_def = self.hir.context.type_of(member_id);
            if let DefKind::Const(const_ty) = &member_def.kind {
                let val = self.format_numeric(w, &const_ty.value);
                py!(w, member_def, " = ", val, "\n");
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_union(&self, w: &mut PyWriter, def: &Def, union_ty: &UnionTy) {
        let disc_type = self.py_type(w, &union_ty.disc.ty);
        let mut value_types: Vec<_> = union_ty
            .variants
            .iter()
            .filter(|v| !matches!(v.ty.kind, TyKind::Null))
            .map(|v| self.py_type(w, &v.ty))
            .collect();

        value_types.sort();
        value_types.dedup();

        let value_type_annotation = if value_types.is_empty() {
            "None".to_string()
        } else {
            format!("{} | None", value_types.join(" | "))
        };

        let disc_field_default = self.field_default(w, &union_ty.disc.ty);
        let disc_runtime_default = self.default_value(w, &union_ty.disc.ty);
        py!(w, "@_dataclasses_.dataclass(slots=True, order=True)\n");
        py!(w, "class ", def, ":\n");
        w.indent();
        py!(w, "_discriminator: ", disc_type, " = ", disc_field_default, "\n");
        py!(w, "_value: ", value_type_annotation, " = None\n");
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

            let variant_type = self.py_type(w, &variant.ty);

            w.indent();
            py!(w, "@property\n");
            py!(w, "def ", variant.ident.name, "(self) -> ", variant_type, ":\n");
            w.indent();

            if variant.is_default {
                py!(w, "return _typing_.cast(", variant_type, ", self._value)\n");
            } else if variant.labels.len() == 1 {
                let label = self.format_numeric(w, &variant.labels[0].value);
                let cmp = if matches!(variant.labels[0].value, Numeric::Bool(_)) {
                    "is not"
                } else {
                    "!="
                };

                py!(w, "if self._discriminator ", cmp, " ", label, ":\n");
                w.indent();
                py!(w, "raise ValueError(\"", variant.ident.name, " not selected\")\n");
                w.dedent();
                py!(w, "return _typing_.cast(", variant_type, ", self._value)\n");
            } else {
                let labels: Vec<_> = variant
                    .labels
                    .iter()
                    .map(|l| self.format_numeric(w, &l.value))
                    .collect();
                py!(w, "if self._discriminator not in (", labels.join(", "), "):\n");
                w.indent();
                py!(w, "raise ValueError(\"", variant.ident.name, " not selected\")\n");
                w.dedent();
                py!(w, "return _typing_.cast(", variant_type, ", self._value)\n");
            }
            w.dedent();
            w.dedent();
            py!(w, "\n");

            w.indent();
            py!(w, "@", variant.ident.name, ".setter\n");
            py!(w, "def ", variant.ident.name, "(self, val: ", variant_type, ") -> None:\n");
            w.indent();
            if !variant.labels.is_empty() {
                let first_label = self.format_numeric(w, &variant.labels[0].value);
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
        py!(w, "self._discriminator = ", disc_runtime_default, "\n");
        py!(w, "self._value = None\n");
        w.dedent();
        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_alias(&self, w: &mut PyWriter, def: &Def, alias: &AliasTy) {
        if let Some(parent) = self.valuetype_parent(def) {
            let ty_str = self.py_type_relative_to(w, &alias.ty, Some(parent));

            py!(w, "if _typing_.TYPE_CHECKING:\n");
            w.indent();
            py!(w, def, ": _typing_.TypeAlias = \"", ty_str, "\"\n");
            w.dedent();
            py!(w, "\n\n");
            return;
        }

        let ty_str = self.py_type(w, &alias.ty);
        if w.deferred_aliases.contains(&def.id) {
            py!(w, def, ": _typing_.TypeAlias = \"", ty_str, "\"\n");
        } else {
            py!(w, def, ": _typing_.TypeAlias = ", ty_str, "\n");
        }
        py!(w, "\n\n");
    }

    fn emit_alias_rebinds(&self, w: &mut PyWriter, deferred: &[DefId]) {
        if deferred.is_empty() {
            return;
        }

        py!(w, "if not _typing_.TYPE_CHECKING:\n");
        w.indent();

        for &def_id in deferred {
            let DefKind::Alias(alias) = &self.hir.context.type_of(def_id).kind else {
                continue;
            };

            let path = self.nested_type_path(def_id, None);
            let ty_str = self.py_type(w, &alias.ty);
            py!(w, path, " = ", ty_str, "\n");
        }

        w.dedent();
    }

    fn emit_const(&self, w: &mut PyWriter, def: &Def, const_ty: &ConstTy) {
        let ty_str = self.py_type(w, &const_ty.ty);
        let value_str = self.format_numeric(w, &const_ty.value);
        py!(w, def, ": _typing_.Final[", ty_str, "] = ", value_str, "\n");
        py!(w, "\n\n");
    }

    fn emit_exception(&self, w: &mut PyWriter, def: &Def, except_ty: &ExceptTy) {
        py!(w, "@_dataclasses_.dataclass(order=True)\n");
        py!(w, "class ", def, "(_builtins_.Exception):\n");
        w.indent();

        if except_ty.members.is_empty() {
            py!(w, "pass\n");
        } else {
            for member in &except_ty.members {
                let ty_str = self.py_type(w, &member.ty);
                let default = self.field_default(w, &member.ty);
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
                .map(|p| self.py_base(w, p.def_id))
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
                let ty_str = self.py_type(w, &attr.ty);
                py!(w, "@property\n");
                py!(w, "@_abc_.abstractmethod\n");
                py!(w, "def ", attr.ident.name, "(self) -> ", ty_str, ": ...\n");
                py!(w, "\n");
            }

            for proto in &interface_ty.prototypes {
                self.emit_prototype(w, proto);
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    fn emit_prototype(&self, w: &mut PyWriter, proto: &ProtoTy) {
        let ret_ty = self.py_type(w, &proto.ty);

        let params: Vec<_> = proto
            .params
            .iter()
            .filter(|p| p.kind == ParamKind::In || p.kind == ParamKind::InOut)
            .map(|p| format!("{}: {}", p.ident.name, self.py_type(w, &p.ty)))
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
        let mut bases = vec![];
        if let Some(parent) = value_ty.parent {
            bases.push(self.py_base(w, parent.def_id));
        }

        if let Some(supports) = value_ty.supports {
            bases.push(self.py_base(w, supports.def_id));
        }

        if bases.is_empty() && !value_ty.prototypes.is_empty() {
            bases.push("_abc_.ABC".to_string());
        }

        py!(w, "@_dataclasses_.dataclass(slots=True, order=True)\n");
        if bases.is_empty() {
            py!(w, "class ", def, ":\n");
        } else if value_ty.parent.is_some() {
            py!(w, "class ", def, "(", bases.join(", "), "):", ORDER_IGNORE, "\n");
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
            let (nested_aliases, nested_defs): (Vec<DefId>, Vec<DefId>) =
                value_ty.definitions.iter().copied().partition(|&id| {
                    matches!(self.hir.context.type_of(id).kind, DefKind::Alias(_))
                });

            for nested_id in nested_defs.into_iter().chain(nested_aliases) {
                self.emit_definition(w, nested_id);
            }

            if !value_ty.definitions.is_empty() {
                py!(w, "\n");
            }

            for member in &value_ty.members {
                let ty_str = self.py_type(w, &member.ty);
                let default = self.field_default(w, &member.ty);
                py!(w, member.ident.name, ": ", ty_str, " = ", default, "\n");
            }

            if !value_ty.members.is_empty() {
                py!(w, "\n");
            }

            for attr in &value_ty.attributes {
                let ty_str = self.py_type(w, &attr.ty);
                py!(w, "@property\n");
                py!(w, "@_abc_.abstractmethod\n");
                py!(w, "def ", attr.ident.name, "(self) -> ", ty_str, ": ...\n");
                py!(w, "\n");
            }

            for proto in &value_ty.prototypes {
                self.emit_prototype(w, proto);
            }
        }

        w.dedent();
        py!(w, "\n\n");
    }

    pub fn emit_header(w: &mut PyWriter) {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");
        py!(w, "# @generated by ic-idl ", IC_VERSION, "\n\n");
    }

    fn emit_file(
        &self,
        module_path: &[String],
        filename: &str,
        defs: &[DefId],
        deferred: &BTreeSet<DefId>,
        rebinds: &[DefId],
    ) -> File {
        let types_filename = format!("_{filename}");
        let imports = collect_imports(
            self.hir,
            defs,
            module_path,
            &types_filename,
            deferred,
            |id| self.module_path(id),
            |id| self.source_filename(id),
        );

        let mut w = PyWriter::new(imports.context);
        w.deferred_aliases.clone_from(deferred);
        Self::emit_header(&mut w);
        imports.stdlib.emit(&mut w);
        w.emit_module_imports();

        for &def_id in defs {
            self.emit_definition(&mut w, def_id);
        }

        self.emit_alias_rebinds(&mut w, rebinds);

        let mut path: PathBuf = module_path.iter().collect();
        path.push(format!("_{filename}.py"));

        File::Generated {
            path,
            source: w.finish(),
        }
    }
}
