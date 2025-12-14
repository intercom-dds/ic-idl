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

//! Constant expression evaluation with C-like integer promotions and
//! table-driven operator dispatch. This is intentionally compact and
//! focuses on numeric arithmetic/bitwise semantics needed for IDL
//! constants, enum values, bitmask bits, and bounds.

mod cast;
mod ops;
mod rank;

pub use cast::{check_float_to_int_precision_loss, get_type_name};
use ic_diagnostic::{Label, error_span, warn_span};
pub use rank::{FloatRank, IntRank, int_min_max};

use self::cast::{numeric_from_value, value_from_numeric};
use self::ops::{eval_bin, eval_unary, op_from_ast};
use super::LoweringContext;
use super::utils::{literal_to_numeric, path_span, path_to_string};
use crate::ctx::Context;
use crate::hir::{DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};
use crate::scope::ScopeId;

/// A simplified value domain for evaluation.
#[derive(Clone, Debug)]
enum Value {
    Int(i128, IntRank),
    UInt(u128, IntRank),
    Float(f64, FloatRank),
    Bool(bool),
    Char(char),
    String(String),
    Null,
    Const(DefId),
}

#[derive(Clone, Copy, Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    Xor,
    Shl,
    Shr,
}

#[derive(Debug)]
enum EvalError {
    /// Signed overflow occurred; contains a wrapped result to continue with.
    SignedOverflow(Value),
    /// Value does not fit in the target type range.
    RangeError,
    /// Invalid Unicode scalar value for a character type (e.g., surrogate for wchar).
    InvalidChar,
    /// Invalid floating-point value (NaN or infinity) in conversion.
    InvalidFloat,
    DivByZero,
    ModByZero,
    TypeMismatch,
    ShiftOutOfRange,
}

/// Evaluation error with source location context for diagnostic generation.
#[derive(Debug)]
struct ContextualEvalError {
    kind: EvalError,
    span: ic_syntax::Span,
    /// Additional context for error messages (e.g., "value description")
    context: Option<String>,
}

/// Table-driven constant evaluator with promotions.
pub struct ConstEvaluator<'a> {
    ctx: &'a mut LoweringContext,
    scope: ScopeId,
    /// Optional annotation definition scope to check first for resolution
    annotation_scope: Option<ScopeId>,
}

impl<'a> ConstEvaluator<'a> {
    pub fn new(ctx: &'a mut LoweringContext, scope: ScopeId) -> Self {
        Self {
            ctx,
            scope,
            annotation_scope: None,
        }
    }

    /// Create a new evaluator with an annotation definition scope for resolving values.
    pub fn with_annotation_scope(
        ctx: &'a mut LoweringContext,
        scope: ScopeId,
        annotation_scope: ScopeId,
    ) -> Self {
        Self {
            ctx,
            scope,
            annotation_scope: Some(annotation_scope),
        }
    }

    /// Returns a reference to the context for read-only access.
    pub fn context(&self) -> &LoweringContext {
        self.ctx
    }

    /// Returns a mutable reference to the diagnostics.
    pub fn diagnostics(&mut self) -> &mut super::Diagnostics {
        &mut self.ctx.diagnostics
    }

    /// Resolve a path, trying annotation scope first if present, then falling back to regular scope.
    fn resolve_path_with_fallback<'p>(
        &self,
        path: &'p ic_syntax::Path,
    ) -> Result<DefId, crate::ctx::PathResolutionError<'p>> {
        if let Some(ann_scope) = self.annotation_scope {
            self.ctx
                .context
                .resolve_syntax_path(ann_scope, path)
                .or_else(|_| self.ctx.context.resolve_syntax_path(self.scope, path))
        } else {
            self.ctx.context.resolve_syntax_path(self.scope, path)
        }
    }

    /// Convert an evaluation error to a diagnostic and add it to the context.
    /// Returns whether evaluation should continue (true for warnings, false for errors).
    fn report_eval_error(
        &mut self,
        error: ContextualEvalError,
        expected_ty: Option<&Ty>,
    ) -> Option<Value> {
        match error.kind {
            EvalError::SignedOverflow(wrapped_value) => {
                self.ctx.diagnostics.warnings.push(
                    warn_span(
                        "integer overflow in constant expression",
                        Label::new(error.span).message("overflow detected"),
                    )
                    .note("consider using a larger integer type if overflow was not intended"),
                );
                Some(wrapped_value)
            }
            EvalError::RangeError => {
                self.ctx.diagnostics.errors.push(error_span(
                    "value out of range for target type",
                    Label::new(error.span).message("out of range"),
                ));
                None
            }
            EvalError::InvalidChar => {
                self.ctx.diagnostics.errors.push(error_span(
                    "invalid Unicode scalar for character type",
                    Label::new(error.span).message("invalid character value"),
                ));
                None
            }
            EvalError::InvalidFloat => {
                self.ctx.diagnostics.errors.push(error_span(
                    "cannot convert NaN or infinity to integer type",
                    Label::new(error.span).message("invalid floating-point value"),
                ));
                None
            }
            EvalError::DivByZero => {
                self.ctx.diagnostics.errors.push(error_span(
                    "division by zero in constant expression",
                    Label::new(error.span).message("division by zero"),
                ));
                None
            }
            EvalError::ModByZero => {
                self.ctx.diagnostics.errors.push(error_span(
                    "modulo by zero in constant expression",
                    Label::new(error.span).message("modulo by zero"),
                ));
                None
            }
            EvalError::ShiftOutOfRange => {
                self.ctx.diagnostics.errors.push(error_span(
                    "invalid shift amount: shift count >= width of type or negative",
                    Label::new(error.span).message("invalid shift"),
                ));
                None
            }
            EvalError::TypeMismatch => {
                let message = if let Some(context) = error.context {
                    if let Some(ty) = expected_ty {
                        let type_name = get_type_name(ty, self.ctx);
                        format!("{context} cannot be assigned to type {type_name}")
                    } else {
                        "type mismatch in constant expression".to_string()
                    }
                } else {
                    "type mismatch in constant expression".to_string()
                };
                self.ctx.diagnostics.errors.push(error_span(
                    message,
                    Label::new(error.span).message("incompatible types"),
                ));
                None
            }
        }
    }

    /// Evaluate an expression to a HIR Numeric value (best-effort typing).
    pub fn eval_numeric(&mut self, expr: &ic_syntax::Expr) -> Option<Numeric> {
        let v = self.eval_value(expr)?;
        match numeric_from_value(&v) {
            Ok(n) => Some(n),
            Err(err) => {
                let error = ContextualEvalError {
                    kind: err,
                    span: expr.span(),
                    context: None,
                };
                self.report_eval_error(error, None)?;
                None
            }
        }
    }

    /// Evaluate an expression to a Numeric for use in annotations.
    /// This preserves symbolic references to constants (returns `Numeric::Const`).
    pub fn eval_annotation_arg(&mut self, expr: &ic_syntax::Expr) -> Option<Numeric> {
        self.eval_preserving_const_refs(expr, None)
    }

    /// Evaluate an expression expecting a given target type (for constants declared with type).
    pub fn eval_for_type(&mut self, expr: &ic_syntax::Expr, expected_ty: &Ty) -> Option<Numeric> {
        // Handle initializer lists specially based on expected type
        if let ic_syntax::Expr::InitList(init_list) = expr {
            return self.eval_initializer_list(init_list, expected_ty, expr.span());
        }

        // Perform literal range checks before evaluation
        if let Some(lit) = extract_direct_int_literal(expr)
            && !self.check_literal_range(lit, expected_ty, expr.span())
        {
            return None;
        }

        // If assigning from a path to a constant, check compatibility and optionally reuse it
        if let ic_syntax::Expr::Path(path) = expr {
            match self.try_const_path_assignment(path, expected_ty, expr.span()) {
                ConstAssignOutcome::Accepted(n) => return Some(*n),
                ConstAssignOutcome::Rejected => return None,
                ConstAssignOutcome::NotApplicable => {}
            }
        }

        // Special case: if the expected type is void, don't try to evaluate or cast.
        // Just return a dummy value and let the lint catch the invalid usage.
        if let TyKind::Primitive(PrimitiveTy::Void) = &expected_ty.kind {
            return Some(Numeric::Null);
        }

        let v = self.eval_value(expr)?;

        // Warn about precision loss when assigning float literal to integer type
        check_float_to_int_precision_loss(expr, expected_ty, &mut self.ctx.diagnostics);

        self.cast_and_convert(v, expected_ty, expr.span())
    }

    /// Evaluate an initializer list for the expected type.
    fn eval_initializer_list(
        &mut self,
        init_list: &ic_syntax::InitList,
        expected_ty: &Ty,
        span: ic_syntax::Span,
    ) -> Option<Numeric> {
        use super::initializers::InitializerEvaluator;

        let resolved_ty = match &expected_ty.kind {
            TyKind::Adt(def_id) => self.ctx.context.base_type_of(*def_id),
            _ => expected_ty.clone(),
        };

        match &resolved_ty.kind {
            TyKind::Any => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_sequence(init_list, expected_ty);
            }
            TyKind::Adt(def_id) => {
                let def = self.ctx.context.definitions.get(*def_id);
                match &def.kind {
                    DefKind::Struct(_) => {
                        let mut init_eval = InitializerEvaluator::new(self);
                        return init_eval.eval_struct(init_list, *def_id, expected_ty);
                    }
                    DefKind::Decl(_) => {
                        self.ctx.diagnostics.error(
                            format!(
                                "cannot initialize forward-declared type '{}' in constant \
                                 expression",
                                def.ident.name
                            ),
                            Label::new(span).message("incomplete type"),
                        );
                        return None;
                    }
                    _ => {
                        // Other ADT types (union, valuetype, etc.) don't support initialization
                    }
                }
            }
            TyKind::Array { ty, len, .. } => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_array(init_list, ty, *len);
            }
            TyKind::Sequence { ty, .. } => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_sequence(init_list, ty);
            }
            TyKind::Map { key, elem, .. } => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_map(init_list, key, elem);
            }
            _ => {}
        }

        self.ctx.diagnostics.error(
            "initializer lists can only be used to initialize structs, arrays, sequences, or maps"
                .to_string(),
            Label::new(span).message("invalid use of initializer list"),
        );
        None
    }

    /// Check if an integer literal is within range for the expected type.
    fn check_literal_range(&mut self, lit: i128, expected_ty: &Ty, span: ic_syntax::Span) -> bool {
        let TyKind::Primitive(p) = &expected_ty.kind else {
            return true;
        };
        match p {
            PrimitiveTy::Int8 | PrimitiveTy::Int16 | PrimitiveTy::Int32 | PrimitiveTy::Int64 => {
                let rank = match p {
                    PrimitiveTy::Int8 => IntRank::I8,
                    PrimitiveTy::Int16 => IntRank::I16,
                    PrimitiveTy::Int32 => IntRank::I32,
                    PrimitiveTy::Int64 => IntRank::I64,
                    _ => unreachable!(),
                };
                let (min, max) = int_min_max(rank);
                if lit < min || lit > max {
                    let ty_name = get_type_name(expected_ty, self.ctx);
                    self.ctx.diagnostics.errors.push(error_span(
                        format!("integer literal out of range for '{ty_name}'"),
                        Label::new(span).message("out of range"),
                    ));
                    return false;
                }
            }
            PrimitiveTy::UInt8
            | PrimitiveTy::UInt16
            | PrimitiveTy::UInt32
            | PrimitiveTy::UInt64 => {
                // For unsigned targets, only reject direct positive literals
                // that exceed the target's max. Negative literals are allowed
                // (they wrap), per IDL/C integer conversion rules.
                if lit >= 0 {
                    let max_u: u128 = match p {
                        PrimitiveTy::UInt8 => u8::MAX as u128,
                        PrimitiveTy::UInt16 => u16::MAX as u128,
                        PrimitiveTy::UInt32 => u32::MAX as u128,
                        PrimitiveTy::UInt64 => u64::MAX as u128,
                        _ => 0,
                    };
                    if (lit as u128) > max_u {
                        let ty_name = get_type_name(expected_ty, self.ctx);
                        self.ctx.diagnostics.errors.push(error_span(
                            format!("integer literal out of range for '{ty_name}'"),
                            Label::new(span).message("out of range"),
                        ));
                        return false;
                    }
                }
            }
            _ => {}
        }
        true
    }

    /// Internal method: cast a value to the expected type and convert to Numeric.
    /// Returns Result for cleaner error handling.
    fn cast_and_convert_internal(
        v: Value,
        expected_ty: &Ty,
        ctx: &Context,
    ) -> Result<Numeric, EvalError> {
        let casted = cast::cast_value_to_type(v, expected_ty, ctx)?;
        numeric_from_value(&casted)
    }

    /// Cast a value to the expected type and convert to Numeric.
    /// Reports diagnostics on error.
    fn cast_and_convert(
        &mut self,
        v: Value,
        expected_ty: &Ty,
        span: ic_syntax::Span,
    ) -> Option<Numeric> {
        // Store value description before moving v
        let value_desc = match &v {
            Value::String(_) => "string value",
            Value::Bool(_) => "boolean value",
            Value::Char(_) => "character value",
            Value::Int(_, _) => "integer value",
            Value::UInt(_, _) => "unsigned integer value",
            Value::Float(_, _) => "floating-point value",
            Value::Null => "null value",
            Value::Const(_) => "constant reference",
        };

        let error = ContextualEvalError {
            kind: match Self::cast_and_convert_internal(v, expected_ty, &self.ctx.context) {
                Ok(numeric) => return Some(numeric),
                Err(e) => e,
            },
            span,
            context: Some(value_desc.to_string()),
        };

        self.report_eval_error(error, Some(expected_ty))
            .and_then(|v| numeric_from_value(&v).ok())
    }

    /// Evaluate an expression to a simplified Value.
    fn eval_value(&mut self, expr: &ic_syntax::Expr) -> Option<Value> {
        use ic_syntax::Expr::{Binary, Group, InitList, Literal, Path, Unary};
        match expr {
            Literal(lit) => value_from_numeric(&literal_to_numeric(&lit.value)),
            Path(path) => self.eval_path_value(path),
            Binary(bin) => self.eval_binary_value(bin, expr.span()),
            Unary(un) => self.eval_unary_value(un, expr.span()),
            Group(group) => self.eval_value(&group.expr),
            InitList(_) => {
                self.ctx.diagnostics.error(
                    "initializer lists cannot be used in arithmetic expressions".to_string(),
                    Label::new(expr.span()).message("not allowed in arithmetic context"),
                );
                None
            }
        }
    }

    /// Evaluate a path to a constant value.
    fn eval_path_value(&mut self, path: &ic_syntax::Path) -> Option<Value> {
        let Ok(def_id) = self.resolve_path_with_fallback(path) else {
            self.ctx.diagnostics.errors.push(
                error_span(
                    format!(
                        "undefined constant or enum value `{}`",
                        path_to_string(path)
                    ),
                    Label::new(path_span(path)).message("evaluation error"),
                )
                .note("check that the name is spelled correctly"),
            );
            return None;
        };

        // Constants, enumerators and flags are Const
        let def = self.ctx.context.definitions.get(def_id);
        if let DefKind::Const(c) = &def.kind {
            // Always resolve to the actual value - only eval_for_type preserves references
            if let Some(v) = value_from_numeric(&c.value) {
                self.resolve_const_value(&v)
            } else {
                None
            }
        } else {
            self.ctx.diagnostics.errors.push(error_span(
                format!("`{}` is not a constant value", path_to_string(path)),
                Label::new(path_span(path)).message("expected constant, enumerator, or flag"),
            ));
            None
        }
    }

    /// Evaluate a binary expression.
    fn eval_binary_value(
        &mut self,
        bin: &ic_syntax::Binary,
        expr_span: ic_syntax::Span,
    ) -> Option<Value> {
        let Some(op) = op_from_ast(bin.op.kind) else {
            self.ctx.diagnostics.errors.push(error_span(
                "unsupported binary operation in constant expression",
                Label::new(expr_span).message("unsupported operation"),
            ));
            return None;
        };

        // Evaluate operands
        let l = self.eval_value(&bin.lhs)?;
        let r = self.eval_value(&bin.rhs)?;

        // Check for string operands early for better error messages
        if self.check_string_operands_value(&l, &r, bin) {
            return None;
        }

        // For division/modulo/shift errors, use the RHS span
        let op_span = match op {
            Op::Div | Op::Mod | Op::Shl | Op::Shr => bin.rhs.span(),
            _ => expr_span,
        };

        self.handle_binary_result_value(eval_bin(op, l, r), expr_span, op_span)
    }

    /// Check if either operand is a string and report an error.
    fn check_string_operands_value(
        &mut self,
        l: &Value,
        r: &Value,
        bin: &ic_syntax::Binary,
    ) -> bool {
        let has_string_operand = matches!(l, Value::String(_)) || matches!(r, Value::String(_));
        if has_string_operand {
            let string_span = if matches!(l, Value::String(_)) {
                bin.lhs.span()
            } else {
                bin.rhs.span()
            };

            self.ctx.diagnostics.errors.push(
                error_span(
                    "string literals cannot be used in arithmetic expressions",
                    Label::new(string_span).message("string operand"),
                )
                .note(
                    "string literals can only be used in struct initialization or string constants",
                ),
            );
            true
        } else {
            false
        }
    }

    /// Handle the result of a binary operation evaluation.
    fn handle_binary_result_value(
        &mut self,
        result: Result<Value, EvalError>,
        expr_span: ic_syntax::Span,
        op_span: ic_syntax::Span,
    ) -> Option<Value> {
        match result {
            Ok(v) => Some(v),
            Err(err) => {
                // Use the operator span for division/modulo/shift errors, expression span for others
                let span = match err {
                    EvalError::DivByZero | EvalError::ModByZero | EvalError::ShiftOutOfRange => {
                        op_span
                    }
                    _ => expr_span,
                };
                self.report_eval_error(
                    ContextualEvalError {
                        kind: err,
                        span,
                        context: None,
                    },
                    None,
                )
            }
        }
    }

    /// Evaluate a unary expression.
    fn eval_unary_value(
        &mut self,
        un: &ic_syntax::Unary,
        expr_span: ic_syntax::Span,
    ) -> Option<Value> {
        let v = self.eval_value(&un.expr)?;
        match eval_unary(un.op.kind, v) {
            Ok(v) => Some(v),
            Err(err) => self.report_eval_error(
                ContextualEvalError {
                    kind: err,
                    span: expr_span,
                    context: None,
                },
                None,
            ),
        }
    }

    /// Evaluate an integer bound (non-negative). Returns None on error.
    pub fn eval_nonneg_bound(&mut self, expr: &ic_syntax::Expr) -> Option<usize> {
        let v = self.eval_value(expr)?;
        match v {
            Value::Int(i, _) if i >= 0 => Some(i as usize),
            Value::UInt(u, _) => Some(u as usize),
            _ => {
                self.ctx.diagnostics.error(
                    "bound must be a non-negative integer".to_string(),
                    Label::new(expr.span()).message("expected non-negative integer"),
                );
                None
            }
        }
    }

    /// Evaluate an expression to a Numeric, preserving Const references for union case labels.
    pub fn eval_union_case_label(
        &mut self,
        expr: &ic_syntax::Expr,
        disc_ty: &Ty,
    ) -> Option<Numeric> {
        self.eval_preserving_const_refs(expr, Some(disc_ty))
    }

    /// Evaluate an expression while preserving symbolic references to constants.
    /// If `expected_ty` is provided, type checking is performed and const refs are preserved automatically.
    /// If `expected_ty` is None, const refs are preserved but no type checking is done.
    fn eval_preserving_const_refs(
        &mut self,
        expr: &ic_syntax::Expr,
        expected_ty: Option<&Ty>,
    ) -> Option<Numeric> {
        // If type checking is requested, eval_for_type already preserves const refs via try_const_path_assignment
        if let Some(ty) = expected_ty {
            return self.eval_for_type(expr, ty);
        }

        // No type checking: handle special cases then evaluate
        match expr {
            // Initializer lists without type - return Null for now
            ic_syntax::Expr::InitList(_) => {
                // TODO: Store the initializer list for later evaluation during annotation validation
                Some(Numeric::Null)
            }
            // For paths, preserve const refs
            ic_syntax::Expr::Path(path) => {
                if let Ok(def_id) = self.resolve_path_with_fallback(path) {
                    let def = self.ctx.context.definitions.get(def_id);
                    if let DefKind::Const(_) = &def.kind {
                        return Some(Numeric::Const(def_id));
                    }
                }
                // Not a const, fall through to normal evaluation
                let v = self.eval_value(expr)?;
                numeric_from_value(&v).ok()
            }
            // For non-path expressions, just evaluate normally
            _ => {
                let v = self.eval_value(expr)?;
                numeric_from_value(&v).ok()
            }
        }
    }
}

/// Outcome of attempting to assign a constant path to a target type.
enum ConstAssignOutcome {
    /// Not a constant path (or not applicable) — caller should continue with normal evaluation.
    NotApplicable,
    /// Assignment accepted; use the returned numeric (typically a Const reference).
    Accepted(Box<Numeric>),
    /// Assignment rejected and a diagnostic was emitted; caller should stop.
    Rejected,
}

impl ConstEvaluator<'_> {
    /// Resolve a `Value::Const` to its actual value by following the reference.
    fn resolve_const_value(&self, value: &Value) -> Option<Value> {
        match value {
            Value::Const(def_id) => {
                let def = self.ctx.context.definitions.get(*def_id);
                if let DefKind::Const(c) = &def.kind {
                    // Recursively resolve in case of chained references
                    if let Some(v) = value_from_numeric(&c.value) {
                        self.resolve_const_value(&v)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => Some(value.clone()),
        }
    }

    /// If `path` resolves to a constant, verify it can be assigned to `expected_ty`.
    /// Returns `Accepted(Numeric::Const`(...)) on success, Rejected on hard error,
    /// or `NotApplicable` if the path is not a constant.
    fn try_const_path_assignment(
        &mut self,
        path: &ic_syntax::Path,
        expected_ty: &Ty,
        use_span: ic_syntax::Span,
    ) -> ConstAssignOutcome {
        let Ok(def_id) = self.resolve_path_with_fallback(path) else {
            return ConstAssignOutcome::NotApplicable;
        };

        let def = self.ctx.context.definitions.get(def_id);
        let DefKind::Const(c) = &def.kind else {
            return ConstAssignOutcome::NotApplicable;
        };

        if let Some(val) = value_from_numeric(&c.value) {
            let resolved_val = self.resolve_const_value(&val).unwrap_or(val);
            match cast::cast_value_to_type(resolved_val, expected_ty, &self.ctx.context) {
                Ok(_) => ConstAssignOutcome::Accepted(Box::new(Numeric::Const(def_id))),
                Err(EvalError::RangeError) => {
                    self.ctx.diagnostics.error(
                        "value out of range for target type".to_string(),
                        Label::new(use_span).message("out of range"),
                    );
                    ConstAssignOutcome::Rejected
                }
                Err(_) => {
                    // Provide a precise error mentioning both types and declaration site
                    let from_ty = get_type_name(&c.ty, self.ctx);
                    let to_ty = get_type_name(expected_ty, self.ctx);
                    self.ctx.diagnostics.errors.push(
                        error_span(
                            format!(
                                "constant '{}' of type {} cannot be assigned to {}",
                                def.ident.name, from_ty, to_ty
                            ),
                            Label::new(use_span).message("incompatible types"),
                        )
                        .label(
                            Label::new(def.ident.span).message(format!(
                                "'{}' declared as {} here",
                                def.ident.name, from_ty
                            )),
                        ),
                    );
                    ConstAssignOutcome::Rejected
                }
            }
        } else {
            let resolved_const_ty = match &c.ty.kind {
                TyKind::Adt(def_id) => self.ctx.context.base_type_of(*def_id),
                _ => c.ty.clone(),
            };
            let resolved_expected_ty = match &expected_ty.kind {
                TyKind::Adt(def_id) => self.ctx.context.base_type_of(*def_id),
                _ => expected_ty.clone(),
            };

            if matches!(resolved_expected_ty.kind, TyKind::Any)
                || types_equal_ignore_spans(&resolved_const_ty.kind, &resolved_expected_ty.kind)
            {
                return ConstAssignOutcome::Accepted(Box::new(Numeric::Const(def_id)));
            }

            let to_ty = get_type_name(expected_ty, self.ctx);
            self.ctx.diagnostics.errors.push(
                error_span(
                    format!(
                        "constant '{}' cannot be assigned to {}",
                        def.ident.name, to_ty
                    ),
                    Label::new(use_span).message("incompatible types"),
                )
                .label(Label::new(def.ident.span).message("declared here")),
            );
            ConstAssignOutcome::Rejected
        }
    }
}

fn types_equal_ignore_spans(a: &TyKind, b: &TyKind) -> bool {
    match (a, b) {
        (TyKind::Primitive(p1), TyKind::Primitive(p2)) => p1 == p2,
        (
            TyKind::Array {
                ty: ty1, len: len1, ..
            },
            TyKind::Array {
                ty: ty2, len: len2, ..
            },
        ) => len1 == len2 && types_equal_ignore_spans(&ty1.kind, &ty2.kind),
        (
            TyKind::Sequence {
                ty: ty1, bound: b1, ..
            },
            TyKind::Sequence {
                ty: ty2, bound: b2, ..
            },
        ) => b1 == b2 && types_equal_ignore_spans(&ty1.kind, &ty2.kind),
        (
            TyKind::String {
                wide: w1,
                bound: b1,
                ..
            },
            TyKind::String {
                wide: w2,
                bound: b2,
                ..
            },
        ) => w1 == w2 && b1 == b2,
        (
            TyKind::Map {
                key: k1,
                elem: e1,
                bound: b1,
                ..
            },
            TyKind::Map {
                key: k2,
                elem: e2,
                bound: b2,
                ..
            },
        ) => {
            b1 == b2
                && types_equal_ignore_spans(&k1.kind, &k2.kind)
                && types_equal_ignore_spans(&e1.kind, &e2.kind)
        }
        (TyKind::Adt(id1), TyKind::Adt(id2)) => id1 == id2,
        (TyKind::Null, TyKind::Null)
        | (TyKind::Any, TyKind::Any)
        | (TyKind::Fixed, TyKind::Fixed) => true,
        _ => false,
    }
}

/// Try to extract a direct integer literal from an expression.
/// Handles plain integer literals, parenthesized literals, and a single leading unary '-'.
fn extract_direct_int_literal(expr: &ic_syntax::Expr) -> Option<i128> {
    use ic_syntax::Expr as E;
    match expr {
        E::Literal(lit) => match &lit.value {
            ic_syntax::LiteralValue::Int(i) => Some(*i as i128),
            _ => None,
        },
        E::Group(g) => extract_direct_int_literal(&g.expr),
        E::Unary(u) => {
            if u.op.kind == ic_syntax::OpKind::Sub
                && let Some(v) = extract_direct_int_literal(&u.expr)
            {
                return v.checked_neg();
            }
            None
        }
        _ => None,
    }
}
