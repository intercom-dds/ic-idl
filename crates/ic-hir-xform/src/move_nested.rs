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

//! Move nested types transformation
//!
//! This transformation moves type definitions out of interfaces and valuetypes
//! into separate modules. This is necessary for Rust code generation where
//! types cannot be nested inside traits.

use std::collections::HashSet;

use ic_hir::{ResolvedGraph, hir};

/// Transform HIR to move nested types out of interfaces and valuetypes
/// Returns the transformed HIR and a set of `DefIds` that were moved
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> (ResolvedGraph, HashSet<hir::DefId>) {
    let mut moved_defs = HashSet::new();

    // Process top-level definitions
    let order = std::mem::take(&mut hir.order);
    hir.order = move_nested_from_list(&mut hir, order, None, &mut moved_defs);

    // Process builtin definitions
    let builtin_order = std::mem::take(&mut hir.builtin_order);
    hir.builtin_order = move_nested_from_list(&mut hir, builtin_order, None, &mut moved_defs);

    (hir, moved_defs)
}

/// Process a list of definitions and move nested types
fn move_nested_from_list(
    hir: &mut ResolvedGraph,
    def_ids: Vec<hir::DefId>,
    parent_scope: Option<hir::DefId>,
    moved_defs: &mut HashSet<hir::DefId>,
) -> Vec<hir::DefId> {
    let mut result = Vec::new();

    for def_id in def_ids {
        let def = hir.context.type_of(def_id);

        match &def.kind {
            hir::DefKind::Interface(interface) => {
                // Check if interface has nested definitions
                let nested_defs = interface.definitions.clone();
                result.push(def_id);

                if !nested_defs.is_empty() {
                    // Recursively process nested definitions
                    let processed_defs =
                        move_nested_from_list(hir, nested_defs.clone(), Some(def_id), moved_defs);

                    // Create a module to hold the extracted types
                    let module_def = create_module_for_parent(
                        hir,
                        def_id,
                        &processed_defs,
                        parent_scope,
                        moved_defs,
                    );

                    // Insert module directly after the interface
                    result.push(module_def);

                    // Clear the nested definitions from the interface
                    if let hir::DefKind::Interface(interface) =
                        &mut hir.context.definitions.get_mut(def_id).kind
                    {
                        interface.definitions.clear();
                    }

                    // Mark all nested definitions as moved
                    for &nested_id in &nested_defs {
                        moved_defs.insert(nested_id);
                    }
                }
            }
            hir::DefKind::Valuetype(valuetype) => {
                // Check if valuetype has nested definitions
                let nested_defs = valuetype.definitions.clone();
                result.push(def_id);

                if !nested_defs.is_empty() {
                    // Recursively process nested definitions
                    let processed_defs =
                        move_nested_from_list(hir, nested_defs.clone(), Some(def_id), moved_defs);

                    // Create a module to hold the extracted types
                    let module_def = create_module_for_parent(
                        hir,
                        def_id,
                        &processed_defs,
                        parent_scope,
                        moved_defs,
                    );

                    // Insert module directly after the valuetype
                    result.push(module_def);

                    // Clear the nested definitions from the valuetype
                    if let hir::DefKind::Valuetype(valuetype) =
                        &mut hir.context.definitions.get_mut(def_id).kind
                    {
                        valuetype.definitions.clear();
                    }

                    // Mark all nested definitions as moved
                    for &nested_id in &nested_defs {
                        moved_defs.insert(nested_id);
                    }
                }
            }
            hir::DefKind::Module(module) => {
                // Recursively process module members
                let members = module.definitions.clone();
                let new_members = move_nested_from_list(hir, members, Some(def_id), moved_defs);

                // Update the module's members
                if let hir::DefKind::Module(m) = &mut hir.context.definitions.get_mut(def_id).kind {
                    m.definitions = new_members;
                }

                result.push(def_id);
            }
            _ => {
                result.push(def_id);
            }
        }
    }

    result
}

/// Create a module to hold extracted types
fn create_module_for_parent(
    hir: &mut ResolvedGraph,
    parent_id: hir::DefId,
    extracted_types: &[hir::DefId],
    parent_scope: Option<hir::DefId>,
    moved_defs: &mut HashSet<hir::DefId>,
) -> hir::DefId {
    // Get parent information before borrowing mutably
    let (module_name, parent_span, parent_ident_span) = {
        let parent_def = hir.context.type_of(parent_id);
        (
            parent_def.ident.name.clone(),
            parent_def.span,
            parent_def.ident.span,
        )
    };

    // Create a new module definition
    let module_id = hir.context.definitions.alloc_with_id(|id| hir::Def {
        id,
        ident: hir::Ident {
            name: module_name,
            span: parent_ident_span,
        },
        parent: parent_scope,
        annotations: Vec::new(),
        span: parent_span,
        flags: hir::DefFlags::nil(),
        kind: hir::DefKind::Module(hir::ModuleTy {
            definitions: extracted_types.to_vec(),
        }),
    });

    // Update parent references for extracted types
    for &type_id in extracted_types {
        hir.context.definitions.get_mut(type_id).parent = Some(module_id);
    }

    module_id
}
