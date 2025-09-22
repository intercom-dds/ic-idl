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
//! This module provides functionality to merge multiple HIR trees into a single
//! unified tree, handling deduplication of identical definitions while preserving
//! the structure of distinct modules.

use std::collections::HashMap;

use ic_diagnostic::{Color, Diag, Label as DiagLabel};
use ic_syntax::Span;

use crate::hir::{
    AliasTy, Ann, AnnArg, AnnotationTy, Attribute, BitmaskTy, BitsetField, BitsetTy, ConstTy, Decl,
    Def, DefId, DefKind, Disc, EnumTy, ExceptTy, InterfaceTy, Label, Member, ModuleTy, Numeric,
    Parameter, ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use crate::scope::ScopeId;
use crate::{Context, ResolvedGraph};

/// Represents the result of merging multiple HIR trees.
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
/// # Returns
///
/// A new `MergedGraph` containing the unified HIR tree.
#[must_use]
pub fn merge_hir_trees(graphs: &[ResolvedGraph]) -> MergedGraph {
    if graphs.is_empty() {
        return MergedGraph {
            context: Context::new(),
            order: Vec::new(),
            builtin_order: Vec::new(),
            errors: Vec::new(),
        };
    }

    let mut merger = HirMerger::new();
    for graph in graphs {
        merger.add_graph(graph);
    }
    merger.finish()
}

/// Internal state for the HIR merging process.
struct HirMerger {
    /// The new context being built
    new_context: Context,

    /// Maps from (`graph_index`, `old_def_id`) to `new_def_id`
    def_id_maps: Vec<DefIdMap>,

    /// Maps from (`graph_index`, `old_scope_id`) to `new_scope_id`
    scope_id_maps: Vec<ScopeIdMap>,

    /// Tracks definitions by their qualified name for deduplication
    /// Maps from `qualified_name` to new `DefId`
    dedup_map: HashMap<String, DefId>,

    /// Tracks all module definitions by qualified name to handle multiple reopenings
    /// Maps from `qualified_name` to list of (`DefId`, `Span`) pairs
    module_defs: HashMap<String, Vec<(DefId, Span)>>,

    /// Maps from `DefId` to the `ScopeId` it belongs to
    /// Used to properly register definitions in their correct scopes
    def_to_scope_map: HashMap<DefId, ScopeId>,

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
            def_id_maps: Vec::new(),
            scope_id_maps: Vec::new(),
            dedup_map: HashMap::new(),
            module_defs: HashMap::new(),
            def_to_scope_map: HashMap::new(),
            order: Vec::new(),
            builtin_order: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Maps an optional `DefId` from old to new using the graph's `DefId` map
    /// Also checks previous graphs' mappings since a `DefId` might come from an earlier file
    fn map_def_id(&self, graph_index: usize, def_id: Option<DefId>) -> Option<DefId> {
        def_id.and_then(|id| {
            // First check the current graph's mapping
            if let Some(&mapped) = self.def_id_maps[graph_index].get(&id) {
                return Some(mapped);
            }

            // Then check all previous graphs' mappings
            // This handles the case where a definition references a type from an earlier file
            for i in 0..graph_index {
                if let Some(&mapped) = self.def_id_maps[i].get(&id) {
                    return Some(mapped);
                }
            }

            None
        })
    }

    /// Maps a vector of `DefIds` from old to new, filtering out any that don't exist
    fn map_def_ids(&self, graph_index: usize, def_ids: &[DefId]) -> Vec<DefId> {
        def_ids
            .iter()
            .filter_map(|&id| {
                // Use the same logic as map_def_id to handle cross-graph references
                // First check the current graph's mapping
                if let Some(&mapped) = self.def_id_maps[graph_index].get(&id) {
                    return Some(mapped);
                }

                // Then check all previous graphs' mappings
                for i in 0..graph_index {
                    if let Some(&mapped) = self.def_id_maps[i].get(&id) {
                        return Some(mapped);
                    }
                }

                None
            })
            .collect()
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
            // Parameter names must match
            if p1.ident.name != p2.ident.name {
                return false;
            }

            // Parameter types must match
            if !Self::types_are_identical(&p1.ty, &p2.ty) {
                return false;
            }

            // Default values must match
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
            (Numeric::Octet(v1), Numeric::Octet(v2)) => v1 == v2,
            (Numeric::UInt16(v1), Numeric::UInt16(v2)) => v1 == v2,
            (Numeric::UInt32(v1), Numeric::UInt32(v2)) => v1 == v2,
            (Numeric::UInt64(v1), Numeric::UInt64(v2)) => v1 == v2,
            (Numeric::Float(v1), Numeric::Float(v2)) => v1.to_bits() == v2.to_bits(),
            (Numeric::Double(v1), Numeric::Double(v2)) => v1.to_bits() == v2.to_bits(),
            (Numeric::Char(c1), Numeric::Char(c2)) => c1 == c2,
            (Numeric::String(s1), Numeric::String(s2)) => s1 == s2,
            // For complex types, we need more sophisticated comparison
            _ => false,
        }
    }

    /// Adds a graph to the merge, handling deduplication and reference updating.
    ///
    /// The merge process follows these steps:
    /// 1. Copy scope hierarchy - preserves the nested scope structure
    /// 2. Copy all definitions - handles deduplication based on qualified name and span
    /// 3. Update scope-definition relationships
    /// 4. Update all internal references (`DefIds`, `TypeIds`) to point to merged definitions
    fn add_graph(&mut self, graph: &ResolvedGraph) {
        let graph_index = self.def_id_maps.len();
        self.def_id_maps.push(HashMap::new());
        self.scope_id_maps.push(HashMap::new());

        // First pass: copy scopes (just the structure, def_ids will be updated later)
        self.copy_scopes(graph_index, &graph.context);

        // Second pass: copy all definitions from the arena
        let all_def_ids: Vec<DefId> = graph.context.definitions.iter().map(|(id, _)| id).collect();

        // First, copy all definitions without mapping parents yet
        let mut parent_fixups: Vec<(DefId, Option<DefId>)> = Vec::new();
        let mut scope_parent_fixups: Vec<(DefId, ScopeId)> = Vec::new();

        for old_def_id in all_def_ids {
            // Find which scope contains this definition
            let old_scope = graph
                .context
                .scopes
                .find_scope_containing_def(old_def_id)
                .unwrap_or(graph.context.scopes.root());

            let new_def_id =
                self.copy_definition(graph_index, &graph.context, old_def_id, old_scope);

            // Record the original parent for later fixup
            let old_def = graph.context.definitions.get(old_def_id);
            if old_def.parent.is_some() {
                parent_fixups.push((new_def_id, old_def.parent));
            } else {
                // If the definition has no parent field but is in a non-root scope,
                // we need to find its parent through the scope hierarchy
                if old_scope != graph.context.scopes.root() {
                    let old_scope_data = &graph.context.scopes.scopes[old_scope.0];
                    if let Some(_scope_def_id) = old_scope_data.def_id {
                        // This scope belongs to a definition, record for fixup
                        scope_parent_fixups.push((new_def_id, old_scope));
                    }
                }
            }
        }

        // Fix up parent relationships from scope hierarchy
        for (new_def_id, old_scope) in scope_parent_fixups {
            self.fix_scope_parent(graph_index, &graph.context, new_def_id, old_scope);
        }

        // Now fix up all parent relationships from parent field
        for (new_def_id, original_parent) in parent_fixups {
            self.fix_parent_relationship(graph_index, new_def_id, original_parent);
        }

        // Add top-level definitions to order
        self.add_to_order(graph_index, &graph.order);

        // Add built-in definitions to builtin_order
        self.add_to_builtin_order(graph_index, &graph.builtin_order);

        // Third pass: update scope def_ids now that definitions are copied
        self.update_scope_def_ids(graph_index);

        // Fourth pass: update scope def_id fields to point to new definitions
        self.update_scope_def_id_fields(graph_index, &graph.context);

        // Fifth pass: update all references in the copied definitions
        self.update_references(graph_index);
    }

    fn fix_scope_parent(
        &mut self,
        graph_index: usize,
        old_context: &Context,
        new_def_id: DefId,
        old_scope: ScopeId,
    ) {
        let old_scope_data = &old_context.scopes.scopes[old_scope.0];
        if let Some(old_parent_def_id) = old_scope_data.def_id {
            if let Some(mapped_parent) = self.map_def_id(graph_index, Some(old_parent_def_id)) {
                self.update_parent_child_relationship(new_def_id, mapped_parent);
            }
        }
    }

    fn fix_parent_relationship(
        &mut self,
        graph_index: usize,
        new_def_id: DefId,
        original_parent: Option<DefId>,
    ) {
        if let Some(mapped_parent) = self.map_def_id(graph_index, original_parent) {
            self.update_parent_child_relationship(new_def_id, mapped_parent);
        }
    }

    fn update_parent_child_relationship(&mut self, child_id: DefId, parent_id: DefId) {
        // Update the child's parent pointer
        let def = self.new_context.definitions.get_mut(child_id);
        def.parent = Some(parent_id);

        // Add the child to the parent's definitions list
        match &mut self.new_context.definitions.get_mut(parent_id).kind {
            DefKind::Module(module) => {
                if !module.definitions.contains(&child_id) {
                    module.definitions.push(child_id);
                }
            }
            DefKind::Interface(interface) => {
                if !interface.definitions.contains(&child_id) {
                    interface.definitions.push(child_id);
                }
            }
            DefKind::Annotation(annotation) => {
                if !annotation.types.contains(&child_id) {
                    annotation.types.push(child_id);
                }
            }
            DefKind::Valuetype(valuetype) => {
                if !valuetype.definitions.contains(&child_id) {
                    valuetype.definitions.push(child_id);
                }
            }
            _ => {}
        }
    }

    fn add_to_order(&mut self, graph_index: usize, order: &[DefId]) {
        for &def_id in order {
            if let Some(&new_def_id) = self.def_id_maps[graph_index].get(&def_id) {
                if self.is_new_definition(graph_index, new_def_id)
                    && !self.order.contains(&new_def_id)
                {
                    self.order.push(new_def_id);
                }
            }
        }
    }

    fn add_to_builtin_order(&mut self, graph_index: usize, order: &[DefId]) {
        for &def_id in order {
            if let Some(&new_def_id) = self.def_id_maps[graph_index].get(&def_id) {
                if self.is_new_definition(graph_index, new_def_id)
                    && !self.builtin_order.contains(&new_def_id)
                {
                    self.builtin_order.push(new_def_id);
                }
            }
        }
    }

    fn is_new_definition(&self, graph_index: usize, new_def_id: DefId) -> bool {
        // Check if this new_def_id was created in this graph or was mapped to an existing one
        !self.def_id_maps[..graph_index]
            .iter()
            .any(|earlier_map| earlier_map.values().any(|&id| id == new_def_id))
    }

    /// Copies a definition from an old context to the new merged context.
    ///
    /// Handles deduplication based on:
    /// - Qualified name (full path including parent modules)
    /// - Identifier span (to distinguish between identical definitions vs conflicts)
    ///
    /// Special cases:
    /// - Modules are never deduplicated (each reopening creates a separate module)
    /// - Conflicting definitions (same name, different spans) generate errors
    #[allow(clippy::too_many_lines)]
    fn copy_definition(
        &mut self,
        graph_index: usize,
        old_context: &Context,
        old_def_id: DefId,
        old_scope: ScopeId,
    ) -> DefId {
        // Check if we've already processed this definition in this graph
        if let Some(&existing_def_id) = self.def_id_maps[graph_index].get(&old_def_id) {
            return existing_def_id;
        }

        let old_def = old_context.definitions.get(old_def_id);

        // Get the qualified name for deduplication
        let qualified_name = Self::get_qualified_name(old_context, old_def_id);

        // For modules, check module_defs first for same-span deduplication
        if matches!(&old_def.kind, DefKind::Module(_)) {
            if let Some(existing_modules) = self.module_defs.get(&qualified_name) {
                // Check if any existing module has the same span
                for &(existing_def_id, existing_span) in existing_modules {
                    if old_def.ident.span == existing_span {
                        // Same span = same module (from include), deduplicate
                        self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);

                        // Ensure children are registered in parent if needed
                        if let Some(parent_def_id) = old_def.parent {
                            if let Some(&mapped_parent) =
                                self.def_id_maps[graph_index].get(&parent_def_id)
                            {
                                if let DefKind::Module(module) =
                                    &mut self.new_context.definitions.get_mut(mapped_parent).kind
                                {
                                    if !module.definitions.contains(&existing_def_id) {
                                        module.definitions.push(existing_def_id);
                                    }
                                }
                            }
                        }

                        return existing_def_id;
                    }
                }
                // Different spans = module reopening, fall through to create new module
            }
        }

        // Check for existing definition with same qualified name
        if let Some(&existing_def_id) = self.dedup_map.get(&qualified_name) {
            let existing_def = self.new_context.definitions.get(existing_def_id);

            // Same span = same definition (from include), deduplicate
            if old_def.ident.span == existing_def.ident.span {
                self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);

                // For modules, ensure children are registered
                if matches!(&old_def.kind, DefKind::Module(_)) {
                    // If this deduplicated definition has children in the new graph,
                    // they will be handled by the parent fixup phase
                }

                // If this deduplicated definition has a parent, ensure it's in the parent's definitions list
                // This handles the case where a module is deduplicated but its children need to be registered
                if let Some(parent_def_id) = old_def.parent {
                    if let Some(&mapped_parent) = self.def_id_maps[graph_index].get(&parent_def_id)
                    {
                        match &mut self.new_context.definitions.get_mut(mapped_parent).kind {
                            DefKind::Module(module) => {
                                if !module.definitions.contains(&existing_def_id) {
                                    module.definitions.push(existing_def_id);
                                }
                            }
                            DefKind::Interface(interface) => {
                                if !interface.definitions.contains(&existing_def_id) {
                                    interface.definitions.push(existing_def_id);
                                }
                            }
                            DefKind::Annotation(annotation) => {
                                if !annotation.types.contains(&existing_def_id) {
                                    annotation.types.push(existing_def_id);
                                }
                            }
                            DefKind::Valuetype(valuetype) => {
                                if !valuetype.definitions.contains(&existing_def_id) {
                                    valuetype.definitions.push(existing_def_id);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                return existing_def_id;
            }

            // Different spans = different definitions
            // For modules, each reopening creates a separate DefId
            if matches!(&old_def.kind, DefKind::Module(_)) {
                // Module reopening - create a new definition
                // Fall through to create new definition
            } else {
                // For non-modules, check for conflicts
                // Check if they are compatible (forward declaration + full definition)
                let compatible = match (&old_def.kind, &existing_def.kind) {
                    (DefKind::Decl(Decl::Struct), DefKind::Struct(_))
                    | (DefKind::Struct(_), DefKind::Decl(Decl::Struct))
                    | (DefKind::Decl(Decl::Union), DefKind::Union(_))
                    | (DefKind::Union(_), DefKind::Decl(Decl::Union))
                    | (DefKind::Decl(Decl::Interface), DefKind::Interface(_))
                    | (DefKind::Interface(_), DefKind::Decl(Decl::Interface))
                    | (DefKind::Decl(Decl::Valuetype), DefKind::Valuetype(_))
                    | (DefKind::Valuetype(_), DefKind::Decl(Decl::Valuetype)) => true,
                    // Multiple forward declarations are allowed
                    (DefKind::Decl(a), DefKind::Decl(b)) if a == b => true,
                    // Identical annotation definitions are allowed (per IDL spec)
                    (DefKind::Annotation(ann1), DefKind::Annotation(ann2)) => {
                        Self::annotations_are_identical(ann1, ann2)
                    }
                    _ => false,
                };

                if compatible {
                    // For forward declaration + full definition pairs, we need to keep BOTH
                    // Don't deduplicate them - create a new definition
                    let is_decl_and_def = matches!(
                        (&old_def.kind, &existing_def.kind),
                        (
                            DefKind::Decl(_),
                            DefKind::Struct(_)
                                | DefKind::Union(_)
                                | DefKind::Interface(_)
                                | DefKind::Valuetype(_)
                        ) | (
                            DefKind::Struct(_)
                                | DefKind::Union(_)
                                | DefKind::Interface(_)
                                | DefKind::Valuetype(_),
                            DefKind::Decl(_)
                        )
                    );

                    if !is_decl_and_def {
                        // Check if both are forward declarations
                        let both_are_decls = matches!(
                            (&old_def.kind, &existing_def.kind),
                            (DefKind::Decl(_), DefKind::Decl(_))
                        );

                        if both_are_decls {
                            // For forward declarations, only deduplicate if spans are identical
                            if old_def.ident.span == existing_def.ident.span {
                                self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);
                                return existing_def_id;
                            }
                            // Different spans = different forward declarations, keep both
                            // Fall through to create a new definition
                        } else {
                            // For other compatible cases (identical annotations),
                            // deduplicate as before
                            self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);
                            return existing_def_id;
                        }
                    }
                    // Fall through to create a new definition for forward decl + full def pairs
                } else {
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
                        .label(
                            DiagLabel::new(existing_def.ident.span).message("first defined here"),
                        ),
                    );
                    // Map to existing to avoid cascading errors
                    self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);
                    return existing_def_id;
                }
            }
        }

        // Create a new definition - parent will be set later in the fixup phase
        let new_def_id = self.new_context.definitions.alloc_with_id(|id| Def {
            id,
            ident: old_def.ident.clone(),
            parent: None,                             // Parent will be fixed up later
            annotations: old_def.annotations.clone(), // Will need updating
            span: old_def.span,
            kind: old_def.kind.clone(), // Will need updating
            flags: old_def.flags,
        });

        // Record mapping
        self.def_id_maps[graph_index].insert(old_def_id, new_def_id);

        // For modules, add to module list; for others, add to dedup_map
        if matches!(&old_def.kind, DefKind::Module(_)) {
            self.module_defs
                .entry(qualified_name.clone())
                .or_default()
                .push((new_def_id, old_def.ident.span));
        } else {
            // Only add to dedup_map if it's not already there or if this is a full definition
            // replacing a forward declaration
            if let Some(&existing_id) = self.dedup_map.get(&qualified_name) {
                let existing_def = self.new_context.definitions.get(existing_id);
                let should_replace = matches!(
                    (&existing_def.kind, &old_def.kind),
                    (
                        DefKind::Decl(_),
                        DefKind::Struct(_)
                            | DefKind::Union(_)
                            | DefKind::Interface(_)
                            | DefKind::Valuetype(_)
                    )
                );
                if should_replace {
                    self.dedup_map.insert(qualified_name, new_def_id);
                }
            } else {
                self.dedup_map.insert(qualified_name, new_def_id);
            }
        }

        // Parent relationship will be handled in the fixup phase

        // Map the new definition to its scope
        if let Some(&new_scope) = self.scope_id_maps[graph_index].get(&old_scope) {
            self.def_to_scope_map.insert(new_def_id, new_scope);
        }

        new_def_id
    }

    fn get_qualified_name(context: &Context, def_id: DefId) -> String {
        let def = context.definitions.get(def_id);
        // Prefix annotations with "@" to separate their namespace
        let name = if matches!(&def.kind, DefKind::Annotation(_)) {
            format!("@{}", def.ident.name)
        } else {
            def.ident.name.clone()
        };
        let mut parts = vec![name];

        let mut current = def.parent;
        while let Some(parent_id) = current {
            let parent_def = context.definitions.get(parent_id);
            parts.push(parent_def.ident.name.clone());
            current = parent_def.parent;
        }

        parts.reverse();
        parts.join("::")
    }

    fn copy_scopes(&mut self, graph_index: usize, old_context: &Context) {
        let scope_map = &mut self.scope_id_maps[graph_index];

        // Map old root to new root
        scope_map.insert(old_context.scopes.root(), self.new_context.scopes.root());

        // Process all scopes in the old context
        for (old_scope_idx, old_scope) in old_context.scopes.scopes.iter().enumerate() {
            let old_scope_id = ScopeId(old_scope_idx);

            // Skip root scope as it's already mapped
            if old_scope_id == old_context.scopes.root() {
                continue;
            }

            // Get parent scope in new context
            let new_parent = if let Some(old_parent) = old_scope.parent {
                scope_map
                    .get(&old_parent)
                    .copied()
                    .expect("Parent scope should be processed before child")
            } else {
                // This shouldn't happen for non-root scopes
                self.new_context.scopes.root()
            };

            // Find the name of this scope in its parent
            let scope_name = if let Some(old_parent) = old_scope.parent {
                old_context.scopes.scopes[old_parent.0]
                    .children
                    .iter()
                    .find(|(_, id)| **id == old_scope_id)
                    .map_or_else(|| String::from("_unknown_"), |(name, _)| name.to_string())
            } else {
                String::from("_unknown_")
            };

            // Create new scope (def_id will be set later)
            let new_scope_id = self
                .new_context
                .scopes
                .create_child_scope(new_parent, scope_name, None);

            // Map old scope to new scope
            scope_map.insert(old_scope_id, new_scope_id);
        }
    }

    fn update_scope_def_ids(&mut self, graph_index: usize) {
        // Register all definitions in their correct scopes
        let def_map = &self.def_id_maps[graph_index];

        for &new_def_id in def_map.values() {
            // Get the scope this definition belongs to
            let scope_id = self
                .def_to_scope_map
                .get(&new_def_id)
                .copied()
                .unwrap_or_else(|| self.new_context.scopes.root());

            let def = self.new_context.definitions.get(new_def_id);
            let def_name = def.ident.name.clone();

            // Add to the appropriate namespace based on definition kind
            if matches!(&def.kind, DefKind::Annotation(_)) {
                self.new_context
                    .scopes
                    .add_annotation(scope_id, def_name, new_def_id);
            } else {
                self.new_context
                    .scopes
                    .add_definition(scope_id, def_name, new_def_id);
            }
        }
    }

    fn update_scope_def_id_fields(&mut self, graph_index: usize, old_context: &Context) {
        // Update the def_id field in scopes to point to new definitions
        for (old_scope_id, &new_scope_id) in &self.scope_id_maps[graph_index] {
            // Get the old scope's def_id
            if let Some(old_def_id) = old_context.scopes.scopes[old_scope_id.0].def_id {
                // Map it to the new def_id
                if let Some(&new_def_id) = self.def_id_maps[graph_index].get(&old_def_id) {
                    // Update the new scope's def_id
                    self.new_context.scopes.scopes[new_scope_id.0].def_id = Some(new_def_id);
                }
            }
        }
    }

    fn update_references(&mut self, graph_index: usize) {
        // Get all the new DefIds we need to update
        let new_def_ids: Vec<DefId> = self.def_id_maps[graph_index].values().copied().collect();

        for new_def_id in new_def_ids {
            self.update_def_references(graph_index, new_def_id);
        }
    }

    /// Updates all references within a definition to point to the new merged definitions
    fn update_def_references(&mut self, graph_index: usize, new_def_id: DefId) {
        // Check if this definition was created in this graph (not deduplicated)
        // A definition was created in this graph if it doesn't appear in any earlier graph's values
        let was_created_in_this_graph = !self.def_id_maps[..graph_index]
            .iter()
            .any(|earlier_map| earlier_map.values().any(|&id| id == new_def_id));

        // Only update if this definition was created in this graph
        if !was_created_in_this_graph {
            return;
        }

        // Collect updates first to avoid borrowing conflicts
        let updated_data = {
            let def = self.new_context.definitions.get(new_def_id);

            (
                def.annotations
                    .iter()
                    .map(|ann| self.update_annotation(graph_index, ann))
                    .collect::<Vec<_>>(),
                self.update_def_kind(graph_index, &def.kind),
            )
        };

        // Apply all updates at once
        let def_mut = self.new_context.definitions.get_mut(new_def_id);
        def_mut.annotations = updated_data.0;
        def_mut.kind = updated_data.1;
    }

    #[allow(clippy::too_many_lines)]
    fn update_def_kind(&self, graph_index: usize, kind: &DefKind) -> DefKind {
        match kind {
            DefKind::Struct(s) => DefKind::Struct(StructTy {
                parent: self.map_def_id(graph_index, s.parent),
                members: s
                    .members
                    .iter()
                    .map(|m| self.update_member(graph_index, m))
                    .collect(),
            }),
            DefKind::Union(u) => DefKind::Union(UnionTy {
                disc: Disc {
                    annotations: u
                        .disc
                        .annotations
                        .iter()
                        .map(|ann| self.update_annotation(graph_index, ann))
                        .collect(),
                    ty: self.update_type(graph_index, &u.disc.ty),
                },
                variants: u
                    .variants
                    .iter()
                    .map(|v| self.update_variant(graph_index, v))
                    .collect(),
            }),
            DefKind::Enum(e) => DefKind::Enum(EnumTy {
                ty: e.ty,
                fields: self.map_def_ids(graph_index, &e.fields),
            }),
            DefKind::Interface(i) => DefKind::Interface(InterfaceTy {
                parents: self.map_def_ids(graph_index, &i.parents),
                prototypes: i
                    .prototypes
                    .iter()
                    .map(|p| self.update_proto(graph_index, p))
                    .collect(),
                attributes: i
                    .attributes
                    .iter()
                    .map(|a| self.update_attribute(graph_index, a))
                    .collect(),
                definitions: self.map_def_ids(graph_index, &i.definitions),
                is_local: i.is_local,
            }),
            DefKind::Module(m) => DefKind::Module(ModuleTy {
                definitions: self.map_def_ids(graph_index, &m.definitions),
            }),
            DefKind::Annotation(a) => DefKind::Annotation(AnnotationTy {
                params: a
                    .params
                    .iter()
                    .map(|p| self.update_ann_param(graph_index, p))
                    .collect(),
                types: self.map_def_ids(graph_index, &a.types),
            }),
            DefKind::Alias(a) => DefKind::Alias(AliasTy {
                ty: self.update_type(graph_index, &a.ty),
            }),
            DefKind::Const(c) => DefKind::Const(ConstTy {
                ty: self.update_type(graph_index, &c.ty),
                value: self.update_numeric(graph_index, &c.value),
            }),
            DefKind::Bitmask(b) => DefKind::Bitmask(BitmaskTy {
                ty: b.ty,
                flags: b
                    .flags
                    .iter()
                    .filter_map(|&f| self.map_def_id(graph_index, Some(f)))
                    .collect(),
            }),
            DefKind::Bitset(b) => DefKind::Bitset(BitsetTy {
                parent: self.map_def_id(graph_index, b.parent),
                fields: b
                    .fields
                    .iter()
                    .map(|f| self.update_bitset_field(graph_index, f))
                    .collect(),
            }),
            DefKind::Valuetype(v) => DefKind::Valuetype(ValueTy {
                parent: self.map_def_id(graph_index, v.parent),
                supports: self.map_def_id(graph_index, v.supports),
                prototypes: v
                    .prototypes
                    .iter()
                    .map(|p| self.update_proto(graph_index, p))
                    .collect(),
                attributes: v
                    .attributes
                    .iter()
                    .map(|a| self.update_attribute(graph_index, a))
                    .collect(),
                members: v
                    .members
                    .iter()
                    .map(|m| self.update_member(graph_index, m))
                    .collect(),
                definitions: self.map_def_ids(graph_index, &v.definitions),
            }),
            DefKind::Except(e) => DefKind::Except(ExceptTy {
                members: e
                    .members
                    .iter()
                    .map(|m| self.update_member(graph_index, m))
                    .collect(),
            }),
            DefKind::Decl(d) => DefKind::Decl(*d),
        }
    }

    fn update_type(&self, graph_index: usize, ty: &Ty) -> Ty {
        let kind = match &ty.kind {
            TyKind::Adt(def_id) => {
                // Check current graph first
                if let Some(&new_id) = self.def_id_maps[graph_index].get(def_id) {
                    TyKind::Adt(new_id)
                } else {
                    // Check previous graphs
                    let mut found = None;
                    for i in 0..graph_index {
                        if let Some(&new_id) = self.def_id_maps[i].get(def_id) {
                            found = Some(new_id);
                            break;
                        }
                    }
                    if let Some(new_id) = found {
                        TyKind::Adt(new_id)
                    } else {
                        TyKind::Adt(*def_id)
                    }
                }
            }
            TyKind::Array { ty, len, len_span } => TyKind::Array {
                ty: Box::new(self.update_type(graph_index, ty)),
                len: *len,
                len_span: *len_span,
            },
            TyKind::Sequence {
                ty,
                bound,
                bound_span,
            } => TyKind::Sequence {
                ty: Box::new(self.update_type(graph_index, ty)),
                bound: *bound,
                bound_span: *bound_span,
            },
            TyKind::Map {
                key,
                elem,
                bound,
                bound_span,
            } => TyKind::Map {
                key: Box::new(self.update_type(graph_index, key)),
                elem: Box::new(self.update_type(graph_index, elem)),
                bound: *bound,
                bound_span: *bound_span,
            },
            other => other.clone(),
        };

        Ty {
            kind,
            span: ty.span,
        }
    }

    fn update_member(&self, graph_index: usize, member: &Member) -> Member {
        Member {
            ident: member.ident.clone(),
            ty: self.update_type(graph_index, &member.ty),
            annotations: member
                .annotations
                .iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect(),
        }
    }

    fn update_ann_param(
        &self,
        graph_index: usize,
        param: &crate::hir::AnnParam,
    ) -> crate::hir::AnnParam {
        crate::hir::AnnParam {
            ident: param.ident.clone(),
            ty: self.update_type(graph_index, &param.ty),
            default: param
                .default
                .as_ref()
                .map(|v| self.update_numeric(graph_index, v)),
        }
    }

    fn update_variant(&self, graph_index: usize, variant: &Variant) -> Variant {
        Variant {
            annotations: variant
                .annotations
                .iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect(),
            ident: variant.ident.clone(),
            ty: self.update_type(graph_index, &variant.ty),
            labels: variant
                .labels
                .iter()
                .map(|label| Label {
                    value: self.update_numeric(graph_index, &label.value),
                    span: label.span,
                })
                .collect(),
            is_default: variant.is_default,
        }
    }

    fn update_proto(&self, graph_index: usize, proto: &ProtoTy) -> ProtoTy {
        ProtoTy {
            ident: proto.ident.clone(),
            ty: self.update_type(graph_index, &proto.ty),
            params: proto
                .params
                .iter()
                .map(|p| self.update_parameter(graph_index, p))
                .collect(),
        }
    }

    fn update_attribute(&self, graph_index: usize, attr: &Attribute) -> Attribute {
        Attribute {
            ident: attr.ident.clone(),
            ty: self.update_type(graph_index, &attr.ty),
            is_readonly: attr.is_readonly,
            getraises: self.map_def_ids(graph_index, &attr.getraises),
            setraises: self.map_def_ids(graph_index, &attr.setraises),
        }
    }

    fn update_parameter(&self, graph_index: usize, param: &Parameter) -> Parameter {
        Parameter {
            ident: param.ident.clone(),
            ty: self.update_type(graph_index, &param.ty),
            kind: param.kind,
        }
    }

    fn update_bitset_field(&self, graph_index: usize, field: &BitsetField) -> BitsetField {
        BitsetField {
            ident: field.ident.clone(),
            size: field.size,
            ty: self.update_type(graph_index, &field.ty),
            annotations: field
                .annotations
                .iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect(),
        }
    }

    fn update_numeric(&self, graph_index: usize, num: &Numeric) -> Numeric {
        match num {
            Numeric::Const(def_id) => {
                // Check current graph first
                if let Some(&new_id) = self.def_id_maps[graph_index].get(def_id) {
                    Numeric::Const(new_id)
                } else {
                    // Check previous graphs
                    for i in 0..graph_index {
                        if let Some(&new_id) = self.def_id_maps[i].get(def_id) {
                            return Numeric::Const(new_id);
                        }
                    }
                    num.clone()
                }
            }
            Numeric::Array { ty, values } => Numeric::Array {
                ty: ty.clone(),
                values: values
                    .iter()
                    .map(|v| self.update_numeric(graph_index, v))
                    .collect(),
            },
            Numeric::Sequence { ty, values } => Numeric::Sequence {
                ty: ty.clone(),
                values: values
                    .iter()
                    .map(|v| self.update_numeric(graph_index, v))
                    .collect(),
            },
            Numeric::Map {
                key,
                value,
                entries: values,
            } => Numeric::Map {
                key: key.clone(),
                value: value.clone(),
                entries: values
                    .iter()
                    .map(|(k, v)| {
                        (
                            self.update_numeric(graph_index, k),
                            self.update_numeric(graph_index, v),
                        )
                    })
                    .collect(),
            },
            Numeric::Struct { ty, fields } => {
                // Map the type DefId, checking all graphs
                let new_ty = if let Some(&new_id) = self.def_id_maps[graph_index].get(ty) {
                    new_id
                } else {
                    // Check previous graphs
                    let mut found = None;
                    for i in 0..graph_index {
                        if let Some(&new_id) = self.def_id_maps[i].get(ty) {
                            found = Some(new_id);
                            break;
                        }
                    }
                    found.unwrap_or(*ty)
                };
                Numeric::Struct {
                    ty: new_ty,
                    fields: fields
                        .iter()
                        .map(|(ident, v)| (ident.clone(), self.update_numeric(graph_index, v)))
                        .collect(),
                }
            }
            Numeric::Union {
                ty,
                discriminant,
                field,
                value,
            } => {
                let new_ty = if let Some(&new_id) = self.def_id_maps[graph_index].get(ty) {
                    new_id
                } else {
                    *ty
                };
                Numeric::Union {
                    ty: new_ty,
                    discriminant: Box::new(self.update_numeric(graph_index, discriminant)),
                    field: field.clone(),
                    value: Box::new(self.update_numeric(graph_index, value)),
                }
            }
            other => other.clone(),
        }
    }

    fn update_annotation(&self, graph_index: usize, ann: &Ann) -> Ann {
        Ann {
            ident: ann.ident.clone(),
            def_id: if let Some(def_id) = ann.def_id {
                // Try to update the def_id if it exists
                if let Some(&new_id) = self.def_id_maps[graph_index].get(&def_id) {
                    Some(new_id)
                } else {
                    // Check previous graphs
                    let mut found = None;
                    for i in 0..graph_index {
                        if let Some(&new_id) = self.def_id_maps[i].get(&def_id) {
                            found = Some(new_id);
                            break;
                        }
                    }
                    found.or(Some(def_id))
                }
            } else {
                // No def_id to update
                None
            },
            args: ann
                .args
                .iter()
                .map(|arg| AnnArg {
                    ident: arg.ident.clone(),
                    value: self.update_numeric(graph_index, &arg.value),
                    ty: arg.ty.as_ref().map(|ty| self.update_type(graph_index, ty)),
                })
                .collect(),
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
