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

use ic_hir::hir::{DefFlags, DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir::{Context, ResolvedGraph};
use ic_hir_analysis::annotation::is_external;
use tracing::{debug, debug_span};

fn analyze_def(
    def_id: DefId,
    context: &mut Context,
    seen: &mut HashSet<DefId>,
    changed: &mut bool,
) {
    if !seen.insert(def_id) {
        return;
    }

    {
        let def = context.definitions.get_mut(def_id);

        // Skip built-in types
        if def.flags.contains(DefFlags::IS_BUILTIN) {
            return;
        }
    }

    let def = context.definitions.get(def_id);

    // Collect parent `DefId`s, member types, and check for indirect annotations
    let (parents, types, has_indirect): (Vec<DefId>, Vec<Ty>, bool) = match &def.kind {
        DefKind::Struct(s) => (
            s.parent.into_iter().map(|p| p.def_id).collect(),
            s.members.iter().map(|m| m.ty.clone()).collect(),
            s.members.iter().any(|member| is_external(context, member)),
        ),
        DefKind::Union(u) => (
            vec![],
            u.variants.iter().map(|v| v.ty.clone()).collect(),
            u.variants
                .iter()
                .any(|variant| is_external(context, variant)),
        ),
        DefKind::Valuetype(v) => (
            v.parent.into_iter().map(|p| p.def_id).collect(),
            v.members.iter().map(|m| m.ty.clone()).collect(),
            v.members.iter().any(|member| is_external(context, member)),
        ),
        DefKind::Alias(a) => (vec![], vec![a.ty.clone()], is_external(context, def)),
        DefKind::Except(e) => (
            vec![],
            e.members.iter().map(|m| m.ty.clone()).collect(),
            e.members.iter().any(|member| is_external(context, member)),
        ),
        DefKind::Const(c) => (vec![], vec![c.ty.clone()], false),
        DefKind::Module(_)
        | DefKind::Interface(_)
        | DefKind::Enum(_)
        | DefKind::Bitmask(_)
        | DefKind::Annotation(_)
        | DefKind::Decl(_)
        | DefKind::Bitset(_) => (vec![], vec![], false),
    };

    // Indirect members imply heap allocation, so the type is not trivial
    if has_indirect {
        let current_def = context.definitions.get_mut(def_id);
        if current_def.flags.contains(DefFlags::IS_TRIVIAL) {
            current_def.flags.unset(DefFlags::IS_TRIVIAL);
            *changed = true;
        }
    }

    // Analyze parent types and propagate flags
    for parent_id in parents {
        check_def(parent_id, def_id, context, seen, changed);
    }

    // Analyze member types and propagate flags
    for ty in types {
        analyze_type(&ty, def_id, context, seen, changed);
    }
}

fn check_def(
    ref_def_id: DefId,
    parent_def_id: DefId,
    context: &mut Context,
    seen: &mut HashSet<DefId>,
    changed: &mut bool,
) {
    analyze_def(ref_def_id, context, seen, changed);

    let ref_def = context.definitions.get(ref_def_id);
    let ref_is_trivial = ref_def.flags.contains(DefFlags::IS_TRIVIAL);
    let ref_total_order = ref_def.flags.contains(DefFlags::TOTAL_ORDER);
    let parent_def = context.definitions.get_mut(parent_def_id);

    if !ref_is_trivial && parent_def.flags.contains(DefFlags::IS_TRIVIAL) {
        parent_def.flags.unset(DefFlags::IS_TRIVIAL);
        *changed = true;
    }
    if !ref_total_order && parent_def.flags.contains(DefFlags::TOTAL_ORDER) {
        parent_def.flags.unset(DefFlags::TOTAL_ORDER);
        *changed = true;
    }
}

fn analyze_type(
    ty: &Ty,
    parent_def_id: DefId,
    context: &mut Context,
    seen: &mut HashSet<DefId>,
    changed: &mut bool,
) {
    match &ty.kind {
        TyKind::Primitive(prim) => match prim {
            PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128 => {
                let def = context.definitions.get_mut(parent_def_id);
                if def.flags.contains(DefFlags::TOTAL_ORDER) {
                    def.flags.unset(DefFlags::TOTAL_ORDER);
                    *changed = true;
                }
            }
            _ => {}
        },

        TyKind::Any | TyKind::String { .. } => {
            let def = context.definitions.get_mut(parent_def_id);
            if def.flags.contains(DefFlags::IS_TRIVIAL) {
                def.flags.unset(DefFlags::IS_TRIVIAL);
                *changed = true;
            }
        }

        TyKind::Sequence { ty: inner, .. } => {
            let def = context.definitions.get_mut(parent_def_id);
            if def.flags.contains(DefFlags::IS_TRIVIAL) {
                def.flags.unset(DefFlags::IS_TRIVIAL);
                *changed = true;
            }
            analyze_type(inner, parent_def_id, context, seen, changed);
        }

        TyKind::Array { ty: inner, .. } => {
            analyze_type(inner, parent_def_id, context, seen, changed);
        }

        TyKind::Map { key, elem, .. } => {
            let def = context.definitions.get_mut(parent_def_id);
            if def.flags.contains(DefFlags::IS_TRIVIAL) {
                def.flags.unset(DefFlags::IS_TRIVIAL);
                *changed = true;
            }
            analyze_type(key, parent_def_id, context, seen, changed);
            analyze_type(elem, parent_def_id, context, seen, changed);
        }

        TyKind::Adt(ref_def_id) => {
            check_def(*ref_def_id, parent_def_id, context, seen, changed);
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

    // Set flags opimistically once so we don't overwrite computed `false` states during iteration.
    for def_id in &def_ids {
        let def = hir.context.definitions.get_mut(*def_id);
        if !def.flags.contains(DefFlags::IS_BUILTIN) {
            def.flags.set(DefFlags::IS_TRIVIAL);
            def.flags.set(DefFlags::TOTAL_ORDER);

            if def.flags.contains(DefFlags::IS_CIRCULAR) {
                def.flags.unset(DefFlags::IS_TRIVIAL);
            }
        }
    }

    // Iterate over types as long as flags change
    let mut changed = true;
    while changed {
        changed = false;
        // The seen set is reset for each full iteration over the graph.
        // This allows cyclic dependencies to be re-evaluated against the newest flag states
        // while still preventing infinite recursion within a single depth-first chain.
        let mut seen = HashSet::new();

        for def_id in &def_ids {
            analyze_def(*def_id, &mut hir.context, &mut seen, &mut changed);
        }
    }

    hir
}
