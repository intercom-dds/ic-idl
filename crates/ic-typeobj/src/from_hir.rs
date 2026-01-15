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

#![allow(clippy::cast_possible_wrap, clippy::wildcard_imports)]

use std::collections::{BTreeMap, BTreeSet};

use ic_alloc::graph::{DiGraph, VertexId};
use ic_hir::Context;
use ic_hir::hir::{
    AliasTy, Ann, AnnotationTy, BitmaskTy, BitsetTy, Def, DefFlags, DefId, DefKind, EnumTy, Member,
    PrimitiveTy, Ty, TyKind, UnionTy,
};
use ic_omgidl::types::xtypes::*;
use tracing::{debug, debug_span, error, trace, warn};

use crate::{annotations, util};

const SMALL_BOUND_LIMIT: usize = 256;

fn is_small_bound(bound: usize) -> bool {
    bound < SMALL_BOUND_LIMIT
}

fn truncate_to_u16(val: u64, field: &str) -> u16 {
    if val > u64::from(u16::MAX) {
        warn!(value = val, field, "value truncated to u16");
    }
    val as u16
}

fn truncate_to_i32(val: i64, field: &str) -> i32 {
    if val > i64::from(i32::MAX) || val < i64::from(i32::MIN) {
        warn!(value = val, field, "value truncated to i32");
    }
    val as i32
}

fn primitive_to_type_identifier(prim: PrimitiveTy) -> TypeIdentifier {
    match prim {
        PrimitiveTy::Bool => TypeIdentifier::TkBoolean(Empty {}),
        PrimitiveTy::Int8 => TypeIdentifier::TkInt8(Empty {}),
        PrimitiveTy::UInt8 => TypeIdentifier::TkUint8(Empty {}),
        PrimitiveTy::Char => TypeIdentifier::TkChar8(Empty {}),
        PrimitiveTy::WChar => TypeIdentifier::TkChar16(Empty {}),
        PrimitiveTy::Int16 => TypeIdentifier::TkInt16(Empty {}),
        PrimitiveTy::UInt16 => TypeIdentifier::TkUint16(Empty {}),
        PrimitiveTy::Int32 => TypeIdentifier::TkInt32(Empty {}),
        PrimitiveTy::UInt32 => TypeIdentifier::TkUint32(Empty {}),
        PrimitiveTy::Int64 => TypeIdentifier::TkInt64(Empty {}),
        PrimitiveTy::UInt64 => TypeIdentifier::TkUint64(Empty {}),
        PrimitiveTy::Float32 => TypeIdentifier::TkFloat32(Empty {}),
        PrimitiveTy::Float64 => TypeIdentifier::TkFloat64(Empty {}),
        PrimitiveTy::Float128 => TypeIdentifier::TkFloat128(Empty {}),
        PrimitiveTy::Void => TypeIdentifier::TkNone(Empty {}),
    }
}

fn is_type_object_kind(kind: &DefKind) -> bool {
    matches!(
        kind,
        DefKind::Struct(_)
            | DefKind::Except(_)
            | DefKind::Union(_)
            | DefKind::Enum(_)
            | DefKind::Alias(_)
            | DefKind::Bitmask(_)
            | DefKind::Bitset(_)
            | DefKind::Annotation(_)
            | DefKind::Valuetype(_)
    )
}

/// Cache for `TypeObject` generation, allowing reuse across multiple type
/// definitions.
pub struct TypeObjectCache<'ctx> {
    ctx: &'ctx Context,
    type_id_map: BTreeMap<DefId, TypeIdentifier>,
    complete_type_map: BTreeMap<TypeIdentifier, TypeObject>,
}

impl<'ctx> TypeObjectCache<'ctx> {
    /// Creates a new empty cache.
    #[must_use]
    pub fn new(ctx: &'ctx Context) -> Self {
        Self {
            ctx,
            type_id_map: BTreeMap::new(),
            complete_type_map: BTreeMap::new(),
        }
    }

    /// Returns the calculated `TypeIdentifier` for a `DefId`, if it exists.
    #[must_use]
    pub fn type_identifier(&self, def_id: DefId) -> Option<&TypeIdentifier> {
        self.type_id_map.get(&def_id)
    }

    /// Returns the cached `TypeObject` for a `DefId`, if it exists.
    #[must_use]
    pub fn type_object(&self, def_id: DefId) -> Option<&TypeObject> {
        self.type_identifier(def_id)
            .and_then(|v| self.complete_type_map.get(v))
    }
}

impl TypeObjectCache<'_> {
    fn lookup_type_identifier(&mut self, ty: &Ty) -> TypeIdentifier {
        match &ty.kind {
            TyKind::Primitive(prim) => primitive_to_type_identifier(*prim),
            TyKind::String { wide, bound, .. } => {
                let bound_val = bound.unwrap_or(0);
                if is_small_bound(bound_val) {
                    if *wide {
                        TypeIdentifier::TiString16Small(StringSTypeDefn {
                            bound: bound_val as u8,
                        })
                    } else {
                        TypeIdentifier::TiString8Small(StringSTypeDefn {
                            bound: bound_val as u8,
                        })
                    }
                } else if *wide {
                    TypeIdentifier::TiString16Large(StringLTypeDefn {
                        bound: bound_val as u32,
                    })
                } else {
                    TypeIdentifier::TiString8Large(StringLTypeDefn {
                        bound: bound_val as u32,
                    })
                }
            }
            TyKind::Array { ty, len, .. } => {
                let elem_id = self.lookup_type_identifier(ty);
                if is_small_bound(*len) {
                    TypeIdentifier::ArraySdefn(PlainArraySElemDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: MemberFlag::new(),
                        },
                        array_bound_seq: vec![*len as u8],
                        element_identifier: Box::new(elem_id),
                    })
                } else {
                    TypeIdentifier::ArrayLdefn(PlainArrayLElemDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: MemberFlag::new(),
                        },
                        array_bound_seq: vec![*len as u32],
                        element_identifier: Box::new(elem_id),
                    })
                }
            }
            TyKind::Sequence { ty, bound, .. } => {
                let elem_id = self.lookup_type_identifier(ty);
                let bound_val = bound.unwrap_or(0);
                if is_small_bound(bound_val) {
                    TypeIdentifier::SeqSdefn(PlainSequenceSElemDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: MemberFlag::new(),
                        },
                        bound: bound_val as u8,
                        element_identifier: Box::new(elem_id),
                    })
                } else {
                    TypeIdentifier::SeqLdefn(PlainSequenceLElemDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: MemberFlag::new(),
                        },
                        bound: bound_val as u32,
                        element_identifier: Box::new(elem_id),
                    })
                }
            }
            TyKind::Map {
                key, elem, bound, ..
            } => {
                let key_id = self.lookup_type_identifier(key);
                let elem_id = self.lookup_type_identifier(elem);
                let bound_val = bound.unwrap_or(0);
                if is_small_bound(bound_val) {
                    TypeIdentifier::MapSdefn(PlainMapSTypeDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: MemberFlag::new(),
                        },
                        bound: bound_val as u8,
                        key_flags: MemberFlag::new(),
                        key_identifier: Box::new(key_id),
                        element_identifier: Box::new(elem_id),
                    })
                } else {
                    TypeIdentifier::MapLdefn(PlainMapLTypeDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: MemberFlag::new(),
                        },
                        bound: bound_val as u32,
                        key_flags: MemberFlag::new(),
                        key_identifier: Box::new(key_id),
                        element_identifier: Box::new(elem_id),
                    })
                }
            }
            TyKind::Adt(def_id) => {
                if let Some(type_id) = self.type_id_map.get(def_id) {
                    type_id.clone()
                } else {
                    error!(
                        def_id = ?def_id,
                        name = %self.ctx.qualified_name(*def_id),
                        "type identifier not found - dependency graph may be incomplete"
                    );
                    TypeIdentifier::TkNone(Empty {})
                }
            }
            _ => TypeIdentifier::TkNone(Empty {}),
        }
    }

    fn format_build_order(
        &self,
        sccs: &[Vec<VertexId>],
        vertex_to_def: &BTreeMap<VertexId, DefId>,
    ) -> Vec<Vec<String>> {
        sccs.iter()
            .map(|scc| {
                let mut names: Vec<_> = scc
                    .iter()
                    .filter_map(|v| vertex_to_def.get(v))
                    .map(|&id| self.ctx.qualified_name(id))
                    .collect();
                names.sort();
                names
            })
            .filter(|names| !names.is_empty())
            .collect()
    }

    fn is_builtin_annotation(&self, def_id: DefId) -> bool {
        self.ctx
            .type_of(def_id)
            .flags
            .contains(DefFlags::IS_BUILTIN)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn build_dependency_graph(&self, root_id: DefId) -> DiGraph<DefId> {
        let mut graph = DiGraph::new();
        let mut id_to_vertex = BTreeMap::new();
        let mut visited = BTreeSet::new();
        self.collect_dependencies(root_id, &mut graph, &mut id_to_vertex, &mut visited);
        trace!(vertices = graph.len(), "dependency graph built");
        graph
    }

    fn collect_dependencies(
        &self,
        def_id: DefId,
        graph: &mut DiGraph<DefId>,
        id_to_vertex: &mut BTreeMap<DefId, VertexId>,
        visited: &mut BTreeSet<DefId>,
    ) {
        if visited.contains(&def_id) {
            return;
        }
        visited.insert(def_id);

        let vertex = *id_to_vertex
            .entry(def_id)
            .or_insert_with(|| graph.add_vertex(def_id));

        let def = self.ctx.type_of(def_id);

        match &def.kind {
            DefKind::Struct(s) => {
                if let Some(parent_id) = s.parent {
                    self.add_dependency(parent_id, vertex, graph, id_to_vertex, visited);
                }
            }
            DefKind::Bitset(b) => {
                if let Some(parent_id) = b.parent {
                    self.add_dependency(parent_id, vertex, graph, id_to_vertex, visited);
                }
            }
            DefKind::Valuetype(v) => {
                if let Some(parent_id) = v.parent {
                    self.add_dependency(parent_id, vertex, graph, id_to_vertex, visited);
                }
            }
            _ => {}
        }

        self.collect_annotation_dependencies(
            &def.annotations,
            vertex,
            graph,
            id_to_vertex,
            visited,
        );

        match &def.kind {
            DefKind::Struct(s) => {
                for member in &s.members {
                    self.collect_ty_dependencies(&member.ty, vertex, graph, id_to_vertex, visited);
                    self.collect_annotation_dependencies(
                        &member.annotations,
                        vertex,
                        graph,
                        id_to_vertex,
                        visited,
                    );
                }
            }
            DefKind::Except(e) => {
                for member in &e.members {
                    self.collect_ty_dependencies(&member.ty, vertex, graph, id_to_vertex, visited);
                    self.collect_annotation_dependencies(
                        &member.annotations,
                        vertex,
                        graph,
                        id_to_vertex,
                        visited,
                    );
                }
            }
            DefKind::Valuetype(v) => {
                for member in &v.members {
                    self.collect_ty_dependencies(&member.ty, vertex, graph, id_to_vertex, visited);
                    self.collect_annotation_dependencies(
                        &member.annotations,
                        vertex,
                        graph,
                        id_to_vertex,
                        visited,
                    );
                }
            }
            DefKind::Union(u) => {
                for variant in &u.variants {
                    self.collect_ty_dependencies(&variant.ty, vertex, graph, id_to_vertex, visited);
                    self.collect_annotation_dependencies(
                        &variant.annotations,
                        vertex,
                        graph,
                        id_to_vertex,
                        visited,
                    );
                }
                self.collect_ty_dependencies(&u.disc.ty, vertex, graph, id_to_vertex, visited);
                self.collect_annotation_dependencies(
                    &u.disc.annotations,
                    vertex,
                    graph,
                    id_to_vertex,
                    visited,
                );
            }
            DefKind::Alias(a) => {
                self.collect_ty_dependencies(&a.ty, vertex, graph, id_to_vertex, visited);
            }
            DefKind::Bitset(b) => {
                for field in &b.fields {
                    self.collect_ty_dependencies(&field.ty, vertex, graph, id_to_vertex, visited);
                    self.collect_annotation_dependencies(
                        &field.annotations,
                        vertex,
                        graph,
                        id_to_vertex,
                        visited,
                    );
                }
            }
            DefKind::Annotation(a) => {
                for param in &a.params {
                    self.collect_ty_dependencies(&param.ty, vertex, graph, id_to_vertex, visited);
                }
            }
            _ => {}
        }
    }

    fn collect_ty_dependencies(
        &self,
        ty: &Ty,
        source_vertex: VertexId,
        graph: &mut DiGraph<DefId>,
        id_to_vertex: &mut BTreeMap<DefId, VertexId>,
        visited: &mut BTreeSet<DefId>,
    ) {
        match &ty.kind {
            TyKind::Adt(def_id) => {
                self.add_dependency(*def_id, source_vertex, graph, id_to_vertex, visited);
            }
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                self.collect_ty_dependencies(ty, source_vertex, graph, id_to_vertex, visited);
            }
            TyKind::Map { key, elem, .. } => {
                self.collect_ty_dependencies(key, source_vertex, graph, id_to_vertex, visited);
                self.collect_ty_dependencies(elem, source_vertex, graph, id_to_vertex, visited);
            }
            _ => {}
        }
    }

    fn collect_annotation_dependencies(
        &self,
        annotations: &[Ann],
        source_vertex: VertexId,
        graph: &mut DiGraph<DefId>,
        id_to_vertex: &mut BTreeMap<DefId, VertexId>,
        visited: &mut BTreeSet<DefId>,
    ) {
        for ann in annotations {
            if let Some(def_id) = ann.def_id
                && !self.is_builtin_annotation(def_id)
            {
                self.add_dependency(def_id, source_vertex, graph, id_to_vertex, visited);
            }
        }
    }

    fn add_dependency(
        &self,
        target_id: DefId,
        source_vertex: VertexId,
        graph: &mut DiGraph<DefId>,
        id_to_vertex: &mut BTreeMap<DefId, VertexId>,
        visited: &mut BTreeSet<DefId>,
    ) {
        let target_vertex = *id_to_vertex
            .entry(target_id)
            .or_insert_with(|| graph.add_vertex(target_id));

        graph.add_edge(source_vertex, target_vertex);

        self.collect_dependencies(target_id, graph, id_to_vertex, visited);
    }

    fn build(&mut self, def_id: DefId) -> TypeDefinition {
        debug!("building dependency graph");
        let graph = self.build_dependency_graph(def_id);
        let sccs = graph.scc_kosaraju();
        let vertex_to_def: BTreeMap<VertexId, DefId> =
            graph.vertices().map(|(v, &d)| (v, d)).collect();

        debug!(
            scc_count = sccs.len(),
            "found strongly connected components"
        );

        if tracing::enabled!(tracing::Level::TRACE) {
            let build_order = self.format_build_order(&sccs, &vertex_to_def);
            trace!(?build_order);
        }

        for scc in &sccs {
            self.process_scc(scc, &vertex_to_def);
        }

        let type_id = self
            .type_id_map
            .get(&def_id)
            .cloned()
            .expect("TypeIdentifier should exist after processing all SCCs");

        let mut type_def = TypeDefinition::new();
        type_def.type_name = self.ctx.qualified_name(def_id);
        type_def.type_info.complete.typeid_with_size.type_id = type_id.clone();

        if let Some(type_obj) = self.complete_type_map.get(&type_id) {
            type_def
                .type_info
                .complete
                .typeid_with_size
                .typeobject_serialized_size = util::type_object_size(type_obj) as u32;

            if let TypeObject::Complete(complete) = type_obj {
                let minimal = util::complete_to_minimal(complete.clone());
                let minimal_obj = TypeObject::Minimal(minimal);
                let minimal_id = util::equivalence_hash(&minimal_obj);

                type_def.type_info.minimal.typeid_with_size.type_id = minimal_id.clone();
                type_def
                    .type_info
                    .minimal
                    .typeid_with_size
                    .typeobject_serialized_size = util::type_object_size(&minimal_obj) as u32;

                let mut complete_deps = BTreeSet::new();
                let mut minimal_deps = BTreeSet::new();
                collect_type_dependencies(complete, &mut complete_deps, &mut minimal_deps);

                type_def.type_info.complete.dependent_typeids = complete_deps
                    .into_iter()
                    .map(|type_id| {
                        let size = self
                            .complete_type_map
                            .get(&type_id)
                            .map_or(0, |obj| util::type_object_size(obj) as u32);
                        TypeIdentifierWithSize {
                            type_id,
                            typeobject_serialized_size: size,
                        }
                    })
                    .collect();
                type_def.type_info.complete.dependent_typeid_count =
                    type_def.type_info.complete.dependent_typeids.len() as i32;

                type_def.type_info.minimal.dependent_typeids = minimal_deps
                    .into_iter()
                    .map(|type_id| {
                        let size = self
                            .complete_type_map
                            .get(&type_id)
                            .map_or(0, |obj| util::type_object_size(obj) as u32);
                        TypeIdentifierWithSize {
                            type_id,
                            typeobject_serialized_size: size,
                        }
                    })
                    .collect();
                type_def.type_info.minimal.dependent_typeid_count =
                    type_def.type_info.minimal.dependent_typeids.len() as i32;
            }
        }

        for (type_id, type_obj) in &self.complete_type_map {
            type_def.type_objects.push(TypeIdentifierTypeObjectPair {
                type_identifier: type_id.clone(),
                type_object: type_obj.clone(),
            });
        }

        type_def
    }

    fn is_self_referential(&self, def_id: DefId) -> bool {
        let def = self.ctx.type_of(def_id);
        match &def.kind {
            DefKind::Struct(s) => s
                .members
                .iter()
                .any(|m| self.type_references_def(&m.ty, def_id)),
            DefKind::Union(u) => u
                .variants
                .iter()
                .any(|v| self.type_references_def(&v.ty, def_id)),
            _ => false,
        }
    }

    fn type_references_def(&self, ty: &Ty, target_def_id: DefId) -> bool {
        match &ty.kind {
            TyKind::Adt(id) => {
                if *id == target_def_id {
                    return true;
                }
                // Follow through typedefs
                let def = self.ctx.type_of(*id);
                if let DefKind::Alias(alias) = &def.kind {
                    self.type_references_def(&alias.ty, target_def_id)
                } else {
                    false
                }
            }
            TyKind::Sequence { ty, .. } | TyKind::Array { ty, .. } => {
                self.type_references_def(ty, target_def_id)
            }
            TyKind::Map { key, elem, .. } => {
                self.type_references_def(key, target_def_id)
                    || self.type_references_def(elem, target_def_id)
            }
            _ => false,
        }
    }

    fn process_scc(&mut self, scc: &[VertexId], vertex_to_def: &BTreeMap<VertexId, DefId>) {
        if scc.is_empty() {
            return;
        }

        let mut def_ids: Vec<DefId> = scc
            .iter()
            .filter_map(|&v| vertex_to_def.get(&v).copied())
            .filter(|&def_id| is_type_object_kind(&self.ctx.type_of(def_id).kind))
            .collect();

        if def_ids.is_empty() {
            return;
        }

        def_ids.sort_by_key(|&def_id| self.ctx.qualified_name(def_id));

        if def_ids.len() == 1 && !self.is_self_referential(def_ids[0]) {
            let def_id = def_ids[0];
            let name = self.ctx.qualified_name(def_id);
            let Some(type_obj) = self.create_type_object(def_id) else {
                return;
            };

            let type_id = util::equivalence_hash(&type_obj);
            debug!(
                name = %name,
                type_id = %util::format_type_id(&type_id),
                "created type object"
            );
            self.complete_type_map.insert(type_id.clone(), type_obj);
            self.type_id_map.insert(def_id, type_id);
        } else {
            self.process_multi_type_scc(&def_ids);
        }
    }

    fn process_multi_type_scc(&mut self, def_ids: &[DefId]) {
        let names: Vec<_> = def_ids
            .iter()
            .map(|&id| self.ctx.qualified_name(id))
            .collect();
        debug!(types = ?names, "processing mutually recursive SCC");

        // Create placeholder identifiers with zeroed hashes for each type in the SCC
        let scc_len = def_ids.len() as i32;
        let placeholders: Vec<_> = (0..def_ids.len())
            .map(|i| {
                TypeIdentifier::ScComponentId(StronglyConnectedComponentId {
                    sc_component_id: TypeObjectHashId::EkComplete([0; 14]),
                    scc_length: scc_len,
                    scc_index: (i + 1) as i32,
                })
            })
            .collect();

        for (&def_id, placeholder) in def_ids.iter().zip(&placeholders) {
            self.type_id_map.insert(def_id, placeholder.clone());
        }

        let type_objects: Vec<_> = def_ids
            .iter()
            .filter_map(|&def_id| self.create_type_object(def_id))
            .collect();

        if type_objects.is_empty() {
            return;
        }

        // Compute the real SCC hash from all type objects
        let scc_base = util::scc_equivalence_hash(&type_objects, EK_COMPLETE);

        // Build mapping from placeholder -> final identifier
        let placeholder_to_final: BTreeMap<_, _> = placeholders
            .iter()
            .enumerate()
            .map(|(i, placeholder)| {
                let final_id = TypeIdentifier::ScComponentId(StronglyConnectedComponentId {
                    sc_component_id: scc_base.sc_component_id,
                    scc_length: scc_len,
                    scc_index: (i + 1) as i32,
                });
                (placeholder.clone(), final_id)
            })
            .collect();

        // Update type_id_map with final identifiers
        for (i, &def_id) in def_ids.iter().enumerate() {
            let final_id = TypeIdentifier::ScComponentId(StronglyConnectedComponentId {
                sc_component_id: scc_base.sc_component_id,
                scc_length: scc_len,
                scc_index: (i + 1) as i32,
            });
            trace!(
                name = %self.ctx.qualified_name(def_id),
                type_id = %util::format_type_id(&final_id),
                "assigned SCC type identifier"
            );
            self.type_id_map.insert(def_id, final_id);
        }

        // Replace placeholders in type objects and insert into complete_type_map
        for (i, mut type_obj) in type_objects.into_iter().enumerate() {
            util::update_type_object_identifiers(&mut type_obj, &placeholder_to_final);

            let final_id = TypeIdentifier::ScComponentId(StronglyConnectedComponentId {
                sc_component_id: scc_base.sc_component_id,
                scc_length: scc_len,
                scc_index: (i + 1) as i32,
            });
            self.complete_type_map.insert(final_id, type_obj);
        }

        let scc_type_id = TypeIdentifier::ScComponentId(scc_base);
        debug!(
            scc_hash = %util::format_type_id(&scc_type_id),
            "completed SCC processing",
        );
    }

    #[tracing::instrument(
        level = "debug",
        skip(self),
        fields(name = %self.ctx.qualified_name(def_id)),
    )]
    fn create_type_object(&mut self, def_id: DefId) -> Option<TypeObject> {
        let def = self.ctx.type_of(def_id);
        let complete = match &def.kind {
            DefKind::Struct(s) => CompleteTypeObject::StructType(
                self.create_complete_struct_type(def, s.parent, &s.members),
            ),
            DefKind::Except(e) => CompleteTypeObject::StructType(
                self.create_complete_struct_type(def, None, &e.members),
            ),
            DefKind::Valuetype(v) => CompleteTypeObject::StructType(
                self.create_complete_struct_type(def, v.parent, &v.members),
            ),
            DefKind::Union(u) => {
                CompleteTypeObject::UnionType(self.create_complete_union_type(def, u))
            }
            DefKind::Enum(e) => {
                CompleteTypeObject::EnumeratedType(self.create_complete_enum_type(def, e))
            }
            DefKind::Alias(a) => {
                CompleteTypeObject::AliasType(self.create_complete_alias_type(def, a))
            }
            DefKind::Bitmask(b) => {
                CompleteTypeObject::BitmaskType(self.create_complete_bitmask_type(def, b))
            }
            DefKind::Bitset(b) => {
                CompleteTypeObject::BitsetType(self.create_complete_bitset_type(def, b))
            }
            DefKind::Annotation(a) => {
                CompleteTypeObject::AnnotationType(self.create_complete_annotation_type(def, a))
            }
            DefKind::Module(_) | DefKind::Interface(_) | DefKind::Const(_) | DefKind::Decl(_) => {
                return None;
            }
        };

        Some(TypeObject::Complete(complete))
    }

    fn create_complete_annotation_type(
        &mut self,
        def: &Def,
        ann_ty: &AnnotationTy,
    ) -> CompleteAnnotationType {
        let mut ty = CompleteAnnotationType::new();
        ty.annotation_flag = AnnotationTypeFlag::new();
        ty.header.annotation_name = self.ctx.qualified_name(def.id);

        for param in &ann_ty.params {
            let mut complete_param = CompleteAnnotationParameter::new();
            complete_param.common.member_type_id = self.lookup_type_identifier(&param.ty);
            complete_param.common.member_flags = MemberFlag::new();
            complete_param.name.clone_from(&param.ident.name);

            if let Some(default_value) = &param.default {
                complete_param.default_value =
                    annotations::numeric_to_annotation_value(self.ctx, default_value);
            }
            ty.member_seq.push(complete_param);
        }

        ty
    }

    fn create_complete_struct_type(
        &mut self,
        def: &Def,
        parent: Option<DefId>,
        members: &[Member],
    ) -> CompleteStructType {
        let mut ty = CompleteStructType::new();
        ty.struct_flags = annotations::get_struct_flags(def);

        if let Some(parent_id) = parent {
            ty.header.base_type = self.lookup_type_identifier(&Ty {
                kind: TyKind::Adt(parent_id),
                span: def.span,
            });
        }

        let mut current_id = annotations::get_parents_last_member_id(self.ctx, def.id);
        for member in members {
            current_id = annotations::get_member_id(self.ctx, &member.annotations, current_id);

            let mut complete_member = CompleteStructMember::new();
            complete_member.common.member_id = current_id;
            complete_member.common.member_flags =
                annotations::get_member_flags(self.ctx, &member.annotations);
            complete_member.common.member_type_id = self.lookup_type_identifier(&member.ty);
            complete_member.detail = annotations::create_complete_member_detail(
                self.ctx,
                &self.type_id_map,
                &member.ident.name,
                &member.annotations,
            );

            ty.member_seq.push(complete_member);
        }

        ty.header.detail =
            annotations::create_complete_type_detail(self.ctx, &self.type_id_map, def);
        ty
    }

    fn create_complete_union_type(&mut self, def: &Def, union_ty: &UnionTy) -> CompleteUnionType {
        let mut ty = CompleteUnionType::new();
        ty.union_flags = annotations::get_union_flags(def);

        let disc_type_id = self.lookup_type_identifier(&union_ty.disc.ty);
        ty.discriminator.common.type_id = disc_type_id;
        ty.discriminator.common.member_flags =
            annotations::get_member_flags(self.ctx, &union_ty.disc.annotations)
                | MemberFlag::IS_MUST_UNDERSTAND;
        annotations::populate_discriminator_annotations(
            self.ctx,
            &self.type_id_map,
            &union_ty.disc.annotations,
            &mut ty.discriminator,
        );

        let mut current_id = 0;
        for variant in &union_ty.variants {
            current_id = annotations::get_member_id(self.ctx, &variant.annotations, current_id);

            let mut complete_member = CompleteUnionMember::new();
            complete_member.common.member_id = current_id;
            complete_member.common.member_flags =
                annotations::get_member_flags(self.ctx, &variant.annotations);

            if variant.is_default {
                complete_member.common.member_flags |= MemberFlag::IS_DEFAULT;
            }

            complete_member.common.type_id = self.lookup_type_identifier(&variant.ty);
            complete_member.detail = annotations::create_complete_member_detail(
                self.ctx,
                &self.type_id_map,
                &variant.ident.name,
                &variant.annotations,
            );

            for label in &variant.labels {
                let val = self.ctx.integer_value(&label.value);
                let truncated = truncate_to_i32(val, &variant.ident.name);
                complete_member.common.label_seq.push(truncated);
            }

            ty.member_seq.push(complete_member);
        }

        ty.header.detail =
            annotations::create_complete_type_detail(self.ctx, &self.type_id_map, def);
        ty
    }

    fn create_complete_enum_type(&mut self, def: &Def, enum_ty: &EnumTy) -> CompleteEnumeratedType {
        let mut ty = CompleteEnumeratedType::new();
        ty.enum_flags = annotations::get_enumerated_flags(def);
        ty.header.common.bit_bound = annotations::get_bit_bound_for_enum(def);

        for field_id in &enum_ty.fields {
            let field_def = self.ctx.type_of(*field_id);
            let DefKind::Const(const_ty) = &field_def.kind else {
                continue;
            };

            let mut literal = CompleteEnumeratedLiteral::new();
            literal.common.flags = annotations::get_literal_flags(self.ctx, &field_def.annotations);

            let val = self.ctx.integer_value(&const_ty.value);
            literal.common.value = truncate_to_i32(val, &field_def.ident.name);

            literal.detail = annotations::create_complete_member_detail(
                self.ctx,
                &self.type_id_map,
                &field_def.ident.name,
                &field_def.annotations,
            );
            ty.literal_seq.push(literal);
        }

        ty.literal_seq.sort_by_key(|lit| lit.common.value);
        ty.header.detail =
            annotations::create_complete_type_detail(self.ctx, &self.type_id_map, def);
        ty
    }

    fn create_complete_alias_type(&mut self, def: &Def, alias_ty: &AliasTy) -> CompleteAliasType {
        let mut ty = CompleteAliasType::new();
        ty.alias_flags = TypeFlag::new();
        ty.body.common.related_type = self.lookup_type_identifier(&alias_ty.ty);
        ty.body.common.related_flags = MemberFlag::new();
        ty.header.detail =
            annotations::create_complete_type_detail(self.ctx, &self.type_id_map, def);
        ty
    }

    fn create_complete_bitmask_type(
        &mut self,
        def: &Def,
        bitmask_ty: &BitmaskTy,
    ) -> CompleteBitmaskType {
        let mut ty = CompleteBitmaskType::new();
        ty.bitmask_flags = annotations::get_bitmask_flags(def);
        ty.header.common.bit_bound = annotations::get_bit_bound_for_bitmask(def);

        for flag_id in &bitmask_ty.flags {
            let flag_def = self.ctx.type_of(*flag_id);
            let DefKind::Const(const_ty) = &flag_def.kind else {
                continue;
            };

            let mut flag = CompleteBitflag::new();
            flag.common.flags = annotations::get_literal_flags(self.ctx, &flag_def.annotations);

            let val = self.ctx.unsigned_value(&const_ty.value);
            flag.common.position = truncate_to_u16(val, &flag_def.ident.name);

            flag.detail = annotations::create_complete_member_detail(
                self.ctx,
                &self.type_id_map,
                &flag_def.ident.name,
                &flag_def.annotations,
            );
            ty.flag_seq.push(flag);
        }

        ty.header.detail =
            annotations::create_complete_type_detail(self.ctx, &self.type_id_map, def);
        ty
    }

    fn create_complete_bitset_type(
        &mut self,
        def: &Def,
        bitset_ty: &BitsetTy,
    ) -> CompleteBitsetType {
        let mut ty = CompleteBitsetType::new();
        ty.bitset_flags = annotations::get_bitset_flags(def);

        for field in &bitset_ty.fields {
            let mut bitfield = CompleteBitfield::new();
            bitfield.common.position =
                annotations::get_bitfield_position(self.ctx, &field.annotations);
            bitfield.common.flags = annotations::get_member_flags(self.ctx, &field.annotations);
            bitfield.common.bitcount = field.size as u8;
            bitfield.common.holder_type = util::get_holder_type(&field.ty);
            bitfield.detail = annotations::create_complete_member_detail(
                self.ctx,
                &self.type_id_map,
                &field.ident.name,
                &field.annotations,
            );

            ty.field_seq.push(bitfield);
        }

        ty.header.detail =
            annotations::create_complete_type_detail(self.ctx, &self.type_id_map, def);
        ty
    }
}

fn collect_type_dependencies(
    complete: &CompleteTypeObject,
    complete_deps: &mut BTreeSet<TypeIdentifier>,
    minimal_deps: &mut BTreeSet<TypeIdentifier>,
) {
    match complete {
        CompleteTypeObject::AliasType(alias) => {
            complete_deps.insert(alias.body.common.related_type.clone());
            minimal_deps.insert(alias.body.common.related_type.clone());
        }
        CompleteTypeObject::AnnotationType(ann) => {
            for param in &ann.member_seq {
                complete_deps.insert(param.common.member_type_id.clone());
                minimal_deps.insert(param.common.member_type_id.clone());
            }
        }
        CompleteTypeObject::StructType(struct_ty) => {
            if !matches!(struct_ty.header.base_type, TypeIdentifier::TkNone(_)) {
                complete_deps.insert(struct_ty.header.base_type.clone());
                minimal_deps.insert(struct_ty.header.base_type.clone());
            }
            for member in &struct_ty.member_seq {
                complete_deps.insert(member.common.member_type_id.clone());
                minimal_deps.insert(member.common.member_type_id.clone());
            }
        }
        CompleteTypeObject::UnionType(union) => {
            if !matches!(union.header.base_type, TypeIdentifier::TkNone(_)) {
                complete_deps.insert(union.header.base_type.clone());
                minimal_deps.insert(union.header.base_type.clone());
            }
            complete_deps.insert(union.discriminator.common.type_id.clone());
            minimal_deps.insert(union.discriminator.common.type_id.clone());
            for member in &union.member_seq {
                complete_deps.insert(member.common.type_id.clone());
                minimal_deps.insert(member.common.type_id.clone());
            }
        }
        CompleteTypeObject::SequenceType(seq) => {
            complete_deps.insert(seq.element.common.type_.clone());
            minimal_deps.insert(seq.element.common.type_.clone());
        }
        CompleteTypeObject::ArrayType(arr) => {
            complete_deps.insert(arr.element.common.type_.clone());
            minimal_deps.insert(arr.element.common.type_.clone());
        }
        CompleteTypeObject::MapType(map) => {
            complete_deps.insert(map.key.common.type_.clone());
            complete_deps.insert(map.element.common.type_.clone());
            minimal_deps.insert(map.key.common.type_.clone());
            minimal_deps.insert(map.element.common.type_.clone());
        }
        _ => {}
    }
}

pub(crate) fn type_definition(def_id: DefId, cache: &mut TypeObjectCache<'_>) -> TypeDefinition {
    let _span = debug_span!("typeobj", root = %cache.ctx.qualified_name(def_id)).entered();
    let type_def = cache.build(def_id);
    debug!(
        type_objects = type_def.type_objects.len(),
        "type definition complete",
    );
    trace!(?type_def);
    type_def
}
