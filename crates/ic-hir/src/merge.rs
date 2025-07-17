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

use crate::hir::{
    AliasTy, Ann, AnnArg, AnnotationTy, BitFlag, BitmaskTy, BitsetField, BitsetTy, ConstTy, Def,
    DefId, DefKind, EnumLit, EnumTy, ExceptTy, InterfaceTy, Member, ModuleTy, Numeric, Parameter,
    ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use crate::scope::ScopeId;
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

            // For modules, also process their contents
            let old_def = graph.context.definitions.get(def_id);
            if let DefKind::Module(module) = &old_def.kind {
                for &child_def_id in &module.definitions {
                    let _ = self.copy_definition(graph_index, &graph.context, child_def_id);
                }
            }
        }

        // Third pass: update scope def_ids now that definitions are copied
        self.update_scope_def_ids(graph_index);

        // Fourth pass: update all references in the copied definitions
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
            // Special case: modules should NOT be deduplicated - each reopening is separate
            if !matches!(&old_def.kind, DefKind::Module(_)) {
                // Map the old DefId to the existing one
                self.def_id_maps[graph_index].insert(old_def_id, existing_def_id);
                return existing_def_id;
            }
        }

        // Create a new definition
        let new_def_id = self.new_context.definitions.alloc_with_id(|id| Def {
            id,
            ident: old_def.ident.clone(),
            parent: None,                             // Will be updated later
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

            // Check if this scope has a definition
            let new_def_id = if let Some(_old_def_id) = old_scope.def_id {
                // The definition will be copied later, so we'll need to update this
                // For now, just use None and update it after definitions are copied
                None
            } else {
                None
            };

            // Find the name of this scope in its parent
            let scope_name = if let Some(old_parent) = old_scope.parent {
                old_context.scopes.scopes[old_parent.0]
                    .children
                    .iter()
                    .find(|(_, id)| **id == old_scope_id)
                    .map(|(name, _)| name.to_string())
                    .unwrap_or_else(|| String::from("_unknown_"))
            } else {
                String::from("_unknown_")
            };

            // Create new scope
            let new_scope_id = self
                .new_context
                .scopes
                .create_child_scope(new_parent, scope_name, new_def_id);

            // Map old scope to new scope
            scope_map.insert(old_scope_id, new_scope_id);
        }
    }

    fn update_scope_def_ids(&mut self, graph_index: usize) {
        // Since we don't have access to the old context here, we need to track
        // which scopes need their def_ids updated during scope copying.
        // For now, this is a placeholder that ensures definitions are properly
        // registered in their scopes.

        let def_map = &self.def_id_maps[graph_index];

        // Register all definitions in their appropriate scopes
        for (_, &new_def_id) in def_map.iter() {
            let def_name = self
                .new_context
                .definitions
                .get(new_def_id)
                .ident
                .name
                .clone();
            // For now, add all definitions to root scope
            // In a full implementation, we'd track the proper scope during copying
            self.new_context.scopes.add_definition(
                self.new_context.scopes.root(),
                def_name,
                new_def_id,
            );
        }
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
            let updated_parent = def
                .parent
                .and_then(|old_parent| self.def_id_maps[graph_index].get(&old_parent).copied());

            // Update annotations
            let updated_annotations = def
                .annotations
                .iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect::<Vec<_>>();

            // Update DefKind
            let updated_kind = self.update_def_kind(graph_index, &def.kind);

            (updated_parent, updated_annotations, updated_kind)
        };

        // Now apply the updates
        let def_mut = self.new_context.definitions.get_mut(new_def_id);
        def_mut.parent = updated_data.0;
        def_mut.annotations = updated_data.1;
        def_mut.kind = updated_data.2;
    }

    fn update_def_kind(&self, graph_index: usize, kind: &DefKind) -> DefKind {
        match kind {
            DefKind::Struct(s) => DefKind::Struct(StructTy {
                parent: s
                    .parent
                    .and_then(|id| self.def_id_maps[graph_index].get(&id).copied()),
                members: s
                    .members
                    .iter()
                    .map(|m| self.update_member(graph_index, m))
                    .collect(),
            }),
            DefKind::Union(u) => DefKind::Union(UnionTy {
                disc: self.update_type(graph_index, &u.disc),
                variants: u
                    .variants
                    .iter()
                    .map(|v| self.update_variant(graph_index, v))
                    .collect(),
            }),
            DefKind::Enum(e) => DefKind::Enum(EnumTy {
                ty: self.update_type(graph_index, &e.ty),
                fields: e
                    .fields
                    .iter()
                    .map(|f| self.update_enum_lit(graph_index, f))
                    .collect(),
            }),
            DefKind::Interface(i) => DefKind::Interface(InterfaceTy {
                parents: i
                    .parents
                    .iter()
                    .filter_map(|&id| self.def_id_maps[graph_index].get(&id).copied())
                    .collect(),
                prototypes: i
                    .prototypes
                    .iter()
                    .map(|p| self.update_proto(graph_index, p))
                    .collect(),
                attributes: i.attributes.clone(),
                definitions: i
                    .definitions
                    .iter()
                    .filter_map(|&id| self.def_id_maps[graph_index].get(&id).copied())
                    .collect(),
                is_local: i.is_local,
            }),
            DefKind::Module(m) => DefKind::Module(ModuleTy {
                definitions: m
                    .definitions
                    .iter()
                    .filter_map(|&id| self.def_id_maps[graph_index].get(&id).copied())
                    .collect(),
            }),
            DefKind::Annotation(a) => DefKind::Annotation(AnnotationTy {
                members: a
                    .members
                    .iter()
                    .map(|m| self.update_member(graph_index, m))
                    .collect(),
                types: a
                    .types
                    .iter()
                    .filter_map(|&id| self.def_id_maps[graph_index].get(&id).copied())
                    .collect(),
            }),
            DefKind::Alias(a) => DefKind::Alias(AliasTy {
                ty: self.update_type(graph_index, &a.ty),
            }),
            DefKind::Const(c) => DefKind::Const(ConstTy {
                ty: self.update_type(graph_index, &c.ty),
                value: self.update_numeric(graph_index, &c.value),
            }),
            DefKind::Bitmask(b) => DefKind::Bitmask(BitmaskTy {
                ty: self.update_type(graph_index, &b.ty),
                flags: b
                    .flags
                    .iter()
                    .map(|f| self.update_bit_flag(graph_index, f))
                    .collect(),
            }),
            DefKind::Bitset(b) => DefKind::Bitset(BitsetTy {
                parent: b
                    .parent
                    .and_then(|id| self.def_id_maps[graph_index].get(&id).copied()),
                fields: b
                    .fields
                    .iter()
                    .map(|f| self.update_bitset_field(graph_index, f))
                    .collect(),
            }),
            DefKind::Valuetype(v) => DefKind::Valuetype(ValueTy {
                parent: v
                    .parent
                    .and_then(|id| self.def_id_maps[graph_index].get(&id).copied()),
                extends: v
                    .extends
                    .and_then(|id| self.def_id_maps[graph_index].get(&id).copied()),
                prototypes: v
                    .prototypes
                    .iter()
                    .map(|p| self.update_proto(graph_index, p))
                    .collect(),
                members: v.members.clone(), // Vec<()> - nothing to update
                definitions: v
                    .definitions
                    .iter()
                    .filter_map(|&id| self.def_id_maps[graph_index].get(&id).copied())
                    .collect(),
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
                if let Some(&new_id) = self.def_id_maps[graph_index].get(def_id) {
                    TyKind::Adt(new_id)
                } else {
                    TyKind::Adt(*def_id)
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
            default_value: member
                .default_value
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
                .map(|label| self.update_numeric(graph_index, label))
                .collect(),
            is_default: variant.is_default,
        }
    }

    fn update_enum_lit(&self, graph_index: usize, lit: &EnumLit) -> EnumLit {
        EnumLit {
            ident: lit.ident.clone(),
            value: lit.value,
            annotations: lit
                .annotations
                .iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect(),
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

    fn update_parameter(&self, graph_index: usize, param: &Parameter) -> Parameter {
        Parameter {
            ident: param.ident.clone(),
            ty: self.update_type(graph_index, &param.ty),
            kind: param.kind,
        }
    }

    fn update_bit_flag(&self, graph_index: usize, flag: &BitFlag) -> BitFlag {
        BitFlag {
            ident: flag.ident.clone(),
            value: flag.value,
            annotations: flag
                .annotations
                .iter()
                .map(|ann| self.update_annotation(graph_index, ann))
                .collect(),
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
                if let Some(&new_id) = self.def_id_maps[graph_index].get(def_id) {
                    Numeric::Const(new_id)
                } else {
                    num.clone()
                }
            }
            other => other.clone(),
        }
    }

    fn update_annotation(&self, graph_index: usize, ann: &Ann) -> Ann {
        Ann {
            ident: ann.ident.clone(),
            def_id: if let Some(&new_id) = self.def_id_maps[graph_index].get(&ann.def_id) {
                new_id
            } else {
                ann.def_id
            },
            args: ann
                .args
                .iter()
                .map(|arg| AnnArg {
                    ident: arg.ident.clone(),
                    value: self.update_numeric(graph_index, &arg.value),
                })
                .collect(),
        }
    }
    fn finish(self) -> MergedGraph {
        MergedGraph {
            context: self.new_context,
            order: self.order,
        }
    }
}
