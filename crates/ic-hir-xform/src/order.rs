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

use std::collections::{HashMap, HashSet};

use ic_alloc::graph::DiGraph;
use ic_hir::hir::{Decl, Def, DefFlags, DefId, DefKind, Ident, ModuleTy};
use ic_hir::{Context, ResolvedGraph};
use ic_vfs::FileId;
use tracing::{debug, debug_span, warn};

struct Item {
    def_id: DefId,
    path: Vec<DefId>,
    file_id: FileId,
}

fn flatten(hir: &ResolvedGraph, order: &[DefId], path: &[DefId], out: &mut Vec<Item>) {
    for &def_id in order {
        let def = hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Module(module) => {
                let mut nested = path.to_vec();
                nested.push(def_id);
                flatten(hir, &module.definitions, &nested, out);
            }
            _ => out.push(Item {
                def_id,
                path: path.to_vec(),
                file_id: def.ident.span.start.file_id,
            }),
        }
    }
}

fn alias_body(ctx: &Context, def_id: DefId) -> Option<ic_hir::hir::Ty> {
    use ic_hir::hir::TyKind;

    let mut visited: HashSet<DefId> = HashSet::new();
    let mut current = def_id;

    loop {
        if !visited.insert(current) {
            return None;
        }

        let ty = match &ctx.definitions.get(current).kind {
            DefKind::Alias(a) => a.ty.clone(),
            DefKind::Const(c) => c.ty.clone(),
            _ => return None,
        };

        match ty.kind {
            TyKind::Adt(next) => current = next,
            _ => return Some(ty),
        }
    }
}

fn indirect_targets(ctx: &Context, def_id: DefId) -> HashSet<DefId> {
    use ic_hir::hir::{Ty, TyKind};

    fn walk(
        ctx: &Context,
        ty: &Ty,
        behind: bool,
        visited: &mut HashSet<DefId>,
        out: &mut HashSet<DefId>,
    ) {
        match &ty.kind {
            TyKind::Adt(id) => {
                if behind {
                    out.insert(*id);
                }

                if !visited.insert(*id) {
                    return;
                }

                if let Some(base) = alias_body(ctx, *id) {
                    walk(ctx, &base, behind, visited, out);
                }
            }
            TyKind::Sequence { ty, .. } => walk(ctx, ty, true, visited, out),
            TyKind::Map { key, elem, .. } => {
                walk(ctx, key, true, visited, out);
                walk(ctx, elem, true, visited, out);
            }
            TyKind::Array { ty, .. } => walk(ctx, ty, behind, visited, out),
            _ => {}
        }
    }

    let mut out = HashSet::new();
    let def = ctx.definitions.get(def_id);

    match &def.kind {
        DefKind::Struct(s) => {
            for member in &s.members {
                let behind = ic_hir_analysis::annotation::is_external(ctx, member);
                walk(ctx, &member.ty, behind, &mut HashSet::new(), &mut out);
            }
        }
        DefKind::Union(u) => {
            for variant in &u.variants {
                let behind = ic_hir_analysis::annotation::is_external(ctx, variant);
                walk(ctx, &variant.ty, behind, &mut HashSet::new(), &mut out);
            }
        }
        DefKind::Alias(a) => walk(ctx, &a.ty, false, &mut HashSet::new(), &mut out),
        _ => {}
    }

    out
}

fn sort_key(ctx: &Context, item: &Item) -> (FileId, u32, String) {
    let def = ctx.definitions.get(item.def_id);
    (item.file_id, def.span.start.offset, def.ident.name.clone())
}

fn shared_prefix(left: &[DefId], right: &[DefId]) -> usize {
    left.iter().zip(right).take_while(|(a, b)| a == b).count()
}

fn affinity(current: Option<(FileId, &[DefId])>, item: &Item) -> (usize, usize) {
    match current {
        Some((file_id, path)) => (
            usize::from(file_id == item.file_id),
            shared_prefix(path, &item.path),
        ),
        None => (0, 0),
    }
}

fn sorted_items(ctx: &Context, items: Vec<Item>) -> (Vec<Item>, Vec<HashSet<DefId>>) {
    let present: HashSet<DefId> = items.iter().map(|i| i.def_id).collect();

    let mut graph: DiGraph<DefId> = DiGraph::new();
    for item in &items {
        graph.add_node(item.def_id);
    }
    for item in &items {
        let mut deps: Vec<DefId> = ctx.deps(item.def_id).into_iter().collect();
        deps.sort_unstable();
        for dep in deps {
            if present.contains(&dep) && dep != item.def_id {
                graph.add_edge(&dep, &item.def_id, ());
            }
        }
    }

    let cycles: Vec<HashSet<DefId>> = graph
        .cyclic_scc()
        .into_iter()
        .map(|scc| scc.into_iter().collect())
        .collect();

    let group_of: HashMap<DefId, usize> = cycles
        .iter()
        .enumerate()
        .flat_map(|(index, scc)| scc.iter().map(move |&id| (id, index)))
        .collect();

    let mut blockers: HashMap<DefId, HashSet<DefId>> = HashMap::new();
    for item in &items {
        let same_group = group_of.get(&item.def_id);
        let set = ctx
            .deps(item.def_id)
            .into_iter()
            .filter(|dep| present.contains(dep) && *dep != item.def_id)
            .filter(|dep| {
                matches!(ctx.definitions.get(*dep).kind, DefKind::Alias(_))
                    || same_group.is_none()
                    || group_of.get(dep) != same_group
            })
            .collect();
        blockers.insert(item.def_id, set);
    }

    let mut by_id: HashMap<DefId, Item> = items.into_iter().map(|i| (i.def_id, i)).collect();
    let mut remaining: Vec<DefId> = by_id.keys().copied().collect();
    remaining.sort_by_key(|id| sort_key(ctx, &by_id[id]));

    let mut emitted: HashSet<DefId> = HashSet::new();
    let mut sorted = Vec::with_capacity(remaining.len());
    let mut reported = false;
    let mut current: Option<(FileId, Vec<DefId>)> = None;

    while !remaining.is_empty() {
        let here = current
            .as_ref()
            .map(|(file_id, path)| (*file_id, path.as_slice()));

        let mut best: Option<((usize, usize), usize)> = None;

        for (index, id) in remaining.iter().enumerate() {
            if !blockers[id].iter().all(|dep| emitted.contains(dep)) {
                continue;
            }

            let rank = affinity(here, &by_id[id]);

            if best.is_none_or(|(top, _)| rank > top) {
                best = Some((rank, index));
            }
        }

        let ready = best.map(|(_, index)| index);

        if ready.is_none() && !reported {
            reported = true;

            let names: Vec<String> = remaining
                .iter()
                .filter(|id| {
                    blockers[*id]
                        .iter()
                        .any(|dep| !emitted.contains(dep) && group_of.get(dep) == group_of.get(*id))
                })
                .map(|&id| ctx.qualified_name(id))
                .collect();

            warn!(defs = ?names, "cyclic typedef dependency: no ordering can satisfy it");
        }

        let def_id = remaining.remove(ready.unwrap_or(0));
        emitted.insert(def_id);

        let item = by_id.remove(&def_id).unwrap();
        current = Some((item.file_id, item.path.clone()));
        sorted.push(item);
    }

    (sorted, cycles)
}

fn forward_decl_target(ctx: &Context, def_id: DefId) -> Option<(DefId, Decl)> {
    use ic_hir::hir::TyKind;

    match &ctx.definitions.get(def_id).kind {
        DefKind::Alias(_) => {}
        DefKind::Union(_) => return Some((def_id, Decl::Union)),
        _ => return Some((def_id, Decl::Struct)),
    }

    let mut chain: Vec<DefId> = Vec::new();
    let mut visited: HashSet<DefId> = HashSet::new();
    let mut current = def_id;

    while let DefKind::Alias(alias) = &ctx.definitions.get(current).kind {
        if !visited.insert(current) {
            let names: Vec<String> = chain.iter().map(|&id| ctx.qualified_name(id)).collect();
            warn!(defs = ?names, "cyclic alias chain: nothing can be forward declared");
            return None;
        }

        chain.push(current);

        match alias.ty.kind {
            TyKind::Adt(next) => current = next,
            _ => return None,
        }
    }

    match &ctx.definitions.get(current).kind {
        DefKind::Struct(_) => Some((current, Decl::Struct)),
        DefKind::Union(_) => Some((current, Decl::Union)),
        _ => None,
    }
}

fn break_cycles(
    hir: &mut ResolvedGraph,
    cycles: &[HashSet<DefId>],
    sorted: &[Item],
) -> HashMap<DefId, Vec<(Vec<DefId>, DefId)>> {
    let position: HashMap<DefId, usize> = sorted
        .iter()
        .enumerate()
        .map(|(i, it)| (it.def_id, i))
        .collect();

    let paths: HashMap<DefId, &[DefId]> = sorted
        .iter()
        .map(|it| (it.def_id, it.path.as_slice()))
        .collect();

    let mut decls_before: HashMap<DefId, Vec<(Vec<DefId>, DefId)>> = HashMap::new();

    for scc in cycles {
        let mut has_indirect = false;
        for &def_id in scc {
            let indirect = indirect_targets(&hir.context, def_id);
            if indirect.iter().any(|target| scc.contains(target)) {
                has_indirect = true;
                break;
            }
        }

        let mut needs_decl: Vec<(DefId, Decl)> = Vec::new();
        for &def_id in scc {
            let here = position[&def_id];
            for dep in hir.context.deps(def_id) {
                if scc.contains(&dep) && position.get(&dep).is_some_and(|&there| there > here) {
                    let Some((target, decl)) = forward_decl_target(&hir.context, dep) else {
                        continue;
                    };

                    if !needs_decl.iter().any(|&(id, _)| id == target) {
                        needs_decl.push((target, decl));
                    }
                }
            }
        }
        needs_decl.sort_by_key(|&(id, _)| {
            (
                position.get(&id).copied().unwrap_or(usize::MAX),
                hir.context.definitions.get(id).ident.name.clone(),
            )
        });

        let Some(anchor) = scc.iter().copied().min_by_key(|id| position[id]) else {
            continue;
        };

        let anchor_path: &[DefId] = paths.get(&anchor).copied().unwrap_or(&[]);

        for (target, decl) in needs_decl {
            let def = hir.context.definitions.get(target);
            let ident = def.ident.clone();
            let span = def.span;
            let parent = def.parent;

            let path = paths.get(&target).copied().unwrap_or(anchor_path).to_vec();

            let decl_id = hir.context.definitions.alloc_with_id(|id| Def {
                id,
                ident,
                parent,
                annotations: Vec::new(),
                span,
                kind: DefKind::Decl(decl),
                flags: DefFlags::nil(),
            });

            decls_before
                .entry(anchor)
                .or_default()
                .push((path, decl_id));
        }

        if !has_indirect {
            mark_external(hir, scc, &position);
        }
    }

    decls_before
}

#[derive(Debug, Clone, Copy)]
enum ExternalSite {
    Member(usize),
    Variant(usize),
    Alias,
}

fn refers_into(ctx: &Context, ty: &ic_hir::hir::Ty, scc: &HashSet<DefId>) -> bool {
    use ic_hir::hir::{Ty, TyKind};

    fn walk(ctx: &Context, ty: &Ty, scc: &HashSet<DefId>, visited: &mut HashSet<DefId>) -> bool {
        match &ty.kind {
            TyKind::Adt(id) => {
                if scc.contains(id) {
                    return true;
                }

                if !visited.insert(*id) {
                    return false;
                }

                alias_body(ctx, *id).is_some_and(|base| walk(ctx, &base, scc, visited))
            }
            TyKind::Array { ty, .. } => walk(ctx, ty, scc, visited),
            _ => false,
        }
    }

    walk(ctx, ty, scc, &mut HashSet::new())
}

fn external_site(ctx: &Context, owner: DefId, scc: &HashSet<DefId>) -> Option<ExternalSite> {
    let def = ctx.definitions.get(owner);

    match &def.kind {
        DefKind::Struct(s) => s
            .members
            .iter()
            .position(|member| refers_into(ctx, &member.ty, scc))
            .map(ExternalSite::Member),
        DefKind::Union(u) => u
            .variants
            .iter()
            .position(|variant| refers_into(ctx, &variant.ty, scc))
            .map(ExternalSite::Variant),
        DefKind::Alias(a) => refers_into(ctx, &a.ty, scc).then_some(ExternalSite::Alias),

        DefKind::Except(_) | DefKind::Valuetype(_) => {
            debug!(
                owner = ?owner,
                kind = def.kind.kind_name(),
                "@external synthesis is not implemented for this kind"
            );
            None
        }

        DefKind::Annotation(_)
        | DefKind::Module(_)
        | DefKind::Enum(_)
        | DefKind::Const(_)
        | DefKind::Bitmask(_)
        | DefKind::Bitset(_)
        | DefKind::Interface(_)
        | DefKind::Decl(_) => None,
    }
}

fn site_rank(site: ExternalSite) -> usize {
    match site {
        ExternalSite::Member(_) => 0,
        ExternalSite::Variant(_) => 1,
        ExternalSite::Alias => 2,
    }
}

fn mark_external(hir: &mut ResolvedGraph, scc: &HashSet<DefId>, position: &HashMap<DefId, usize>) {
    let mut owners: Vec<DefId> = scc.iter().copied().collect();
    owners.sort_by_key(|id| position.get(id).copied().unwrap_or(usize::MAX));

    let root = hir.context.root_scope();
    let external = hir.context.scopes.resolve_annotation(root, "external");

    let candidate = owners
        .iter()
        .filter_map(|&owner| external_site(&hir.context, owner, scc).map(|site| (owner, site)))
        .min_by_key(|&(owner, site)| {
            (
                site_rank(site),
                position.get(&owner).copied().unwrap_or(usize::MAX),
            )
        });

    let Some((owner, site)) = candidate else {
        let names: Vec<String> = owners
            .iter()
            .map(|&id| hir.context.qualified_name(id))
            .collect();

        warn!(defs = ?names, "unbreakable dependency cycle: no @external candidate found");
        return;
    };

    let span = hir.context.definitions.get(owner).span;

    let ann = ic_hir::hir::Ann {
        ident: Ident {
            name: "external".to_string(),
            span,
        },
        def_id: external,
        args: Vec::new(),
    };

    let def = hir.context.definitions.get_mut(owner);

    match site {
        ExternalSite::Member(index) => {
            if let DefKind::Struct(s) = &mut def.kind {
                s.members[index].annotations.push(ann);
            }
        }
        ExternalSite::Variant(index) => {
            if let DefKind::Union(u) = &mut def.kind {
                u.variants[index].annotations.push(ann);
            }
        }
        ExternalSite::Alias => def.annotations.push(ann),
    }

    debug!(owner = ?owner, site = ?site, "synthesized @external to break cycle");
}

struct Run {
    path: Vec<DefId>,
    file_id: FileId,
    defs: Vec<DefId>,
}

fn group_runs(
    ctx: &Context,
    sorted: Vec<Item>,
    decls_before: &HashMap<DefId, Vec<(Vec<DefId>, DefId)>>,
) -> Vec<Run> {
    let mut flat: Vec<(Vec<DefId>, FileId, DefId)> = Vec::new();

    for item in sorted {
        if let Some(decls) = decls_before.get(&item.def_id) {
            for (path, decl_id) in decls {
                let file_id = ctx.definitions.get(*decl_id).ident.span.start.file_id;
                flat.push((path.clone(), file_id, *decl_id));
            }
        }

        flat.push((item.path, item.file_id, item.def_id));
    }

    let mut runs: Vec<Run> = Vec::new();

    for (path, file_id, def_id) in flat {
        match runs.last_mut() {
            Some(run) if run.path == path && run.file_id == file_id => run.defs.push(def_id),
            _ => runs.push(Run {
                path,
                file_id,
                defs: vec![def_id],
            }),
        }
    }

    runs
}

fn place_in_file(def: &mut Def, file_id: FileId) {
    def.span.start.file_id = file_id;
    def.span.end.file_id = file_id;
    def.ident.span.start.file_id = file_id;
    def.ident.span.end.file_id = file_id;
}

fn block(
    hir: &mut ResolvedGraph,
    source_id: DefId,
    file_id: FileId,
    children: Vec<DefId>,
    reused: &mut HashSet<DefId>,
    opened: &mut HashSet<(DefId, FileId)>,
) -> DefId {
    let reopened = !opened.insert((source_id, file_id));

    let block_id = if reused.insert(source_id) {
        let def = hir.context.definitions.get_mut(source_id);

        place_in_file(def, file_id);
        def.parent = None;
        def.kind = DefKind::Module(ModuleTy {
            definitions: children,
        });

        source_id
    } else {
        let mut source = hir.context.definitions.get(source_id).clone();

        place_in_file(&mut source, file_id);

        if reopened {
            source.annotations = Vec::new();
        }

        hir.context.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            kind: DefKind::Module(ModuleTy {
                definitions: children,
            }),
            ..source
        })
    };

    let nested = hir.context.definitions.get(block_id).kind.module_defs();
    for child in nested {
        hir.context.definitions.get_mut(child).parent = Some(block_id);
    }

    block_id
}

fn level(
    hir: &mut ResolvedGraph,
    runs: &[Run],
    depth: usize,
    reused: &mut HashSet<DefId>,
    opened: &mut HashSet<(DefId, FileId)>,
) -> Vec<DefId> {
    let mut order = Vec::new();
    let mut index = 0;

    while index < runs.len() {
        let run = &runs[index];

        if run.path.len() == depth {
            order.extend(run.defs.iter().copied());
            index += 1;
            continue;
        }

        let source_id = run.path[depth];
        let file_id = run.file_id;

        let mut end = index + 1;
        while end < runs.len()
            && runs[end].path.len() > depth
            && runs[end].path[depth] == source_id
            && runs[end].file_id == file_id
        {
            end += 1;
        }

        let children = level(hir, &runs[index..end], depth + 1, reused, opened);
        order.push(block(hir, source_id, file_id, children, reused, opened));

        index = end;
    }

    order
}

fn materialize(hir: &mut ResolvedGraph, runs: &[Run]) -> Vec<DefId> {
    let mut reused: HashSet<DefId> = HashSet::new();
    let mut opened: HashSet<(DefId, FileId)> = HashSet::new();

    level(hir, runs, 0, &mut reused, &mut opened)
}

trait ModuleDefs {
    fn module_defs(&self) -> Vec<DefId>;
}

impl ModuleDefs for DefKind {
    fn module_defs(&self) -> Vec<DefId> {
        match self {
            DefKind::Module(m) => m.definitions.clone(),
            _ => Vec::new(),
        }
    }
}

/// Orders definitions for emission into a language that requires
/// declare-before-use, inserting forward declarations, `@external`
/// annotations, and reopened module blocks as needed.
///
/// `ic-hir-xform::squash_modules` must not run after this transformation.
/// Squashing merges same-named module defs so that backends with no notion of
/// module reopening can traverse the graph, which undoes the runs this
/// transformation produces.
#[must_use]
pub fn apply(mut hir: ResolvedGraph) -> ResolvedGraph {
    let _span = debug_span!("order").entered();

    let mut items = Vec::new();
    let order = hir.order.clone();
    flatten(&hir, &order, &[], &mut items);

    debug!(definitions = items.len(), "flattened graph");

    let (sorted, cycles) = sorted_items(&hir.context, items);

    debug!(cycles = cycles.len(), "detected cyclic groups");

    let decls_before = break_cycles(&mut hir, &cycles, &sorted);
    let runs = group_runs(&hir.context, sorted, &decls_before);

    debug!(runs = runs.len(), "grouped emission runs");

    hir.order = materialize(&mut hir, &runs);

    hir
}
