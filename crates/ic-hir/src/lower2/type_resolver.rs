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

use ic_diagnostic::Label;
use ic_syntax::{Path, Type as AstType};

use super::eval::ConstEvaluator;
use super::utils::{path_span, path_to_string};
use super::{LoweringContext, ResolveMode};
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
                    bound_span: s.bound.as_ref().map(|e| e.span()),
                },
            }),
            AstType::Sequence(seq) => {
                let elem_ty = self.resolve_type(&seq.ty)?;
                Some(Ty {
                    span: (seq.span),
                    kind: TyKind::Sequence {
                        ty: Box::new(elem_ty),
                        bound: seq.bound.as_ref().and_then(|e| self.evaluate_bound(e)),
                        bound_span: seq.bound.as_ref().map(|e| e.span()),
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
                        bound_span: m.bound.as_ref().map(|e| e.span()),
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
        match self.ctx.scopes.resolve_path(&self.ctx.context, start, path) {
            Some(def_id) => {
                let def = self.ctx.context.definitions.get(def_id);
                if self.is_type_definition(&def.kind) {
                    Some(Ty {
                        span: (path_span(path)),
                        kind: TyKind::Adt(def_id),
                    })
                } else {
                    use ic_diagnostic::error_span;
                    self.ctx.diagnostics.errors.push(error_span(
                        format!("`{}` is not a type", path_to_string(path)),
                        Label::new(path_span(path)).message("expected a type"),
                    ));
                    None
                }
            }
            None => {
                use ic_diagnostic::error_span;
                self.ctx.diagnostics.errors.push(error_span(
                    format!("unresolved type `{}`", path_to_string(path)),
                    Label::new(path_span(path)).message("unknown type"),
                ));
                None
            }
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
            "short" | "int16" => PrimitiveTy::Int16,
            "unsigned short" | "uint16" => PrimitiveTy::UInt16,
            "long" | "int32" => PrimitiveTy::Int32,
            "unsigned long" | "uint32" => PrimitiveTy::UInt32,
            "long long" | "int64" => PrimitiveTy::Int64,
            "unsigned long long" | "uint64" => PrimitiveTy::UInt64,
            "float" => PrimitiveTy::Float32,
            "double" => PrimitiveTy::Float64,
            "long double" => PrimitiveTy::Float128,
            "int8" => PrimitiveTy::Int8,
            _ => return None,
        })
    }

    /// Evaluate a bound expression to a numeric value.
    fn evaluate_bound(&mut self, expr: &ic_syntax::Expr) -> Option<usize> {
        // Use the full expression evaluator
        let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
        evaluator.eval_nonneg_bound(expr)
    }

    /// Check if a DefKind represents a type definition.
    fn is_type_definition(&self, kind: &DefKind) -> bool {
        !matches!(
            kind,
            DefKind::Annotation(_) | DefKind::Module(_) | DefKind::Const(_)
        )
    }

    /// Resolve a type in the context of an interface (for inherited type visibility).
    pub fn resolve_in_interface_context(
        &mut self,
        interface_id: DefId,
        type_name: &str,
    ) -> Option<DefId> {
        self.ctx.scopes.resolve_name(
            &self.ctx.context,
            self.current_scope,
            type_name,
            ResolveMode::InsideInterface(interface_id),
        )
    }
}
