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

//! Typedef stripping transformation
//!
//! This transformation resolves and inlines all intermediate typedefs, effectively
//! removing all typedef definitions from the HIR tree. This is useful for target
//! languages that do not support typedefs, such as C# and Java.
//!
//! # Transformation
//!
//! The transformation performs the following steps:
//! 1. Collect all typedef definitions that can be inlined
//! 2. Pre-compute the resolved types for all typedefs
//! 3. Replace all type references (`TyKind::Adt`) that point to typedefs with
//!    their resolved underlying types
//! 4. Remove typedef definitions from the tree
//!
//! # Example
//!
//! Input IDL:
//! ```idl
//! module A {
//!     typedef long MyInt;
//!     typedef MyInt AnotherInt;
//!     typedef sequence<MyInt> IntSeq;
//!
//!     struct Example {
//!         AnotherInt value;
//!         IntSeq data;
//!     };
//! };
//! ```
//!
//! Output HIR (conceptually):
//! ```idl
//! module A {
//!     struct Example {
//!         long value;       // AnotherInt -> MyInt -> long
//!         sequence<long> data;  // IntSeq -> sequence<MyInt> -> sequence<long>
//!     };
//! };
//! ```
//!
//! # Limitations
//!
//! This transformation will NOT strip typedefs that alias non-typedef ADTs
//! (structs, unions, enums, etc.). For example:
//! ```idl
//! struct Foo { long x; };
//! typedef Foo Bar;  // Bar will be inlined to reference Foo directly
//! ```

use std::collections::HashMap;

use ic_hir::ResolvedGraph;
use ic_hir::fold::{self, Fold};
use ic_hir::hir::{DefId, DefKind, Ty, TyKind};

/// Transform HIR by stripping all typedefs and inlining their underlying types.
///
/// Returns the transformed HIR with all typedef references resolved to their
/// underlying types, and all typedef definitions removed.
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    // Step 1: Pre-compute the resolved types for all typedefs
    let resolved_types = compute_resolved_types(&hir);

    // Step 2: Create folder and apply it to all definitions
    let mut folder = TypedefStripper { resolved_types };

    // Collect all definition IDs upfront
    let def_ids: Vec<DefId> = hir.context.definitions.iter().map(|(id, _)| id).collect();

    // Apply the fold to each definition
    for id in def_ids {
        hir.context.definitions.fold(id, |def| folder.fold_def(def));
    }

    // Step 3: Remove typedef definitions from the tree
    remove_typedef_definitions(&mut hir, &folder.resolved_types);

    hir
}

/// Pre-compute the fully resolved types for all typedefs
fn compute_resolved_types(hir: &ResolvedGraph) -> HashMap<DefId, Ty> {
    let mut typedef_ids = Vec::new();

    // First pass: collect all typedef IDs
    for (def_id, def) in &hir.context.definitions {
        if matches!(def.kind, DefKind::Alias(_)) {
            typedef_ids.push(def_id);
        }
    }

    // Second pass: resolve each typedef to its underlying type
    let mut resolved_types = HashMap::new();
    for def_id in &typedef_ids {
        let resolved = resolve_typedef_fully(hir, *def_id, &typedef_ids);
        resolved_types.insert(*def_id, resolved);
    }

    resolved_types
}

/// Fully resolve a typedef to its underlying non-typedef type
fn resolve_typedef_fully(hir: &ResolvedGraph, def_id: DefId, typedef_ids: &[DefId]) -> Ty {
    let def = hir.context.type_of(def_id);

    if let DefKind::Alias(alias) = &def.kind {
        let mut resolved_ty = alias.ty.clone();
        resolve_type_deep(&mut resolved_ty, hir, typedef_ids);
        resolved_ty
    } else {
        Ty {
            span: def.span,
            kind: TyKind::Adt(def_id),
        }
    }
}

/// Deep resolution - resolves types within container types as well
fn resolve_type_deep(ty: &mut Ty, hir: &ResolvedGraph, typedef_ids: &[DefId]) {
    match &mut ty.kind {
        TyKind::Adt(def_id) => {
            if typedef_ids.contains(def_id) {
                let resolved = resolve_typedef_fully(hir, *def_id, typedef_ids);
                *ty = resolved;
            }
        }
        TyKind::Array { ty: elem_ty, .. } | TyKind::Sequence { ty: elem_ty, .. } => {
            resolve_type_deep(elem_ty, hir, typedef_ids);
        }
        TyKind::Map { key, elem, .. } => {
            resolve_type_deep(key, hir, typedef_ids);
            resolve_type_deep(elem, hir, typedef_ids);
        }
        _ => {}
    }
}

/// A folder that replaces typedef references with their resolved types
struct TypedefStripper {
    resolved_types: HashMap<DefId, Ty>,
}

impl Fold for TypedefStripper {
    fn fold_ty(&mut self, ty: Ty) -> Ty {
        match &ty.kind {
            TyKind::Adt(def_id) => {
                if let Some(resolved) = self.resolved_types.get(def_id) {
                    // Return the resolved type, but apply fold to it as well
                    // in case there are nested typedefs (though we've already
                    // resolved them in compute_resolved_types)
                    fold::fold_ty(self, resolved.clone())
                } else {
                    ty
                }
            }
            _ => fold::fold_ty(self, ty),
        }
    }
}

/// Remove typedef definitions from the HIR tree
fn remove_typedef_definitions(hir: &mut ResolvedGraph, resolved_types: &HashMap<DefId, Ty>) {
    // Remove from top-level order
    hir.order
        .retain(|def_id| !resolved_types.contains_key(def_id));

    // Remove from builtin order
    hir.builtin_order
        .retain(|def_id| !resolved_types.contains_key(def_id));

    // Collect all container IDs that need to have typedefs removed from their definitions
    let container_ids: Vec<DefId> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(def_id, def)| match &def.kind {
            DefKind::Module(_) | DefKind::Interface(_) | DefKind::Valuetype(_) => Some(def_id),
            _ => None,
        })
        .collect();

    // Remove typedefs from each container's definitions list
    for def_id in container_ids {
        hir.context.definitions.fold(def_id, |mut def| {
            match &mut def.kind {
                DefKind::Module(module) => {
                    module
                        .definitions
                        .retain(|id| !resolved_types.contains_key(id));
                }
                DefKind::Interface(interface) => {
                    interface
                        .definitions
                        .retain(|id| !resolved_types.contains_key(id));
                }
                DefKind::Valuetype(valuetype) => {
                    valuetype
                        .definitions
                        .retain(|id| !resolved_types.contains_key(id));
                }
                _ => {}
            }
            def
        });
    }
}
