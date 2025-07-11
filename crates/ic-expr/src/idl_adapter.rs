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

//! IDL expression evaluation adapter
//!
//! This module provides types and functions to evaluate IDL expressions
//! using the generic expression evaluation framework with rich type support.

use std::fmt;

use crate::{Error, EvalConfig, EvalContext, NumericValue, OverflowBehavior, Result};

/// Numeric types supported in IDL
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Numeric {
    Bool(bool),
    Octet(u8),
    Int8(i8),
    UInt16(u16),
    Int16(i16),
    UInt32(u32),
    Int32(i32),
    UInt64(u64),
    Int64(i64),
    Float(f32),
    Double(f64),
    Char(char),
}

impl fmt::Display for Numeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{}", v),
            Self::Octet(v) => write!(f, "{}", v),
            Self::Int8(v) => write!(f, "{}", v),
            Self::UInt16(v) => write!(f, "{}", v),
            Self::Int16(v) => write!(f, "{}", v),
            Self::UInt32(v) => write!(f, "{}", v),
            Self::Int32(v) => write!(f, "{}", v),
            Self::UInt64(v) => write!(f, "{}", v),
            Self::Int64(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::Double(v) => write!(f, "{}", v),
            Self::Char(v) => write!(f, "'{}'", v),
        }
    }
}

impl Numeric {
    /// Perform arithmetic operations preserving type
    fn binop_arith<F>(
        &self,
        rhs: &Self,
        op_name: &'static str,
        config: &EvalConfig,
        mut op: F,
    ) -> Result<Self>
    where
        F: FnMut(i128, i128, &EvalConfig) -> Result<i128>,
    {
        // For now, convert to i128, do operation, convert back
        // This matches the current behavior in ic-hir/src/interp.rs
        let lhs_val = self.to_i128();
        let rhs_val = rhs.to_i128();
        let result = op(lhs_val, rhs_val, config)?;

        // Preserve the type of the left operand
        Ok(self.from_i128_preserving_type(result))
    }

    /// Convert to i128 for computation
    fn to_i128(&self) -> i128 {
        match self {
            Self::Bool(v) => *v as i128,
            Self::Octet(v) => *v as i128,
            Self::Int8(v) => *v as i128,
            Self::UInt16(v) => *v as i128,
            Self::Int16(v) => *v as i128,
            Self::UInt32(v) => *v as i128,
            Self::Int32(v) => *v as i128,
            Self::UInt64(v) => *v as i128,
            Self::Int64(v) => *v as i128,
            Self::Float(v) => *v as i128,
            Self::Double(v) => *v as i128,
            Self::Char(v) => *v as i128,
        }
    }

    /// Create from i128 preserving the original type
    fn from_i128_preserving_type(&self, val: i128) -> Self {
        match self {
            Self::Bool(_) => Self::Bool(val != 0),
            Self::Octet(_) => Self::Octet(val as u8),
            Self::Int8(_) => Self::Int8(val as i8),
            Self::UInt16(_) => Self::UInt16(val as u16),
            Self::Int16(_) => Self::Int16(val as i16),
            Self::UInt32(_) => Self::UInt32(val as u32),
            Self::Int32(_) => Self::Int32(val as i32),
            Self::UInt64(_) => Self::UInt64(val as u64),
            Self::Int64(_) => Self::Int64(val as i64),
            Self::Float(_) => Self::Float(val as f32),
            Self::Double(_) => Self::Double(val as f64),
            Self::Char(_) => Self::Char(char::from_u32(val as u32).unwrap_or('\0')),
        }
    }
}

impl NumericValue for Numeric {
    fn from_bool(b: bool) -> Self {
        Self::Bool(b)
    }

    fn to_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            _ => self.to_i128() != 0,
        }
    }

    fn negate(&self, config: &EvalConfig) -> Result<Self> {
        match self {
            // Unsigned types use bitwise NOT to simulate overflow
            Self::Octet(v) => Ok(Self::Octet(!v)),
            Self::UInt16(v) => Ok(Self::UInt16(!v)),
            Self::UInt32(v) => Ok(Self::UInt32(!v)),
            Self::UInt64(v) => Ok(Self::UInt64(!v)),

            // Signed types use arithmetic negation
            Self::Bool(v) => Ok(Self::Bool(!v)),
            Self::Int8(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int8(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int8)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int8(v.saturating_neg())),
            },
            Self::Int16(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int16(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int16)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int16(v.saturating_neg())),
            },
            Self::Int32(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int32(v.saturating_neg())),
            },
            Self::Int64(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int64(v.saturating_neg())),
            },
            Self::Float(v) => Ok(Self::Float(-v)),
            Self::Double(v) => Ok(Self::Double(-v)),
            Self::Char(_) => Ok(self.from_i128_preserving_type(-self.to_i128())),
        }
    }

    fn bit_not(&self) -> Self {
        match self {
            Self::Bool(v) => Self::Bool(!v),
            Self::Octet(v) => Self::Octet(!v),
            Self::Int8(v) => Self::Int8(!v),
            Self::UInt16(v) => Self::UInt16(!v),
            Self::Int16(v) => Self::Int16(!v),
            Self::UInt32(v) => Self::UInt32(!v),
            Self::Int32(v) => Self::Int32(!v),
            Self::UInt64(v) => Self::UInt64(!v),
            Self::Int64(v) => Self::Int64(!v),
            _ => self.from_i128_preserving_type(!self.to_i128()),
        }
    }

    fn add(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        self.binop_arith(rhs, "addition", config, |a, b, cfg| match cfg.overflow {
            OverflowBehavior::Wrap => Ok(a.wrapping_add(b)),
            OverflowBehavior::Error => a.checked_add(b).ok_or(Error::Overflow("addition")),
            OverflowBehavior::Saturate => Ok(a.saturating_add(b)),
        })
    }

    fn sub(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        self.binop_arith(rhs, "subtraction", config, |a, b, cfg| match cfg.overflow {
            OverflowBehavior::Wrap => Ok(a.wrapping_sub(b)),
            OverflowBehavior::Error => a.checked_sub(b).ok_or(Error::Overflow("subtraction")),
            OverflowBehavior::Saturate => Ok(a.saturating_sub(b)),
        })
    }

    fn mul(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        self.binop_arith(rhs, "multiplication", config, |a, b, cfg| {
            match cfg.overflow {
                OverflowBehavior::Wrap => Ok(a.wrapping_mul(b)),
                OverflowBehavior::Error => {
                    a.checked_mul(b).ok_or(Error::Overflow("multiplication"))
                }
                OverflowBehavior::Saturate => Ok(a.saturating_mul(b)),
            }
        })
    }

    fn div(&self, rhs: &Self, _config: &EvalConfig) -> Result<Self> {
        if rhs.to_i128() == 0 {
            return Err(Error::DivisionByZero);
        }
        Ok(self.from_i128_preserving_type(self.to_i128().wrapping_div(rhs.to_i128())))
    }

    fn modulo(&self, rhs: &Self, _config: &EvalConfig) -> Result<Self> {
        if rhs.to_i128() == 0 {
            return Err(Error::ModuloByZero);
        }
        Ok(self.from_i128_preserving_type(self.to_i128().wrapping_rem(rhs.to_i128())))
    }

    fn bit_and(&self, rhs: &Self) -> Self {
        self.from_i128_preserving_type(self.to_i128() & rhs.to_i128())
    }

    fn bit_or(&self, rhs: &Self) -> Self {
        self.from_i128_preserving_type(self.to_i128() | rhs.to_i128())
    }

    fn bit_xor(&self, rhs: &Self) -> Self {
        self.from_i128_preserving_type(self.to_i128() ^ rhs.to_i128())
    }

    fn shl(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        let shift = rhs.to_i128();
        if shift < 0 || shift > config.max_shift as i128 {
            return Err(Error::InvalidShift(shift));
        }
        Ok(self.from_i128_preserving_type(self.to_i128().wrapping_shl(shift as u32)))
    }

    fn shr(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        let shift = rhs.to_i128();
        if shift < 0 || shift > config.max_shift as i128 {
            return Err(Error::InvalidShift(shift));
        }
        Ok(self.from_i128_preserving_type(self.to_i128().wrapping_shr(shift as u32)))
    }

    fn lt(&self, rhs: &Self) -> bool {
        self.to_i128() < rhs.to_i128()
    }

    fn le(&self, rhs: &Self) -> bool {
        self.to_i128() <= rhs.to_i128()
    }

    fn gt(&self, rhs: &Self) -> bool {
        self.to_i128() > rhs.to_i128()
    }

    fn ge(&self, rhs: &Self) -> bool {
        self.to_i128() >= rhs.to_i128()
    }

    fn eq(&self, rhs: &Self) -> bool {
        self.to_i128() == rhs.to_i128()
    }

    fn ne(&self, rhs: &Self) -> bool {
        self.to_i128() != rhs.to_i128()
    }
}

/// IDL expression literal
#[derive(Debug, Clone)]
pub enum IdlLiteral {
    /// Numeric literal with type
    Numeric(Numeric),
    /// String literal
    String(String),
    /// Null value
    Null,
    /// Path to a constant definition
    Path(Vec<String>),
}

/// Context for evaluating IDL expressions
pub struct IdlContext<'a> {
    /// Configuration for evaluation
    config: EvalConfig,
    /// Callback to resolve paths to values
    resolve_path: Box<dyn FnMut(&[String]) -> Option<Numeric> + 'a>,
}

impl<'a> IdlContext<'a> {
    /// Create a new IDL evaluation context
    pub fn new<F>(resolve_path: F) -> Self
    where
        F: FnMut(&[String]) -> Option<Numeric> + 'a,
    {
        Self {
            config: EvalConfig {
                overflow: OverflowBehavior::Wrap,
                max_shift: 63, // More conservative for IDL
            },
            resolve_path: Box::new(resolve_path),
        }
    }

    /// Create a context with custom configuration
    pub fn with_config<F>(config: EvalConfig, resolve_path: F) -> Self
    where
        F: FnMut(&[String]) -> Option<Numeric> + 'a,
    {
        Self {
            config,
            resolve_path: Box::new(resolve_path),
        }
    }
}

impl<'a> EvalContext<IdlLiteral> for IdlContext<'a> {
    type Value = Numeric;

    fn eval_literal(&mut self, lit: &IdlLiteral) -> Result<Self::Value> {
        match lit {
            IdlLiteral::Numeric(n) => Ok(*n),
            IdlLiteral::Path(segments) => (self.resolve_path)(segments).ok_or_else(|| {
                Error::Custom(format!("Undefined constant: {}", segments.join("::")))
            }),
            IdlLiteral::String(_) => Err(Error::Custom(
                "String literals not supported in numeric expressions".into(),
            )),
            IdlLiteral::Null => Err(Error::Custom(
                "Null not supported in numeric expressions".into(),
            )),
        }
    }

    fn config(&self) -> &EvalConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Binary, Expr, Op, eval};

    #[test]
    fn test_type_preservation() {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(100))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(200))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::UInt16(300)));
    }

    #[test]
    fn test_unsigned_negation() {
        // For unsigned types, negation uses bitwise NOT
        let expr = Expr::Unary(Box::new(crate::Unary {
            op: Op::Sub,
            expr: Expr::Lit(IdlLiteral::Numeric(Numeric::Octet(1))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::Octet(254))); // !1 = 254 for u8
    }

    #[test]
    fn test_path_resolution() {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Path(vec![
                "Constants".into(),
                "MAX_SIZE".into(),
            ])),
            op: Op::Div,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(2))),
        }));

        let mut ctx = IdlContext::new(|path| {
            if path == ["Constants", "MAX_SIZE"] {
                Some(Numeric::Int32(1024))
            } else {
                None
            }
        });

        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::Int32(512)));
    }
}
