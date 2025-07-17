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

use ic_alloc::arena::Arena;
use ic_alloc::insensitive::CaseMap;

use crate::hir::{Def, DefId};

/// A scope in the hierarchy.
#[derive(Debug)]
pub struct Scope {
    /// The definition ID of this scope (if it's a module/interface/etc).
    pub def_id: Option<DefId>,

    /// Parent scope.
    pub parent: Option<ScopeId>,

    /// Child scopes by name.
    pub children: CaseMap<ScopeId>,

    /// Local definitions in this scope.
    pub definitions: CaseMap<DefId>,
}

/// Unique identifier for a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

/// Manages the scope hierarchy.
#[derive(Debug)]
pub struct ScopeTree {
    /// All scopes in the tree.
    pub scopes: Vec<Scope>,

    /// The root scope (global scope).
    root: ScopeId,
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
        }
    }

    /// Gets the root scope.
    #[must_use]
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

        // Add to parent's children
        self.scopes[parent.0].children.insert(name, scope_id);

        scope_id
    }

    /// Adds a definition to a scope.
    pub fn add_definition(&mut self, scope: ScopeId, name: String, def_id: DefId) {
        self.scopes[scope.0].definitions.insert(name, def_id);
    }

    /// Gets a scope by ID.
    #[must_use]
    pub fn get_scope(&self, scope: ScopeId) -> &Scope {
        &self.scopes[scope.0]
    }

    /// Resolves a single name segment in a scope (looks in this scope and parents).
    #[must_use]
    pub fn resolve_name(&self, scope: ScopeId, name: &str) -> Option<DefId> {
        let mut current = Some(scope);

        while let Some(scope_id) = current {
            let scope = &self.scopes[scope_id.0];

            // Check local definitions
            if let Some(&def_id) = scope.definitions.get(name) {
                return Some(def_id);
            }

            // Check child scopes (for module names)
            if let Some(&child_scope_id) = scope.children.get(name) {
                if let Some(def_id) = self.scopes[child_scope_id.0].def_id {
                    return Some(def_id);
                }
            }

            // Move to parent
            current = scope.parent;
        }

        None
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

    /// Resolves a path starting from a specific scope.
    fn resolve_path_from_scope(&self, scope: ScopeId, path: &[&str]) -> Option<DefId> {
        if path.is_empty() {
            return None;
        }

        let scope_data = &self.scopes[scope.0];

        if path.len() == 1 {
            // Single segment - check definitions
            return scope_data.definitions.get(path[0]).copied();
        }

        // Multi-segment path - first segment should be a child scope
        if let Some(&child_scope) = scope_data.children.get(path[0]) {
            // Recurse into child scope
            return self.resolve_path_from_scope(child_scope, &path[1..]);
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
        for (_, &def_id) in scope_data.definitions.iter() {
            let def = definitions.get(def_id);
            if let crate::hir::DefKind::Enum(enum_ty) = &def.kind {
                for field in &enum_ty.fields {
                    if field.ident.name == enumerator {
                        results.push(def_id);
                        break;
                    }
                }
            }
        }

        results
    }

    /// Finds the scope ID for a given definition.
    #[must_use]
    pub fn find_scope_for_def(&self, def_id: DefId) -> Option<ScopeId> {
        for (idx, scope) in self.scopes.iter().enumerate() {
            if scope.def_id == Some(def_id) {
                return Some(ScopeId(idx));
            }
        }
        None
    }
    
    /// Finds the scope that contains a definition.
    pub fn find_scope_containing_def(&self, def_id: DefId) -> Option<ScopeId> {
        for (idx, scope) in self.scopes.iter().enumerate() {
            if scope.definitions.values().any(|&id| id == def_id) {
                return Some(ScopeId(idx));
            }
        }
        None
    }

    /// Gets all visible enums from a scope (including parent scopes).
    #[must_use]
    pub fn get_visible_enums(&self, scope: ScopeId, definitions: &Arena<Def>) -> Vec<DefId> {
        let mut results = Vec::new();
        let mut current = Some(scope);

        while let Some(scope_id) = current {
            let scope_data = &self.scopes[scope_id.0];

            // Add all enums from this scope
            for (_, &def_id) in scope_data.definitions.iter() {
                let def = definitions.get(def_id);
                if matches!(def.kind, crate::hir::DefKind::Enum(_)) {
                    results.push(def_id);
                }
            }

            // Move to parent
            current = scope_data.parent;
        }

        results
    }
}
