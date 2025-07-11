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

use std::fmt;

pub mod error;
pub use error::{Error, Result};

pub mod c_adapter;
pub mod idl_adapter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Op {
    Not,
    And,
    Or,
    Gt,
    GtEq,
    Lt,
    LtEq,
    EqEq,
    NotEq,
    BitNot,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum Expr<T> {
    Lit(T),
    Unary(Box<Unary<T>>),
    Binary(Box<Binary<T>>),
    Ternary(Box<Ternary<T>>),
}

#[derive(Debug)]
pub struct Unary<T> {
    pub op: Op,
    pub expr: Expr<T>,
}

#[derive(Debug)]
pub struct Binary<T> {
    pub lhs: Expr<T>,
    pub op: Op,
    pub rhs: Expr<T>,
}

#[derive(Debug)]
pub struct Ternary<T> {
    pub cond: Expr<T>,
    pub then: Expr<T>,
    pub els: Expr<T>,
}

/// Configuration for expression evaluation behavior
#[derive(Debug, Clone, Copy)]
pub struct EvalConfig {
    /// How to handle arithmetic overflow
    pub overflow: OverflowBehavior,
    /// Maximum bit shift amount allowed
    pub max_shift: u32,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            overflow: OverflowBehavior::Wrap,
            max_shift: 127,
        }
    }
}

/// How to handle arithmetic overflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowBehavior {
    /// Wrap around on overflow (like C)
    Wrap,
    /// Return an error on overflow
    Error,
    /// Saturate at min/max values
    Saturate,
}

/// Trait for values that can be evaluated in expressions
pub trait NumericValue: Sized + fmt::Debug + Clone {
    /// Create from a boolean value
    fn from_bool(b: bool) -> Self;

    /// Convert to boolean (zero is false, non-zero is true)
    fn to_bool(&self) -> bool;

    /// Unary negation
    ///
    /// # Errors
    /// Returns an error if overflow occurs and overflow behavior is set to error
    fn negate(&self, config: EvalConfig) -> Result<Self>;

    /// Bitwise NOT
    #[must_use]
    fn bit_not(&self) -> Self;

    /// Addition
    ///
    /// # Errors
    /// Returns an error if overflow occurs and overflow behavior is set to error
    fn add(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Subtraction
    ///
    /// # Errors
    /// Returns an error if overflow occurs and overflow behavior is set to error
    fn sub(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Multiplication
    ///
    /// # Errors
    /// Returns an error if overflow occurs and overflow behavior is set to error
    fn mul(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Division
    ///
    /// # Errors
    /// Returns an error if division by zero occurs
    fn div(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Modulo
    ///
    /// # Errors
    /// Returns an error if modulo by zero occurs
    fn modulo(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Bitwise AND
    #[must_use]
    fn bit_and(&self, rhs: &Self) -> Self;

    /// Bitwise OR
    #[must_use]
    fn bit_or(&self, rhs: &Self) -> Self;

    /// Bitwise XOR
    #[must_use]
    fn bit_xor(&self, rhs: &Self) -> Self;

    /// Left shift
    ///
    /// # Errors
    /// Returns an error if the shift amount exceeds the configured maximum
    fn shl(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Right shift
    ///
    /// # Errors
    /// Returns an error if the shift amount exceeds the configured maximum
    fn shr(&self, rhs: &Self, config: EvalConfig) -> Result<Self>;

    /// Less than
    fn lt(&self, rhs: &Self) -> bool;

    /// Less than or equal
    fn le(&self, rhs: &Self) -> bool;

    /// Greater than
    fn gt(&self, rhs: &Self) -> bool;

    /// Greater than or equal
    fn ge(&self, rhs: &Self) -> bool;

    /// Equal
    fn eq(&self, rhs: &Self) -> bool;

    /// Not equal
    fn ne(&self, rhs: &Self) -> bool;
}

/// Context for expression evaluation
pub trait EvalContext<T> {
    /// The numeric value type
    type Value: NumericValue;

    /// Evaluate a literal to a value
    ///
    /// # Errors
    /// Returns an error if the literal cannot be evaluated
    fn eval_literal(&mut self, lit: &T) -> Result<Self::Value>;

    /// Get the evaluation configuration
    fn config(&self) -> EvalConfig;
}

/// Evaluate an expression with the given context
///
/// # Errors
/// Returns an error if the expression cannot be evaluated
pub fn eval<T, C>(expr: &Expr<T>, ctx: &mut C) -> Result<C::Value>
where
    C: EvalContext<T>,
{
    match expr {
        Expr::Lit(lit) => ctx.eval_literal(lit),

        Expr::Unary(unary) => {
            let val = eval(&unary.expr, ctx)?;
            match unary.op {
                Op::Sub => val.negate(ctx.config()),
                Op::Add => Ok(val),
                Op::Not => Ok(C::Value::from_bool(!val.to_bool())),
                Op::BitNot => Ok(val.bit_not()),
                _ => Err(Error::InvalidUnaryOp(unary.op)),
            }
        }

        Expr::Binary(binary) => {
            match binary.op {
                // Short-circuit evaluation for logical operators
                Op::And => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    if !lhs.to_bool() {
                        return Ok(C::Value::from_bool(false));
                    }
                    let rhs = eval(&binary.rhs, ctx)?;
                    Ok(C::Value::from_bool(rhs.to_bool()))
                }
                Op::Or => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    if lhs.to_bool() {
                        return Ok(C::Value::from_bool(true));
                    }
                    let rhs = eval(&binary.rhs, ctx)?;
                    Ok(C::Value::from_bool(rhs.to_bool()))
                }

                // Non-short-circuit operators
                _ => {
                    let lhs = eval(&binary.lhs, ctx)?;
                    let rhs = eval(&binary.rhs, ctx)?;

                    match binary.op {
                        // Arithmetic
                        Op::Add => lhs.add(&rhs, ctx.config()),
                        Op::Sub => lhs.sub(&rhs, ctx.config()),
                        Op::Mul => lhs.mul(&rhs, ctx.config()),
                        Op::Div => lhs.div(&rhs, ctx.config()),
                        Op::Mod => lhs.modulo(&rhs, ctx.config()),

                        // Bitwise
                        Op::BitAnd => Ok(lhs.bit_and(&rhs)),
                        Op::BitOr => Ok(lhs.bit_or(&rhs)),
                        Op::BitXor => Ok(lhs.bit_xor(&rhs)),
                        Op::LShift => lhs.shl(&rhs, ctx.config()),
                        Op::RShift => lhs.shr(&rhs, ctx.config()),

                        // Comparison
                        Op::Lt => Ok(C::Value::from_bool(lhs.lt(&rhs))),
                        Op::LtEq => Ok(C::Value::from_bool(lhs.le(&rhs))),
                        Op::Gt => Ok(C::Value::from_bool(lhs.gt(&rhs))),
                        Op::GtEq => Ok(C::Value::from_bool(lhs.ge(&rhs))),
                        Op::EqEq => Ok(C::Value::from_bool(lhs.eq(&rhs))),
                        Op::NotEq => Ok(C::Value::from_bool(lhs.ne(&rhs))),

                        Op::And | Op::Or => unreachable!("handled above"),
                        Op::Not | Op::BitNot => unreachable!("unary operator"),
                    }
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

/// Simple i128-based numeric value for C preprocessor compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleInt(pub i128);

impl NumericValue for SimpleInt {
    fn from_bool(b: bool) -> Self {
        Self(i128::from(b))
    }

    fn to_bool(&self) -> bool {
        self.0 != 0
    }

    fn negate(&self, config: EvalConfig) -> Result<Self> {
        match config.overflow {
            OverflowBehavior::Wrap => Ok(Self(self.0.wrapping_neg())),
            OverflowBehavior::Error => self
                .0
                .checked_neg()
                .map(Self)
                .ok_or(Error::Overflow("negation")),
            OverflowBehavior::Saturate => Ok(Self(self.0.saturating_neg())),
        }
    }

    fn bit_not(&self) -> Self {
        Self(!self.0)
    }

    fn add(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        match config.overflow {
            OverflowBehavior::Wrap => Ok(Self(self.0.wrapping_add(rhs.0))),
            OverflowBehavior::Error => self
                .0
                .checked_add(rhs.0)
                .map(Self)
                .ok_or(Error::Overflow("addition")),
            OverflowBehavior::Saturate => Ok(Self(self.0.saturating_add(rhs.0))),
        }
    }

    fn sub(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        match config.overflow {
            OverflowBehavior::Wrap => Ok(Self(self.0.wrapping_sub(rhs.0))),
            OverflowBehavior::Error => self
                .0
                .checked_sub(rhs.0)
                .map(Self)
                .ok_or(Error::Overflow("subtraction")),
            OverflowBehavior::Saturate => Ok(Self(self.0.saturating_sub(rhs.0))),
        }
    }

    fn mul(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        match config.overflow {
            OverflowBehavior::Wrap => Ok(Self(self.0.wrapping_mul(rhs.0))),
            OverflowBehavior::Error => self
                .0
                .checked_mul(rhs.0)
                .map(Self)
                .ok_or(Error::Overflow("multiplication")),
            OverflowBehavior::Saturate => Ok(Self(self.0.saturating_mul(rhs.0))),
        }
    }

    fn div(&self, rhs: &Self, _config: EvalConfig) -> Result<Self> {
        if rhs.0 == 0 {
            return Err(Error::DivisionByZero);
        }
        Ok(Self(self.0.wrapping_div(rhs.0)))
    }

    fn modulo(&self, rhs: &Self, _config: EvalConfig) -> Result<Self> {
        if rhs.0 == 0 {
            return Err(Error::ModuloByZero);
        }
        Ok(Self(self.0.wrapping_rem(rhs.0)))
    }

    fn bit_and(&self, rhs: &Self) -> Self {
        Self(self.0 & rhs.0)
    }

    fn bit_or(&self, rhs: &Self) -> Self {
        Self(self.0 | rhs.0)
    }

    fn bit_xor(&self, rhs: &Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    fn shl(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        if rhs.0 < 0 || rhs.0 > i128::from(config.max_shift) {
            return Err(Error::InvalidShift(rhs.0));
        }
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        Ok(Self(self.0.wrapping_shl(rhs.0 as u32)))
    }

    fn shr(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        if rhs.0 < 0 || rhs.0 > i128::from(config.max_shift) {
            return Err(Error::InvalidShift(rhs.0));
        }
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        Ok(Self(self.0.wrapping_shr(rhs.0 as u32)))
    }

    fn lt(&self, rhs: &Self) -> bool {
        self.0 < rhs.0
    }

    fn le(&self, rhs: &Self) -> bool {
        self.0 <= rhs.0
    }

    fn gt(&self, rhs: &Self) -> bool {
        self.0 > rhs.0
    }

    fn ge(&self, rhs: &Self) -> bool {
        self.0 >= rhs.0
    }

    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }

    fn ne(&self, rhs: &Self) -> bool {
        self.0 != rhs.0
    }
}
