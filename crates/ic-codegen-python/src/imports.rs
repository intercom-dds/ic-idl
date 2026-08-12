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

use std::collections::{BTreeMap, BTreeSet, HashSet};

use ic_hir::hir::{DefFlags, DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir::visit::Visitor;
use ic_hir::{Context, ResolvedGraph};

use crate::py;
use crate::writer::PyWriter;

#[derive(Debug, Clone)]
pub enum ImportStyle {
    /// `from .. import foo` -> `type_prefix.Type`
    Relative {
        module_name: String,
        type_prefix: String,
        depth: usize,
    },

    /// `from .. import foo as _alias` -> `type_prefix.Type`
    Aliased {
        module_name: String,
        alias: String,
        type_prefix: String,
        depth: usize,
    },

    /// `from ... import _types_file as _alias` -> `_alias.Type`
    Ancestor {
        types_file: String,
        alias: String,
        depth: usize,
    },
}

#[derive(Debug, Clone)]
pub struct FileImport {
    pub depth: usize,
    pub types_file: String,
    pub type_name: String,
    pub alias: Option<String>,
}

impl ImportStyle {
    pub fn type_prefix(&self) -> String {
        match self {
            ImportStyle::Relative { type_prefix, .. }
            | ImportStyle::Aliased { type_prefix, .. } => type_prefix.clone(),
            ImportStyle::Ancestor { alias, .. } => alias.clone(),
        }
    }
}

#[derive(Default)]
pub struct ImportContext {
    pub module_imports: BTreeMap<DefId, ImportStyle>,
    pub file_imports: BTreeMap<DefId, FileImport>,
}

impl ImportContext {
    pub fn emit(&self, w: &mut PyWriter) {
        let mut relative_imports: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();

        for style in self.module_imports.values() {
            match style {
                ImportStyle::Relative {
                    module_name, depth, ..
                } => {
                    relative_imports
                        .entry(*depth)
                        .or_default()
                        .insert((module_name.as_str(), None));
                }
                ImportStyle::Aliased {
                    module_name,
                    alias,
                    depth,
                    ..
                } => {
                    relative_imports
                        .entry(*depth)
                        .or_default()
                        .insert((module_name.as_str(), Some(alias.as_str())));
                }
                ImportStyle::Ancestor {
                    types_file,
                    alias,
                    depth,
                } => {
                    relative_imports
                        .entry(*depth)
                        .or_default()
                        .insert((types_file.as_str(), Some(alias.as_str())));
                }
            }
        }

        for (depth, imports) in &relative_imports {
            let dots = ".".repeat(depth + 1);
            for &(module_name, alias) in imports {
                py!(w, "from ", dots, " import ", module_name);
                if let Some(alias) = alias
                    && alias != module_name
                {
                    py!(w, " as ", alias);
                }
                py!(w, "\n");
            }
        }

        let mut grouped_file_imports: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for import in self.file_imports.values() {
            grouped_file_imports
                .entry((import.depth, &import.types_file))
                .or_default()
                .push((&import.type_name, import.alias.as_deref()));
        }

        for ((depth, types_file), imports) in &grouped_file_imports {
            let dots = ".".repeat(depth + 1);
            py!(w, "from ", dots, *types_file, " import (\n");
            w.indent();
            for (type_name, alias) in imports {
                if let Some(alias) = alias {
                    py!(w, *type_name, " as ", *alias, ",\n");
                } else {
                    py!(w, *type_name, ",\n");
                }
            }
            w.dedent();
            py!(w, ")\n");
        }
    }
}

#[derive(Default)]
pub struct Stdlib {
    pub abc: bool,
    pub builtins: bool,
    pub dataclasses: bool,
    pub decimal: bool,
    pub enum_: bool,
    pub typing: bool,
}

#[derive(Default)]
pub struct Imports {
    pub stdlib: Stdlib,
    pub context: ImportContext,
}

impl Stdlib {
    pub fn emit(&self, w: &mut PyWriter) {
        py!(w, "from __future__ import annotations\n\n");

        if self.abc {
            py!(w, "import abc as _abc_\n");
        }
        if self.builtins {
            py!(w, "import builtins as _builtins_\n");
        }
        if self.dataclasses {
            py!(w, "import dataclasses as _dataclasses_\n");
        }
        if self.decimal {
            py!(w, "import decimal as _decimal_\n");
        }
        if self.enum_ {
            py!(w, "import enum as _enum_\n");
        }
        if self.typing {
            py!(w, "import typing as _typing_\n");
        }

        if self.abc
            || self.builtins
            || self.dataclasses
            || self.decimal
            || self.enum_
            || self.typing
        {
            py!(w, "\n");
        }
    }
}

pub fn parent_module(hir: &ResolvedGraph, def_id: DefId) -> Option<DefId> {
    let def = hir.context.type_of(def_id);
    let mut current = def.parent;

    while let Some(id) = current {
        let parent_def = hir.context.type_of(id);
        if matches!(parent_def.kind, DefKind::Module(_)) {
            return Some(id);
        }
        current = parent_def.parent;
    }

    None
}

pub fn is_exportable(hir: &ResolvedGraph, def_id: DefId) -> bool {
    let def = hir.context.type_of(def_id);
    if def.flags.contains(DefFlags::IS_BUILTIN) {
        return false;
    }

    match &def.kind {
        DefKind::Module(_) | DefKind::Bitset(_) | DefKind::Annotation(_) | DefKind::Decl(_) => {
            false
        }
        DefKind::Const(_) => !matches!(
            def.parent.map(|p| &hir.context.type_of(p).kind),
            Some(DefKind::Enum(_) | DefKind::Bitmask(_))
        ),
        _ => true,
    }
}

fn has_collision(
    hir: &ResolvedGraph,
    name: &str,
    current_module: &[String],
    local_defs: &[DefId],
) -> bool {
    for &def_id in local_defs {
        let def = hir.context.type_of(def_id);
        if def.ident.name == name {
            return true;
        }
    }

    current_module.last().is_some_and(|last| last == name)
}

struct ImportCollectorCtx<'a> {
    hir: &'a ResolvedGraph,
    current_module: &'a [String],
    local_defs: &'a [DefId],
    types_filename: &'a str,
}

fn resolve_deferred_aliases(
    hir: &ResolvedGraph,
    def_id: DefId,
    deferred: &BTreeSet<DefId>,
    dep_ids: &mut HashSet<DefId>,
) {
    let aliases: Vec<DefId> = dep_ids
        .iter()
        .filter(|id| deferred.contains(id))
        .copied()
        .collect();

    if aliases.is_empty() {
        return;
    }

    let ty_deps = hir.context.ty_deps(def_id);

    for alias_id in aliases {
        dep_ids.insert(hir.context.base_id_of(alias_id));

        if !ty_deps.contains(&alias_id) {
            dep_ids.remove(&alias_id);
        }
    }
}

fn collect_module_imports(
    ctx: &ImportCollectorCtx,
    def_id: DefId,
    deferred: &BTreeSet<DefId>,
    module_path_fn: &impl Fn(DefId) -> Vec<String>,
    source_filename_fn: &impl Fn(DefId) -> Option<String>,
    context: &mut ImportContext,
) {
    let mut dep_ids = ctx.hir.context.deps(def_id);
    resolve_deferred_aliases(ctx.hir, def_id, deferred, &mut dep_ids);

    for dep_id in dep_ids {
        if !is_exportable(ctx.hir, dep_id) {
            continue;
        }

        if ctx.local_defs.contains(&dep_id) {
            continue;
        }

        let dep_module = module_path_fn(dep_id);
        if dep_module == ctx.current_module || dep_module.is_empty() {
            if context.file_imports.contains_key(&dep_id) {
                continue;
            }

            if let Some(dep_filename) = source_filename_fn(dep_id) {
                let dep_types_file = format!("_{dep_filename}");
                let depth = ctx.current_module.len() - dep_module.len();

                if dep_types_file != ctx.types_filename || depth > 0 {
                    let dep_def = ctx.hir.context.type_of(dep_id);
                    let type_name = dep_def.ident.name.clone();
                    let has_collision = ctx
                        .local_defs
                        .iter()
                        .any(|&id| ctx.hir.context.type_of(id).ident.name == type_name);

                    let alias = if has_collision {
                        Some(format!("_{type_name}"))
                    } else {
                        None
                    };

                    context.file_imports.insert(
                        dep_id,
                        FileImport {
                            depth,
                            types_file: dep_types_file,
                            type_name,
                            alias,
                        },
                    );
                }
            }
            continue;
        }

        let dep_module_id = parent_module(ctx.hir, dep_id);

        if let Some(module_id) = dep_module_id
            && context.module_imports.contains_key(&module_id)
        {
            continue;
        }

        if let Some(module_id) = dep_module_id {
            let style = import_style(
                ctx.hir,
                ctx.current_module,
                &dep_module,
                ctx.local_defs,
                ctx.types_filename,
            );
            context.module_imports.insert(module_id, style);
        }
    }
}

fn is_ancestor(current_module: &[String], target_module: &[String]) -> bool {
    target_module.len() < current_module.len() && current_module.starts_with(target_module)
}

fn import_style(
    hir: &ResolvedGraph,
    current_module: &[String],
    target_module: &[String],
    local_defs: &[DefId],
    types_filename: &str,
) -> ImportStyle {
    // Handle root-level types (no module)
    if target_module.is_empty() {
        return ImportStyle::Relative {
            module_name: String::new(),
            type_prefix: String::new(),
            depth: current_module.len(),
        };
    }

    if is_ancestor(current_module, target_module) {
        let depth = current_module.len() - target_module.len();
        let alias = format!("_{}", target_module.join("_"));
        return ImportStyle::Ancestor {
            types_file: types_filename.to_string(),
            alias,
            depth,
        };
    }

    let common_len = current_module
        .iter()
        .zip(target_module.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let remaining_path = &target_module[common_len..];
    let module_name = remaining_path[0].clone();
    let depth = current_module.len() - common_len;
    let type_prefix = remaining_path.join(".");

    if has_collision(hir, &module_name, current_module, local_defs) {
        let alias = format!("_{module_name}");
        let mut aliased_path = vec![alias.clone()];
        aliased_path.extend(remaining_path[1..].iter().cloned());
        let type_prefix = aliased_path.join(".");
        ImportStyle::Aliased {
            module_name,
            alias,
            type_prefix,
            depth,
        }
    } else {
        ImportStyle::Relative {
            module_name,
            type_prefix,
            depth,
        }
    }
}

pub fn collect_imports(
    hir: &ResolvedGraph,
    defs: &[DefId],
    current_module: &[String],
    types_filename: &str,
    deferred: &BTreeSet<DefId>,
    module_path_fn: impl Fn(DefId) -> Vec<String>,
    source_filename_fn: impl Fn(DefId) -> Option<String>,
) -> Imports {
    let mut imports = Imports::default();

    let ctx = ImportCollectorCtx {
        hir,
        current_module,
        local_defs: defs,
        types_filename,
    };

    for &def_id in defs {
        collect_stdlib_imports(hir, def_id, &mut imports);
        collect_module_imports(
            &ctx,
            def_id,
            deferred,
            &module_path_fn,
            &source_filename_fn,
            &mut imports.context,
        );
    }

    imports
}

struct StdlibVisitor<'a> {
    context: &'a Context,
    stdlib: Stdlib,
}

impl<'a> ic_hir::visit::Visitor<'a> for StdlibVisitor<'a> {
    fn context(&self) -> &'a Context {
        self.context
    }

    fn visit_def(&mut self, def: &'a ic_hir::hir::Def) {
        match &def.kind {
            DefKind::Struct(_) => {
                self.stdlib.dataclasses = true;
            }
            DefKind::Union(_) => {
                self.stdlib.dataclasses = true;
                self.stdlib.typing = true;
            }
            DefKind::Except(_) => {
                self.stdlib.builtins = true;
                self.stdlib.dataclasses = true;
            }
            DefKind::Enum(_) | DefKind::Bitmask(_) => {
                self.stdlib.enum_ = true;
            }
            DefKind::Alias(_) => {
                self.stdlib.typing = true;
            }
            DefKind::Const(_)
                if def.parent.is_none_or(|p| {
                    !matches!(
                        self.context.type_of(p).kind,
                        DefKind::Enum(_) | DefKind::Bitmask(_)
                    )
                }) =>
            {
                self.stdlib.typing = true;
            }
            DefKind::Interface(_) => {
                self.stdlib.abc = true;
            }
            DefKind::Valuetype(value_ty) => {
                self.stdlib.dataclasses = true;
                if !value_ty.prototypes.is_empty() || !value_ty.attributes.is_empty() {
                    self.stdlib.abc = true;
                }
            }
            _ => {}
        }

        ic_hir::visit::walk_def(self, def);
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        let resolved = self.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Primitive(PrimitiveTy::Float128) | TyKind::Fixed => {
                self.stdlib.decimal = true;
            }
            TyKind::Any => {
                self.stdlib.typing = true;
            }
            _ => {}
        }
        ic_hir::visit::walk_ty(self, ty);
    }
}

fn collect_stdlib_imports(hir: &ResolvedGraph, def_id: DefId, imports: &mut Imports) {
    let def = hir.context.definitions.get(def_id);
    let mut visitor = StdlibVisitor {
        context: &hir.context,
        stdlib: std::mem::take(&mut imports.stdlib),
    };
    visitor.visit_def(def);
    imports.stdlib = visitor.stdlib;
}
