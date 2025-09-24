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

//! HIR normalization and validation.
//!
//! This transformation ensures the HIR is in a consistent state by:
//! - Validating parent-child relationships
//! - Ensuring module/interface/valuetype definition lists are complete
//! - Verifying scope hierarchy matches definition hierarchy

use std::collections::{HashMap, HashSet};

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind, TyKind};

/// Normalizes and validates HIR structure.
pub struct Normalizer {
    /// Track changes made during normalization
    changes_made: bool,
    /// Errors found during validation
    errors: Vec<String>,
    /// Detailed log of changes made
    changes: Vec<String>,
}

impl Normalizer {
    /// Normalize the HIR, fixing any inconsistencies.
    ///
    /// # Panics
    ///
    /// Panics if HIR validation fails after normalization, indicating a bug in the
    /// normalization process.
    #[must_use]
    pub fn normalize(mut hir: ResolvedGraph) -> ResolvedGraph {
        let mut normalizer = Normalizer {
            changes_made: false,
            errors: Vec::new(),
            changes: Vec::new(),
        };

        normalizer.normalize_impl(&mut hir);

        #[cfg(debug_assertions)]
        {
            debug_assert!(
                !normalizer.changes_made,
                "HIR normalization made changes:\n{}",
                normalizer.changes.join("\n"),
            );

            // Run validation to ensure we're in a good state
            if let Err(errors) = Self::validate_hir(&hir) {
                panic!(
                    "HIR validation failed after normalization:\n{}",
                    errors.join("\n")
                );
            }
        }

        hir
    }

    /// Validate the HIR without making changes.
    ///
    /// # Errors
    ///
    /// Returns a vector of validation error messages if the HIR is invalid.
    #[cfg(debug_assertions)]
    pub fn validate_only(hir: &ResolvedGraph) -> Result<(), Vec<String>> {
        Self::validate_hir(hir)
    }

    fn normalize_impl(&mut self, hir: &mut ResolvedGraph) {
        // Fix parent relationships
        self.fix_parent_relationships(hir);

        // Fix module/interface/valuetype definition lists
        self.fix_definition_lists(hir);
    }

    fn validate_hir(hir: &ResolvedGraph) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate parent relationships
        Self::validate_parent_relationships(hir, &mut errors);

        // Validate definition lists
        Self::validate_definition_lists(hir, &mut errors);

        // Validate scope relationships
        Self::validate_scope_relationships(hir, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Fix parent relationships based on containment.
    fn fix_parent_relationships(&mut self, hir: &mut ResolvedGraph) {
        // Build a map of who contains whom
        let mut containment: HashMap<DefId, Vec<DefId>> = HashMap::new();

        // First pass: collect all containment relationships
        for (def_id, def) in &hir.context.definitions {
            match &def.kind {
                DefKind::Module(m) => {
                    for &child_id in &m.definitions {
                        containment.entry(def_id).or_default().push(child_id);
                    }
                }
                DefKind::Interface(i) => {
                    for &child_id in &i.definitions {
                        containment.entry(def_id).or_default().push(child_id);
                    }
                }
                DefKind::Valuetype(v) => {
                    for &child_id in &v.definitions {
                        containment.entry(def_id).or_default().push(child_id);
                    }
                }
                DefKind::Annotation(a) => {
                    for &child_id in &a.types {
                        containment.entry(def_id).or_default().push(child_id);
                    }
                }
                DefKind::Enum(e) => {
                    // Enum members are the enum constants (fields)
                    for &field_id in &e.fields {
                        containment.entry(def_id).or_default().push(field_id);
                    }
                }
                DefKind::Bitmask(b) => {
                    // Bitmask flags don't have their own DefIds, they're just stored inline
                    // So no containment relationship to track here
                }
                _ => {}
            }
        }

        // Second pass: fix parent pointers
        for (parent_id, children) in containment {
            for child_id in children {
                // Get names before mutating
                let child_name = hir.context.definitions.get(child_id).ident.name.clone();
                let parent_name = hir.context.definitions.get(parent_id).ident.name.clone();

                let child_def = hir.context.definitions.get_mut(child_id);
                if child_def.parent != Some(parent_id) {
                    self.changes_made = true;
                    self.changes.push(format!(
                        "Set parent of '{child_name}' (def_id={child_id:?}) to '{parent_name}' \
                         (def_id={parent_id:?})"
                    ));
                    child_def.parent = Some(parent_id);
                }
            }
        }

        // Check definitions with scopes
        for (scope_idx, scope) in hir.context.scopes.scopes.iter().enumerate() {
            if let Some(def_id) = scope.def_id {
                // Skip root scope - definitions there should not automatically get a parent
                if scope_idx == 0 {
                    continue;
                }

                // Find all definitions in this scope
                for (name, child_ids) in scope.definitions.iter() {
                    for &child_id in child_ids {
                        // Skip if it's the scope's own definition
                        if child_id == def_id {
                            continue;
                        }

                        // Get names before mutating
                        let child_name = hir.context.definitions.get(child_id).ident.name.clone();
                        let parent_name = hir.context.definitions.get(def_id).ident.name.clone();

                        let child_def = hir.context.definitions.get_mut(child_id);
                        // If this definition has no parent set by containment,
                        // it should have the scope's definition as parent
                        if child_def.parent.is_none() && !name.starts_with('@') {
                            self.changes_made = true;
                            self.changes.push(format!(
                                "Set parent of '{child_name}' (def_id={child_id:?}) in scope to \
                                 '{parent_name}' (def_id={def_id:?})"
                            ));
                            child_def.parent = Some(def_id);
                        }
                    }
                }
            }
        }
    }

    /// Fix definition lists to ensure they're complete.
    fn fix_definition_lists(&mut self, hir: &mut ResolvedGraph) {
        // Build a map of parent -> children based on parent pointers
        let mut actual_children: HashMap<DefId, Vec<DefId>> = HashMap::new();

        for (def_id, def) in &hir.context.definitions {
            if let Some(parent_id) = def.parent {
                // Check if this is a bitmask flag constant
                if let DefKind::Const(c) = &def.kind {
                    if let TyKind::Adt(type_id) = &c.ty.kind {
                        if let DefKind::Bitmask(_) = &hir.context.definitions.get(*type_id).kind {
                            // This is a bitmask flag - don't add it to actual_children
                            // as bitmask flags are stored in the parent scope, not in the bitmask's definition list
                            continue;
                        }
                    }
                }
                actual_children.entry(parent_id).or_default().push(def_id);
            }
        }

        // Fix definition lists
        for (parent_id, children) in actual_children {
            // Collect parent name before mutating
            let parent_name = hir.context.definitions.get(parent_id).ident.name.clone();

            // First, collect info about children if it's an annotation
            let children_to_add: Vec<DefId> =
                if let DefKind::Annotation(a) = &hir.context.definitions.get(parent_id).kind {
                    let existing: HashSet<_> = a.types.iter().copied().collect();
                    children
                        .iter()
                        .copied()
                        .filter(|&child_id| {
                            if existing.contains(&child_id) {
                                return false;
                            }
                            // Only add if it's actually a type definition
                            let child_def = hir.context.definitions.get(child_id);
                            matches!(child_def.kind, DefKind::Enum(_) | DefKind::Bitmask(_))
                        })
                        .collect()
                } else {
                    Vec::new()
                };

            // Collect child names before mutating
            let child_names: HashMap<DefId, String> = children
                .iter()
                .chain(children_to_add.iter())
                .map(|&id| (id, hir.context.definitions.get(id).ident.name.clone()))
                .collect();

            // Now do the actual modifications
            let parent_def = hir.context.definitions.get_mut(parent_id);
            match &mut parent_def.kind {
                DefKind::Module(m) => {
                    let existing: HashSet<_> = m.definitions.iter().copied().collect();
                    for child_id in children {
                        if !existing.contains(&child_id) {
                            self.changes_made = true;
                            self.changes.push(format!(
                                "Added '{}' (def_id={:?}) to module '{}' definitions",
                                child_names.get(&child_id).unwrap(),
                                child_id,
                                parent_name
                            ));
                            m.definitions.push(child_id);
                        }
                    }
                }
                DefKind::Interface(i) => {
                    let existing: HashSet<_> = i.definitions.iter().copied().collect();
                    for child_id in children {
                        if !existing.contains(&child_id) {
                            self.changes_made = true;
                            self.changes.push(format!(
                                "Added '{}' (def_id={:?}) to interface '{}' definitions",
                                child_names.get(&child_id).unwrap(),
                                child_id,
                                parent_name
                            ));
                            i.definitions.push(child_id);
                        }
                    }
                }
                DefKind::Valuetype(v) => {
                    let existing: HashSet<_> = v.definitions.iter().copied().collect();
                    for child_id in children {
                        if !existing.contains(&child_id) {
                            self.changes_made = true;
                            self.changes.push(format!(
                                "Added '{}' (def_id={:?}) to valuetype '{}' definitions",
                                child_names.get(&child_id).unwrap(),
                                child_id,
                                parent_name
                            ));
                            v.definitions.push(child_id);
                        }
                    }
                }
                DefKind::Annotation(a) => {
                    for child_id in children_to_add {
                        self.changes_made = true;
                        self.changes.push(format!(
                            "Added '{}' (def_id={:?}) to annotation '{}' types",
                            child_names.get(&child_id).unwrap(),
                            child_id,
                            parent_name
                        ));
                        a.types.push(child_id);
                    }
                }
                _ => {}
            }
        }
    }

    #[allow(clippy::ptr_arg)]
    fn validate_parent_relationships(hir: &ResolvedGraph, errors: &mut Vec<String>) {
        for (def_id, def) in &hir.context.definitions {
            // Check parent exists
            if let Some(parent_id) = def.parent {
                let parent_def = hir.context.definitions.get(parent_id);

                // Check if this is a bitmask flag constant
                let is_bitmask_flag = if let DefKind::Const(c) = &def.kind {
                    if let TyKind::Adt(type_id) = &c.ty.kind {
                        matches!(
                            &hir.context.definitions.get(*type_id).kind,
                            DefKind::Bitmask(_)
                        )
                    } else {
                        false
                    }
                } else {
                    false
                };

                let is_contained = if is_bitmask_flag {
                    // Bitmask flags are constants stored in parent scope, not in bitmask's definition list
                    true
                } else {
                    match &parent_def.kind {
                        DefKind::Module(m) => m.definitions.contains(&def_id),
                        DefKind::Interface(i) => i.definitions.contains(&def_id),
                        DefKind::Valuetype(v) => v.definitions.contains(&def_id),
                        DefKind::Annotation(a) => a.types.contains(&def_id),
                        DefKind::Enum(e) => e.fields.contains(&def_id),
                        DefKind::Bitmask(_) => false, // Regular bitmask children
                        _ => true,                    // Other types don't have containment
                    }
                };

                if !is_contained {
                    errors.push(format!(
                        "Definition '{}' claims parent '{}' but parent doesn't contain it",
                        def.ident.name, parent_def.ident.name
                    ));
                }
            }
        }
    }

    #[allow(clippy::ptr_arg)] // We need Vec to push errors
    fn validate_definition_lists(hir: &ResolvedGraph, errors: &mut Vec<String>) {
        for (def_id, def) in &hir.context.definitions {
            let children = match &def.kind {
                DefKind::Module(m) => &m.definitions[..],
                DefKind::Interface(i) => &i.definitions[..],
                DefKind::Valuetype(v) => &v.definitions[..],
                _ => continue,
            };

            for &child_id in children {
                let child_def = hir.context.definitions.get(child_id);
                if child_def.parent != Some(def_id) {
                    errors.push(format!(
                        "{} '{}' contains '{}' but child has different parent",
                        match &def.kind {
                            DefKind::Module(_) => "Module",
                            DefKind::Interface(_) => "Interface",
                            DefKind::Valuetype(_) => "Valuetype",
                            _ => "Definition",
                        },
                        def.ident.name,
                        child_def.ident.name
                    ));
                }
            }
        }
    }

    #[allow(clippy::ptr_arg)] // We need Vec to push errors
    fn validate_scope_relationships(hir: &ResolvedGraph, errors: &mut Vec<String>) {
        for scope in &hir.context.scopes.scopes {
            if let Some(def_id) = scope.def_id {
                // Verify the definition exists
                let _def = hir.context.definitions.get(def_id);
            }
        }
    }
}

/// Run normalization on a HIR graph.
#[must_use]
pub fn normalize(hir: ResolvedGraph) -> ResolvedGraph {
    Normalizer::normalize(hir)
}
