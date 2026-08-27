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

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind, ModuleTy};

pub const BARREL_STEM: &str = "index";

pub struct FileImport {
    pub binding: String,
    pub path: String,
    pub type_only: bool,
}

pub type FileImports = BTreeMap<Option<DefId>, FileImport>;

#[derive(Default)]
pub struct ImportMap {
    files: HashMap<Option<DefId>, FileImports>,
}

impl ImportMap {
    pub fn of(&self, file_module: Option<DefId>) -> Option<&FileImports> {
        self.files.get(&file_module)
    }

    pub fn binding(&self, file_module: Option<DefId>, target: Option<DefId>) -> Option<&str> {
        self.files
            .get(&file_module)
            .and_then(|imports| imports.get(&target))
            .map(|import| import.binding.as_str())
    }
}

pub fn collect(
    hir: &ResolvedGraph,
    deps_fn: &impl Fn(&[DefId]) -> HashSet<DefId>,
    type_only_fn: &impl Fn(DefId) -> bool,
) -> ImportMap {
    let mut files = HashMap::new();
    files.insert(None, FileImports::new());

    let mut pending = top_level_modules(hir);
    while let Some(module_id) = pending.pop() {
        files.insert(
            Some(module_id),
            collect_file(hir, module_id, deps_fn, type_only_fn),
        );

        if let DefKind::Module(module_ty) = &hir.context.type_of(module_id).kind {
            pending.extend(
                module_ty
                    .definitions
                    .iter()
                    .copied()
                    .filter(|&id| matches!(hir.context.type_of(id).kind, DefKind::Module(_))),
            );
        }
    }

    ImportMap { files }
}

fn collect_file(
    hir: &ResolvedGraph,
    module_id: DefId,
    deps_fn: &impl Fn(&[DefId]) -> HashSet<DefId>,
    type_only_fn: &impl Fn(DefId) -> bool,
) -> FileImports {
    let DefKind::Module(module_ty) = &hir.context.type_of(module_id).kind else {
        return FileImports::new();
    };

    let (_, defs) = partition_module_defs(hir, module_ty);
    let mut grouped: BTreeMap<Option<_>, Vec<_>> = BTreeMap::new();
    for ref_id in deps_fn(&defs) {
        if defs.contains(&ref_id) || module_ancestors(hir, ref_id).contains(&module_id) {
            continue;
        }

        let (target, _) = import_target(hir, Some(module_id), scope_of(hir, ref_id));
        grouped.entry(target).or_default().push(ref_id);
    }

    let dir_module = dir_module_of(hir, Some(module_id));
    let ups = dir_module.map_or(0, |m| module_ancestors(hir, m).len());

    let mut used = file_scope_names(hir, Some(module_id));
    let mut imports = FileImports::new();

    for (target, refs) in grouped {
        let mut binding = match target {
            None => "types".to_string(),
            Some(id) => hir.context.type_of(id).ident.name.clone(),
        };

        while used.contains(&binding) {
            binding = format!("{binding}_");
        }
        used.insert(binding.clone());

        let path = match target {
            None => match ups {
                0 => ".".to_string(),
                n => vec![".."; n].join("/"),
            },
            Some(id) => relative_path(hir, dir_module, id),
        };

        imports.insert(
            target,
            FileImport {
                binding,
                path,
                type_only: refs.iter().all(|&id| type_only_fn(id)),
            },
        );
    }

    imports
}

pub fn import_target(
    hir: &ResolvedGraph,
    file_module: Option<DefId>,
    target_scope: Option<DefId>,
) -> (Option<DefId>, Vec<DefId>) {
    let Some(scope) = target_scope else {
        return (None, vec![]);
    };

    let target_ancestors = module_ancestors(hir, scope);
    let file_ancestors = file_module
        .map(|m| module_ancestors(hir, m))
        .unwrap_or_default();

    let common = target_ancestors
        .iter()
        .zip(file_ancestors.iter())
        .take_while(|(a, b)| a == b)
        .count();

    match target_ancestors.get(common) {
        Some(&import) => (Some(import), target_ancestors[common + 1..].to_vec()),
        None => (target_ancestors.last().copied(), vec![]),
    }
}

pub fn relative_path(hir: &ResolvedGraph, from_module: Option<DefId>, to_module: DefId) -> String {
    let from_ancestors = from_module
        .map(|m| module_ancestors(hir, m))
        .unwrap_or_default();

    let to_ancestors = module_ancestors(hir, to_module);
    let common = from_ancestors
        .iter()
        .zip(to_ancestors.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = from_ancestors.len() - common;
    let remaining = &to_ancestors[common..];

    if ups == 0 && remaining.is_empty() {
        return ".".to_string();
    }

    let mut path = String::new();
    if ups == 0 {
        path.push('.');
    } else {
        for i in 0..ups {
            if i > 0 {
                path.push('/');
            }
            path.push_str("..");
        }
    }
    for &id in remaining {
        path.push('/');
        path.push_str(&module_file_stem(hir, id));
    }

    path
}

pub fn scope_of(hir: &ResolvedGraph, def_id: DefId) -> Option<DefId> {
    let def = hir.context.type_of(def_id);
    let mut current = def.parent?;

    loop {
        let def = hir.context.type_of(current);
        if matches!(def.kind, DefKind::Module(_)) {
            return Some(current);
        }
        current = def.parent?;
    }
}

pub fn module_ancestors(hir: &ResolvedGraph, def_id: DefId) -> Vec<DefId> {
    let mut ancestors = vec![];
    let mut current = Some(def_id);
    while let Some(id) = current {
        let def = hir.context.type_of(id);
        if matches!(def.kind, DefKind::Module(_)) {
            ancestors.push(id);
        }
        current = def.parent;
    }
    ancestors.reverse();
    ancestors
}

pub fn module_file_stem(hir: &ResolvedGraph, module_id: DefId) -> String {
    let def = hir.context.type_of(module_id);
    if def.ident.name != BARREL_STEM {
        return def.ident.name.clone();
    }

    let siblings = sibling_module_names(hir, def.parent);
    let mut stem = format!("{BARREL_STEM}_");
    while siblings.contains(&stem) {
        stem.push('_');
    }

    stem
}

fn sibling_module_names(hir: &ResolvedGraph, parent: Option<DefId>) -> HashSet<String> {
    let siblings = match parent {
        Some(parent_id) => match &hir.context.type_of(parent_id).kind {
            DefKind::Module(module_ty) => module_ty.definitions.clone(),
            _ => vec![],
        },
        None => top_level_modules(hir),
    };

    siblings
        .into_iter()
        .filter(|&id| matches!(hir.context.type_of(id).kind, DefKind::Module(_)))
        .map(|id| hir.context.type_of(id).ident.name.clone())
        .collect()
}

pub fn top_level_modules(hir: &ResolvedGraph) -> Vec<DefId> {
    hir.order
        .iter()
        .copied()
        .filter(|&id| {
            let def = hir.context.type_of(id);
            def.parent.is_none() && matches!(def.kind, DefKind::Module(_))
        })
        .collect()
}

pub fn partition_module_defs(
    hir: &ResolvedGraph,
    module_ty: &ModuleTy,
) -> (Vec<DefId>, Vec<DefId>) {
    let mut nested_modules = vec![];
    let mut other_defs = vec![];

    for &def_id in &module_ty.definitions {
        let def = hir.context.type_of(def_id);
        if matches!(def.kind, DefKind::Module(_)) {
            nested_modules.push(def_id);
        } else {
            other_defs.push(def_id);
        }
    }

    (nested_modules, other_defs)
}

pub fn dir_module_of(hir: &ResolvedGraph, file_module: Option<DefId>) -> Option<DefId> {
    let module_id = file_module?;
    let def = hir.context.type_of(module_id);
    let DefKind::Module(module_ty) = &def.kind else {
        return None;
    };

    let has_nested = module_ty
        .definitions
        .iter()
        .any(|&id| matches!(hir.context.type_of(id).kind, DefKind::Module(_)));

    if has_nested {
        Some(module_id)
    } else {
        def.parent
            .filter(|&p| matches!(hir.context.type_of(p).kind, DefKind::Module(_)))
    }
}

fn file_scope_names(hir: &ResolvedGraph, file_module: Option<DefId>) -> HashSet<String> {
    let mut names = HashSet::new();
    match file_module {
        Some(module_id) => {
            if let DefKind::Module(module_ty) = &hir.context.type_of(module_id).kind {
                for &def_id in &module_ty.definitions {
                    names.insert(hir.context.type_of(def_id).ident.name.clone());
                }
            }
        }
        None => {
            for &def_id in &hir.order {
                let def = hir.context.type_of(def_id);
                if def.parent.is_none() {
                    names.insert(def.ident.name.clone());
                }
            }
        }
    }
    names
}
