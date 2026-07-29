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

use ic_diagnostic::{Label, error_span, warn_span};
use ic_expr::ops::{self, TyTag};
use ic_expr::{ArithError, EvalContext, FloatRank, IntRank, SpannedError, int_bounds};
use ic_hir::hir::{DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};
use ic_hir::scope::ScopeId;

use super::LoweringContext;
use super::utils::{literal_to_numeric, path_to_string};
use super::value_items::types_equal;

type Value = ic_expr::Value<DefId>;
type Expr = ic_expr::Expr<ExprLeaf, ic_syntax::Span>;
type ConvertError = (&'static str, ic_syntax::Span);
type Error = ArithError<DefId>;

/// Constant evaluator wrapping lowering context and scope.
pub struct ConstEvaluator<'a> {
    pub ctx: &'a mut LoweringContext,
    scope: ScopeId,
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

    pub fn with_annotation_scope(
        ctx: &'a mut LoweringContext,
        scope: ScopeId,
        ann: ScopeId,
    ) -> Self {
        Self {
            ctx,
            scope,
            annotation_scope: Some(ann),
        }
    }

    pub fn diagnostics(&mut self) -> &mut ic_hir::diagnostics::Diagnostics {
        &mut self.ctx.diagnostics
    }

    /// Evaluate expression with target type.
    pub fn eval_for_type(&mut self, expr: &ic_syntax::Expr, ty: &Ty) -> Option<Numeric> {
        if let ic_syntax::ExprKind::InitList(init) = &expr.value {
            return self.eval_init_list(init, ty, expr.span);
        }
        if let ic_syntax::ExprKind::Path(path) = &expr.value {
            match self.try_const_path_assignment(path, ty, expr.span) {
                ConstPathOutcome::Accepted(n) => return Some(*n),
                ConstPathOutcome::Rejected => return None,
                ConstPathOutcome::NotApplicable => {}
            }
        } else if let Some(target_enum) = enum_def_id_for_type(&self.ctx.context, ty)
            && let Some((const_id, src_enum)) = self.find_foreign_enum_ref(expr, target_enum)
        {
            self.emit_enum_mismatch(const_id, src_enum, target_enum, expr.span, ty);
            return None;
        }
        check_precision_loss(expr, ty, &mut self.ctx.diagnostics);
        let v = self.eval_expr(expr)?;
        let is_literal = matches!(expr.value, ic_syntax::ExprKind::Literal(_));
        self.cast_value(v, ty, expr.span, is_literal)
    }

    /// Walks an expression looking for a path that resolves to a constant
    /// belonging to an enum different from `target_enum`. Returns the
    /// offending const's `DefId` along with the enum it belongs to.
    fn find_foreign_enum_ref(
        &self,
        expr: &ic_syntax::Expr,
        target_enum: DefId,
    ) -> Option<(DefId, DefId)> {
        match &expr.value {
            ic_syntax::ExprKind::Path(path) => {
                let id = self.resolve_path(path).ok()?;
                let DefKind::Const(c) = &self.ctx.context.definitions.get(id).kind else {
                    return None;
                };
                let src_enum = enum_def_id_for_type(&self.ctx.context, &c.ty)?;
                (src_enum != target_enum).then_some((id, src_enum))
            }
            ic_syntax::ExprKind::Unary(u) => self.find_foreign_enum_ref(&u.operand, target_enum),
            ic_syntax::ExprKind::Binary(b) => self
                .find_foreign_enum_ref(&b.lhs, target_enum)
                .or_else(|| self.find_foreign_enum_ref(&b.rhs, target_enum)),
            ic_syntax::ExprKind::Group(g) => self.find_foreign_enum_ref(g, target_enum),
            ic_syntax::ExprKind::Literal(_) | ic_syntax::ExprKind::InitList(_) => None,
        }
    }

    fn emit_enum_mismatch(
        &mut self,
        const_id: DefId,
        src_enum: DefId,
        tgt_enum: DefId,
        use_span: ic_syntax::Span,
        target_ty: &Ty,
    ) {
        let const_name = self.ctx.context.type_of(const_id).ident.name.clone();
        let from_name = self.ctx.context.type_of(src_enum).ident.name.clone();
        let to_name = self.ctx.context.type_of(tgt_enum).ident.name.clone();
        self.ctx.diagnostics.errors.push(
            error_span(
                format!(
                    "value '{const_name}' of enum '{from_name}' cannot be assigned to enum \
                     '{to_name}'"
                ),
                Label::new(use_span).message(format!("value of enum '{from_name}'")),
            )
            .label(Label::new(target_ty.span).message(format!("expected enum '{to_name}'"))),
        );
    }

    /// Evaluate expression without target type.
    pub fn eval_numeric(&mut self, expr: &ic_syntax::Expr) -> Option<Numeric> {
        numeric_from_value(&self.eval_expr(expr)?)
            .map_err(|e| self.emit_error(&e, expr.span))
            .ok()
    }

    /// Evaluate non-negative integer bound.
    pub fn eval_nonneg_bound(&mut self, expr: &ic_syntax::Expr) -> Option<usize> {
        match self.eval_expr(expr)? {
            Value::Int(i, _) if i >= 0 => Some(i as usize),
            Value::UInt(u, _) => Some(u as usize),
            _ => {
                self.ctx.diagnostics.error(
                    "bound must be a non-negative integer",
                    Label::new(expr.span).message("expected non-negative integer"),
                );
                None
            }
        }
    }

    /// Evaluate union case label.
    pub fn eval_union_case_label(&mut self, expr: &ic_syntax::Expr, ty: &Ty) -> Option<Numeric> {
        self.eval_for_type(expr, ty)
    }

    /// Evaluate annotation argument (preserves const refs).
    pub fn eval_annotation_arg(&mut self, expr: &ic_syntax::Expr) -> Option<Numeric> {
        if let ic_syntax::ExprKind::Path(path) = &expr.value
            && let Ok(id) = self.resolve_path(path)
            && matches!(self.ctx.context.definitions.get(id).kind, DefKind::Const(_))
        {
            return Some(Numeric::Const(id));
        }
        self.eval_numeric(expr)
    }

    fn eval_expr(&mut self, expr: &ic_syntax::Expr) -> Option<Value> {
        let e = convert_expr(expr)
            .map_err(|e| self.ctx.diagnostics.error(e.0, Label::new(e.1)))
            .ok()?;
        let mut ctx = HirEvalCtx {
            hir: &self.ctx.context,
            scope: self.scope,
            ann_scope: self.annotation_scope,
        };
        ic_expr::eval(&e, &mut ctx)
            .map_err(|(err, span)| self.emit_error(&err, span))
            .ok()
    }

    fn cast_value(
        &mut self,
        v: Value,
        ty: &Ty,
        span: ic_syntax::Span,
        is_literal: bool,
    ) -> Option<Numeric> {
        let value_desc = v.kind_name();
        cast_value_to_type(v, ty, &self.ctx.context, is_literal)
            .and_then(|v| numeric_from_value(&v))
            .map_err(|e| self.emit_type_error(&e, span, ty, value_desc))
            .ok()
    }

    fn resolve_path<'p>(
        &self,
        path: &'p ic_syntax::Path,
    ) -> Result<DefId, crate::resolve::PathResolutionError<'p>> {
        crate::resolve::resolve_with_fallback(
            &self.ctx.context,
            self.scope,
            self.annotation_scope,
            path,
        )
    }

    fn try_const_path_assignment(
        &mut self,
        path: &ic_syntax::Path,
        ty: &Ty,
        use_span: ic_syntax::Span,
    ) -> ConstPathOutcome {
        let Ok(id) = self.resolve_path(path) else {
            return ConstPathOutcome::NotApplicable;
        };
        let def = self.ctx.context.definitions.get(id);
        let DefKind::Const(c) = &def.kind else {
            return ConstPathOutcome::NotApplicable;
        };

        if let Some((src_enum, tgt_enum)) = mismatched_enums(&c.ty, ty, &self.ctx.context) {
            self.emit_enum_mismatch(id, src_enum, tgt_enum, use_span, ty);
            return ConstPathOutcome::Rejected;
        }

        if let Some(val) = value_from_numeric(&c.value) {
            let resolved = resolve_value(&self.ctx.context, &val).unwrap_or(val);
            match cast_value_to_type(resolved, ty, &self.ctx.context, false) {
                Ok(cast) => {
                    if enum_def_id_for_type(&self.ctx.context, &c.ty).is_some()
                        && is_numeric_primitive(&self.ctx.context, ty)
                    {
                        match numeric_from_value(&cast) {
                            Ok(numeric) => ConstPathOutcome::Accepted(Box::new(numeric)),
                            Err(err) => {
                                self.emit_type_error(&err, use_span, ty, cast.kind_name());
                                ConstPathOutcome::Rejected
                            }
                        }
                    } else {
                        ConstPathOutcome::Accepted(Box::new(Numeric::Const(id)))
                    }
                }
                Err(Error::RangeError) => {
                    let to_ty = ty_name(ty, self.ctx);
                    self.ctx.diagnostics.errors.push(
                        error_span(
                            format!("constant '{}' out of range for '{}'", def.ident.name, to_ty),
                            Label::new(use_span).message("out of range"),
                        )
                        .label(
                            Label::new(def.ident.span)
                                .message(format!("'{}' defined here", def.ident.name)),
                        ),
                    );
                    ConstPathOutcome::Rejected
                }
                Err(_) => {
                    let from_ty = ty_name(&c.ty, self.ctx);
                    let to_ty = ty_name(ty, self.ctx);
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
                    ConstPathOutcome::Rejected
                }
            }
        } else {
            let ct = self.ctx.context.resolve_ty(&c.ty);
            let tt = self.ctx.context.resolve_ty(ty);
            if matches!(tt.kind, TyKind::Any) || types_equal(&ct, &tt, &self.ctx.context) {
                ConstPathOutcome::Accepted(Box::new(Numeric::Const(id)))
            } else {
                let from_ty = ty_name(&c.ty, self.ctx);
                let to_ty = ty_name(ty, self.ctx);
                self.ctx.diagnostics.errors.push(
                    error_span(
                        format!(
                            "constant '{}' of type {} cannot be assigned to {}",
                            def.ident.name, from_ty, to_ty
                        ),
                        Label::new(use_span).message("incompatible types"),
                    )
                    .label(
                        Label::new(def.ident.span)
                            .message(format!("'{}' declared as {} here", def.ident.name, from_ty)),
                    ),
                );
                ConstPathOutcome::Rejected
            }
        }
    }

    fn eval_init_list(
        &mut self,
        init: &[ic_syntax::NamedExpr],
        ty: &Ty,
        span: ic_syntax::Span,
    ) -> Option<Numeric> {
        use super::initializers::InitializerEvaluator;
        let rt = self.ctx.context.resolve_ty(ty);
        match &rt.kind {
            TyKind::Any => InitializerEvaluator::new(self).eval_sequence(init, ty, span),
            TyKind::Sequence { ty: et, .. } => {
                InitializerEvaluator::new(self).eval_sequence(init, et, span)
            }
            TyKind::Array { ty: et, len, .. } => {
                InitializerEvaluator::new(self).eval_array(init, et, *len, span)
            }
            TyKind::Map { key, elem, .. } => {
                InitializerEvaluator::new(self).eval_map(init, key, elem, span)
            }
            TyKind::Adt(id) => {
                let def = self.ctx.context.definitions.get(*id);
                match &def.kind {
                    DefKind::Struct(_) => {
                        InitializerEvaluator::new(self).eval_struct(init, *id, ty, span)
                    }
                    DefKind::Decl(_) => {
                        self.ctx.diagnostics.error(
                            format!(
                                "cannot initialize forward-declared type '{}'",
                                def.ident.name
                            ),
                            Label::new(span).message("incomplete type"),
                        );
                        None
                    }
                    _ => self.init_err(span),
                }
            }
            _ => self.init_err(span),
        }
    }

    fn init_err(&mut self, span: ic_syntax::Span) -> Option<Numeric> {
        self.ctx.diagnostics.error(
            "initializer lists can only initialize structs, arrays, sequences, or maps",
            Label::new(span).message("invalid initializer list"),
        );
        None
    }

    fn emit_error(&mut self, err: &Error, span: ic_syntax::Span) {
        match err {
            Error::SignedOverflow(_) => self.ctx.diagnostics.warnings.push(
                warn_span(
                    "integer overflow in constant expression",
                    Label::new(span).message("overflow"),
                )
                .note("consider using a larger integer type"),
            ),
            Error::Custom(msg) => self.ctx.diagnostics.errors.push(
                error_span(msg, Label::new(span).message("evaluation error"))
                    .note("check that the name is spelled correctly"),
            ),
            Error::RangeError => self.ctx.diagnostics.error(
                "value out of range for target type",
                Label::new(span).message("out of range"),
            ),
            Error::InvalidChar => self.ctx.diagnostics.error(
                "invalid Unicode scalar for character type",
                Label::new(span).message("invalid character"),
            ),
            Error::InvalidFloat => self.ctx.diagnostics.error(
                "cannot convert NaN or infinity to integer",
                Label::new(span).message("invalid float"),
            ),
            Error::DivByZero => self.ctx.diagnostics.error(
                "division by zero in constant expression",
                Label::new(span).message("division by zero"),
            ),
            Error::ModByZero => self.ctx.diagnostics.error(
                "modulo by zero in constant expression",
                Label::new(span).message("modulo by zero"),
            ),
            Error::ShiftOutOfRange(_) => self.ctx.diagnostics.error(
                "shift count >= width of type or negative",
                Label::new(span).message("invalid shift"),
            ),
            Error::TypeMismatch | Error::UnresolvedRef(_) | Error::InvalidUnaryOp => {
                self.ctx.diagnostics.error(
                    "type mismatch in constant expression",
                    Label::new(span).message("incompatible types"),
                );
            }
        }
    }

    fn emit_type_error(&mut self, err: &Error, span: ic_syntax::Span, ty: &Ty, value_desc: &str) {
        match err {
            Error::TypeMismatch => {
                let type_name = ty_name(ty, self.ctx);
                self.ctx.diagnostics.errors.push(error_span(
                    format!("{value_desc} cannot be assigned to type {type_name}"),
                    Label::new(span).message("incompatible types"),
                ));
            }
            Error::RangeError => {
                let type_name = ty_name(ty, self.ctx);
                self.ctx.diagnostics.errors.push(error_span(
                    format!("{value_desc} out of range for '{type_name}'"),
                    Label::new(span).message("out of range"),
                ));
            }
            _ => self.emit_error(err, span),
        }
    }
}

enum ConstPathOutcome {
    NotApplicable,
    Accepted(Box<Numeric>),
    Rejected,
}

struct HirEvalCtx<'a> {
    hir: &'a ic_hir::Context,
    scope: ScopeId,
    ann_scope: Option<ScopeId>,
}

impl EvalContext<ExprLeaf, DefId, ic_syntax::Span> for HirEvalCtx<'_> {
    fn eval_literal(
        &mut self,
        leaf: &ExprLeaf,
        span: ic_syntax::Span,
    ) -> Result<Value, SpannedError<DefId, ic_syntax::Span>> {
        match leaf {
            ExprLeaf::Literal(lit) => value_from_numeric(&literal_to_numeric(lit))
                .ok_or_else(|| (ArithError::Custom("unsupported literal".into()), span)),
            ExprLeaf::Path(path) => {
                let id = self.resolve(path).map_err(|_| {
                    (
                        ArithError::Custom(format!(
                            "undefined constant or enum value `{}`",
                            path_to_string(path)
                        )),
                        span,
                    )
                })?;
                let def = self.hir.definitions.get(id);
                if let DefKind::Const(c) = &def.kind {
                    if let Some(v) = value_from_numeric(&c.value) {
                        resolve_value(self.hir, &v).ok_or_else(|| {
                            (
                                ArithError::Custom("could not resolve constant".into()),
                                span,
                            )
                        })
                    } else {
                        Ok(Value::Ref(id))
                    }
                } else {
                    Err((
                        ArithError::Custom(format!("`{}` is not a constant", path_to_string(path))),
                        span,
                    ))
                }
            }
        }
    }
}

impl HirEvalCtx<'_> {
    fn resolve<'p>(
        &self,
        path: &'p ic_syntax::Path,
    ) -> Result<DefId, crate::resolve::PathResolutionError<'p>> {
        crate::resolve::resolve_with_fallback(self.hir, self.scope, self.ann_scope, path)
    }
}

fn resolve_value(ctx: &ic_hir::Context, v: &Value) -> Option<Value> {
    match v {
        Value::Ref(id) => {
            if let DefKind::Const(c) = &ctx.definitions.get(*id).kind {
                value_from_numeric(&c.value).and_then(|v| resolve_value(ctx, &v))
            } else {
                None
            }
        }
        _ => Some(v.clone()),
    }
}

#[derive(Debug, Clone)]
enum ExprLeaf {
    Literal(ic_syntax::Literal),
    Path(ic_syntax::Path),
}

fn convert_expr(expr: &ic_syntax::Expr) -> Result<Expr, ConvertError> {
    match &expr.value {
        ic_syntax::ExprKind::Literal(lit) => {
            Ok(Expr::Lit(ExprLeaf::Literal(lit.clone()), expr.span))
        }
        ic_syntax::ExprKind::Path(path) => Ok(Expr::Lit(
            ExprLeaf::Path(path.clone()),
            ic_syntax::util::path_span(path),
        )),
        ic_syntax::ExprKind::Unary(u) => {
            let op =
                convert_unary_op(u.op.value).ok_or(("unsupported unary operator", u.op.span))?;
            Ok(Expr::Unary(Box::new(ic_expr::Unary {
                op,
                expr: convert_expr(&u.operand)?,
            })))
        }
        ic_syntax::ExprKind::Binary(b) => {
            let op =
                convert_binary_op(b.op.value).ok_or(("unsupported binary operator", b.op.span))?;
            Ok(Expr::Binary(Box::new(ic_expr::Binary {
                lhs: convert_expr(&b.lhs)?,
                op,
                rhs: convert_expr(&b.rhs)?,
            })))
        }
        ic_syntax::ExprKind::Group(g) => convert_expr(g),
        ic_syntax::ExprKind::InitList(_) => Err((
            "initializer lists cannot be used in arithmetic expressions",
            expr.span,
        )),
    }
}

fn convert_unary_op(op: ic_syntax::Op) -> Option<ic_expr::Op> {
    match op {
        ic_syntax::Op::Add => Some(ic_expr::Op::Add),
        ic_syntax::Op::Sub => Some(ic_expr::Op::Sub),
        ic_syntax::Op::Not => Some(ic_expr::Op::BitNot),
        _ => None,
    }
}

fn convert_binary_op(op: ic_syntax::Op) -> Option<ic_expr::Op> {
    Some(match op {
        ic_syntax::Op::Add => ic_expr::Op::Add,
        ic_syntax::Op::Sub => ic_expr::Op::Sub,
        ic_syntax::Op::Multiply => ic_expr::Op::Mul,
        ic_syntax::Op::Divide => ic_expr::Op::Div,
        ic_syntax::Op::Modulo => ic_expr::Op::Mod,
        ic_syntax::Op::And => ic_expr::Op::BitAnd,
        ic_syntax::Op::Or => ic_expr::Op::BitOr,
        ic_syntax::Op::Xor => ic_expr::Op::BitXor,
        ic_syntax::Op::LShift => ic_expr::Op::LShift,
        ic_syntax::Op::RShift => ic_expr::Op::RShift,
        ic_syntax::Op::Not => return None,
    })
}

fn cast_value_to_type(
    v: Value,
    ty: &Ty,
    ctx: &ic_hir::Context,
    strict_unsigned: bool,
) -> Result<Value, Error> {
    let resolved_ty = ctx.resolve_ty(ty);
    match &resolved_ty.kind {
        TyKind::Primitive(p) => match *p {
            PrimitiveTy::Char => match v {
                Value::Char(_) => Ok(v),
                Value::WChar(_) => Err(Error::TypeMismatch),
                _ => {
                    let vv = ops::cast_to(v, TyTag::Int(IntRank::U8, false))?;
                    match vv {
                        Value::UInt(u, IntRank::U8) => Ok(Value::Char((u as u8) as char)),
                        Value::Int(i, IntRank::I8) => Ok(Value::Char((i as u8) as char)),
                        _ => Err(Error::TypeMismatch),
                    }
                }
            },

            PrimitiveTy::WChar => match v {
                Value::WChar(_) => Ok(v),
                Value::Char(_) => Err(Error::TypeMismatch),
                _ => {
                    let vv = ops::cast_to(v, TyTag::Int(IntRank::U16, false))?;
                    let code = match vv {
                        Value::UInt(u, IntRank::U16) => u as u32,
                        Value::Int(i, IntRank::I16) => u32::from(i as u16),
                        _ => return Err(Error::TypeMismatch),
                    };

                    if (0xD800..=0xDFFF).contains(&code) {
                        return Err(Error::InvalidChar);
                    }
                    Ok(Value::WChar(char::from_u32(code).unwrap()))
                }
            },

            PrimitiveTy::Bool => match v {
                Value::Bool(_) => Ok(v),
                _ => Err(Error::TypeMismatch),
            },

            _ => {
                if let Some((signed, rank)) = rank_for_primitive(*p) {
                    ops::cast_to_int(v, rank, signed, strict_unsigned)
                } else if let Some(fr) = float_rank_for_primitive(*p) {
                    Ok(ops::cast_to(v, TyTag::Float(fr))?)
                } else {
                    Err(Error::TypeMismatch)
                }
            }
        },

        TyKind::String { wide: false, .. } => match v {
            Value::String(_) => Ok(v),
            _ => Err(Error::TypeMismatch),
        },

        TyKind::String { wide: true, .. } => match v {
            Value::WString(_) => Ok(v),
            _ => Err(Error::TypeMismatch),
        },

        TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. } => {
            Err(Error::TypeMismatch)
        }

        TyKind::Adt(def_id) => {
            let base_ty = ctx.base_type_of(*def_id);
            if matches!(
                base_ty.kind,
                TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. }
            ) {
                Err(Error::TypeMismatch)
            } else {
                Ok(v)
            }
        }
        _ => Ok(v),
    }
}

fn value_from_numeric(num: &Numeric) -> Option<Value> {
    Some(match num {
        Numeric::Null => Value::Null,
        Numeric::Bool(b) => Value::Bool(*b),
        Numeric::Char(c) => Value::Char(*c),
        Numeric::WChar(c) => Value::WChar(*c),
        Numeric::Int8(v) => Value::Int(i128::from(*v), IntRank::I8),
        Numeric::UInt8(v) => Value::UInt(u128::from(*v), IntRank::U8),
        Numeric::Int16(v) => Value::Int(i128::from(*v), IntRank::I16),
        Numeric::UInt16(v) => Value::UInt(u128::from(*v), IntRank::U16),
        Numeric::Int32(v) => Value::Int(i128::from(*v), IntRank::I32),
        Numeric::UInt32(v) => Value::UInt(u128::from(*v), IntRank::U32),
        Numeric::Int64(v) => Value::Int(i128::from(*v), IntRank::I64),
        Numeric::UInt64(v) => Value::UInt(u128::from(*v), IntRank::U64),
        Numeric::Float(v) => Value::Float(f64::from(*v), FloatRank::F32),
        Numeric::Double(v) => Value::Float(*v, FloatRank::F64),
        Numeric::String(s) => Value::String(s.clone()),
        Numeric::WString(s) => Value::WString(s.clone()),
        Numeric::Const(def_id) => Value::Ref(*def_id),
        Numeric::Array { .. }
        | Numeric::Sequence { .. }
        | Numeric::Map { .. }
        | Numeric::Struct { .. }
        | Numeric::Union { .. } => return None,
    })
}

fn numeric_from_value(v: &Value) -> Result<Numeric, Error> {
    Ok(match v {
        Value::Null => Numeric::Null,
        Value::Bool(b) => Numeric::Bool(*b),
        Value::Char(c) => Numeric::Char(*c),
        Value::WChar(c) => Numeric::WChar(*c),
        Value::Int(i, r) => {
            let (min, max) = int_bounds(*r);
            if *i < min || *i > max {
                return Err(Error::RangeError);
            }

            match r {
                IntRank::I8 => Numeric::Int8(*i as i8),
                IntRank::I16 => Numeric::Int16(*i as i16),
                IntRank::I32 => Numeric::Int32(*i as i32),
                IntRank::I64 => Numeric::Int64(*i as i64),
                IntRank::U8 => Numeric::UInt8(*i as u8),
                IntRank::U16 => Numeric::UInt16(*i as u16),
                IntRank::U32 => Numeric::UInt32(*i as u32),
                IntRank::U64 => Numeric::UInt64(*i as u64),
            }
        }
        Value::UInt(u, r) => {
            let (_, max) = int_bounds(*r);
            if *u > max as u128 {
                return Err(Error::RangeError);
            }

            match r {
                IntRank::I8 => Numeric::Int8(*u as i8),
                IntRank::I16 => Numeric::Int16(*u as i16),
                IntRank::I32 => Numeric::Int32(*u as i32),
                IntRank::I64 => Numeric::Int64(*u as i64),
                IntRank::U8 => Numeric::UInt8(*u as u8),
                IntRank::U16 => Numeric::UInt16(*u as u16),
                IntRank::U32 => Numeric::UInt32(*u as u32),
                IntRank::U64 => Numeric::UInt64(*u as u64),
            }
        }
        Value::Float(f, fr) => match fr {
            FloatRank::F32 => Numeric::Float(*f as f32),
            _ => Numeric::Double(*f),
        },
        Value::String(s) => Numeric::String(s.clone()),
        Value::WString(s) => Numeric::WString(s.clone()),
        Value::Ref(def_id) => Numeric::Const(*def_id),
    })
}

fn rank_for_primitive(prim: PrimitiveTy) -> Option<(bool, IntRank)> {
    match prim {
        PrimitiveTy::Int8 => Some((true, IntRank::I8)),
        PrimitiveTy::UInt8 => Some((false, IntRank::U8)),
        PrimitiveTy::Int16 => Some((true, IntRank::I16)),
        PrimitiveTy::UInt16 => Some((false, IntRank::U16)),
        PrimitiveTy::Int32 => Some((true, IntRank::I32)),
        PrimitiveTy::UInt32 => Some((false, IntRank::U32)),
        PrimitiveTy::Int64 => Some((true, IntRank::I64)),
        PrimitiveTy::UInt64 => Some((false, IntRank::U64)),
        _ => None,
    }
}

fn float_rank_for_primitive(prim: PrimitiveTy) -> Option<FloatRank> {
    match prim {
        PrimitiveTy::Float32 => Some(FloatRank::F32),
        PrimitiveTy::Float64 => Some(FloatRank::F64),
        PrimitiveTy::Float128 => Some(FloatRank::F128),
        _ => None,
    }
}

fn check_precision_loss(
    expr: &ic_syntax::Expr,
    ty: &Ty,
    diag: &mut ic_hir::diagnostics::Diagnostics,
) {
    if let ic_syntax::ExprKind::Literal(ic_syntax::Literal::Float(f)) = &expr.value
        && let TyKind::Primitive(p) = &ty.kind
        && matches!(
            p,
            PrimitiveTy::Int8
                | PrimitiveTy::UInt8
                | PrimitiveTy::Int16
                | PrimitiveTy::UInt16
                | PrimitiveTy::Int32
                | PrimitiveTy::UInt32
                | PrimitiveTy::Int64
                | PrimitiveTy::UInt64
        )
    {
        let t = f.trunc();
        if (f - t).abs() > f64::EPSILON {
            diag.warnings.push(warn_span(
                format!(
                    "implicit conversion from 'double' to '{}' changes value from {} to {}",
                    p.name(),
                    f,
                    t as i64
                ),
                Label::new(expr.span).message("precision loss here"),
            ));
        }
    }
}

/// Returns `Some((src_enum, tgt_enum))` if `src` and `tgt` resolve to two
/// different enum definitions. Used to reject assignments that mix enum types.
fn mismatched_enums(src: &Ty, tgt: &Ty, ctx: &ic_hir::Context) -> Option<(DefId, DefId)> {
    let src_enum = enum_def_id_for_type(ctx, src)?;
    let tgt_enum = enum_def_id_for_type(ctx, tgt)?;
    (src_enum != tgt_enum).then_some((src_enum, tgt_enum))
}

/// Returns the enum `DefId` that `ty` ultimately refers to, or `None` if
/// `ty` is not an enum (after alias resolution).
fn enum_def_id_for_type(ctx: &ic_hir::Context, ty: &Ty) -> Option<DefId> {
    let TyKind::Adt(id) = ctx.resolve_ty(ty).kind else {
        return None;
    };
    matches!(ctx.type_of(id).kind, DefKind::Enum(_)).then_some(id)
}

/// Returns true if `ty` resolves to a primitive integer or floating-point
/// type. Used to decide when to inline an enum value as a numeric literal
/// rather than preserving the enum reference.
fn is_numeric_primitive(ctx: &ic_hir::Context, ty: &Ty) -> bool {
    if let TyKind::Primitive(p) = ctx.resolve_ty(ty).kind {
        rank_for_primitive(p).is_some() || float_rank_for_primitive(p).is_some()
    } else {
        false
    }
}

fn ty_name<'a>(ty: &'a Ty, ctx: &'a LoweringContext) -> &'a str {
    match &ty.kind {
        TyKind::Primitive(p) => p.name(),
        TyKind::Adt(def_id) => &ctx.context.type_of(*def_id).ident.name,
        TyKind::String { wide, .. } => {
            if *wide {
                "wstring"
            } else {
                "string"
            }
        }
        TyKind::Array { .. } => "array",
        TyKind::Sequence { .. } => "sequence",
        TyKind::Map { .. } => "map",
        TyKind::Fixed => "fixed",
        TyKind::Any => "any",
        TyKind::Null => "null",
    }
}
