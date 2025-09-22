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

//! Module squashing transformation
//!
//! This transformation merges re-opened modules into a single definition.
//! For example:
//! ```idl
//! module A {
//!     struct Foo {};
//! };
//! module A {
//!     struct Bar {};
//! };
//! ```
//! Will be transformed into:
//! ```idl
//! module A {
//!     struct Foo {};
//!     struct Bar {};
//! };
//! ```

use std::collections::HashMap;

use ic_hir::{ResolvedGraph, hir};

/// Transform HIR to squash re-opened modules into single definitions
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    // Process top-level definitions
    let mut modules = HashMap::new();
    let order = std::mem::take(&mut hir.order);
    hir.order = squash_modules_list(&mut hir, order, &mut modules);

    // Process builtin definitions
    let mut builtin_modules = HashMap::new();
    let builtin_order = std::mem::take(&mut hir.builtin_order);
    hir.builtin_order = squash_modules_list(&mut hir, builtin_order, &mut builtin_modules);

    hir
}

/// Process a list of definitions and squash duplicate modules
fn squash_modules_list(
    hir: &mut ResolvedGraph,
    def_ids: Vec<hir::DefId>,
    modules: &mut HashMap<String, hir::DefId>,
) -> Vec<hir::DefId> {
    let mut result = Vec::new();

    for def_id in def_ids {
        let def = hir.context.type_of(def_id);
        if let hir::DefKind::Module(module_ty) = &def.kind {
            // Process the module's members recursively FIRST
            let members = module_ty.definitions.clone();
            let new_members = squash_modules_list(hir, members, modules);

            // Update the module's members
            if let hir::DefKind::Module(module_ty) =
                &mut hir.context.definitions.get_mut(def_id).kind
            {
                module_ty.definitions = new_members;
            }

            // Check if this module should be merged with an existing one
            let qualified_name = build_qualified_name(hir, def_id);
            if let Some(&target_id) = modules.get(&qualified_name) {
                // This is a duplicate module, merge it into the target
                merge_module_contents(hir, target_id, def_id);
                // Don't add to result since it's being merged
            } else {
                // This is the first instance of this module
                modules.insert(qualified_name, def_id);
                result.push(def_id);
            }
        } else {
            result.push(def_id);
        }
    }

    result
}

/// Build a scoped name for a module (similar to `lc_scoped_name` in C++)
fn build_qualified_name(hir: &ResolvedGraph, module_id: hir::DefId) -> String {
    let def = hir.context.type_of(module_id);
    let mut parts = vec![def.ident.name.clone()];
    let mut current_parent = def.parent;

    while let Some(parent_id) = current_parent {
        let parent_def = hir.context.type_of(parent_id);
        // Only add module names to build the qualified path
        if matches!(parent_def.kind, hir::DefKind::Module(_)) {
            parts.push(parent_def.ident.name.clone());
        }
        current_parent = parent_def.parent;
    }

    parts.reverse();
    parts.join("::")
}

/// Merge the contents of source module into target module
fn merge_module_contents(hir: &mut ResolvedGraph, target_id: hir::DefId, source_id: hir::DefId) {
    // Get the definitions from the source module
    let source_definitions =
        if let hir::DefKind::Module(module_ty) = &hir.context.type_of(source_id).kind {
            module_ty.definitions.clone()
        } else {
            return;
        };

    // Collect definitions to update parent
    let mut defs_to_update = Vec::new();

    // Add all definitions from source to target
    if let hir::DefKind::Module(target_module) =
        &mut hir.context.definitions.get_mut(target_id).kind
    {
        for def_id in source_definitions {
            if !target_module.definitions.contains(&def_id) {
                target_module.definitions.push(def_id);
                defs_to_update.push(def_id);
            }
        }
    }

    // Update the parent of the moved definitions
    for def_id in defs_to_update {
        hir.context.definitions.get_mut(def_id).parent = Some(target_id);
    }
}
