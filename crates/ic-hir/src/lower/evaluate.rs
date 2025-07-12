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

//! Phase 3: Expression evaluation.
//!
//! This phase:
//! - Evaluates constant expressions
//! - Computes array bounds, sequence bounds, map bounds
//! - Assigns values to enum fields and bitmask flags
//! - Evaluates initializer expressions for constants
//!
//! At this point, all types are resolved, so we can properly evaluate expressions.

use ic_diagnostic::{Diag, Label, error_span};
use ic_syntax::{Item, Expr, ExprKind, Lit, UnaryOp, BinaryOp};

use crate::{Context, hir::*};

/// Evaluates expressions in the HIR.
pub struct ExpressionEvaluator<'a> {
    ctx: &'a mut Context,
    errors: Vec<Diag>,
}

impl<'a> ExpressionEvaluator<'a> {
    fn new(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            errors: Vec::new(),
        }
    }
    
    /// Evaluates an expression to a numeric value.
    fn eval_expr(&mut self, expr: &Expr) -> Numeric {
        match &expr.kind {
            ExprKind::Lit(lit) => self.eval_literal(lit),
            ExprKind::Path(path) => self.eval_path(path, expr.span),
            ExprKind::Unary(op, inner) => self.eval_unary(*op, inner),
            ExprKind::Binary(lhs, op, rhs) => self.eval_binary(lhs, *op, rhs),
            ExprKind::Ternary(cond, then_expr, else_expr) => {
                self.eval_ternary(cond, then_expr, else_expr)
            },
            ExprKind::Paren(inner) => self.eval_expr(inner),
            _ => {
                self.errors.push(error_span(
                    "unsupported expression in constant context",
                    Label::new(expr.span).message("cannot evaluate this expression"),
                ));
                Numeric::Null
            }
        }
    }
    
    /// Evaluates a literal.
    fn eval_literal(&self, lit: &Lit) -> Numeric {
        match lit {
            Lit::Null(_) => Numeric::Null,
            Lit::Bool(b) => Numeric::Bool(b.value),
            Lit::Char(c) => Numeric::Char(c.value),
            Lit::Int(i) => {
                // TODO: Handle different integer types based on suffix
                Numeric::Int32(i.value as i32)
            },
            Lit::Float(f) => {
                // TODO: Handle float vs double based on suffix
                Numeric::Float(f.value as f32)
            },
            Lit::String(s) => Numeric::String(s.value.clone()),
        }
    }
    
    /// Evaluates a path expression (constant reference).
    fn eval_path(&mut self, path: &ic_syntax::Path, span: ic_syntax::Span) -> Numeric {
        // For now, just return null - proper implementation would look up the constant
        // TODO: Implement constant lookup
        self.errors.push(error_span(
            "constant references not yet implemented",
            Label::new(span).message("cannot evaluate constant reference"),
        ));
        Numeric::Null
    }
    
    /// Evaluates a unary expression.
    fn eval_unary(&mut self, op: UnaryOp, expr: &Expr) -> Numeric {
        let value = self.eval_expr(expr);
        
        match (op, value) {
            (UnaryOp::Plus, v) => v,
            (UnaryOp::Minus, Numeric::Int8(v)) => Numeric::Int8(-v),
            (UnaryOp::Minus, Numeric::Int16(v)) => Numeric::Int16(-v),
            (UnaryOp::Minus, Numeric::Int32(v)) => Numeric::Int32(-v),
            (UnaryOp::Minus, Numeric::Int64(v)) => Numeric::Int64(-v),
            (UnaryOp::Minus, Numeric::Float(v)) => Numeric::Float(-v),
            (UnaryOp::Minus, Numeric::Double(v)) => Numeric::Double(-v),
            (UnaryOp::Not, Numeric::Bool(v)) => Numeric::Bool(!v),
            (UnaryOp::Not, Numeric::Int32(v)) => Numeric::Int32(!v),
            (UnaryOp::Not, Numeric::Int64(v)) => Numeric::Int64(!v),
            _ => {
                self.errors.push(error_span(
                    format!("invalid unary operation {:?} on value", op),
                    Label::new(expr.span).message("cannot apply this operator"),
                ));
                Numeric::Null
            }
        }
    }
    
    /// Evaluates a binary expression.
    fn eval_binary(&mut self, lhs: &Expr, op: BinaryOp, rhs: &Expr) -> Numeric {
        let lhs_val = self.eval_expr(lhs);
        let rhs_val = self.eval_expr(rhs);
        
        // For now, just handle integer arithmetic
        match (lhs_val, op, rhs_val) {
            (Numeric::Int32(a), BinaryOp::Add, Numeric::Int32(b)) => Numeric::Int32(a + b),
            (Numeric::Int32(a), BinaryOp::Sub, Numeric::Int32(b)) => Numeric::Int32(a - b),
            (Numeric::Int32(a), BinaryOp::Mul, Numeric::Int32(b)) => Numeric::Int32(a * b),
            (Numeric::Int32(a), BinaryOp::Div, Numeric::Int32(b)) => {
                if b == 0 {
                    self.errors.push(error_span(
                        "division by zero",
                        Label::new(rhs.span).message("divisor is zero"),
                    ));
                    Numeric::Null
                } else {
                    Numeric::Int32(a / b)
                }
            },
            (Numeric::Int32(a), BinaryOp::Mod, Numeric::Int32(b)) => {
                if b == 0 {
                    self.errors.push(error_span(
                        "modulo by zero",
                        Label::new(rhs.span).message("divisor is zero"),
                    ));
                    Numeric::Null
                } else {
                    Numeric::Int32(a % b)
                }
            },
            (Numeric::Int32(a), BinaryOp::And, Numeric::Int32(b)) => Numeric::Int32(a & b),
            (Numeric::Int32(a), BinaryOp::Or, Numeric::Int32(b)) => Numeric::Int32(a | b),
            (Numeric::Int32(a), BinaryOp::Xor, Numeric::Int32(b)) => Numeric::Int32(a ^ b),
            (Numeric::Int32(a), BinaryOp::Lshift, Numeric::Int32(b)) => Numeric::Int32(a << b),
            (Numeric::Int32(a), BinaryOp::Rshift, Numeric::Int32(b)) => Numeric::Int32(a >> b),
            // TODO: Handle more type combinations
            _ => {
                self.errors.push(error_span(
                    format!("invalid binary operation {:?}", op),
                    Label::new(lhs.span).message("cannot apply this operator to these types"),
                ));
                Numeric::Null
            }
        }
    }
    
    /// Evaluates a ternary expression.
    fn eval_ternary(&mut self, cond: &Expr, then_expr: &Expr, else_expr: &Expr) -> Numeric {
        let cond_val = self.eval_expr(cond);
        
        match cond_val {
            Numeric::Bool(true) => self.eval_expr(then_expr),
            Numeric::Bool(false) => self.eval_expr(else_expr),
            Numeric::Int32(v) if v != 0 => self.eval_expr(then_expr),
            Numeric::Int32(0) => self.eval_expr(else_expr),
            _ => {
                self.errors.push(error_span(
                    "invalid condition type in ternary expression",
                    Label::new(cond.span).message("condition must be boolean or integer"),
                ));
                Numeric::Null
            }
        }
    }
    
    /// Evaluates a bound expression to a usize.
    fn eval_bound(&mut self, expr: &Expr) -> usize {
        match self.eval_expr(expr) {
            Numeric::Int32(v) if v >= 0 => v as usize,
            Numeric::UInt32(v) => v as usize,
            Numeric::Int64(v) if v >= 0 => v as usize,
            Numeric::UInt64(v) => v as usize,
            _ => {
                self.errors.push(error_span(
                    "invalid bound expression",
                    Label::new(expr.span).message("bound must be a positive integer"),
                ));
                0
            }
        }
    }
    
    /// Updates array bounds in a type.
    fn update_type_bounds(&mut self, ty: &mut Ty, bounds: &[ic_syntax::Expr]) {
        let mut current = ty;
        let mut bound_iter = bounds.iter().rev();  // Process from outermost to innermost
        
        loop {
            match &mut current.kind {
                TyKind::Array { ty: inner_ty, len } => {
                    if let Some(bound_expr) = bound_iter.next() {
                        *len = self.eval_bound(bound_expr);
                    }
                    current = inner_ty;
                },
                _ => break,
            }
        }
    }
    
    /// Evaluates all expressions in struct/union/etc definitions.
    fn evaluate_struct(&mut self, id: DefId, def: &ic_syntax::StructDef) {
        // Structs don't have expressions to evaluate in their definition
        // (member default values would be handled separately)
    }
    
    /// Evaluates expressions in a union definition.
    fn evaluate_union(&mut self, id: DefId, def: &ic_syntax::UnionDef) {
        let hir_def = self.ctx.definitions.get_mut(id);
        
        if let DefKind::Union(union_ty) = &mut hir_def.kind {
            // Evaluate case labels
            for (idx, field) in def.fields.iter().enumerate() {
                let mut labels = Vec::new();
                
                for label in &field.labels {
                    if let ic_syntax::Label::Case(expr) = label {
                        labels.push(self.eval_expr(expr));
                    }
                }
                
                if let Some(variant) = union_ty.variants.get_mut(idx) {
                    variant.labels = labels;
                }
            }
        }
    }
    
    /// Evaluates enum values.
    fn evaluate_enum(&mut self, id: DefId, def: &ic_syntax::EnumDef) {
        let hir_def = self.ctx.definitions.get_mut(id);
        
        if let DefKind::Enum(enum_ty) = &mut hir_def.kind {
            let mut last_value = -1isize;
            
            for (idx, field) in def.fields.iter().enumerate() {
                let value = if let Some(expr) = &field.value {
                    self.eval_bound(expr) as isize
                } else {
                    last_value + 1
                };
                
                last_value = value;
                
                if let Some(enum_lit) = enum_ty.fields.get_mut(idx) {
                    enum_lit.value = value;
                }
            }
        }
    }
    
    /// Evaluates bitmask values.
    fn evaluate_bitmask(&mut self, id: DefId, def: &ic_syntax::BitmaskDef) {
        let hir_def = self.ctx.definitions.get_mut(id);
        
        if let DefKind::Bitmask(bitmask_ty) = &mut hir_def.kind {
            let mut last_value = 0usize;
            
            for (idx, bit) in def.bits.iter().enumerate() {
                let value = if let Some(expr) = &bit.value {
                    self.eval_bound(expr)
                } else {
                    if last_value == 0 {
                        1
                    } else {
                        last_value << 1
                    }
                };
                
                last_value = value;
                
                if let Some(flag) = bitmask_ty.flags.get_mut(idx) {
                    flag.value = value;
                }
            }
        }
    }
    
    /// Evaluates a constant definition.
    fn evaluate_const(&mut self, id: DefId, def: &ic_syntax::ConstDef) {
        let value = self.eval_expr(&def.value);
        
        let hir_def = self.ctx.definitions.get_mut(id);
        
        if let DefKind::Const(const_ty) = &mut hir_def.kind {
            const_ty.value = value;
            
            // Also update array bounds if this is an array constant
            if let ic_syntax::Declarator::Array(arr) = &def.decl {
                self.update_type_bounds(&mut const_ty.ty, &arr.bounds);
            }
        }
    }
    
    /// Evaluates expressions in type definitions.
    fn evaluate_types(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::UnionValue(v) => {
                    if let Some(&id) = self.ctx.registered.get(&v.ident.name) {
                        self.evaluate_union(id, v);
                    }
                },
                Item::EnumValue(v) => {
                    if let Some(&id) = self.ctx.registered.get(&v.ident.name) {
                        self.evaluate_enum(id, v);
                    }
                },
                Item::BitmaskValue(v) => {
                    if let Some(&id) = self.ctx.registered.get(&v.ident.name) {
                        self.evaluate_bitmask(id, v);
                    }
                },
                Item::ConstValue(v) => {
                    let name = match &v.decl {
                        ic_syntax::Declarator::Simple(n) => n.clone(),
                        ic_syntax::Declarator::Array(a) => a.ident.name.clone(),
                    };
                    
                    if let Some(&id) = self.ctx.registered.get(&name) {
                        self.evaluate_const(id, v);
                    }
                },
                // TODO: Handle sequence/map/string bounds
                _ => {},
            }
        }
    }
}

/// Evaluates all expressions in the HIR.
pub fn evaluate_expressions(
    ctx: &mut Context,
    items: &[Item],
) -> Vec<Diag> {
    let mut evaluator = ExpressionEvaluator::new(ctx);
    evaluator.evaluate_types(items);
    evaluator.errors
}