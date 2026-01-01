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

//! Hierarchical scope management for name resolution.
//!
//! The `ScopeTree` manages hierarchical scopes for IDL name resolution. It supports:
//! - Nested scopes (modules, interfaces, valuetypes)
//! - Case-insensitive lookups (IDL requirement)
//! - Module reopening (multiple definitions with same name)
//! - Separate annotation namespace
//!
//! Resolution is performed entirely within `ScopeTree`, making it reusable
//! for HIR construction from sources other than the AST.

use std::collections::HashMap;

use ic_alloc::arena::Arena;
use ic_alloc::insensitive::CaseMap;

use crate::hir::{Def, DefId};

/// Error returned when path resolution fails.
#[derive(Debug, Clone)]
pub struct ResolutionError {
    /// Index of the segment that failed to resolve.
    pub failed_segment: usize,
    /// The container `DefId` we were searching in when resolution failed.
    /// `None` if we failed at the top-level scope.
    pub container: Option<DefId>,
}

/// A scope in the hierarchy.
#[derive(Clone, Debug)]
pub struct Scope {
    /// The definition ID of this scope (if it's a module/interface/etc).
    pub def_id: Option<DefId>,

    /// Parent scope.
    pub parent: Option<ScopeId>,

    /// Child scopes by name.
    pub children: CaseMap<ScopeId>,

    /// Local definitions in this scope.
    /// Uses Vec<DefId> to support multiple definitions with the same name (e.g., reopened modules).
    /// Annotation definitions are stored with an `@` prefix (e.g., `@annotation foo` → `@foo`).
    pub definitions: CaseMap<Vec<DefId>>,
}

impl Scope {
    /// Gets the most recent definition with the given name, if any.
    #[must_use]
    pub fn get_definition(&self, name: &str) -> Option<DefId> {
        self.definitions
            .get(name)
            .and_then(|ids| ids.last().copied())
    }
}

/// Unique identifier for a scope.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

/// Manages the scope hierarchy.
#[derive(Clone, Debug)]
pub struct ScopeTree {
    /// All scopes in the tree.
    pub scopes: Vec<Scope>,

    /// The root scope (global scope).
    root: ScopeId,

    /// Maps `DefId` → scope where the definition is registered.
    /// Used by `find_scope_containing_def`.
    containing_scope: HashMap<DefId, ScopeId>,

    /// Maps `DefId` → scope that the definition owns (for modules, interfaces, etc.).
    /// Used by `find_scope_for_def` to navigate into a definition's child scope.
    owned_scope: HashMap<DefId, ScopeId>,
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeTree {
    /// Creates a new scope tree with a root scope.
    #[must_use]
    pub fn new() -> Self {
        let root_scope = Scope {
            def_id: None,
            parent: None,
            children: CaseMap::new(),
            definitions: CaseMap::new(),
        };

        Self {
            scopes: vec![root_scope],
            root: ScopeId(0),
            containing_scope: HashMap::new(),
            owned_scope: HashMap::new(),
        }
    }

    /// Gets the root scope.
    pub fn root(&self) -> ScopeId {
        self.root
    }

    /// Creates a new child scope.
    pub fn create_child_scope(
        &mut self,
        parent: ScopeId,
        name: String,
        def_id: Option<DefId>,
    ) -> ScopeId {
        let scope_id = ScopeId(self.scopes.len());

        let scope = Scope {
            def_id,
            parent: Some(parent),
            children: CaseMap::new(),
            definitions: CaseMap::new(),
        };

        self.scopes.push(scope);
        self.scopes[parent.0].children.insert(name, scope_id);

        scope_id
    }

    /// Adds a definition to a scope.
    /// Multiple definitions with the same name are allowed (e.g., reopened modules).
    pub fn add_definition(&mut self, scope: ScopeId, name: String, def_id: DefId) {
        Self::add_to_definitions(&mut self.scopes[scope.0].definitions, name, def_id);
        self.containing_scope.insert(def_id, scope);
    }

    /// Adds an annotation definition to a scope.
    /// Annotations are stored with an `@` prefix to avoid collisions with regular definitions.
    pub fn add_annotation(&mut self, scope: ScopeId, name: &str, def_id: DefId) {
        self.add_definition(scope, format!("@{name}"), def_id);
    }

    /// Helper to add a `DefId` to a `CaseMap<Vec<DefId>>`
    fn add_to_definitions(definitions: &mut CaseMap<Vec<DefId>>, name: String, def_id: DefId) {
        definitions
            .entry(name)
            .or_insert_with(Vec::new)
            .push(def_id);
    }

    /// Gets a scope by ID.
    #[must_use]
    pub fn get_scope(&self, scope: ScopeId) -> &Scope {
        &self.scopes[scope.0]
    }

    /// Gets a mutable scope by ID.
    pub fn get_scope_mut(&mut self, scope: ScopeId) -> &mut Scope {
        &mut self.scopes[scope.0]
    }

    /// Sets the `def_id` for a scope and maintains the reverse mapping.
    /// This associates a definition (e.g., module, interface) with the scope it owns.
    pub fn set_scope_def_id(&mut self, scope_id: ScopeId, def_id: DefId) {
        self.scopes[scope_id.0].def_id = Some(def_id);
        self.owned_scope.insert(def_id, scope_id);
    }

    /// Resolves a single name segment in a scope (looks in this scope and parents).
    #[must_use]
    pub fn resolve_name(&self, scope: ScopeId, name: &str) -> Option<DefId> {
        let mut current = Some(scope);

        while let Some(scope_id) = current {
            let scope_data = &self.scopes[scope_id.0];

            if let Some(def_id) = scope_data.get_definition(name) {
                return Some(def_id);
            }

            // Check child scopes (for module names)
            if let Some(&child_scope_id) = scope_data.children.get(name)
                && let Some(def_id) = self.scopes[child_scope_id.0].def_id
            {
                return Some(def_id);
            }

            current = scope_data.parent;
        }
        None
    }

    /// Resolves an annotation name in a scope (looks in this scope and parents).
    /// Annotations are stored with an `@` prefix, so this prepends `@` before lookup.
    #[must_use]
    pub fn resolve_annotation(&self, scope: ScopeId, name: &str) -> Option<DefId> {
        self.resolve_name(scope, &format!("@{name}"))
    }

    /// Resolves a single name with interface visibility rules.
    /// Types inside interfaces are only visible within the interface unless qualified.
    #[must_use]
    pub fn resolve_name_with_visibility(
        &self,
        scope: ScopeId,
        name: &str,
        definitions: &Arena<Def>,
    ) -> Option<DefId> {
        let mut current = Some(scope);
        let starting_scope = scope;

        while let Some(scope_id) = current {
            let scope_data = &self.scopes[scope_id.0];

            if let Some(def_id) = scope_data.get_definition(name) {
                // Check if it's accessible (interface visibility rules)
                if self.is_interface_scope(scope_id, definitions)
                    && !self.is_inside_scope(starting_scope, scope_id)
                {
                    return None;
                }
                return Some(def_id);
            }

            // Check child scopes (for module names and interfaces)
            if let Some(&child_scope_id) = scope_data.children.get(name)
                && let Some(def_id) = self.scopes[child_scope_id.0].def_id
            {
                return Some(def_id);
            }

            // Before moving to parent, check if we would cross an interface boundary
            if let Some(parent_scope_id) = scope_data.parent
                && self.is_interface_scope(parent_scope_id, definitions)
                && !self.is_inside_scope(starting_scope, parent_scope_id)
            {
                return None;
            }

            current = scope_data.parent;
        }
        None
    }

    /// Checks if a scope is inside (or is) another scope.
    fn is_inside_scope(&self, inner: ScopeId, outer: ScopeId) -> bool {
        if inner == outer {
            return true;
        }

        let mut current = Some(inner);
        while let Some(scope_id) = current {
            if scope_id == outer {
                return true;
            }
            current = self.scopes[scope_id.0].parent;
        }
        false
    }

    /// Resolves a path starting from a scope. Returns `None` on failure.
    /// For detailed error information, use `try_resolve_path` instead.
    #[must_use]
    pub fn resolve_path(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        self.try_resolve_path(scope, path, false).ok()
    }

    /// Resolves an annotation path starting from a scope.
    /// The last segment has `@` prepended since annotations are stored with that prefix.
    #[must_use]
    pub fn resolve_annotation_path(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        if path.is_empty() {
            return None;
        }

        // Prepend @ to the last segment (the annotation name)
        let mut ann_path: Vec<String> = path.iter().map(std::string::ToString::to_string).collect();
        if let Some(last) = ann_path.last_mut() {
            *last = format!("@{last}");
        }
        let ann_path_refs: Vec<&str> = ann_path.iter().map(std::string::String::as_str).collect();

        self.try_resolve_path(scope, &ann_path_refs, false).ok()
    }

    /// Resolves a path, trying relative resolution first, then absolute.
    ///
    /// For relative paths, walks up parent scopes trying each one.
    /// Tracks the "best" error (the one that made most progress through the path).
    ///
    /// # Errors
    ///
    /// Returns `ResolutionError` if the path cannot be resolved, containing the
    /// index of the failing segment and the container definition (if any).
    pub fn try_resolve_path(
        &self,
        scope: ScopeId,
        path: &[&str],
        absolute: bool,
    ) -> Result<DefId, ResolutionError> {
        if path.is_empty() {
            return Err(ResolutionError {
                failed_segment: 0,
                container: None,
            });
        }

        if absolute {
            return self.try_resolve_path_from_scope(self.root, path, 0, None);
        }

        // Relative resolution: try from current scope and walk up parents
        let mut current = Some(scope);
        let mut best_error: Option<ResolutionError> = None;

        while let Some(scope_id) = current {
            match self.try_resolve_path_from_scope(scope_id, path, 0, None) {
                Ok(def_id) => return Ok(def_id),
                Err(e) => {
                    // Keep the error that made the most progress
                    if best_error
                        .as_ref()
                        .is_none_or(|best| e.failed_segment > best.failed_segment)
                    {
                        best_error = Some(e);
                    }
                    current = self.scopes[scope_id.0].parent;
                }
            }
        }

        Err(best_error.unwrap_or(ResolutionError {
            failed_segment: 0,
            container: None,
        }))
    }

    /// Resolves a path starting from a specific scope, returning detailed error info on failure.
    ///
    /// The `segment_offset` parameter is used to track the original segment index when
    /// recursing into child scopes, so error messages can point to the correct segment.
    fn try_resolve_path_from_scope(
        &self,
        scope: ScopeId,
        path: &[&str],
        segment_offset: usize,
        container: Option<DefId>,
    ) -> Result<DefId, ResolutionError> {
        if path.is_empty() {
            return Err(ResolutionError {
                failed_segment: segment_offset,
                container,
            });
        }

        let scope_data = &self.scopes[scope.0];

        if path.len() == 1 {
            // Single segment - check definitions
            if let Some(def_id) = scope_data.get_definition(path[0]) {
                return Ok(def_id);
            }
            return Err(ResolutionError {
                failed_segment: segment_offset,
                container,
            });
        }

        // Multi-segment path - first segment might be a child scope or a definition

        // First, check if it's a child scope
        if let Some(&child_scope) = scope_data.children.get(path[0]) {
            let child_container = self.scopes[child_scope.0].def_id;
            return self.try_resolve_path_from_scope(
                child_scope,
                &path[1..],
                segment_offset + 1,
                child_container,
            );
        }

        // If not a child scope, check if it's a definition (like an enum) with its own scope
        if let Some(def_id) = scope_data.get_definition(path[0])
            && let Some(def_scope) = self.find_scope_for_def(def_id)
        {
            return self.try_resolve_path_from_scope(
                def_scope,
                &path[1..],
                segment_offset + 1,
                Some(def_id),
            );
        }

        Err(ResolutionError {
            failed_segment: segment_offset,
            container,
        })
    }

    /// Finds all enums in a scope that contain a specific enumerator.
    #[must_use]
    pub fn find_enums_with_enumerator(
        &self,
        scope: ScopeId,
        enumerator: &str,
        definitions: &Arena<Def>,
    ) -> Vec<DefId> {
        let mut results = Vec::new();
        let scope_data = &self.scopes[scope.0];

        // Check all definitions in this scope
        for (_, def_ids) in scope_data.definitions.iter() {
            for &def_id in def_ids {
                let def = definitions.get(def_id);
                if let crate::hir::DefKind::Enum(enum_ty) = &def.kind {
                    // Check each field constant
                    for &field_id in &enum_ty.fields {
                        let field_def = definitions.get(field_id);
                        if field_def.ident.name == enumerator {
                            results.push(def_id);
                            break;
                        }
                    }
                }
            }
        }

        results
    }

    /// Looks up all module definitions matching a path.
    /// Returns all `DefId`s for reopened modules, not just the most recent.
    ///
    /// For reopened modules, the definitions Vec contains the first `def_id`,
    /// while the child scope's `def_id` contains the most recent. This collects both.
    #[must_use]
    pub fn lookup_all_modules(&self, scope: ScopeId, path: &[&str]) -> Vec<DefId> {
        if path.is_empty() {
            return Vec::new();
        }

        let scope_data = &self.scopes[scope.0];

        if path.len() == 1 {
            let mut result = Vec::new();

            // Get DefIds from definitions (includes first module block)
            if let Some(def_ids) = scope_data.definitions.get(path[0]) {
                result.extend(def_ids.iter().copied());
            }

            // Also check child scope's def_id (has the most recent for reopened modules)
            if let Some(&child_scope_id) = scope_data.children.get(path[0])
                && let Some(def_id) = self.scopes[child_scope_id.0].def_id
                && !result.contains(&def_id)
            {
                result.push(def_id);
            }

            result
        } else {
            // Navigate to child scope and continue
            if let Some(&child_scope) = scope_data.children.get(path[0]) {
                self.lookup_all_modules(child_scope, &path[1..])
            } else {
                Vec::new()
            }
        }
    }

    /// Finds the scope ID for a given definition.
    #[must_use]
    pub fn find_scope_for_def(&self, def_id: DefId) -> Option<ScopeId> {
        self.owned_scope.get(&def_id).copied()
    }

    /// Finds the scope that contains a definition.
    #[must_use]
    pub fn find_scope_containing_def(&self, def_id: DefId) -> Option<ScopeId> {
        self.containing_scope.get(&def_id).copied()
    }

    /// Checks if a scope belongs to an interface definition.
    #[must_use]
    pub fn is_interface_scope(&self, scope_id: ScopeId, definitions: &Arena<Def>) -> bool {
        if let Some(def_id) = self.scopes[scope_id.0].def_id {
            let def = definitions.get(def_id);
            matches!(def.kind, crate::hir::DefKind::Interface(_))
        } else {
            false
        }
    }

    /// Gets all visible enums from a scope (including parent scopes).
    #[must_use]
    pub fn get_visible_enums(&self, scope: ScopeId, definitions: &Arena<Def>) -> Vec<DefId> {
        let mut results = Vec::new();
        let mut current = Some(scope);

        while let Some(scope_id) = current {
            let scope_data = &self.scopes[scope_id.0];

            // Add all enums from this scope
            for (_, def_ids) in scope_data.definitions.iter() {
                for &def_id in def_ids {
                    let def = definitions.get(def_id);
                    if matches!(def.kind, crate::hir::DefKind::Enum(_)) {
                        results.push(def_id);
                    }
                }
            }

            // Move to parent
            current = scope_data.parent;
        }

        results
    }
}
