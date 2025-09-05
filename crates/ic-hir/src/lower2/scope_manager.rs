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

//! Scope management for name resolution.

use std::collections::HashMap;

use ic_alloc::insensitive::CaseMap;
use ic_syntax::Path;

use crate::hir::DefId;
use crate::scope::ScopeId;

/// Mode for name resolution.
#[derive(Clone, Copy, Debug)]
pub enum ResolveMode {
    /// Regular unqualified name lookup.
    Unqualified,
    /// Qualified path lookup.
    Qualified,
    /// Augment search with parent interfaces.
    InsideInterface(DefId),
}

/// Wrapper around the Context's scope tree to provide additional functionality.
pub struct ScopeTree {
    /// Root scope ID from the context.
    root: ScopeId,

    /// Additional scope metadata for module reopening.
    /// Maps from `parent_scope` to a `CaseMap` of module names to (`scope_id`, `original_span`).
    /// `CaseMap` automatically handles case-insensitive lookups while preserving original names.
    module_scopes: HashMap<ScopeId, CaseMap<(ScopeId, ic_syntax::Span)>>,
}

impl ScopeTree {
    pub fn new(root: ScopeId) -> Self {
        Self {
            root,
            module_scopes: HashMap::new(),
        }
    }

    pub fn root(&self) -> ScopeId {
        self.root
    }

    /// Find or create a module scope.
    /// This handles module reopening by returning the existing scope if found.
    pub fn find_or_create_module(
        &mut self,
        parent: ScopeId,
        name: &str,
        span: ic_syntax::Span,
        ctx: &mut crate::Context,
        diagnostics: &mut super::Diagnostics,
    ) -> ScopeId {
        // Get or create the CaseMap for this parent scope
        let parent_modules = self
            .module_scopes
            .entry(parent)
            .or_insert_with(CaseMap::new);

        if let Some(&(scope_id, original_span)) = parent_modules.get(name) {
            // Module already exists - check if the name differs in case
            if let Some(canonical_name) = parent_modules.get_key(name) {
                if canonical_name != name {
                    use ic_diagnostic::{Label, warn_span};
                    diagnostics.warnings.push(
                        warn_span(
                            format!(
                                "inconsistent capitalization: module `{name}` was previously \
                                 defined as `{canonical_name}`"
                            ),
                            Label::new(span).message("module reopened here"),
                        )
                        .label(Label::new(original_span).message("first defined here")),
                    );
                }
            }
            return scope_id;
        }

        // Create new module scope
        let scope_id = ctx
            .scopes
            .create_child_scope(parent, name.to_string(), None);
        parent_modules.insert(name, (scope_id, span));
        scope_id
    }

    /// Resolve a name in the given scope with the specified mode.
    pub fn resolve_name(
        &self,
        ctx: &crate::Context,
        start: ScopeId,
        name: &str,
        mode: ResolveMode,
    ) -> Option<DefId> {
        match mode {
            ResolveMode::Unqualified => {
                // Search current scope and parents
                self.resolve_unqualified(ctx, start, name)
            }
            ResolveMode::Qualified => {
                // Only search the specified scope
                self.lookup_in_scope(ctx, start, name)
            }
            ResolveMode::InsideInterface(interface_id) => {
                // Search current scope, parents, and inherited interfaces
                self.resolve_in_interface_context(ctx, start, name, interface_id)
            }
        }
    }

    /// Resolve a path starting from the given scope.
    pub fn resolve_path(&self, ctx: &crate::Context, start: ScopeId, path: &Path) -> Option<DefId> {
        let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();

        if segments.is_empty() {
            return None;
        }

        // Start from root for absolute paths
        let start_scope = if path.leading_colons.is_some() {
            self.root
        } else {
            start
        };

        // Use the core ScopeTree's resolve_path which properly handles multi-segment paths
        ctx.scopes.resolve_path(start_scope, &segments)
    }

    /// Look up a name in a specific scope only.
    fn lookup_in_scope(&self, ctx: &crate::Context, scope: ScopeId, name: &str) -> Option<DefId> {
        ctx.scopes.resolve_name(scope, name)
    }

    /// Resolve a name by searching current scope and parents.
    fn resolve_unqualified(
        &self,
        ctx: &crate::Context,
        start: ScopeId,
        name: &str,
    ) -> Option<DefId> {
        let mut current = Some(start);

        while let Some(scope_id) = current {
            // Check current scope
            if let Some(def_id) = self.lookup_in_scope(ctx, scope_id, name) {
                return Some(def_id);
            }

            // Move to parent scope
            current = ctx.scopes.get_scope(scope_id).parent;

            // Skip interface scopes during unqualified lookup from outside
            if let Some(parent_id) = current {
                if self.is_interface_scope(ctx, parent_id) {
                    // Skip to the interface's parent
                    current = ctx.scopes.get_scope(parent_id).parent;
                }
            }
        }

        None
    }

    /// Resolve in interface context, including inherited interfaces.
    fn resolve_in_interface_context(
        &self,
        ctx: &crate::Context,
        start: ScopeId,
        name: &str,
        interface_id: DefId,
    ) -> Option<DefId> {
        // First try normal unqualified resolution
        if let Some(def_id) = self.resolve_unqualified(ctx, start, name) {
            return Some(def_id);
        }

        // Then check inherited interfaces
        self.search_inherited_interfaces(ctx, name, interface_id)
    }

    /// Search for a name in inherited interfaces.
    fn search_inherited_interfaces(
        &self,
        ctx: &crate::Context,
        name: &str,
        interface_id: DefId,
    ) -> Option<DefId> {
        let def = ctx.definitions.get(interface_id);

        if let crate::hir::DefKind::Interface(interface) = &def.kind {
            // Check each parent interface
            for &parent_id in &interface.parents {
                // Check parent's scope
                if let Some(parent_scope) = ctx.scopes.find_scope_for_def(parent_id) {
                    if let Some(def_id) = self.lookup_in_scope(ctx, parent_scope, name) {
                        return Some(def_id);
                    }
                }

                // Recursively check parent's parents
                if let Some(def_id) = self.search_inherited_interfaces(ctx, name, parent_id) {
                    return Some(def_id);
                }
            }
        }

        None
    }

    /// Check if a scope belongs to an interface.
    fn is_interface_scope(&self, ctx: &crate::Context, scope_id: ScopeId) -> bool {
        // Check if this scope has an interface as its definition
        if let Some(def_id) = ctx.scopes.get_scope(scope_id).def_id {
            let def = ctx.definitions.get(def_id);
            matches!(def.kind, crate::hir::DefKind::Interface(_))
        } else {
            false
        }
    }
}
