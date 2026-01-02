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

//! Expression evaluation for constant expressions.
//!
//! This crate provides generic expression evaluation functionality using
//! `Value<R>` as the unified value type. It's used for evaluating constant
//! expressions in IDL files and C preprocessor expressions.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

mod rank;
pub use rank::{FloatRank, IntRank, int_bounds};

mod value;
pub use value::Value;

/// Arithmetic operations on typed values.
pub mod ops;
pub use ops::{ArithError, BinOp, UnaryOp, eval_bin, eval_unary};

/// Result type for expression evaluation.
pub type Result<T, R> = std::result::Result<T, ArithError<R>>;

/// Binary and unary operators supported in expressions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Op {
    /// Logical NOT (!)
    Not,

    /// Logical AND (&&)
    And,

    /// Logical OR (||)
    Or,

    /// Greater than (>)
    Gt,

    /// Greater than or equal (>=)
    GtEq,

    /// Less than (<)
    Lt,

    /// Less than or equal (<=)
    LtEq,

    /// Equality (==)
    EqEq,

    /// Inequality (!=)
    NotEq,

    /// Bitwise NOT (~)
    BitNot,

    /// Bitwise AND (&)
    BitAnd,

    /// Bitwise OR (|)
    BitOr,

    /// Bitwise XOR (^)
    BitXor,

    /// Left shift (<<)
    LShift,

    /// Right shift (>>)
    RShift,

    /// Addition (+)
    Add,

    /// Subtraction (-)
    Sub,

    /// Multiplication (*)
    Mul,

    /// Division (/)
    Div,

    /// Modulo (%)
    Mod,
}

/// An expression tree that can be evaluated.
///
/// `T` is the literal/leaf type, `S` is the span type for error reporting.
#[derive(Debug)]
pub enum Expr<T, S = ()> {
    Lit(T, S),
    Var(String, S),
    Unary(Box<Unary<T, S>>),
    Binary(Box<Binary<T, S>>),
    Ternary(Box<Ternary<T, S>>),
}

/// A unary operation on an expression.
#[derive(Debug)]
pub struct Unary<T, S = ()> {
    /// The unary operator.
    pub op: Op,
    /// The expression to apply the operator to.
    pub expr: Expr<T, S>,
}

/// A binary operation on two expressions.
#[derive(Debug)]
pub struct Binary<T, S = ()> {
    pub lhs: Expr<T, S>,
    pub op: Op,
    pub rhs: Expr<T, S>,
}

/// A ternary conditional expression (cond ? then : else).
#[derive(Debug)]
pub struct Ternary<T, S = ()> {
    pub cond: Expr<T, S>,
    pub then: Expr<T, S>,
    pub els: Expr<T, S>,
}

/// Context for expression evaluation using `Value<R>`.
pub trait EvalContext<T, R: Clone, S: Clone> {
    /// Evaluate a literal to a value.
    ///
    /// # Errors
    /// Returns an error if the literal cannot be evaluated.
    fn eval_literal(
        &mut self,
        lit: &T,
        span: S,
    ) -> std::result::Result<Value<R>, SpannedError<R, S>>;

    /// Look up a variable by name.
    ///
    /// # Errors
    /// Returns an error if the variable is not found or cannot be evaluated.
    fn lookup_var(
        &mut self,
        name: &str,
        span: S,
    ) -> std::result::Result<Value<R>, SpannedError<R, S>> {
        Err((
            ArithError::Custom(format!("undefined variable: {name}")),
            span,
        ))
    }
}

/// Spanned error result from evaluation.
pub type SpannedError<R, S> = (ArithError<R>, S);

/// Evaluate an expression with the given context.
///
/// # Errors
///
/// Returns an error with the span where it occurred.
#[allow(clippy::too_many_lines)]
pub fn eval<T, R, S, C>(
    expr: &Expr<T, S>,
    ctx: &mut C,
) -> std::result::Result<Value<R>, SpannedError<R, S>>
where
    R: Clone,
    S: Clone,
    C: EvalContext<T, R, S>,
{
    match expr {
        Expr::Lit(lit, span) => ctx.eval_literal(lit, span.clone()),
        Expr::Var(name, span) => ctx.lookup_var(name, span.clone()),
        Expr::Unary(unary) => {
            let val = eval(&unary.expr, ctx)?;
            let span = unary.expr.span();
            match unary.op {
                Op::Sub => eval_unary(UnaryOp::Neg, val).map_err(|e| (e, span)),
                Op::Add => eval_unary(UnaryOp::Plus, val).map_err(|e| (e, span)),
                Op::BitNot => eval_unary(UnaryOp::BitNot, val).map_err(|e| (e, span)),
                Op::Not => Ok(Value::Int(i128::from(!val.to_bool()), IntRank::I32)),
                _ => Err((ArithError::InvalidUnaryOp, span)),
            }
        }
        Expr::Binary(binary) => {
            let rhs_span = binary.rhs.span();
            match binary.op {
                Op::And => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    if !lhs.to_bool() {
                        return Ok(Value::Int(0, IntRank::I32));
                    }
                    let rhs = eval(&binary.rhs, ctx)?;
                    Ok(Value::Int(i128::from(rhs.to_bool()), IntRank::I32))
                }
                Op::Or => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    if lhs.to_bool() {
                        return Ok(Value::Int(1, IntRank::I32));
                    }
                    let rhs = eval(&binary.rhs, ctx)?;
                    Ok(Value::Int(i128::from(rhs.to_bool()), IntRank::I32))
                }
                Op::Lt => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let result = compare_values(&lhs, &rhs, std::cmp::Ordering::Less);
                    Ok(Value::Int(i128::from(result), IntRank::I32))
                }
                Op::LtEq => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let result = compare_values(&lhs, &rhs, std::cmp::Ordering::Less)
                        || compare_values(&lhs, &rhs, std::cmp::Ordering::Equal);
                    Ok(Value::Int(i128::from(result), IntRank::I32))
                }
                Op::Gt => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let result = compare_values(&lhs, &rhs, std::cmp::Ordering::Greater);
                    Ok(Value::Int(i128::from(result), IntRank::I32))
                }
                Op::GtEq => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let result = compare_values(&lhs, &rhs, std::cmp::Ordering::Greater)
                        || compare_values(&lhs, &rhs, std::cmp::Ordering::Equal);
                    Ok(Value::Int(i128::from(result), IntRank::I32))
                }
                Op::EqEq => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let result = compare_values(&lhs, &rhs, std::cmp::Ordering::Equal);
                    Ok(Value::Int(i128::from(result), IntRank::I32))
                }
                Op::NotEq => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let result = !compare_values(&lhs, &rhs, std::cmp::Ordering::Equal);
                    Ok(Value::Int(i128::from(result), IntRank::I32))
                }
                _ => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;
                    let bin_op = match binary.op {
                        Op::Add => BinOp::Add,
                        Op::Sub => BinOp::Sub,
                        Op::Mul => BinOp::Mul,
                        Op::Div => BinOp::Div,
                        Op::Mod => BinOp::Mod,
                        Op::BitAnd => BinOp::BitAnd,
                        Op::BitOr => BinOp::BitOr,
                        Op::BitXor => BinOp::Xor,
                        Op::LShift => BinOp::Shl,
                        Op::RShift => BinOp::Shr,
                        _ => unreachable!("handled above"),
                    };
                    eval_bin(bin_op, lhs, rhs).map_err(|e| (e, rhs_span))
                }
            }
        }
        Expr::Ternary(ternary) => {
            let cond = eval(&ternary.cond, ctx)?;
            if cond.to_bool() {
                eval(&ternary.then, ctx)
            } else {
                eval(&ternary.els, ctx)
            }
        }
    }
}

impl<T, S: Clone> Expr<T, S> {
    /// Get the span of this expression.
    pub fn span(&self) -> S {
        match self {
            Expr::Lit(_, s) | Expr::Var(_, s) => s.clone(),
            Expr::Unary(u) => u.expr.span(),
            Expr::Binary(b) => b.lhs.span(), // Use LHS span as representative
            Expr::Ternary(t) => t.cond.span(),
        }
    }
}

fn compare_values<R>(lhs: &Value<R>, rhs: &Value<R>, expected: std::cmp::Ordering) -> bool {
    match (lhs, rhs) {
        (Value::Int(a, _), Value::Int(b, _)) => a.cmp(b) == expected,
        (Value::UInt(a, _), Value::UInt(b, _)) => a.cmp(b) == expected,
        (Value::Int(a, _), Value::UInt(b, _)) => {
            if *a < 0 {
                expected == std::cmp::Ordering::Less
            } else {
                (*a as u128).cmp(b) == expected
            }
        }
        (Value::UInt(a, _), Value::Int(b, _)) => {
            if *b < 0 {
                expected == std::cmp::Ordering::Greater
            } else {
                a.cmp(&(*b as u128)) == expected
            }
        }
        (Value::Float(a, _), Value::Float(b, _)) => a.partial_cmp(b).is_some_and(|o| o == expected),
        (Value::Float(a, _), Value::Int(b, _)) => {
            a.partial_cmp(&(*b as f64)).is_some_and(|o| o == expected)
        }
        (Value::Int(a, _), Value::Float(b, _)) => {
            (*a as f64).partial_cmp(b).is_some_and(|o| o == expected)
        }
        (Value::Float(a, _), Value::UInt(b, _)) => {
            a.partial_cmp(&(*b as f64)).is_some_and(|o| o == expected)
        }
        (Value::UInt(a, _), Value::Float(b, _)) => {
            (*a as f64).partial_cmp(b).is_some_and(|o| o == expected)
        }
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b) == expected,
        (Value::Char(a), Value::Char(b)) => a.cmp(b) == expected,
        _ => false,
    }
}
