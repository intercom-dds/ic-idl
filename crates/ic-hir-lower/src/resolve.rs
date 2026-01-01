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

//! IDL-specific path resolution and module handling utilities.

use std::collections::HashMap;

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::{Label, error_span};
use ic_hir::Context;
use ic_hir::diagnostics::Diagnostics;
use ic_hir::hir::DefId;
use ic_hir::scope::{ScopeId, ScopeTree};
use ic_syntax::{Ident, Span};
use tracing::trace;

/// Error returned when path resolution fails.
#[derive(Debug, Clone)]
pub struct PathResolutionError<'a> {
    /// The identifier segment that could not be resolved.
    pub segment: &'a Ident,
    /// The container definition we were searching in, if any.
    pub container: Option<DefId>,
}

/// Resolve a path to a `DefId`.
pub fn resolve_path<'a>(
    ctx: &Context,
    scope: ScopeId,
    path: &'a ic_syntax::Path,
) -> Result<DefId, PathResolutionError<'a>> {
    let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();
    let absolute = path.leading_colons.is_some();

    let result = ctx
        .scopes
        .try_resolve_path(scope, &segments, absolute)
        .map_err(|e| PathResolutionError {
            segment: &path.segments[e.failed_segment],
            container: e.container,
        });

    if tracing::enabled!(tracing::Level::TRACE) {
        let path_str = segments.join("::");
        match &result {
            Ok(def_id) => {
                let def = ctx.definitions.get(*def_id);
                trace!(
                    path = %path_str,
                    ?def_id,
                    kind = def.kind.kind_name(),
                    absolute,
                    "resolved"
                );
            }
            Err(e) => {
                trace!(
                    path = %path_str,
                    failed_segment = %e.segment.name,
                    absolute,
                    "unresolved"
                );
            }
        }
    }

    result
}

/// Find or create a module scope, handling IDL module reopening semantics.
///
/// IDL allows reopening modules (defining the same module multiple times to add more content).
/// This function tracks module definitions and warns on inconsistent capitalization.
pub fn find_or_create_module(
    scopes: &mut ScopeTree,
    parent: ScopeId,
    name: &str,
    span: Span,
    module_scopes: &mut HashMap<ScopeId, CaseMap<(ScopeId, Span)>>,
    diagnostics: &mut Diagnostics,
) -> ScopeId {
    let parent_modules = module_scopes.entry(parent).or_default();
    if let Some(&(scope_id, original_span)) = parent_modules.get(name) {
        if let Some(canonical_name) = parent_modules.get_key(name)
            && canonical_name != name
        {
            diagnostics.errors.push(
                error_span(
                    format!(
                        "inconsistent capitalization: module `{name}` was previously defined as \
                         `{canonical_name}`"
                    ),
                    Label::new(span).message("module reopened here"),
                )
                .label(Label::new(original_span).message("first defined here")),
            );
        }
        return scope_id;
    }

    let scope_id = scopes.create_child_scope(parent, name.to_string(), None);
    parent_modules.insert(name, (scope_id, span));
    scope_id
}

/// Resolve an annotation path to a `DefId`.
#[must_use]
pub fn resolve_annotation(ctx: &Context, scope: ScopeId, path: &ic_syntax::Path) -> Option<DefId> {
    let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();
    ctx.scopes.resolve_annotation_path(scope, &segments)
}
