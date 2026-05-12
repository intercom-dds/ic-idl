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

use ic_alloc::graph::{DiGraph, VertexId};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind, PrimitiveTy, Ty, TyKind};

pub struct TypeGroup {
    pub types: Vec<DefId>,
}

pub fn group_types_by_scc(hir: &ResolvedGraph, types: &[DefId]) -> Vec<TypeGroup> {
    let mut graph = DiGraph::new();
    let mut def_to_vertex: HashMap<DefId, VertexId> = HashMap::new();
    let mut vertex_to_def: HashMap<VertexId, DefId> = HashMap::new();

    for &def_id in types {
        let vertex = graph.add_vertex(def_id);
        def_to_vertex.insert(def_id, vertex);
        vertex_to_def.insert(vertex, def_id);
    }

    for &def_id in types {
        let deps = collect_type_dependencies(hir, def_id);
        let from_vertex = def_to_vertex[&def_id];

        for dep_id in deps {
            if let Some(&to_vertex) = def_to_vertex.get(&dep_id) {
                graph.add_edge(from_vertex, to_vertex);
            }
        }
    }

    let sccs = graph.scc_tarjan();
    let mut groups = vec![];

    for scc in sccs {
        if scc.is_empty() {
            continue;
        }

        let type_ids: Vec<DefId> = scc.iter().map(|&v| vertex_to_def[&v]).collect();
        groups.push(TypeGroup { types: type_ids });
    }

    groups
}

pub fn collect_type_dependencies(hir: &ResolvedGraph, def_id: DefId) -> HashSet<DefId> {
    let mut deps = HashSet::new();
    let members = collect_struct_members(hir, def_id);

    for member in members {
        find_type_dependencies(hir, &member.ty, &mut deps);
    }

    deps.remove(&def_id);
    deps
}

pub struct MemberInfo {
    pub name: String,
    pub ty: Ty,
}

pub fn collect_struct_members(hir: &ResolvedGraph, def_id: DefId) -> Vec<MemberInfo> {
    let def = hir.context.definitions.get(def_id);
    let mut members = Vec::new();

    match &def.kind {
        DefKind::Struct(struct_ty) => {
            if let Some(parent) = struct_ty.parent {
                members.extend(collect_struct_members(hir, parent.def_id));
            }

            for member in &struct_ty.members {
                members.push(MemberInfo {
                    name: member.ident.name.clone(),
                    ty: member.ty.clone(),
                });
            }
        }
        DefKind::Except(except_ty) => {
            for member in &except_ty.members {
                members.push(MemberInfo {
                    name: member.ident.name.clone(),
                    ty: member.ty.clone(),
                });
            }
        }
        DefKind::Valuetype(valuetype_ty) => {
            if let Some(parent) = valuetype_ty.parent {
                members.extend(collect_struct_members(hir, parent.def_id));
            }

            for member in &valuetype_ty.members {
                members.push(MemberInfo {
                    name: member.ident.name.clone(),
                    ty: member.ty.clone(),
                });
            }
        }
        DefKind::Union(union_ty) => {
            for variant in &union_ty.variants {
                members.push(MemberInfo {
                    name: variant.ident.name.clone(),
                    ty: variant.ty.clone(),
                });
            }
        }
        _ => {}
    }

    members
}

pub fn resolve_typedef(hir: &ResolvedGraph, def_id: DefId) -> DefId {
    let def = hir.context.definitions.get(def_id);
    if let DefKind::Alias(alias_ty) = &def.kind
        && let TyKind::Adt(aliased_id) = alias_ty.ty.kind
    {
        return resolve_typedef(hir, aliased_id);
    }
    def_id
}

pub fn is_proto_type(hir: &ResolvedGraph, def_id: DefId) -> bool {
    let def = hir.context.definitions.get(def_id);
    matches!(
        def.kind,
        DefKind::Struct(_)
            | DefKind::Except(_)
            | DefKind::Union(_)
            | DefKind::Enum(_)
            | DefKind::Valuetype(_)
    )
}

fn find_type_dependencies(hir: &ResolvedGraph, ty: &Ty, deps: &mut HashSet<DefId>) {
    match &ty.kind {
        TyKind::Adt(def_id) => {
            let resolved_id = resolve_typedef(hir, *def_id);
            let resolved_def = hir.context.definitions.get(resolved_id);

            if let DefKind::Alias(alias_ty) = &resolved_def.kind {
                find_type_dependencies(hir, &alias_ty.ty, deps);
            } else if is_proto_type(hir, resolved_id) {
                deps.insert(resolved_id);
            }
        }
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. }
            if !matches!(
                ty.kind,
                TyKind::Primitive(PrimitiveTy::UInt8 | PrimitiveTy::Int8)
            ) =>
        {
            find_type_dependencies(hir, ty, deps);
        }
        TyKind::Map { key, elem, .. } => {
            find_type_dependencies(hir, key, deps);
            find_type_dependencies(hir, elem, deps);
        }
        _ => {}
    }
}
