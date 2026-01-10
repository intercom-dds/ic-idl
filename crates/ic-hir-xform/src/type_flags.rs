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

//! Marks types with `IS_TRIVIAL` and `TOTAL_ORDER` flags.
//!
//! This transformation analyzes types to determine:
//! - `IS_TRIVIAL`: Types that consist only of primitive types and arrays
//! - `TOTAL_ORDER`: Types whose members can form a well-ordered set
//!
//! The algorithm works by optimistically setting both flags on first visit,
//! then clearing them as we discover disqualifying properties. This handles
//! recursive types correctly: when we revisit a node in a cycle, we skip it
//! (it already has flags set), and after the full traversal, flags propagate
//! back up to clear any that should be unset.

use std::collections::HashSet;

use ic_hir::hir::{Ann, DefFlags, DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir::{Context, ResolvedGraph};
use tracing::{debug, debug_span};

fn has_external_annotation(annotations: &[Ann]) -> bool {
    annotations.iter().any(|a| a.ident.name == "external")
}

fn analyze_def(def_id: DefId, context: &mut Context, seen: &mut HashSet<DefId>) {
    if !seen.insert(def_id) {
        return;
    }

    let def = context.definitions.get_mut(def_id);

    // Skip built-in types
    if def.flags.contains(DefFlags::IS_BUILTIN) {
        return;
    }

    def.flags.set(DefFlags::IS_TRIVIAL);
    def.flags.set(DefFlags::TOTAL_ORDER);

    if def.flags.contains(DefFlags::IS_CIRCULAR) {
        def.flags.unset(DefFlags::IS_TRIVIAL);
    }

    // Collect parent `DefId`s, member types, and check for @external annotations
    let (parents, types, has_external): (Vec<DefId>, Vec<Ty>, bool) = match &def.kind {
        DefKind::Struct(s) => (
            s.parent.into_iter().collect(),
            s.members.iter().map(|m| m.ty.clone()).collect(),
            s.members
                .iter()
                .any(|m| has_external_annotation(&m.annotations)),
        ),
        DefKind::Union(u) => (
            vec![],
            u.variants.iter().map(|v| v.ty.clone()).collect(),
            u.variants
                .iter()
                .any(|v| has_external_annotation(&v.annotations)),
        ),
        DefKind::Valuetype(v) => (
            v.parent.into_iter().collect(),
            v.members.iter().map(|m| m.ty.clone()).collect(),
            v.members
                .iter()
                .any(|m| has_external_annotation(&m.annotations)),
        ),
        DefKind::Alias(a) => (vec![], vec![a.ty.clone()], false),
        DefKind::Except(e) => (
            vec![],
            e.members.iter().map(|m| m.ty.clone()).collect(),
            e.members
                .iter()
                .any(|m| has_external_annotation(&m.annotations)),
        ),
        DefKind::Module(_)
        | DefKind::Interface(_)
        | DefKind::Enum(_)
        | DefKind::Bitmask(_)
        | DefKind::Const(_)
        | DefKind::Annotation(_)
        | DefKind::Decl(_)
        | DefKind::Bitset(_) => (vec![], vec![], false),
    };

    // @external members imply heap allocation (Box<T>), so the type is not trivial
    if has_external {
        context
            .definitions
            .get_mut(def_id)
            .flags
            .unset(DefFlags::IS_TRIVIAL);
    }

    // Analyze parent types and propagate flags
    for parent_id in parents {
        check_def(parent_id, def_id, context, seen);
    }

    // Analyze member types and propagate flags
    for ty in types {
        analyze_type(&ty, def_id, context, seen);
    }
}

fn check_def(
    ref_def_id: DefId,
    parent_def_id: DefId,
    context: &mut Context,
    seen: &mut HashSet<DefId>,
) {
    analyze_def(ref_def_id, context, seen);

    let ref_def = context.definitions.get(ref_def_id);
    let ref_is_trivial = ref_def.flags.contains(DefFlags::IS_TRIVIAL);
    let ref_total_order = ref_def.flags.contains(DefFlags::TOTAL_ORDER);
    let parent_def = context.definitions.get_mut(parent_def_id);

    if !ref_is_trivial {
        parent_def.flags.unset(DefFlags::IS_TRIVIAL);
    }
    if !ref_total_order {
        parent_def.flags.unset(DefFlags::TOTAL_ORDER);
    }
}

fn analyze_type(ty: &Ty, parent_def_id: DefId, context: &mut Context, seen: &mut HashSet<DefId>) {
    match &ty.kind {
        TyKind::Primitive(prim) => match prim {
            PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128 => {
                context
                    .definitions
                    .get_mut(parent_def_id)
                    .flags
                    .unset(DefFlags::TOTAL_ORDER);
            }
            _ => {}
        },

        TyKind::Any | TyKind::String { .. } => {
            context
                .definitions
                .get_mut(parent_def_id)
                .flags
                .unset(DefFlags::IS_TRIVIAL);
        }

        TyKind::Sequence { ty: inner, .. } => {
            context
                .definitions
                .get_mut(parent_def_id)
                .flags
                .unset(DefFlags::IS_TRIVIAL);
            analyze_type(inner, parent_def_id, context, seen);
        }

        TyKind::Array { ty: inner, .. } => {
            analyze_type(inner, parent_def_id, context, seen);
        }

        TyKind::Map { key, elem, .. } => {
            context
                .definitions
                .get_mut(parent_def_id)
                .flags
                .unset(DefFlags::IS_TRIVIAL);
            analyze_type(key, parent_def_id, context, seen);
            analyze_type(elem, parent_def_id, context, seen);
        }

        TyKind::Adt(ref_def_id) => {
            check_def(*ref_def_id, parent_def_id, context, seen);
        }

        TyKind::Fixed | TyKind::Null => {}
    }
}

/// Analyzes and marks types with `IS_TRIVIAL` and `TOTAL_ORDER` flags.
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    let _span = debug_span!("xform", name = "type_flags").entered();
    debug!("applying transform");

    let def_ids: Vec<DefId> = hir.context.definitions.iter().map(|(id, _)| id).collect();
    let mut seen = HashSet::new();

    for def_id in def_ids {
        analyze_def(def_id, &mut hir.context, &mut seen);
    }
    hir
}
