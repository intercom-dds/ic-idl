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

use std::collections::HashMap;

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::{Label, error_span};
use ic_hir::Context;
use ic_hir::hir::{DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir::scope::{ScopeId, ScopeTree};
use ic_syntax::{Ident, Path, Span, Type as AstType};
use tracing::trace;

use crate::eval::ConstEvaluator;
use crate::utils::{path_span, path_to_string};
use crate::{Diagnostics, LoweringContext};

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

/// Resolve a path, trying `annotation_scope` first (if set) before falling
/// back to `scope`. Shared by `eval.rs`'s two const-evaluation contexts,
/// which both need the same annotation-argument-vs-enclosing-scope
/// resolution policy.
pub fn resolve_with_fallback<'a>(
    ctx: &Context,
    scope: ScopeId,
    annotation_scope: Option<ScopeId>,
    path: &'a ic_syntax::Path,
) -> Result<DefId, PathResolutionError<'a>> {
    if let Some(ann) = annotation_scope {
        resolve_path(ctx, ann, path).or_else(|_| resolve_path(ctx, scope, path))
    } else {
        resolve_path(ctx, scope, path)
    }
}

/// Resolves AST types to HIR types.
pub struct TypeResolver<'ctx> {
    ctx: &'ctx mut LoweringContext,
    current_scope: ScopeId,
}

impl<'ctx> TypeResolver<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext, current_scope: ScopeId) -> Self {
        Self { ctx, current_scope }
    }

    /// Resolve an AST type to a HIR type.
    pub fn resolve_type(&mut self, ast_type: &AstType) -> Option<Ty> {
        match ast_type {
            AstType::Named(path) => self.resolve_path_type(path),
            AstType::String(s) => Some(Ty {
                span: s.span,
                kind: TyKind::String {
                    wide: matches!(s.kind, ic_syntax::StringKind::Wide),
                    bound: s.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                    bound_span: s.bound.as_ref().map(|e| e.span),
                },
            }),
            AstType::Sequence(seq) => {
                let elem_ty = self.resolve_type(&seq.element)?;
                Some(Ty {
                    span: seq.span,
                    kind: TyKind::Sequence {
                        ty: Box::new(elem_ty),
                        bound: seq.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                        bound_span: seq.bound.as_ref().map(|e| e.span),
                    },
                })
            }
            AstType::Map(m) => {
                let key_ty = self.resolve_type(&m.key)?;
                let elem_ty = self.resolve_type(&m.value)?;
                Some(Ty {
                    span: m.span,
                    kind: TyKind::Map {
                        key: Box::new(key_ty),
                        elem: Box::new(elem_ty),
                        bound: m.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                        bound_span: m.bound.as_ref().map(|e| e.span),
                    },
                })
            }
            AstType::Fixed(f) => Some(Ty {
                span: f.span,
                kind: TyKind::Fixed,
            }),
        }
    }

    /// Resolve a path type (could be a primitive or named type).
    pub fn resolve_path_type(&mut self, path: &Path) -> Option<Ty> {
        let span = path_span(path);

        // Check if it's a single identifier that could be a primitive
        if path.segments.len() == 1 && path.leading_colons.is_none() {
            let name = &path.segments[0].name;
            if name == "any" {
                return Some(Ty {
                    span: (span),
                    kind: TyKind::Any,
                });
            }

            // Check if it's a primitive type
            if let Some(prim) = Self::resolve_primitive(name) {
                return Some(Ty {
                    span: (span),
                    kind: TyKind::Primitive(prim),
                });
            }
        }

        // Otherwise, resolve as a named type. `resolve_path` derives
        // absolute-vs-relative from the path's leading `::` itself and
        // substitutes the root scope internally when needed, so the
        // current scope is always the right thing to pass here.
        match resolve_path(&self.ctx.context, self.current_scope, path) {
            Ok(def_id) => {
                let def = self.ctx.context.definitions.get(def_id);
                if Self::is_type_definition(&def.kind) {
                    // Check for case sensitivity issues on the entire path
                    self.check_case_consistency(path, def_id);

                    Some(Ty {
                        span: (path_span(path)),
                        kind: TyKind::Adt(def_id),
                    })
                } else {
                    self.ctx.diagnostics.errors.push(error_span(
                        format!("`{}` is not a type", path_to_string(path)),
                        Label::new(path_span(path)).message("expected a type"),
                    ));
                    None
                }
            }
            Err(err) => {
                let context = if let Some(def_id) = err.container {
                    let def = self.ctx.context.definitions.get(def_id);
                    format!("{} '{}'", def.kind.kind_name(), def.ident.name)
                } else {
                    "this scope".to_string()
                };

                self.ctx.diagnostics.errors.push(error_span(
                    format!("no type named '{}' in {context}", err.segment.name),
                    Label::new(err.segment.span).message("unknown type"),
                ));
                None
            }
        }
    }

    fn resolve_primitive(name: &str) -> Option<PrimitiveTy> {
        Some(match name {
            "void" => PrimitiveTy::Void,
            "boolean" => PrimitiveTy::Bool,
            "char" => PrimitiveTy::Char,
            "wchar" => PrimitiveTy::WChar,
            "octet" | "uint8" => PrimitiveTy::UInt8,
            "int8" => PrimitiveTy::Int8,
            "short" | "int16" => PrimitiveTy::Int16,
            "unsigned short" | "uint16" => PrimitiveTy::UInt16,
            "long" | "int32" => PrimitiveTy::Int32,
            "unsigned long" | "uint32" => PrimitiveTy::UInt32,
            "long long" | "int64" => PrimitiveTy::Int64,
            "unsigned long long" | "uint64" => PrimitiveTy::UInt64,
            "float" => PrimitiveTy::Float32,
            "double" => PrimitiveTy::Float64,
            "long double" => PrimitiveTy::Float128,
            _ => return None,
        })
    }

    fn evaluate_bound(&mut self, expr: &ic_syntax::Expr) -> Option<usize> {
        let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
        evaluator.eval_nonneg_bound(expr)
    }

    fn is_type_definition(kind: &DefKind) -> bool {
        !matches!(
            kind,
            DefKind::Annotation(_) | DefKind::Module(_) | DefKind::Const(_)
        )
    }

    /// Check if a path reference has consistent capitalization with the definition.
    fn check_case_consistency(&mut self, path: &Path, _def_id: DefId) {
        let mut current_scope = if path.leading_colons.is_some() {
            self.ctx.context.root_scope()
        } else {
            self.current_scope
        };

        for (i, segment) in path.segments.iter().enumerate() {
            let is_last = i == path.segments.len() - 1;
            let name = &segment.name;
            let span = segment.span;

            // For the last segment, look for a definition
            if is_last {
                let mut check_scope = Some(current_scope);
                while let Some(scope_id) = check_scope {
                    let scope = self.ctx.context.scopes.get_scope(scope_id);

                    // Check if it exists as a definition
                    if let Some(canonical_name) = scope.definitions.get_key(name) {
                        if canonical_name != name.as_str() {
                            self.ctx.diagnostics.errors.push(
                                error_span(
                                    format!(
                                        "inconsistent capitalization: `{name}` should be \
                                         `{canonical_name}`",
                                    ),
                                    Label::new(span).message("used here"),
                                )
                                .note(format!("the canonical name is `{canonical_name}`")),
                            );
                        }
                        return;
                    }

                    check_scope = scope.parent;
                }
            } else {
                // For non-last segments, look for child scopes (modules)
                let scope = self.ctx.context.scopes.get_scope(current_scope);

                if let Some(canonical_name) = scope.children.get_key(name) {
                    if canonical_name != name.as_str() {
                        self.ctx.diagnostics.errors.push(
                            error_span(
                                format!(
                                    "inconsistent capitalization: `{name}` should be \
                                     `{canonical_name}`"
                                ),
                                Label::new(span).message("module name used here"),
                            )
                            .note(format!("the canonical module name is `{canonical_name}`")),
                        );
                    }

                    // Move into the child scope
                    if let Some(&child_scope) = scope.children.get(name) {
                        current_scope = child_scope;
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
    }
}

/// Resolves a declarator to produce an identifier and type.
/// Handles array declarators by building array types from the base type.
pub fn resolve_declarator(
    decl: &ic_syntax::Declarator,
    base_ty: Ty,
    ctx: &mut LoweringContext,
    scope: ScopeId,
) -> (ic_syntax::Ident, Ty) {
    match decl {
        ic_syntax::Declarator::Name(ident) => (ident.clone(), base_ty),
        ic_syntax::Declarator::Array(arr) => {
            let mut ty = base_ty;
            for bound_expr in arr.bounds.iter().rev() {
                let mut evaluator = ConstEvaluator::new(ctx, scope);
                let len = evaluator.eval_nonneg_bound(bound_expr).unwrap_or(1);

                ty = Ty {
                    span: ty.span,
                    kind: TyKind::Array {
                        ty: Box::new(ty.clone()),
                        len,
                        len_span: bound_expr.span,
                    },
                };
            }
            (arr.name.clone(), ty)
        }
    }
}
