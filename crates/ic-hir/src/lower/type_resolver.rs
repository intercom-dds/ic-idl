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

//! Type resolution for converting AST types to HIR types.

use ic_diagnostic::{Label, error_span, warn_span};
use ic_syntax::{Path, Type as AstType};

use super::LoweringContext;
use super::eval::ConstEvaluator;
use super::utils::{path_span, path_to_string};
use crate::hir::{DefId, DefKind, PrimitiveTy, Ty, TyKind};
use crate::scope::ScopeId;

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
            AstType::Path(path) => self.resolve_path_type(path),
            AstType::String(s) => Some(Ty {
                span: (s.span),
                kind: TyKind::String {
                    wide: s.wide,
                    bound: s.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                    bound_span: s.bound.as_ref().map(ic_syntax::Expr::span),
                },
            }),
            AstType::Sequence(seq) => {
                let elem_ty = self.resolve_type(&seq.ty)?;
                Some(Ty {
                    span: (seq.span),
                    kind: TyKind::Sequence {
                        ty: Box::new(elem_ty),
                        bound: seq.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                        bound_span: seq.bound.as_ref().map(ic_syntax::Expr::span),
                    },
                })
            }
            AstType::Map(m) => {
                let key_ty = self.resolve_type(&m.key)?;
                let elem_ty = self.resolve_type(&m.value)?;
                Some(Ty {
                    span: (m.span),
                    kind: TyKind::Map {
                        key: Box::new(key_ty),
                        elem: Box::new(elem_ty),
                        bound: m.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                        bound_span: m.bound.as_ref().map(ic_syntax::Expr::span),
                    },
                })
            }
            AstType::Fixed(f) => Some(Ty {
                span: (f.span),
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

            // Special case for "any" type
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

        // Otherwise, resolve as a named type
        // Determine starting scope for resolution
        let start = if path.leading_colons.is_some() {
            self.ctx.scopes.root()
        } else {
            self.current_scope
        };

        // Resolve the path
        if let Some(def_id) = self.ctx.scopes.resolve_path(&self.ctx.context, start, path) {
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
        } else {
            self.ctx.diagnostics.errors.push(error_span(
                format!("unresolved type `{}`", path_to_string(path)),
                Label::new(path_span(path)).message("unknown type"),
            ));
            None
        }
    }

    /// Resolve a primitive type by name.
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

    /// Evaluate a bound expression to a numeric value.
    fn evaluate_bound(&mut self, expr: &ic_syntax::Expr) -> Option<usize> {
        // Use the full expression evaluator
        let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
        evaluator.eval_nonneg_bound(expr)
    }

    /// Check if a `DefKind` represents a type definition.
    fn is_type_definition(kind: &DefKind) -> bool {
        !matches!(
            kind,
            DefKind::Annotation(_) | DefKind::Module(_) | DefKind::Const(_)
        )
    }

    /// Check if a path reference has consistent capitalization with the definition.
    fn check_case_consistency(&mut self, path: &Path, _def_id: DefId) {
        // We need to check each segment of the path for case consistency
        // This requires resolving the path step by step

        let start_scope = if path.leading_colons.is_some() {
            self.ctx.scopes.root()
        } else {
            self.current_scope
        };

        // Call our custom path resolution that checks case consistency
        self.resolve_path_with_case_check(start_scope, path);
    }

    /// Resolve a path while checking case consistency for each segment
    fn resolve_path_with_case_check(&mut self, start_scope: ScopeId, path: &Path) -> Option<DefId> {
        let segments: Vec<(&str, ic_syntax::Span)> = path
            .segments
            .iter()
            .map(|s| (s.name.as_str(), s.span))
            .collect();

        if segments.is_empty() {
            return None;
        }

        // Try resolving as a relative path first
        let mut current = Some(start_scope);

        while let Some(scope_id) = current {
            if let Some(def_id) = self.resolve_path_from_scope_with_case_check(scope_id, &segments)
            {
                return Some(def_id);
            }

            // Move to parent scope
            current = self.ctx.context.scopes.get_scope(scope_id).parent;
        }

        None
    }

    /// Resolve a path starting from a specific scope, checking case consistency
    fn resolve_path_from_scope_with_case_check(
        &mut self,
        scope: ScopeId,
        segments: &[(&str, ic_syntax::Span)],
    ) -> Option<DefId> {
        if segments.is_empty() {
            return None;
        }

        let (name, span) = segments[0];

        // First check if it's a single segment definition
        if segments.len() == 1 {
            let scope_data = self.ctx.context.scopes.get_scope(scope);
            // Find the definition (case-insensitive)
            let found = scope_data
                .definitions
                .iter()
                .find(|(canonical_name, _)| canonical_name.eq_ignore_ascii_case(name))
                .map(|(canonical_name, &def_id)| (canonical_name, def_id));

            if let Some((canonical_name, def_id)) = found {
                // Check case consistency
                if canonical_name != name {
                    self.ctx.diagnostics.warnings.push(
                        warn_span(
                            format!(
                                "inconsistent capitalization: `{name}` should be \
                                 `{canonical_name}`"
                            ),
                            Label::new(span).message("used here"),
                        )
                        .note(format!("the canonical name is `{canonical_name}`")),
                    );
                }
                return Some(def_id);
            }
        }

        // Multi-segment path or not found as definition - check child scopes
        let scope_data = self.ctx.context.scopes.get_scope(scope);
        let found_child = scope_data
            .children
            .iter()
            .find(|(canonical_name, _)| canonical_name.eq_ignore_ascii_case(name))
            .map(|(canonical_name, &child_scope)| (canonical_name, child_scope));

        if let Some((canonical_name, child_scope)) = found_child {
            // Check case consistency for module name
            if canonical_name != name {
                self.ctx.diagnostics.warnings.push(
                    warn_span(
                        format!(
                            "inconsistent capitalization: `{name}` should be `{canonical_name}`"
                        ),
                        Label::new(span).message("module name used here"),
                    )
                    .note(format!("the canonical module name is `{canonical_name}`")),
                );
            }

            // Recurse into child scope
            return self.resolve_path_from_scope_with_case_check(child_scope, &segments[1..]);
        }

        None
    }
}
