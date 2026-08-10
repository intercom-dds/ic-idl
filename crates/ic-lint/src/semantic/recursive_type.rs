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

use std::collections::{HashMap, HashSet, VecDeque};

use ic_alloc::graph::DiGraph;
use ic_diagnostic::Label;
use ic_hir::hir::{
    Ann, DefFlags, DefId, DefKind, Member, PrimitiveTy, Ty, TyKind, UnionTy, Variant,
};
use ic_hir::{Context, ResolvedGraph};
use ic_syntax::Span;

use crate::{Category, Lint, LintCtx};

pub struct RecursiveType<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for RecursiveType<'a> {
    fn name() -> &'static str {
        "recursive-type"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Recursive types without a proper indirection"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &'a ResolvedGraph) {
        let lint = RecursiveType { ctx, hir };
        lint.run();
    }
}

enum Failure {
    InfiniteSize,
    NotConstructible,
}

impl Failure {
    fn message(&self, name: &str) -> String {
        match self {
            Failure::InfiniteSize => format!("type `{name}` has infinite size"),
            Failure::NotConstructible => {
                format!("type `{name}` can never be finitely constructed")
            }
        }
    }

    fn help(&self) -> &'static str {
        match self {
            Failure::InfiniteSize => "break the cycle with an indirection",
            Failure::NotConstructible => "break the cycle with an indirection that terminates",
        }
    }
}

type Edge = (Span, DefId, DefId);

impl RecursiveType<'_> {
    fn run(&self) {
        let graph = TypeGraph::build(&self.hir.context);
        let mut reported: HashSet<DefId> = HashSet::new();

        for component in Self::ordered(graph.size.cyclic_scc(), &graph.size) {
            self.report_cycle(&component, &graph.size, &Failure::InfiniteSize);
            reported.extend(component);
        }

        for component in Self::ordered(non_terminating_cycles(&graph), &graph.value) {
            if component.iter().any(|def| reported.contains(def)) {
                continue;
            }

            self.report_cycle(&component, &graph.value, &Failure::NotConstructible);
        }
    }

    fn ordered(mut components: Vec<Vec<DefId>>, relation: &Relation) -> Vec<Vec<DefId>> {
        components.sort_by_key(|component| {
            Self::edges_within(component, relation)
                .into_iter()
                .map(|(span, _, _)| span)
                .min()
        });

        components
    }

    fn edges_within(component: &[DefId], relation: &Relation) -> Vec<Edge> {
        let members: HashSet<_> = component.iter().copied().collect();
        let mut edges = vec![];

        for from in component {
            for (to, spans) in relation.neighbors(from) {
                if !members.contains(to) {
                    continue;
                }

                edges.extend(spans.iter().map(|span| (*span, *from, *to)));
            }
        }

        edges.sort_unstable();
        edges
    }

    fn report_cycle(&self, component: &[DefId], relation: &Relation, failure: &Failure) {
        let edges = Self::edges_within(component, relation);
        let Some((primary, owner, _)) = edges.first().copied() else {
            return;
        };

        let name = self.name_of(owner);
        let mut diag = ic_diagnostic::error_span(
            failure.message(&name),
            Label::new(primary).message(self.edge_message(edges[0])),
        );

        for edge in &edges[1..] {
            diag = diag.label(Label::new(edge.0).message(self.edge_message(*edge)));
        }

        let path = shortest_cycle(relation, component, owner);
        let path = path
            .iter()
            .map(|def| self.name_of(*def))
            .collect::<Vec<_>>()
            .join(" -> ");

        Self::report(
            self.ctx,
            diag.note(format!("cycle: {path}")).help(failure.help()),
        );
    }

    fn edge_message(&self, (_, from, to): Edge) -> String {
        format!(
            "`{}` contains `{}` here",
            self.name_of(from),
            self.name_of(to)
        )
    }

    fn name_of(&self, def: DefId) -> String {
        self.hir.context.definitions.get(def).ident.name.clone()
    }
}

fn non_terminating_cycles(graph: &TypeGraph) -> Vec<Vec<DefId>> {
    let terminating = terminating_set(graph);
    graph
        .value
        .cyclic_scc_where(|def| !terminating.contains(def))
}

fn terminating_set(graph: &TypeGraph) -> HashSet<DefId> {
    let mut dependents: HashMap<DefId, Vec<DefId>> = HashMap::new();
    for (def, termination) in &graph.termination {
        for target in targets_of(termination) {
            dependents.entry(target).or_default().push(*def);
        }
    }

    let mut terminating = HashSet::new();
    let mut queue = VecDeque::new();

    for (def, termination) in &graph.termination {
        if reaches_base(termination, &terminating) {
            terminating.insert(*def);
            queue.push_back(*def);
        }
    }

    while let Some(def) = queue.pop_front() {
        for dependent in dependents.get(&def).into_iter().flatten().copied() {
            if terminating.contains(&dependent) {
                continue;
            }

            let Some(termination) = graph.termination.get(&dependent) else {
                continue;
            };

            if reaches_base(termination, &terminating) {
                terminating.insert(dependent);
                queue.push_back(dependent);
            }
        }
    }

    terminating
}

fn targets_of(termination: &Termination) -> Vec<DefId> {
    match termination {
        Termination::All(targets) => targets.clone(),
        Termination::Any(arms) => arms.iter().flatten().copied().collect(),
    }
}

fn reaches_base(termination: &Termination, terminating: &HashSet<DefId>) -> bool {
    match termination {
        Termination::All(targets) => targets.iter().all(|target| terminating.contains(target)),
        Termination::Any(arms) => arms
            .iter()
            .any(|arm| arm.iter().all(|target| terminating.contains(target))),
    }
}

fn shortest_cycle(relation: &Relation, component: &[DefId], start: DefId) -> Vec<DefId> {
    let members: HashSet<_> = component.iter().copied().collect();
    let mut previous = HashMap::new();
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);

    while let Some(current) = queue.pop_front() {
        for (next, _) in relation.neighbors(&current) {
            if !members.contains(next) {
                continue;
            }

            if *next == start {
                return unwind(&previous, start, current);
            }

            if seen.insert(*next) {
                previous.insert(*next, current);
                queue.push_back(*next);
            }
        }
    }

    vec![start]
}

fn unwind(previous: &HashMap<DefId, DefId>, start: DefId, end: DefId) -> Vec<DefId> {
    let mut tail = vec![];
    let mut current = end;

    while current != start {
        tail.push(current);
        current = previous[&current];
    }

    tail.reverse();

    let mut path = vec![start];
    path.append(&mut tail);
    path.push(start);
    path
}

type Relation = DiGraph<DefId, Vec<Span>>;

enum Termination {
    All(Vec<DefId>),
    Any(Vec<Vec<DefId>>),
}

struct TypeGraph {
    size: Relation,
    value: Relation,
    termination: HashMap<DefId, Termination>,
}

impl TypeGraph {
    fn build(ctx: &Context) -> Self {
        let mut graph = TypeGraph {
            size: Relation::new(),
            value: Relation::new(),
            termination: HashMap::new(),
        };

        for (id, def) in &ctx.definitions {
            if is_node(&def.kind) {
                graph.size.add_node(id);
                graph.value.add_node(id);
            }
        }

        for (id, def) in &ctx.definitions {
            match &def.kind {
                DefKind::Struct(data) => {
                    let mut targets = vec![];
                    if let Some(parent) = &data.parent {
                        graph.link(id, parent.def_id, parent.span, false);
                        targets.push(parent.def_id);
                    }
                    graph.members(ctx, id, &data.members, &mut targets);
                    graph.termination.insert(id, Termination::All(targets));
                }
                DefKind::Except(data) => {
                    let mut targets = vec![];
                    graph.members(ctx, id, &data.members, &mut targets);
                    graph.termination.insert(id, Termination::All(targets));
                }
                DefKind::Valuetype(data) => {
                    let mut targets = vec![];
                    if let Some(parent) = &data.parent {
                        graph.link(id, parent.def_id, parent.span, false);
                        targets.push(parent.def_id);
                    }
                    graph.members(ctx, id, &data.members, &mut targets);
                    graph.termination.insert(id, Termination::All(targets));
                }
                DefKind::Union(data) => {
                    let mut arms = graph.variants(ctx, id, &data.variants);
                    if !is_exhaustive(ctx, data) {
                        arms.push(vec![]);
                    }

                    graph.termination.insert(id, Termination::Any(arms));
                }
                DefKind::Alias(data) => {
                    let mut targets = vec![];
                    graph.edges(id, &data.ty, data.ty.span, false, &mut targets);
                    graph.termination.insert(id, Termination::All(targets));
                }
                _ => {}
            }
        }

        graph
    }

    fn members(
        &mut self,
        ctx: &Context,
        from: DefId,
        members: &[Member],
        targets: &mut Vec<DefId>,
    ) {
        for member in members {
            let pointer = is_external(ctx, &member.annotations);
            self.edges(from, &member.ty, member.ident.span, pointer, targets);
        }
    }

    fn variants(&mut self, ctx: &Context, from: DefId, variants: &[Variant]) -> Vec<Vec<DefId>> {
        variants
            .iter()
            .map(|variant| {
                let pointer = is_external(ctx, &variant.annotations);
                let mut targets = vec![];
                self.edges(from, &variant.ty, variant.ident.span, pointer, &mut targets);
                targets
            })
            .collect()
    }

    fn edges(&mut self, from: DefId, ty: &Ty, span: Span, pointer: bool, targets: &mut Vec<DefId>) {
        let mut found = vec![];
        collect_targets(ty, &mut found);

        for to in found {
            if !self.size.contains(&to) {
                continue;
            }

            targets.push(to);
            self.link(from, to, span, pointer);
        }
    }

    fn link(&mut self, from: DefId, to: DefId, span: Span, pointer: bool) {
        push_span(&mut self.value, from, to, span);

        if !pointer {
            push_span(&mut self.size, from, to, span);
        }
    }
}

fn push_span(relation: &mut Relation, from: DefId, to: DefId, span: Span) {
    if let Some(spans) = relation.edge_or_default(&from, &to) {
        spans.push(span);
    }
}

fn is_node(kind: &DefKind) -> bool {
    matches!(
        kind,
        DefKind::Struct(_)
            | DefKind::Except(_)
            | DefKind::Union(_)
            | DefKind::Alias(_)
            | DefKind::Valuetype(_)
    )
}

fn is_exhaustive(ctx: &Context, union: &UnionTy) -> bool {
    if union.variants.iter().any(|variant| variant.is_default) {
        return true;
    }

    let labels: usize = union.variants.iter().map(|v| v.labels.len()).sum();

    match &ctx.resolve_ty(&union.disc.ty).kind {
        TyKind::Primitive(PrimitiveTy::Bool) => labels >= 2,
        TyKind::Primitive(PrimitiveTy::UInt8 | PrimitiveTy::Int8 | PrimitiveTy::Char) => {
            labels >= 256
        }
        TyKind::Adt(id) => match &ctx.definitions.get(*id).kind {
            DefKind::Enum(data) => labels >= data.fields.len(),
            _ => false,
        },
        _ => false,
    }
}

fn collect_targets(ty: &Ty, out: &mut Vec<DefId>) {
    match &ty.kind {
        TyKind::Adt(id) => out.push(*id),
        TyKind::Array { ty, .. } => collect_targets(ty, out),
        _ => {}
    }
}

fn is_external(ctx: &Context, annotations: &[Ann]) -> bool {
    annotations.iter().any(|ann| {
        ann.def_id.is_some_and(|id| {
            let def = ctx.definitions.get(id);

            def.flags.contains(DefFlags::IS_BUILTIN)
                && matches!(def.ident.name.as_str(), "shared" | "external")
        })
    })
}
