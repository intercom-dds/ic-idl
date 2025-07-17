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

use ic_alloc::insensitive::CaseMap;

use crate::hir::{
    AliasTy, Ann, AnnArg, AnnotationTy, BitFlag, BitsetField, BitsetTy, BitmaskTy, ConstTy, 
    Decl, Def, DefId, DefKind, EnumLit, EnumTy, ExceptTy, InterfaceTy, Member, ModuleTy, 
    Numeric, Parameter, ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use crate::scope::{Scope, ScopeId};
use crate::{Context, ResolvedGraph};

/// Represents the result of merging multiple HIR trees.
pub struct MergedGraph {
    pub context: Context,
    pub order: Vec<DefId>,
}

/// A mapping from old DefIds to new DefIds after merging.
type DefIdMap = HashMap<DefId, DefId>;

/// A mapping from old ScopeIds to new ScopeIds after merging.
type ScopeIdMap = HashMap<ScopeId, ScopeId>;

/// Merges multiple HIR trees into a single unified tree.
///
/// This function takes multiple resolved HIR graphs and merges them into a single
/// graph, deduplicating identical definitions while preserving distinct modules.
///
/// # Arguments
///
/// * `graphs` - A slice of resolved HIR graphs to merge
///
/// # Returns
///
/// A new `MergedGraph` containing the unified HIR tree.
pub fn merge_hir_trees(graphs: &[ResolvedGraph]) -> MergedGraph {
    if graphs.is_empty() {
        return MergedGraph {
            context: Context::new(),
            order: Vec::new(),
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
    
    /// Maps from (graph_index, old_def_id) to new_def_id
    def_id_maps: Vec<DefIdMap>,
    
    /// Maps from (graph_index, old_scope_id) to new_scope_id
    scope_id_maps: Vec<ScopeIdMap>,
    
    /// Tracks definitions by their qualified name for deduplication
    /// Maps from qualified_name to new DefId
    dedup_map: HashMap<String, DefId>,
    
    /// The final order of definitions
    order: Vec<DefId>,
}

impl HirMerger {
    fn new() -> Self {
        Self {
            new_context: Context::new(),
            def_id_maps: Vec::new(),
            scope_id_maps: Vec::new(),
            dedup_map: HashMap::new(),
            order: Vec::new(),
        }
    }

    fn add_graph(&mut self, graph: &ResolvedGraph) {
        let graph_index = self.def_id_maps.len();
        self.def_id_maps.push(HashMap::new());
        self.scope_id_maps.push(HashMap::new());

        // First pass: copy scopes
        self.copy_scopes(graph_index, &graph.context);

        // Second pass: copy all definitions and build DefId mapping
        for &def_id in &graph.order {
            let new_def_id = self.copy_definition(graph_index, &graph.context, def_id);
            // Only add to order if it's a new definition (not deduplicated)
            if !self.order.contains(&new_def_id) {
                self.order.push(new_def_id);
            }
        }

        // Third pass: update all references in the copied definitions
        self.update_references(graph_index);
    }

    fn copy_definition(
        &mut self,
        graph_index: usize,
        old_context: &Context,
        old_def_id: DefId,
    ) -> DefId {
        let old_def = old_context.definitions.get(old_def_id);
        
        // Get the qualified name for deduplication
        let qualified_name = self.get_qualified_name(old_context, old_def_id);
        
        // Check if we've already copied this definition
        if let Some(&existing_def_id) = self.dedup_map.get(&qualified_name) {
            // Map the old DefId to the existing one
            self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);
            return existing_def_id;
        }
        
        // Create a new definition
        let new_def_id = self.new_context.definitions.alloc_with_id(|id| Def {
            id,
            ident: old_def.ident.clone(),
            parent: None, // Will be updated later
            annotations: old_def.annotations.clone(), // Will need updating
            span: old_def.span,
            kind: old_def.kind.clone(), // Will need updating
            flags: old_def.flags,
        });

        // Record mapping
        self.def_id_maps[graph_index].insert(old_def_id, new_def_id);
        self.dedup_map.insert(qualified_name, new_def_id);
        
        new_def_id
    }

    fn get_qualified_name(&self, context: &Context, def_id: DefId) -> String {
        let def = context.definitions.get(def_id);
        let mut parts = vec![def.ident.name.clone()];
        
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
        // Map old root to new root
        let scope_map = &mut self.scope_id_maps[graph_index];
        scope_map.insert(old_context.scopes.root(), self.new_context.scopes.root());
        
        // For simplicity, we'll just copy the scope structure without full implementation
        // TODO: Implement proper scope copying
    }

    fn update_references(&mut self, graph_index: usize) {
        // Get all the new DefIds we need to update
        let new_def_ids: Vec<DefId> = self.def_id_maps[graph_index].values().copied().collect();
        
        for new_def_id in new_def_ids {
            self.update_def_references(graph_index, new_def_id);
        }
    }

    fn update_def_references(&mut self, graph_index: usize, new_def_id: DefId) {
        // We need to be careful about borrowing here
        let updated_data = {
            let def = self.new_context.definitions.get(new_def_id);
            
            // Update parent reference
            let updated_parent = def.parent.and_then(|old_parent| {
                self.def_id_maps[graph_index].get(&old_parent).copied()
            });
            
            // Update annotations
            let updated_annotations = def.annotations.iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect::<Vec<_>>();
            
            // Update DefKind - for now just clone it
            // TODO: Implement proper DefKind updating
            let updated_kind = def.kind.clone();
            
            (updated_parent, updated_annotations, updated_kind)
        };
        
        // Now apply the updates
        let def_mut = self.new_context.definitions.get_mut(new_def_id);
        def_mut.parent = updated_data.0;
        def_mut.annotations = updated_data.1;
        def_mut.kind = updated_data.2;
    }

    fn update_annotation(&self, graph_index: usize, ann: &Ann) -> Ann {
        Ann {
            ident: ann.ident.clone(),
            def_id: if let Some(&new_id) = self.def_id_maps[graph_index].get(&ann.def_id) {
                new_id
            } else {
                ann.def_id
            },
            args: ann.args.clone(), // TODO: Update args that reference DefIds
        }
    }

    fn finish(self) -> MergedGraph {
        MergedGraph {
            context: self.new_context,
            order: self.order,
        }
    }
}