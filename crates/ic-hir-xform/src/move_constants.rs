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

//! Move constants into dedicated `Constants` modules.
//!
//! This transformation collects all constants from each module (and top-level)
//! and moves them into a new nested module called `Constants`. This is useful
//! for languages like C# and Java where constants need to be grouped into a
//! separate container (emitted as a static class).
//!
//! # Transformation
//!
//! For each module (including top-level), the transformation:
//! 1. Iterates over direct definitions only (not nested modules)
//! 2. Collects all constants
//! 3. Creates a new `Constants` module and moves the constants into it
//! 4. Recursively processes nested modules
//!
//! # Name Collision Handling
//!
//! If a `Constants` module already exists, constants are merged into it.
//! If a constant being moved has the same name as an existing definition
//! in the `Constants` module, the `escape` closure is called repeatedly
//! until a unique name is found.
//!
//! # Example
//!
//! Input IDL:
//! ```idl
//! module A {
//!     const long X = 1;
//!     const long Y = 2;
//!     struct Foo { long x; };
//!
//!     module B {
//!         const long Z = 3;
//!         struct Bar { long y; };
//!     };
//! };
//! ```
//!
//! Output HIR (conceptually):
//! ```idl
//! module A {
//!     module Constants {
//!         const long X = 1;
//!         const long Y = 2;
//!     };
//!     struct Foo { long x; };
//!
//!     module B {
//!         module Constants {
//!             const long Z = 3;
//!         };
//!         struct Bar { long y; };
//!     };
//! };
//! ```
//!
//! Note: Enum constants are NOT moved - only standalone `const` definitions.

use std::collections::HashSet;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{self, DefFlags, DefId, DefKind, Ident, ModuleTy};

/// The name of the generated constants module.
pub const CONSTANTS_MODULE_NAME: &str = "Constants";

/// Transform HIR by moving constants into dedicated `Constants` modules.
///
/// The `escape` closure is called when a constant name collides with an existing
/// definition. It receives the current name and should return an escaped version.
/// The closure is called repeatedly until a unique name is found.
///
/// Example escape functions:
/// - `|name| format!("{}_", name)` - appends underscore
/// - `|name| format!("_{}", name)` - prepends underscore
/// - `|name| format!("{}Const", name)` - appends "Const" suffix
#[must_use]
pub fn transform<F>(mut hir: ResolvedGraph, escape: F) -> ResolvedGraph
where
    F: Fn(&str) -> String + Copy,
{
    // Process top-level definitions
    let order = std::mem::take(&mut hir.order);
    hir.order = process_definition_list(&mut hir, order, None, escape);

    hir
}

/// Process a list of definitions, moving constants into a Constants module.
/// Returns the new definition list with constants replaced by the Constants module.
fn process_definition_list<F>(
    hir: &mut ResolvedGraph,
    def_ids: Vec<DefId>,
    parent: Option<DefId>,
    escape: F,
) -> Vec<DefId>
where
    F: Fn(&str) -> String + Copy,
{
    // First, recursively process any nested modules (but skip Constants modules)
    for &def_id in &def_ids {
        let def = hir.context.definitions.get(def_id);
        if let DefKind::Module(module_ty) = &def.kind {
            // Don't recursively process Constants modules - they're the destination
            if def.ident.name == CONSTANTS_MODULE_NAME {
                continue;
            }

            let nested_defs = module_ty.definitions.clone();
            let new_nested_defs = process_definition_list(hir, nested_defs, Some(def_id), escape);

            // Update the module's definitions
            hir.context.definitions.fold(def_id, |mut def| {
                if let DefKind::Module(module_ty) = &mut def.kind {
                    module_ty.definitions = new_nested_defs;
                }
                def
            });
        }
    }

    // Separate constants from other definitions (only direct children, not from nested modules)
    let mut constants: Vec<DefId> = Vec::new();
    let mut others: Vec<DefId> = Vec::new();
    let mut existing_constants_module: Option<DefId> = None;

    for def_id in def_ids {
        let def = hir.context.definitions.get(def_id);

        // Check if this is a standalone constant (not an enum field)
        let is_standalone_const =
            matches!(&def.kind, DefKind::Const(_)) && !is_enum_field(hir, def_id);

        // Check if this is an existing Constants module
        let is_constants_module =
            matches!(&def.kind, DefKind::Module(_)) && def.ident.name == CONSTANTS_MODULE_NAME;

        if is_standalone_const {
            constants.push(def_id);
        } else if is_constants_module {
            existing_constants_module = Some(def_id);
            others.push(def_id);
        } else {
            others.push(def_id);
        }
    }

    // If no constants found, return the original list (with recursively processed modules)
    if constants.is_empty() {
        return others;
    }

    // Handle collision detection and renaming
    let existing_names = collect_existing_names(hir, existing_constants_module);
    rename_colliding_constants(hir, &constants, &existing_names, escape);

    // Either merge into existing Constants module or create a new one
    let constants_module_id = if let Some(existing_id) = existing_constants_module {
        // Merge constants into existing module
        merge_into_constants_module(hir, existing_id, &constants);
        existing_id
    } else {
        // Create new Constants module
        let new_module_id = create_constants_module(hir, &constants, parent);

        // Insert at the beginning of others
        others.insert(0, new_module_id);
        new_module_id
    };

    // Update parent references for moved constants
    for &const_id in &constants {
        hir.context.definitions.get_mut(const_id).parent = Some(constants_module_id);
    }

    others
}

/// Collect names of all definitions in an existing Constants module
fn collect_existing_names(hir: &ResolvedGraph, module_id: Option<DefId>) -> HashSet<String> {
    let mut names = HashSet::new();

    if let Some(id) = module_id {
        let def = hir.context.definitions.get(id);
        if let DefKind::Module(module_ty) = &def.kind {
            for &child_id in &module_ty.definitions {
                let child = hir.context.definitions.get(child_id);
                names.insert(child.ident.name.clone());
            }
        }
    }

    names
}

/// Rename constants that would collide with existing definitions
fn rename_colliding_constants<F>(
    hir: &mut ResolvedGraph,
    constants: &[DefId],
    existing_names: &HashSet<String>,
    escape: F,
) where
    F: Fn(&str) -> String,
{
    // Build a set of all names (existing + constants being moved)
    let mut all_names = existing_names.clone();

    for &const_id in constants {
        let def = hir.context.definitions.get(const_id);
        let mut name = def.ident.name.clone();

        // Keep escaping until we find a unique name
        while all_names.contains(&name) {
            name = escape(&name);
        }

        // Update the constant's name if it changed
        let original_name = &hir.context.definitions.get(const_id).ident.name;
        if &name != original_name {
            hir.context
                .definitions
                .get_mut(const_id)
                .ident
                .name
                .clone_from(&name);
        }

        all_names.insert(name);
    }
}

/// Merge constants into an existing Constants module
fn merge_into_constants_module(hir: &mut ResolvedGraph, module_id: DefId, constants: &[DefId]) {
    hir.context.definitions.fold(module_id, |mut def| {
        if let DefKind::Module(module_ty) = &mut def.kind {
            module_ty.definitions.extend(constants.iter().copied());
        }
        def
    });
}

/// Check if a constant is an enum field (we don't want to move those)
fn is_enum_field(hir: &ResolvedGraph, def_id: DefId) -> bool {
    let def = hir.context.definitions.get(def_id);
    if let Some(parent_id) = def.parent {
        let parent = hir.context.definitions.get(parent_id);
        return matches!(parent.kind, DefKind::Enum(_) | DefKind::Bitmask(_));
    }
    false
}

/// Create a new Constants module containing the given constants
fn create_constants_module(
    hir: &mut ResolvedGraph,
    constants: &[DefId],
    parent: Option<DefId>,
) -> DefId {
    // Get span from first constant, or use a default
    let span = constants
        .first()
        .map(|&id| hir.context.definitions.get(id).span)
        .unwrap_or_default();

    // Create the Constants module
    hir.context.definitions.alloc_with_id(|id| hir::Def {
        id,
        ident: Ident {
            name: CONSTANTS_MODULE_NAME.to_string(),
            span,
        },
        parent,
        annotations: Vec::new(),
        span,
        kind: DefKind::Module(ModuleTy {
            definitions: constants.to_vec(),
        }),
        flags: DefFlags::IS_SYNTHESIZED,
    })
}
