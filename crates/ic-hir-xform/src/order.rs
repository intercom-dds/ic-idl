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

use std::collections::{BTreeSet, HashMap, HashSet};

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

/// A trie over module paths.
///
/// Every node holds the slots of the candidates in its subtree, so the node
/// reached by walking a module path holds exactly the candidates whose own
/// module path starts with that path.
struct PathTrie {
    children: Vec<HashMap<DefId, usize>>,
    slots: Vec<BTreeSet<usize>>,
}

impl PathTrie {
    fn new() -> Self {
        Self {
            children: vec![HashMap::new()],
            slots: vec![BTreeSet::new()],
        }
    }

    fn intern(&mut self, path: &[DefId]) -> Vec<usize> {
        let mut chain = Vec::with_capacity(path.len() + 1);
        let mut node = 0;
        chain.push(node);

        for &segment in path {
            node = if let Some(&next) = self.children[node].get(&segment) {
                next
            } else {
                let next = self.slots.len();
                self.children.push(HashMap::new());
                self.slots.push(BTreeSet::new());
                self.children[node].insert(segment, next);
                next
            };

            chain.push(node);
        }

        chain
    }

    fn insert(&mut self, chain: &[usize], slot: usize) {
        for &node in chain {
            self.slots[node].insert(slot);
        }
    }

    fn remove(&mut self, chain: &[usize], slot: usize) {
        for &node in chain {
            self.slots[node].remove(&slot);
        }
    }

    fn is_empty(&self) -> bool {
        self.slots[0].is_empty()
    }

    fn deepest(&self, chain: &[usize]) -> Option<usize> {
        chain
            .iter()
            .rev()
            .find_map(|&node| self.slots[node].first().copied())
    }
}

/// The definitions whose blockers have all been emitted, indexed so that the
/// best of them is found without scanning the ones that lose.
///
/// `anywhere` holds every candidate. `per_file` holds one trie per file, each
/// holding only the candidates in that file. Walking the current module path
/// from its end back towards the root and stopping at the first non-empty node
/// finds the candidates sharing the longest prefix of that path, and the lowest
/// slot in that node applies the sorted order as the tie-break.
struct Candidates {
    anywhere: PathTrie,
    per_file: Vec<PathTrie>,
    anywhere_chain: Vec<Vec<usize>>,
    file_chain: Vec<Vec<usize>>,
    file_index: Vec<usize>,
}

impl Candidates {
    fn new(order: &[DefId], by_id: &HashMap<DefId, Item>) -> Self {
        let mut anywhere = PathTrie::new();
        let mut per_file: Vec<PathTrie> = Vec::new();
        let mut file_slot: HashMap<FileId, usize> = HashMap::new();

        let mut anywhere_chain = Vec::with_capacity(order.len());
        let mut file_chain = Vec::with_capacity(order.len());
        let mut file_index = Vec::with_capacity(order.len());

        for id in order {
            let item = &by_id[id];

            anywhere_chain.push(anywhere.intern(&item.path));

            let index = if let Some(&index) = file_slot.get(&item.file_id) {
                index
            } else {
                let index = per_file.len();
                per_file.push(PathTrie::new());
                file_slot.insert(item.file_id, index);
                index
            };

            file_chain.push(per_file[index].intern(&item.path));
            file_index.push(index);
        }

        Self {
            anywhere,
            per_file,
            anywhere_chain,
            file_chain,
            file_index,
        }
    }

    fn insert(&mut self, slot: usize) {
        self.anywhere.insert(&self.anywhere_chain[slot], slot);
        self.per_file[self.file_index[slot]].insert(&self.file_chain[slot], slot);
    }

    fn remove(&mut self, slot: usize) {
        self.anywhere.remove(&self.anywhere_chain[slot], slot);
        self.per_file[self.file_index[slot]].remove(&self.file_chain[slot], slot);
    }

    fn is_empty(&self) -> bool {
        self.anywhere.is_empty()
    }

    fn best(&self, current: Option<usize>) -> Option<usize> {
        let Some(previous) = current else {
            return self.anywhere.deepest(&[0]);
        };

        let here = &self.per_file[self.file_index[previous]];

        if here.is_empty() {
            self.anywhere.deepest(&self.anywhere_chain[previous])
        } else {
            here.deepest(&self.file_chain[previous])
        }
    }
}

/// Orders the flattened definitions so that each one follows the definitions it
/// depends on, and returns the cyclic groups that no ordering can satisfy.
///
/// The work happens in three steps.
///
/// 1. Build a dependency graph over the definitions present in this emission
///    and find its cyclic strongly connected components. A dependency between
///    two members of one component cannot be satisfied by ordering alone, so it
///    is dropped from the blocker set and `break_cycles` later repairs it with a
///    forward declaration. A dependency on an alias is always kept as a blocker
///    because an alias cannot be forward declared.
/// 2. Sort the definitions by file, then by byte offset, then by name. That
///    order is the tie-break for the rest of the algorithm and is what makes the
///    output stable across runs. A definition's position in it is its slot.
/// 3. Emit the definitions one at a time. See `emit_order`.
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
    let mut order: Vec<DefId> = by_id.keys().copied().collect();
    order.sort_by_key(|id| sort_key(ctx, &by_id[id]));

    let sorted = emit_order(ctx, &order, &mut by_id, &blockers, &group_of);

    (sorted, cycles)
}

/// Emits the definitions in dependency order, preferring the candidates that
/// keep the current output file and module block open.
///
/// This is Kahn's algorithm with a global choice of what to emit next. A
/// definition becomes a candidate once every definition in its blocker set has
/// been emitted. Among all candidates the winner is the one with the greatest
/// affinity to the definition just emitted, and the lowest slot breaks ties.
///
/// Affinity ranks sharing the current file above sharing module path segments.
/// A candidate in the current file beats a candidate in any other file, however
/// many module segments that other candidate shares. Among the candidates that
/// do share the current file, the one sharing the longest prefix of the current
/// module path wins. Both rules exist to avoid closing and reopening a file or a
/// module block for no reason.
///
/// The choice is global and not per component, so an unrelated definition whose
/// slot falls between the slots of two members of one cyclic component is
/// emitted between them. Emitting each component as one contiguous block would
/// change the output.
///
/// Affinity is measured against the definition just emitted, so it is not a
/// fixed sort key and a plain priority queue cannot express it. `Candidates`
/// indexes the candidates by module path instead, which answers the same
/// question without a scan.
///
/// Definitions whose blockers can never all be emitted leave no candidate at
/// all. That is reported once, and the lowest remaining slot is emitted anyway
/// so that the emission still terminates and still covers every definition.
fn emit_order(
    ctx: &Context,
    order: &[DefId],
    by_id: &mut HashMap<DefId, Item>,
    blockers: &HashMap<DefId, HashSet<DefId>>,
    group_of: &HashMap<DefId, usize>,
) -> Vec<Item> {
    let count = order.len();
    let slot_of: HashMap<DefId, usize> = order
        .iter()
        .enumerate()
        .map(|(slot, &id)| (id, slot))
        .collect();

    let mut candidates = Candidates::new(order, by_id);

    let mut pending: Vec<usize> = Vec::with_capacity(count);
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); count];

    for (slot, id) in order.iter().enumerate() {
        let deps = &blockers[id];
        pending.push(deps.len());

        for dep in deps {
            dependents[slot_of[dep]].push(slot);
        }
    }

    for (slot, &blocked) in pending.iter().enumerate() {
        if blocked == 0 {
            candidates.insert(slot);
        }
    }

    let mut alive = vec![true; count];
    let mut left = count;

    let mut emitted: HashSet<DefId> = HashSet::new();
    let mut sorted = Vec::with_capacity(count);
    let mut reported = false;
    let mut current: Option<usize> = None;
    let mut cursor = 0;

    while left > 0 {
        let chosen = if candidates.is_empty() {
            if !reported {
                reported = true;
                report_unsatisfiable(ctx, order, &alive, blockers, group_of, &emitted);
            }

            while !alive[cursor] {
                cursor += 1;
            }

            cursor
        } else {
            candidates
                .best(current)
                .expect("a non-empty candidate set must yield a best candidate")
        };

        alive[chosen] = false;
        left -= 1;

        candidates.remove(chosen);

        let def_id = order[chosen];
        emitted.insert(def_id);

        for &dependent in &dependents[chosen] {
            pending[dependent] -= 1;

            if pending[dependent] == 0 && alive[dependent] {
                candidates.insert(dependent);
            }
        }

        current = Some(chosen);
        sorted.push(by_id.remove(&def_id).unwrap());
    }

    sorted
}

fn report_unsatisfiable(
    ctx: &Context,
    order: &[DefId],
    alive: &[bool],
    blockers: &HashMap<DefId, HashSet<DefId>>,
    group_of: &HashMap<DefId, usize>,
    emitted: &HashSet<DefId>,
) {
    let names: Vec<String> = order
        .iter()
        .enumerate()
        .filter(|&(slot, _)| alive[slot])
        .map(|(_, &id)| id)
        .filter(|id| {
            blockers[id]
                .iter()
                .any(|dep| !emitted.contains(dep) && group_of.get(dep) == group_of.get(id))
        })
        .map(|id| ctx.qualified_name(id))
        .collect();

    warn!(defs = ?names, "cyclic typedef dependency: no ordering can satisfy it");
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
