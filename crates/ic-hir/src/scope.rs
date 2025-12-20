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

use std::collections::HashMap;

use ic_alloc::arena::Arena;
use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::{Label, error_span};

use crate::hir::{Def, DefId};

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
    /// Uses Vec<DefId> to support multiple definitions with the same name (e.g., reopened modules)
    pub definitions: CaseMap<Vec<DefId>>,

    /// Local annotation definitions in this scope (separate namespace).
    /// Uses Vec<DefId> to support multiple definitions with the same name
    pub annotations: CaseMap<Vec<DefId>>,
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

    /// Reverse mapping from `DefId` to the scope that contains it.
    /// This is maintained for efficient lookup.
    def_to_scope: HashMap<DefId, ScopeId>,

    /// Reverse mapping from `DefId` to the scope it owns.
    /// Maps definitions to the scopes where `scope.def_id == def_id`.
    /// This is maintained for efficient O(1) lookup in `find_scope_for_def`.
    def_to_owned_scope: HashMap<DefId, ScopeId>,
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
            annotations: CaseMap::new(),
        };

        Self {
            scopes: vec![root_scope],
            root: ScopeId(0),
            def_to_scope: HashMap::new(),
            def_to_owned_scope: HashMap::new(),
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
            annotations: CaseMap::new(),
        };

        self.scopes.push(scope);

        // Add to parent's children
        self.scopes[parent.0].children.insert(name, scope_id);

        scope_id
    }

    /// Adds a definition to a scope.
    /// Multiple definitions with the same name are allowed (e.g., reopened modules).
    pub fn add_definition(&mut self, scope: ScopeId, name: String, def_id: DefId) {
        Self::add_to_definitions(&mut self.scopes[scope.0].definitions, name, def_id);
        // Update reverse mapping
        self.def_to_scope.insert(def_id, scope);
    }

    /// Adds an annotation definition to a scope.
    /// Multiple definitions with the same name are allowed.
    pub fn add_annotation(&mut self, scope: ScopeId, name: String, def_id: DefId) {
        Self::add_to_definitions(&mut self.scopes[scope.0].annotations, name, def_id);
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
        self.def_to_owned_scope.insert(def_id, scope_id);
    }

    /// Resolves a single name segment in a scope (looks in this scope and parents).
    #[must_use]
    pub fn resolve_name(&self, scope: ScopeId, name: &str) -> Option<DefId> {
        let mut current = Some(scope);

        while let Some(scope_id) = current {
            let scope = &self.scopes[scope_id.0];

            // Check local definitions
            if let Some(def_ids) = scope.definitions.get(name)
                && let Some(&def_id) = def_ids.last()
            {
                return Some(def_id);
            }

            // Check child scopes (for module names)
            if let Some(&child_scope_id) = scope.children.get(name)
                && let Some(def_id) = self.scopes[child_scope_id.0].def_id
            {
                return Some(def_id);
            }

            // Move to parent
            current = scope.parent;
        }

        None
    }

    /// Resolves an annotation name in a scope (looks in this scope and parents).
    #[must_use]
    pub fn resolve_annotation(&self, scope: ScopeId, name: &str) -> Option<DefId> {
        let mut current = Some(scope);

        while let Some(scope_id) = current {
            let scope = &self.scopes[scope_id.0];

            // Check local annotation definitions
            if let Some(def_ids) = scope.annotations.get(name)
                && let Some(&def_id) = def_ids.last()
            {
                return Some(def_id);
            }

            // Move to parent
            current = scope.parent;
        }

        None
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
            let scope = &self.scopes[scope_id.0];

            // Check local definitions
            if let Some(def_ids) = scope.definitions.get(name)
                && let Some(&def_id) = def_ids.last()
            {
                // Found the definition - but check if it's accessible
                // If we found it in an interface scope and we started outside that interface,
                // it's not accessible
                if self.is_interface_scope(scope_id, definitions)
                    && !self.is_inside_scope(starting_scope, scope_id)
                {
                    // This type is inside an interface but we're outside - not accessible
                    return None;
                }
                return Some(def_id);
            }

            // Check child scopes (for module names and interfaces)
            if let Some(&child_scope_id) = scope.children.get(name)
                && let Some(def_id) = self.scopes[child_scope_id.0].def_id
            {
                return Some(def_id);
            }

            // Before moving to parent, check if we would cross an interface boundary
            if let Some(parent_scope_id) = scope.parent {
                // If the parent is an interface scope and we started outside of it,
                // don't enter the interface scope when resolving unqualified names
                if self.is_interface_scope(parent_scope_id, definitions)
                    && !self.is_inside_scope(starting_scope, parent_scope_id)
                {
                    // Don't cross into the interface when resolving unqualified names
                    return None;
                }
            }

            // Move to parent
            current = scope.parent;
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

    /// Resolves a path starting from a scope.
    #[must_use]
    pub fn resolve_path(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        if path.is_empty() {
            return None;
        }

        // Try resolving as a relative path first
        if let Some(def_id) = self.resolve_relative_path(scope, path) {
            return Some(def_id);
        }

        // Try resolving as an absolute path from root
        self.resolve_absolute_path(path)
    }

    /// Resolves an annotation path starting from a scope.
    #[must_use]
    pub fn resolve_annotation_path(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        if path.is_empty() {
            return None;
        }

        // Try resolving as a relative path first
        if let Some(def_id) = self.resolve_relative_annotation_path(scope, path) {
            return Some(def_id);
        }

        // Try resolving as an absolute path from root
        self.resolve_absolute_annotation_path(path)
    }

    /// Resolves a relative path from a scope.
    fn resolve_relative_path(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        let mut current = Some(scope);

        // Try starting from current scope and walking up
        while let Some(scope_id) = current {
            if let Some(def_id) = self.resolve_path_from_scope(scope_id, path) {
                return Some(def_id);
            }

            // Move to parent
            current = self.scopes[scope_id.0].parent;
        }

        None
    }

    /// Resolves an absolute path from root.
    fn resolve_absolute_path(&self, path: &[&str]) -> Option<DefId> {
        self.resolve_path_from_scope(self.root, path)
    }

    /// Resolves a relative annotation path from a scope.
    fn resolve_relative_annotation_path(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        let mut current = Some(scope);

        // Try starting from current scope and walking up
        while let Some(scope_id) = current {
            if let Some(def_id) = self.resolve_annotation_path_from_scope(scope_id, path) {
                return Some(def_id);
            }

            // Move to parent
            current = self.scopes[scope_id.0].parent;
        }

        None
    }

    /// Resolves an absolute annotation path from root.
    fn resolve_absolute_annotation_path(&self, path: &[&str]) -> Option<DefId> {
        self.resolve_annotation_path_from_scope(self.root, path)
    }

    /// Resolves a path starting from a specific scope.
    fn resolve_path_from_scope(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        if path.is_empty() {
            return None;
        }

        let scope_data = &self.scopes[scope.0];

        if path.len() == 1 {
            // Single segment - check definitions
            if let Some(def_ids) = scope_data.definitions.get(path[0]) {
                return def_ids.last().copied();
            }
            return None;
        }

        // Multi-segment path - first segment might be a definition or a child scope

        // First, check if it's a child scope
        if let Some(&child_scope) = scope_data.children.get(path[0]) {
            // Recurse into child scope
            return self.resolve_path_from_scope(child_scope, &path[1..]);
        }

        // If not a child scope, check if it's a definition (like an enum)
        // whose own scope we should look into
        if let Some(def_ids) = scope_data.definitions.get(path[0])
            && let Some(&def_id) = def_ids.last()
        {
            // Find the scope for this definition
            if let Some(def_scope) = self.find_scope_for_def(def_id) {
                // Continue resolution from the definition's scope
                return self.resolve_path_from_scope(def_scope, &path[1..]);
            }
        }

        None
    }

    /// Resolves an annotation path starting from a specific scope.
    fn resolve_annotation_path_from_scope(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        if path.is_empty() {
            return None;
        }

        let scope_data = &self.scopes[scope.0];

        if path.len() == 1 {
            // Single segment - check annotation definitions
            if let Some(def_ids) = scope_data.annotations.get(path[0]) {
                return def_ids.last().copied();
            }
            return None;
        }

        // Multi-segment path - first segment might be a child scope
        // (modules can contain annotation definitions)
        if let Some(&child_scope) = scope_data.children.get(path[0]) {
            // Recurse into child scope
            return self.resolve_annotation_path_from_scope(child_scope, &path[1..]);
        }

        None
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

    /// Finds the scope ID for a given definition.
    #[must_use]
    pub fn find_scope_for_def(&self, def_id: DefId) -> Option<ScopeId> {
        self.def_to_owned_scope.get(&def_id).copied()
    }

    /// Finds the scope that contains a definition.
    #[must_use]
    pub fn find_scope_containing_def(&self, def_id: DefId) -> Option<ScopeId> {
        self.def_to_scope.get(&def_id).copied()
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

    /// Find or create a module scope.
    /// This handles module reopening by returning the existing scope if found.
    /// Used during lowering to support IDL's module reopening feature.
    ///
    /// The `module_scopes` parameter tracks module reopening across the lowering process.
    /// It maps from `parent_scope` to a `CaseMap` of module names to `(scope_id, original_span)`.
    pub fn find_or_create_module(
        &mut self,
        parent: ScopeId,
        name: &str,
        span: ic_syntax::Span,
        module_scopes: &mut HashMap<ScopeId, CaseMap<(ScopeId, ic_syntax::Span)>>,
        diagnostics: &mut crate::lower::Diagnostics,
    ) -> ScopeId {
        let parent_modules = module_scopes.entry(parent).or_insert_with(CaseMap::new);

        if let Some(&(scope_id, original_span)) = parent_modules.get(name) {
            if let Some(canonical_name) = parent_modules.get_key(name)
                && canonical_name != name
            {
                diagnostics.errors.push(
                    error_span(
                        format!(
                            "inconsistent capitalization: module `{name}` was previously defined \
                             as `{canonical_name}`"
                        ),
                        Label::new(span).message("module reopened here"),
                    )
                    .label(Label::new(original_span).message("first defined here")),
                );
            }
            return scope_id;
        }

        let scope_id = self.create_child_scope(parent, name.to_string(), None);
        let parent_modules = module_scopes.entry(parent).or_insert_with(CaseMap::new);
        parent_modules.insert(name, (scope_id, span));
        scope_id
    }
}
