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

//! HIR tree merging functionality.
//!
//! This module merges the HIR trees of all translation units into a single
//! unified tree. Definitions are deduplicated by textual identity: all
//! translation units share one source map, so two definitions are the same
//! definition if and only if their identifier spans are equal, i.e. they are
//! literally the same declaration reached through different translation units
//! (`#include`). Everything else is preserved, including forward declarations
//! at distinct source locations.

use std::collections::{HashMap, HashSet};

use ic_diagnostic::{Color, Diag, Label as DiagLabel};
use ic_syntax::Span;
use tracing::{debug, debug_span};

use crate::hir::{AnnotationTy, Decl, Def, DefId, DefKind, Numeric, Ty, TyKind};
use crate::scope::ScopeId;
use crate::{Context, ResolvedGraph, rewrite};

pub struct MergedGraph {
    pub context: Context,
    pub order: Vec<DefId>,
    pub builtin_order: Vec<DefId>,
    pub errors: Vec<Diag>,
}

/// A mapping from old `DefIds` to new `DefIds` after merging.
type DefIdMap = HashMap<DefId, DefId>;

/// A mapping from old `ScopeIds` to new `ScopeIds` after merging.
type ScopeIdMap = HashMap<ScopeId, ScopeId>;

/// Merges multiple HIR trees into a single unified tree.
///
/// This function takes multiple resolved HIR graphs and merges them into a single
/// graph, deduplicating identical definitions while preserving distinct modules.
///
/// All graphs must originate from the same `SourceMap`: deduplication relies
/// on identifier spans being globally unique identifiers of source text.
///
/// # Returns
///
/// A new `MergedGraph` containing the unified HIR tree.
#[must_use]
pub fn merge_hir_trees(graphs: &[ResolvedGraph]) -> MergedGraph {
    let _span = debug_span!("merge_hir_trees", graph_count = graphs.len()).entered();
    debug!("merging {} HIR trees", graphs.len());

    if graphs.is_empty() {
        return MergedGraph {
            context: Context::new(),
            order: vec![],
            builtin_order: vec![],
            errors: vec![],
        };
    }

    let mut merger = HirMerger::new();
    for graph in graphs {
        merger.add_graph(graph);
    }

    let result = merger.finish();
    debug!(
        defs = result.context.definitions.len(),
        merged_order = ?result.order,
        "merged",
    );
    result
}

/// Internal state for the HIR merging process.
struct HirMerger {
    /// The new context being built
    new_context: Context,

    /// Maps an identifier span to the merged definition it produced.
    /// Two defs are the same definition iff their identifier spans are equal
    /// (the same source text reached through different translation units).
    span_map: HashMap<Span, DefId>,

    /// Maps from (`graph_index`, `old_def_id`) to `new_def_id`.
    /// Total per graph: every old id has an entry, deduplicated or fresh.
    def_id_maps: Vec<DefIdMap>,

    /// Maps from (`graph_index`, `old_scope_id`) to `new_scope_id`
    scope_id_maps: Vec<ScopeIdMap>,

    /// The final order of definitions
    order: Vec<DefId>,

    /// The final order of built-in definitions
    builtin_order: Vec<DefId>,

    /// Errors collected during merging
    errors: Vec<Diag>,
}

impl HirMerger {
    fn new() -> Self {
        Self {
            new_context: Context::new(),
            span_map: HashMap::new(),
            def_id_maps: vec![],
            scope_id_maps: vec![],
            order: vec![],
            builtin_order: vec![],
            errors: vec![],
        }
    }

    /// Checks if two annotation definitions are identical.
    /// Two annotations are considered identical if they have the same members with
    /// the same types in the same order.
    fn annotations_are_identical(ann1: &AnnotationTy, ann2: &AnnotationTy) -> bool {
        // Check if they have the same number of parameters
        if ann1.params.len() != ann2.params.len() {
            return false;
        }

        // Check if all parameters match in order
        for (p1, p2) in ann1.params.iter().zip(ann2.params.iter()) {
            if p1.ident.name != p2.ident.name {
                return false;
            }

            if !Self::types_are_identical(&p1.ty, &p2.ty) {
                return false;
            }

            match (&p1.default, &p2.default) {
                (None, None) => {}
                (Some(v1), Some(v2)) => {
                    if !Self::numerics_are_identical(v1, v2) {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // Check if types arrays match
        if ann1.types.len() != ann2.types.len() {
            return false;
        }

        true
    }

    /// Checks if two types are identical.
    fn types_are_identical(ty1: &Ty, ty2: &Ty) -> bool {
        match (&ty1.kind, &ty2.kind) {
            (TyKind::Any, TyKind::Any)
            | (TyKind::Fixed, TyKind::Fixed)
            | (TyKind::Null, TyKind::Null) => true,

            (TyKind::Primitive(p1), TyKind::Primitive(p2)) => p1 == p2,

            (
                TyKind::Array {
                    ty: ty1, len: len1, ..
                },
                TyKind::Array {
                    ty: ty2, len: len2, ..
                },
            ) => len1 == len2 && Self::types_are_identical(ty1, ty2),

            (
                TyKind::Sequence {
                    ty: ty1,
                    bound: bound1,
                    ..
                },
                TyKind::Sequence {
                    ty: ty2,
                    bound: bound2,
                    ..
                },
            ) => bound1 == bound2 && Self::types_are_identical(ty1, ty2),

            (
                TyKind::String {
                    wide: w1,
                    bound: b1,
                    ..
                },
                TyKind::String {
                    wide: w2,
                    bound: b2,
                    ..
                },
            ) => w1 == w2 && b1 == b2,

            (
                TyKind::Map {
                    key: k1,
                    elem: e1,
                    bound: b1,
                    ..
                },
                TyKind::Map {
                    key: k2,
                    elem: e2,
                    bound: b2,
                    ..
                },
            ) => b1 == b2 && Self::types_are_identical(k1, k2) && Self::types_are_identical(e1, e2),

            // For ADT types, we can't easily check identity without the DefId mapping
            // This is a limitation but shouldn't matter for built-in annotations
            _ => false,
        }
    }

    /// Checks if two numeric values are identical.
    fn numerics_are_identical(n1: &Numeric, n2: &Numeric) -> bool {
        match (n1, n2) {
            (Numeric::Null, Numeric::Null) => true,
            (Numeric::Bool(b1), Numeric::Bool(b2)) => b1 == b2,
            (Numeric::Int8(v1), Numeric::Int8(v2)) => v1 == v2,
            (Numeric::Int16(v1), Numeric::Int16(v2)) => v1 == v2,
            (Numeric::Int32(v1), Numeric::Int32(v2)) => v1 == v2,
            (Numeric::Int64(v1), Numeric::Int64(v2)) => v1 == v2,
            (Numeric::UInt8(v1), Numeric::UInt8(v2)) => v1 == v2,
            (Numeric::UInt16(v1), Numeric::UInt16(v2)) => v1 == v2,
            (Numeric::UInt32(v1), Numeric::UInt32(v2)) => v1 == v2,
            (Numeric::UInt64(v1), Numeric::UInt64(v2)) => v1 == v2,
            (Numeric::Float(v1), Numeric::Float(v2)) => v1.to_bits() == v2.to_bits(),
            (Numeric::Double(v1), Numeric::Double(v2)) => v1.to_bits() == v2.to_bits(),
            (Numeric::Char(c1), Numeric::Char(c2)) | (Numeric::WChar(c1), Numeric::WChar(c2)) => {
                c1 == c2
            }
            (Numeric::String(s1), Numeric::String(s2))
            | (Numeric::WString(s1), Numeric::WString(s2)) => s1 == s2,
            // For complex types, we need more sophisticated comparison
            _ => false,
        }
    }

    /// Adds a graph to the merge.
    ///
    /// Definitions are deduplicated by textual identity: two defs are the
    /// same iff their identifier spans are equal, i.e. they are literally the
    /// same declaration reached through different translation units. Nothing
    /// else is deduplicated, with one exception: an annotation that is
    /// structurally identical to a same-named annotation already merged into
    /// the same scope maps to the existing one.
    ///
    /// Steps:
    /// 1. Merge the scope skeleton (find-or-create child scopes by name).
    /// 2. Copy definitions in arena order, deduplicating by span.
    /// 3. Fix up references, parents and scope owners for created defs.
    /// 4. Merge the emission orders.
    fn add_graph(&mut self, graph: &ResolvedGraph) {
        let graph_index = self.def_id_maps.len();
        let _span = debug_span!("add_graph", graph_index).entered();
        debug!(
            def_count = graph.context.definitions.len(),
            scope_count = graph.context.scopes.scopes.len(),
            "merging graph"
        );

        self.def_id_maps.push(HashMap::new());
        self.scope_id_maps.push(HashMap::new());

        self.merge_scope_skeleton(graph_index, &graph.context);
        let created = self.copy_definitions(graph_index, &graph.context);
        self.fix_up_created_definitions(graph_index, &graph.context, &created);

        let created_ids: HashSet<_> = created.iter().map(|&(_, new_id)| new_id).collect();
        Self::merge_order_into(
            &self.def_id_maps[graph_index],
            &created_ids,
            &graph.order,
            &mut self.order,
        );
        Self::merge_order_into(
            &self.def_id_maps[graph_index],
            &created_ids,
            &graph.builtin_order,
            &mut self.builtin_order,
        );
    }

    /// Unifies the graph's scope tree with the merged scope tree by walking
    /// parent-before-child (scopes are created in that order) and
    /// finding-or-creating each child scope by name within its merged parent.
    fn merge_scope_skeleton(&mut self, graph_index: usize, old_context: &Context) {
        let old_scopes = &old_context.scopes;
        self.scope_id_maps[graph_index].insert(old_scopes.root(), self.new_context.scopes.root());

        for old_index in 0..old_scopes.scopes.len() {
            let old_scope_id = ScopeId(old_index);
            if old_scope_id == old_scopes.root() {
                continue;
            }

            let old_scope = old_scopes.get_scope(old_scope_id);
            let old_parent = old_scope.parent.expect("non-root scope has a parent");
            let &merged_parent = self.scope_id_maps[graph_index]
                .get(&old_parent)
                .expect("parent scope is processed before its children");

            let name = old_scopes
                .get_scope(old_parent)
                .children
                .iter()
                .find(|(_, id)| **id == old_scope_id)
                .map(|(name, _)| name.to_string())
                .expect("scope is registered in its parent's children");

            let merged_scope = match self
                .new_context
                .scopes
                .get_scope(merged_parent)
                .children
                .get(&name)
            {
                Some(&existing) => existing,
                None => self
                    .new_context
                    .scopes
                    .create_child_scope(merged_parent, name, None),
            };

            self.scope_id_maps[graph_index].insert(old_scope_id, merged_scope);
        }
    }

    /// Copies this graph's definitions into the merged context, deduplicating
    /// by identifier span. Returns the (old, new) id pairs actually created.
    fn copy_definitions(
        &mut self,
        graph_index: usize,
        old_context: &Context,
    ) -> Vec<(DefId, DefId)> {
        let mut created = vec![];

        for (old_def_id, old_def) in &old_context.definitions {
            if let Some(&merged_id) = self.span_map.get(&old_def.ident.span) {
                self.def_id_maps[graph_index].insert(old_def_id, merged_id);
                continue;
            }

            let merged_scope = self.merged_scope_of(graph_index, old_context, old_def_id);

            if let DefKind::Annotation(ann) = &old_def.kind
                && let Some(existing_id) =
                    self.find_identical_annotation(merged_scope, &old_def.ident.name, ann)
            {
                self.def_id_maps[graph_index].insert(old_def_id, existing_id);
                continue;
            }

            self.check_name_conflict(merged_scope, old_def);

            let new_def_id = self.new_context.definitions.alloc_with_id(|id| Def {
                id,
                ident: old_def.ident.clone(),
                parent: None,
                annotations: old_def.annotations.clone(),
                span: old_def.span,
                kind: old_def.kind.clone(),
                flags: old_def.flags,
            });

            self.span_map.insert(old_def.ident.span, new_def_id);
            self.def_id_maps[graph_index].insert(old_def_id, new_def_id);
            self.register_in_scope(merged_scope, old_def, new_def_id);
            created.push((old_def_id, new_def_id));
        }

        created
    }

    fn merged_scope_of(
        &self,
        graph_index: usize,
        old_context: &Context,
        old_def_id: DefId,
    ) -> ScopeId {
        let old_scope = old_context
            .scopes
            .find_scope_containing_def(old_def_id)
            .unwrap_or_else(|| old_context.scopes.root());

        self.scope_id_maps[graph_index][&old_scope]
    }

    fn find_identical_annotation(
        &self,
        scope: ScopeId,
        name: &str,
        ann: &AnnotationTy,
    ) -> Option<DefId> {
        let ids = self
            .new_context
            .scopes
            .get_scope(scope)
            .definitions
            .get(format!("@{name}"))?;

        ids.iter().copied().find(|&id| {
            matches!(
                &self.new_context.definitions.get(id).kind,
                DefKind::Annotation(existing) if Self::annotations_are_identical(ann, existing)
            )
        })
    }

    /// Emits a conflict diagnostic when a definition's name is already taken
    /// in its merged scope by something it cannot legally coexist with.
    /// Modules never conflict here (reopening), declarations coexist with
    /// compatible definitions, and identical annotations were deduplicated
    /// before this check.
    fn check_name_conflict(&mut self, merged_scope: ScopeId, old_def: &Def) {
        if matches!(&old_def.kind, DefKind::Module(_)) {
            return;
        }

        let table_name = Self::table_name(old_def);
        let existing_id = self
            .new_context
            .scopes
            .get_scope(merged_scope)
            .definitions
            .get(&table_name)
            .and_then(|ids| ids.last().copied());

        let Some(existing_id) = existing_id else {
            return;
        };

        let existing = self.new_context.definitions.get(existing_id);
        if matches!(&existing.kind, DefKind::Module(_))
            || Self::are_compatible_defs(&old_def.kind, &existing.kind)
        {
            return;
        }

        let existing_span = existing.ident.span;
        self.errors.push(
            Diag::error(format!(
                "conflicting definitions for `{}`",
                old_def.ident.name
            ))
            .label(
                DiagLabel::new(old_def.ident.span)
                    .message("redefined here")
                    .color(Color::Red),
            )
            .label(DiagLabel::new(existing_span).message("first defined here")),
        );
    }

    fn table_name(def: &Def) -> String {
        if matches!(&def.kind, DefKind::Annotation(_)) {
            format!("@{}", def.ident.name)
        } else {
            def.ident.name.clone()
        }
    }

    /// Registers a created definition in its merged scope's name table,
    /// in per-graph copy order, mirroring how lowering registers names in
    /// source order within one translation unit.
    fn register_in_scope(&mut self, merged_scope: ScopeId, old_def: &Def, new_def_id: DefId) {
        if matches!(&old_def.kind, DefKind::Annotation(_)) {
            self.new_context
                .scopes
                .add_annotation(merged_scope, &old_def.ident.name, new_def_id);
        } else {
            self.new_context.scopes.add_definition(
                merged_scope,
                old_def.ident.name.clone(),
                new_def_id,
            );
        }
    }

    /// Determines if two definition kinds are compatible (e.g., forward decl + full definition).
    fn are_compatible_defs(old_kind: &DefKind, existing_kind: &DefKind) -> bool {
        match (old_kind, existing_kind) {
            (DefKind::Decl(Decl::Struct), DefKind::Struct(_))
            | (DefKind::Struct(_), DefKind::Decl(Decl::Struct))
            | (DefKind::Decl(Decl::Union), DefKind::Union(_))
            | (DefKind::Union(_), DefKind::Decl(Decl::Union))
            | (DefKind::Decl(Decl::Interface), DefKind::Interface(_))
            | (DefKind::Interface(_), DefKind::Decl(Decl::Interface))
            | (DefKind::Decl(Decl::Valuetype), DefKind::Valuetype(_))
            | (DefKind::Valuetype(_), DefKind::Decl(Decl::Valuetype)) => true,
            (DefKind::Decl(a), DefKind::Decl(b)) if a == b => true,
            (DefKind::Annotation(ann1), DefKind::Annotation(ann2)) => {
                Self::annotations_are_identical(ann1, ann2)
            }
            _ => false,
        }
    }

    /// Rewrites references, parent links and scope ownership for the
    /// definitions created from this graph. Deduplicated definitions are
    /// already consistent. The per-graph map is total, so child lists and
    /// parents resolve through the same mapping and containment stays
    /// consistent by construction.
    fn fix_up_created_definitions(
        &mut self,
        graph_index: usize,
        old_context: &Context,
        created: &[(DefId, DefId)],
    ) {
        for &(old_def_id, new_def_id) in created {
            rewrite::replace_all_def_ids_in_def(
                &mut self.new_context,
                new_def_id,
                &self.def_id_maps[graph_index],
            );

            let old_parent = old_context.definitions.get(old_def_id).parent;
            let new_parent = old_parent.map(|p| self.def_id_maps[graph_index][&p]);
            self.new_context.definitions.get_mut(new_def_id).parent = new_parent;

            if let Some(old_owned) = old_context.scopes.find_scope_for_def(old_def_id) {
                let merged_scope = self.scope_id_maps[graph_index][&old_owned];
                self.new_context
                    .scopes
                    .set_scope_def_id(merged_scope, new_def_id);
            }
        }
    }

    fn merge_order_into(
        def_id_map: &DefIdMap,
        created: &HashSet<DefId>,
        old_order: &[DefId],
        merged_order: &mut Vec<DefId>,
    ) {
        for old_id in old_order {
            if let Some(new_id) = def_id_map.get(old_id)
                && created.contains(new_id)
                && !merged_order.contains(new_id)
            {
                merged_order.push(*new_id);
            }
        }
    }

    fn finish(self) -> MergedGraph {
        MergedGraph {
            context: self.new_context,
            order: self.order,
            builtin_order: self.builtin_order,
            errors: self.errors,
        }
    }
}
