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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_alloc::arena::Arena;
use ic_syntax::Ident;

use crate::hir::{self, Def, DefId, DefKind, Ty, TyKind};
use crate::scope::ScopeTree;

/// Error returned when path resolution fails.
#[derive(Debug, Clone)]
pub struct PathResolutionError<'a> {
    /// The identifier segment that could not be resolved.
    pub segment: &'a Ident,
    /// The container definition we were searching in, if any.
    pub container: Option<DefId>,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub definitions: Arena<hir::Def>,

    // Scope hierarchy for name resolution
    pub scopes: ScopeTree,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Creates a new context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            definitions: Arena::default(),
            scopes: ScopeTree::new(),
        }
    }

    /// Returns the type definition of the specified type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    #[must_use]
    pub fn type_of(&self, id: DefId) -> &Def {
        self.definitions.get(id)
    }

    /// Similar to `type_of`, but will resolve the underlying type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    #[must_use]
    pub fn base_type_of(&self, id: DefId) -> Ty {
        let ty = self.type_of(id);
        match &ty.kind {
            DefKind::Alias(v) => match v.ty.kind {
                TyKind::Adt(id) => self.base_type_of(id),
                _ => v.ty.clone(),
            },
            _ => Ty {
                kind: TyKind::Adt(id),
                span: ty.span,
            },
        }
    }

    /// Returns the root scope ID.
    #[must_use]
    pub fn root_scope(&self) -> crate::scope::ScopeId {
        self.scopes.root()
    }

    /// Resolves a name in the given scope.
    #[must_use]
    pub fn resolve_name(&self, scope: crate::scope::ScopeId, name: &str) -> Option<DefId> {
        self.scopes.resolve_name(scope, name)
    }

    /// Resolves a path starting from the given scope.
    #[must_use]
    pub fn resolve_path(&self, scope: crate::scope::ScopeId, path: &[&str]) -> Option<DefId> {
        self.scopes.resolve_path(scope, path)
    }

    /// Resolves a syntax path starting from the given scope.
    ///
    /// # Errors
    ///
    /// Returns an error containing the failing segment and the container `DefId` where
    /// resolution failed. The container `DefId` will be None if resolution failed at
    /// the top-level scope.
    pub fn resolve_syntax_path<'a>(
        &self,
        scope: crate::scope::ScopeId,
        path: &'a ic_syntax::Path,
    ) -> Result<DefId, PathResolutionError<'a>> {
        let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();

        if path.leading_colons.is_some() {
            self.resolve_from_scope(self.root_scope(), &segments, &path.segments)
        } else {
            self.resolve_relative_path(scope, &segments, &path.segments)
        }
    }

    /// Resolves a relative path by trying from the current scope and walking up parents.
    fn resolve_relative_path<'a>(
        &self,
        start_scope: crate::scope::ScopeId,
        segments: &[&str],
        path_segments: &'a [Ident],
    ) -> Result<DefId, PathResolutionError<'a>> {
        let mut current = Some(start_scope);
        let mut first_error = None;

        while let Some(scope_to_try) = current {
            match self.resolve_from_scope(scope_to_try, segments, path_segments) {
                Ok(def_id) => return Ok(def_id),
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                    current = self.scopes.get_scope(scope_to_try).parent;
                }
            }
        }

        Err(first_error.unwrap_or_else(|| PathResolutionError {
            segment: path_segments.first().unwrap_or(&path_segments[0]),
            container: None,
        }))
    }

    /// Resolves a path from a specific starting scope.
    /// Returns the `DefId` if found, or an error with the failing segment.
    fn resolve_from_scope<'a>(
        &self,
        start_scope: crate::scope::ScopeId,
        segments: &[&str],
        path_segments: &'a [Ident],
    ) -> Result<DefId, PathResolutionError<'a>> {
        let mut scope_id = start_scope;
        let mut container_def_id = None;

        for (i, &segment_name) in segments.iter().enumerate() {
            let scope_data = self.scopes.get_scope(scope_id);

            if i == segments.len() - 1 {
                if let Some(def_ids) = scope_data.definitions.get(segment_name) {
                    if let Some(&def_id) = def_ids.last() {
                        return Ok(def_id);
                    }
                }
                return Err(PathResolutionError {
                    segment: &path_segments[i],
                    container: container_def_id,
                });
            }

            if let Some(&child_scope) = scope_data.children.get(segment_name) {
                scope_id = child_scope;
                container_def_id = self.scopes.get_scope(child_scope).def_id;
            } else if let Some(def_ids) = scope_data.definitions.get(segment_name) {
                if let Some(&def_id) = def_ids.last() {
                    if let Some(def_scope) = self.scopes.find_scope_for_def(def_id) {
                        scope_id = def_scope;
                        container_def_id = Some(def_id);
                    } else {
                        return Err(PathResolutionError {
                            segment: &path_segments[i],
                            container: container_def_id,
                        });
                    }
                } else {
                    return Err(PathResolutionError {
                        segment: &path_segments[i],
                        container: container_def_id,
                    });
                }
            } else {
                return Err(PathResolutionError {
                    segment: &path_segments[i],
                    container: container_def_id,
                });
            }
        }

        Err(PathResolutionError {
            segment: &path_segments[0],
            container: None,
        })
    }

    /// Creates a child scope with the given name.
    pub fn create_child_scope(
        &mut self,
        parent: crate::scope::ScopeId,
        name: String,
        def_id: Option<DefId>,
    ) -> crate::scope::ScopeId {
        self.scopes.create_child_scope(parent, name, def_id)
    }

    /// Adds a definition to a scope.
    pub fn add_definition_to_scope(
        &mut self,
        scope: crate::scope::ScopeId,
        name: String,
        def_id: DefId,
    ) {
        self.scopes.add_definition(scope, name, def_id);
    }

    /// Adds an annotation definition to a scope.
    pub fn add_annotation_to_scope(
        &mut self,
        scope: crate::scope::ScopeId,
        name: String,
        def_id: DefId,
    ) {
        self.scopes.add_annotation(scope, name, def_id);
    }

    /// Resolves an annotation syntax path starting from the given scope.
    #[must_use]
    pub fn resolve_annotation_syntax_path(
        &self,
        scope: crate::scope::ScopeId,
        path: &ic_syntax::Path,
    ) -> Option<DefId> {
        let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();
        self.scopes.resolve_annotation_path(scope, &segments)
    }

    /// Returns the `DefId` of the given type, if one exists. For arrays,
    /// sequences, and maps, this will return the element type if it points to
    /// a definition.
    #[must_use]
    #[allow(clippy::only_used_in_recursion)]
    pub fn def_of(&self, ty: &Ty) -> Option<DefId> {
        match &ty.kind {
            TyKind::Array { ty, .. }
            | TyKind::Sequence { ty, .. }
            | TyKind::Map { elem: ty, .. } => self.def_of(ty),
            TyKind::Adt(id) => Some(*id),
            _ => None,
        }
    }

    #[must_use]
    pub fn qualified_name(&self, id: DefId) -> String {
        let def = self.type_of(id);
        let mut parts = vec![def.ident.name.clone()];
        let mut current = def.parent;
        while let Some(parent_id) = current {
            let parent_def = self.type_of(parent_id);
            parts.push(parent_def.ident.name.clone());
            current = parent_def.parent;
        }

        parts.reverse();
        parts.join("::")
    }

    /// Looks up a symbol by its qualified name (e.g., "`DDS::XTypes`").
    /// Starts from the root scope.
    /// For reopened modules, returns the last (most recent) definition.
    #[must_use]
    pub fn lookup_symbol(&self, qualified_name: &str) -> Option<DefId> {
        let parts: Vec<&str> = qualified_name.split("::").collect();
        if parts.is_empty() {
            return None;
        }

        self.lookup_path_from_scope(self.root_scope(), &parts)
    }

    /// Looks up all definitions for a module by its qualified name.
    /// This is useful for reopened modules which may have multiple `DefId`s.
    /// Returns empty vector if not found or if not a module.
    #[must_use]
    pub fn lookup_modules(&self, qualified_name: &str) -> Vec<DefId> {
        let parts: Vec<&str> = qualified_name.split("::").collect();
        if parts.is_empty() {
            return Vec::new();
        }

        self.lookup_all_defs_from_scope(self.root_scope(), &parts)
    }

    /// Helper to look up a path from a specific scope
    fn lookup_path_from_scope(
        &self,
        start_scope: crate::scope::ScopeId,
        parts: &[&str],
    ) -> Option<DefId> {
        // First try to resolve as a definition in current scope
        let (name, remaining) = parts.split_first()?;
        let scope = self.scopes.get_scope(start_scope);

        // For the last part, just resolve the name
        if remaining.is_empty() {
            // Get the last DefId for this name (most recent definition)
            if let Some(def_ids) = scope.definitions.get(*name) {
                return def_ids.last().copied();
            }
            return None;
        }

        // For intermediate parts, we need to find the associated scope
        // Check if there's a child scope with this name
        if let Some((_, child_scope)) = scope
            .children
            .iter()
            .find(|(child_name, _)| child_name.eq_ignore_ascii_case(name))
        {
            return self.lookup_path_from_scope(*child_scope, remaining);
        }

        // If no child scope, check if it's a definition with its own scope
        if let Some(def_ids) = scope.definitions.get(*name) {
            // Try all DefIds for this name (in case of reopened modules)
            for &def_id in def_ids {
                let def = self.definitions.get(def_id);
                match &def.kind {
                    DefKind::Module(_) | DefKind::Interface(_) | DefKind::Valuetype(_) => {
                        // These types have child scopes - find it
                        if let Some(def_scope) = self.scopes.find_scope_for_def(def_id) {
                            if let Some(result) = self.lookup_path_from_scope(def_scope, remaining)
                            {
                                return Some(result);
                            }
                        }
                    }
                    _ => {} // Other types don't have child scopes
                }
            }
        }

        None
    }

    /// Helper to look up all definitions for a path from a specific scope.
    /// Returns all `DefId`s for the target (useful for reopened modules).
    fn lookup_all_defs_from_scope(
        &self,
        start_scope: crate::scope::ScopeId,
        parts: &[&str],
    ) -> Vec<DefId> {
        let Some((name, remaining)) = parts.split_first() else {
            return vec![];
        };
        let scope = self.scopes.get_scope(start_scope);

        // For the last part, we need to collect all matching DefIds
        if remaining.is_empty() {
            let mut result = Vec::new();

            // Check definitions in current scope
            if let Some(def_ids) = scope.definitions.get(*name) {
                for &def_id in def_ids {
                    let def = self.definitions.get(def_id);
                    if matches!(def.kind, DefKind::Module(_)) {
                        result.push(def_id);
                    }
                }
            }

            // Also check child scopes with matching names (for modules)
            for (child_name, &child_scope_id) in scope.children.iter() {
                if child_name.eq_ignore_ascii_case(name) {
                    if let Some(def_id) = self.scopes.get_scope(child_scope_id).def_id {
                        let def = self.definitions.get(def_id);
                        if matches!(def.kind, DefKind::Module(_)) {
                            result.push(def_id);
                        }
                    }
                }
            }

            result
        } else {
            // For intermediate parts, we need to find the associated scope
            // Check if there's a child scope with this name
            if let Some((_, child_scope)) = scope
                .children
                .iter()
                .find(|(child_name, _)| child_name.eq_ignore_ascii_case(name))
            {
                return self.lookup_all_defs_from_scope(*child_scope, remaining);
            }

            Vec::new()
        }
    }
}
