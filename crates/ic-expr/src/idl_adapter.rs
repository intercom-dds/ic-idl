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

//! IDL expression evaluation adapter with C semantics
//!
//! This module implements C's integer promotion and usual arithmetic
//! conversion rules for IDL expressions.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

use std::fmt;

use crate::{Error, EvalConfig, EvalContext, NumericValue, OverflowBehavior, Result};

/// Numeric types supported in IDL
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Numeric {
    Bool(bool),
    Octet(u8),   // unsigned char
    Int8(i8),    // signed char
    UInt16(u16), // unsigned short
    Int16(i16),  // short
    UInt32(u32), // unsigned int/long (assuming 32-bit)
    Int32(i32),  // int/long (assuming 32-bit)
    UInt64(u64), // unsigned long long
    Int64(i64),  // long long
    Float(f32),
    Double(f64),
    Char(char), // char (may be signed or unsigned)
}

impl fmt::Display for Numeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{v}"),
            Self::Octet(v) => write!(f, "{v}"),
            Self::Int8(v) => write!(f, "{v}"),
            Self::UInt16(v) => write!(f, "{v}"),
            Self::Int16(v) => write!(f, "{v}"),
            Self::UInt32(v) => write!(f, "{v}"),
            Self::Int32(v) => write!(f, "{v}"),
            Self::UInt64(v) => write!(f, "{v}"),
            Self::Int64(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Double(v) => write!(f, "{v}"),
            Self::Char(v) => write!(f, "'{v}'"),
        }
    }
}

impl Numeric {
    /// Get the rank for integer promotions (C11 6.3.1.1)
    /// Higher rank means "larger" type
    fn rank(&self) -> i32 {
        match self {
            Self::Bool(_) => 0,
            Self::Int8(_) | Self::Octet(_) | Self::Char(_) => 1,
            Self::Int16(_) | Self::UInt16(_) => 2,
            Self::Int32(_) | Self::UInt32(_) => 3,
            Self::Int64(_) | Self::UInt64(_) => 4,
            Self::Float(_) => 5,
            Self::Double(_) => 6,
        }
    }

    /// Check if this is an unsigned type
    fn is_unsigned(&self) -> bool {
        matches!(
            self,
            Self::Bool(_) | Self::Octet(_) | Self::UInt16(_) | Self::UInt32(_) | Self::UInt64(_)
        )
    }

    /// Check if this is a floating-point type
    fn is_floating(&self) -> bool {
        matches!(self, Self::Float(_) | Self::Double(_))
    }

    /// Perform integer promotions (C11 6.3.1.1)
    /// Types smaller than int are promoted to int
    fn promote(&self) -> Self {
        match self {
            // These types are promoted to int (Int32)
            Self::Bool(v) => Self::Int32(i32::from(*v)),
            Self::Int8(v) => Self::Int32(i32::from(*v)),
            Self::Octet(v) => Self::Int32(i32::from(*v)),
            Self::Char(v) => Self::Int32(*v as i32),
            Self::Int16(v) => Self::Int32(i32::from(*v)),
            Self::UInt16(v) => {
                // UInt16 promotes to Int32 if it fits, otherwise UInt32
                // Since all UInt16 values fit in Int32, always promote to Int32
                Self::Int32(i32::from(*v))
            }
            // These types are not promoted
            _ => *self,
        }
    }

    /// Convert to i64 for computation
    fn to_i64(self) -> i64 {
        match self {
            Self::Bool(v) => i64::from(v),
            Self::Octet(v) => i64::from(v),
            Self::Int8(v) => i64::from(v),
            Self::UInt16(v) => i64::from(v),
            Self::Int16(v) => i64::from(v),
            Self::UInt32(v) => i64::from(v),
            Self::Int32(v) => i64::from(v),
            Self::UInt64(v) => v as i64,
            Self::Int64(v) => v,
            Self::Float(v) => v as i64,
            Self::Double(v) => v as i64,
            Self::Char(v) => v as i64,
        }
    }

    /// Convert to u64 for unsigned computation
    fn to_u64(self) -> u64 {
        match self {
            Self::Bool(v) => u64::from(v),
            Self::Octet(v) => u64::from(v),
            Self::Int8(v) => v as u64,
            Self::UInt16(v) => u64::from(v),
            Self::Int16(v) => v as u64,
            Self::UInt32(v) => u64::from(v),
            Self::Int32(v) => v as u64,
            Self::UInt64(v) => v,
            Self::Int64(v) => v as u64,
            Self::Float(v) => v as u64,
            Self::Double(v) => v as u64,
            Self::Char(v) => v as u64,
        }
    }

    /// Convert to f64 for floating-point computation
    fn to_f64(self) -> f64 {
        match self {
            Self::Bool(v) => {
                if v {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Octet(v) => f64::from(v),
            Self::Int8(v) => f64::from(v),
            Self::UInt16(v) => f64::from(v),
            Self::Int16(v) => f64::from(v),
            Self::UInt32(v) => f64::from(v),
            Self::Int32(v) => f64::from(v),
            Self::UInt64(v) => v as f64,
            Self::Int64(v) => v as f64,
            Self::Float(v) => f64::from(v),
            Self::Double(v) => v,
            Self::Char(v) => f64::from(v as u32),
        }
    }

    /// Create a value of the given type from i64
    fn from_i64(value: i64, target_type: &Self) -> Self {
        match target_type {
            Self::Bool(_) => Self::Bool(value != 0),
            Self::Octet(_) => Self::Octet(value as u8),
            Self::Int8(_) => Self::Int8(value as i8),
            Self::UInt16(_) => Self::UInt16(value as u16),
            Self::Int16(_) => Self::Int16(value as i16),
            Self::UInt32(_) => Self::UInt32(value as u32),
            Self::Int32(_) => Self::Int32(value as i32),
            Self::UInt64(_) => Self::UInt64(value as u64),
            Self::Int64(_) => Self::Int64(value),
            Self::Float(_) => Self::Float(value as f32),
            Self::Double(_) => Self::Double(value as f64),
            Self::Char(_) => Self::Char(char::from_u32(value as u32).unwrap_or('\0')),
        }
    }

    /// Create a value of the given type from u64
    fn from_u64(value: u64, target_type: &Self) -> Self {
        match target_type {
            Self::Bool(_) => Self::Bool(value != 0),
            Self::Octet(_) => Self::Octet(value as u8),
            Self::Int8(_) => Self::Int8(value as i8),
            Self::UInt16(_) => Self::UInt16(value as u16),
            Self::Int16(_) => Self::Int16(value as i16),
            Self::UInt32(_) => Self::UInt32(value as u32),
            Self::Int32(_) => Self::Int32(value as i32),
            Self::UInt64(_) => Self::UInt64(value),
            Self::Int64(_) => Self::Int64(value as i64),
            Self::Float(_) => Self::Float(value as f32),
            Self::Double(_) => Self::Double(value as f64),
            Self::Char(_) => Self::Char(char::from_u32(value as u32).unwrap_or('\0')),
        }
    }

    /// Create a value of the given type from f64
    fn from_f64(value: f64, target_type: &Self) -> Self {
        match target_type {
            Self::Bool(_) => Self::Bool(value != 0.0),
            Self::Octet(_) => Self::Octet(value as u8),
            Self::Int8(_) => Self::Int8(value as i8),
            Self::UInt16(_) => Self::UInt16(value as u16),
            Self::Int16(_) => Self::Int16(value as i16),
            Self::UInt32(_) => Self::UInt32(value as u32),
            Self::Int32(_) => Self::Int32(value as i32),
            Self::UInt64(_) => Self::UInt64(value as u64),
            Self::Int64(_) => Self::Int64(value as i64),
            Self::Float(_) => Self::Float(value as f32),
            Self::Double(_) => Self::Double(value),
            Self::Char(_) => Self::Char(char::from_u32(value as u32).unwrap_or('\0')),
        }
    }
}

/// Perform usual arithmetic conversions (C11 6.3.1.8)
/// Returns the common type that both operands should be converted to
fn usual_arithmetic_conversions(lhs: &Numeric, rhs: &Numeric) -> Numeric {
    // First, perform integer promotions on both operands
    let lhs = lhs.promote();
    let rhs = rhs.promote();

    // If either operand is floating-point
    if lhs.is_floating() || rhs.is_floating() {
        // If either is double, result is double
        if matches!(lhs, Numeric::Double(_)) || matches!(rhs, Numeric::Double(_)) {
            return Numeric::Double(0.0);
        }
        // Otherwise, result is float
        return Numeric::Float(0.0);
    }

    // Both operands are integers after promotion
    let lhs_rank = lhs.rank();
    let rhs_rank = rhs.rank();
    let lhs_unsigned = lhs.is_unsigned();
    let rhs_unsigned = rhs.is_unsigned();

    // If both have the same signedness
    if lhs_unsigned == rhs_unsigned {
        // Return the type with greater rank
        if lhs_rank >= rhs_rank { lhs } else { rhs }
    } else {
        // Different signedness
        let (unsigned_op, signed_op) = if lhs_unsigned {
            (&lhs, &rhs)
        } else {
            (&rhs, &lhs)
        };

        let unsigned_rank = unsigned_op.rank();
        let signed_rank = signed_op.rank();

        if unsigned_rank >= signed_rank {
            // The unsigned type has greater or equal rank
            *unsigned_op
        } else {
            // The signed type has greater rank
            // If the signed type can represent all values of the unsigned type,
            // use the signed type. Otherwise, convert to the unsigned version
            // of the signed type.

            // For our type system:
            // Int32 can represent all values of UInt16 (already handled by promotion)
            // Int64 can represent all values of UInt32
            match (signed_op, unsigned_op) {
                (Numeric::Int64(_), Numeric::UInt32(_)) => Numeric::Int64(0),
                _ => {
                    // Convert to unsigned version of the signed type
                    match signed_op {
                        Numeric::Int32(_) => Numeric::UInt32(0),
                        Numeric::Int64(_) => Numeric::UInt64(0),
                        _ => *signed_op,
                    }
                }
            }
        }
    }
}

/// Perform a binary arithmetic operation with proper type conversions
#[allow(clippy::trivially_copy_pass_by_ref)]
fn binop_arithmetic<F>(
    lhs: &Numeric,
    rhs: &Numeric,
    config: &EvalConfig,
    mut op: F,
) -> Result<Numeric>
where
    F: FnMut(Numeric, Numeric, &EvalConfig) -> Result<Numeric>,
{
    // Determine the common type
    let common_type = usual_arithmetic_conversions(lhs, rhs);

    // Convert both operands to the common type
    let lhs_converted = convert_to_type(lhs, &common_type);
    let rhs_converted = convert_to_type(rhs, &common_type);

    // Perform the operation
    op(lhs_converted, rhs_converted, config)
}

/// Convert a numeric value to a specific type
fn convert_to_type(value: &Numeric, target_type: &Numeric) -> Numeric {
    // First promote if needed
    let value = value.promote();

    if std::mem::discriminant(&value) == std::mem::discriminant(target_type) {
        return value;
    }

    // Convert based on target type
    if target_type.is_floating() {
        Numeric::from_f64(value.to_f64(), target_type)
    } else if target_type.is_unsigned() {
        Numeric::from_u64(value.to_u64(), target_type)
    } else {
        Numeric::from_i64(value.to_i64(), target_type)
    }
}

impl NumericValue for Numeric {
    fn from_bool(b: bool) -> Self {
        Self::Bool(b)
    }

    fn to_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Float(v) => *v != 0.0,
            Self::Double(v) => *v != 0.0,
            _ => self.to_i64() != 0,
        }
    }

    fn negate(&self, config: &EvalConfig) -> Result<Self> {
        match self {
            Self::Bool(v) => Ok(Self::Int32(-i32::from(*v))),
            Self::Octet(v) => Ok(Self::Int32(-i32::from(*v))),
            Self::Int8(v) => Ok(Self::Int32(-i32::from(*v))),
            Self::UInt16(v) => Ok(Self::Int32(-i32::from(*v))),
            Self::Int16(v) => Ok(Self::Int32(-i32::from(*v))),

            Self::UInt32(v) => {
                // -UInt32 -> Int64 to avoid overflow
                Ok(Self::Int64(-i64::from(*v)))
            }
            Self::Int32(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int32(v.saturating_neg())),
            },

            Self::UInt64(v) => {
                // -UInt64 wraps around in unsigned arithmetic
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::UInt64(v.wrapping_neg())),
                    OverflowBehavior::Error => Err(Error::Overflow("negation of unsigned")),
                    OverflowBehavior::Saturate => Ok(Self::UInt64(0)),
                }
            }
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
            Self::Char(v) => Ok(Self::Int32(-(*v as i32))),
        }
    }

    fn bit_not(&self) -> Self {
        // Bitwise NOT promotes to int first
        let promoted = self.promote();
        match promoted {
            Self::Int32(v) => Self::Int32(!v),
            Self::UInt32(v) => Self::UInt32(!v),
            Self::Int64(v) => Self::Int64(!v),
            Self::UInt64(v) => Self::UInt64(!v),
            _ => promoted, // Shouldn't happen after promotion
        }
    }

    fn add(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        binop_arithmetic(self, rhs, config, |lhs, rhs, cfg| match (lhs, rhs) {
            (Self::Int32(a), Self::Int32(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_add(b))),
            },
            (Self::UInt32(a), Self::UInt32(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt32(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::UInt32)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::UInt32(a.saturating_add(b))),
            },
            (Self::Int64(a), Self::Int64(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_add(b))),
            },
            (Self::UInt64(a), Self::UInt64(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::UInt64)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(a.saturating_add(b))),
            },
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a + b)),
            (Self::Double(a), Self::Double(b)) => Ok(Self::Double(a + b)),
            _ => unreachable!("Mismatched types after conversion"),
        })
    }

    fn sub(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        binop_arithmetic(self, rhs, config, |lhs, rhs, cfg| match (lhs, rhs) {
            (Self::Int32(a), Self::Int32(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_sub(b))),
            },
            (Self::UInt32(a), Self::UInt32(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt32(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::UInt32)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::UInt32(a.saturating_sub(b))),
            },
            (Self::Int64(a), Self::Int64(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_sub(b))),
            },
            (Self::UInt64(a), Self::UInt64(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::UInt64)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(a.saturating_sub(b))),
            },
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a - b)),
            (Self::Double(a), Self::Double(b)) => Ok(Self::Double(a - b)),
            _ => unreachable!("Mismatched types after conversion"),
        })
    }

    fn mul(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        binop_arithmetic(self, rhs, config, |lhs, rhs, cfg| match (lhs, rhs) {
            (Self::Int32(a), Self::Int32(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_mul(b))),
            },
            (Self::UInt32(a), Self::UInt32(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt32(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::UInt32)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::UInt32(a.saturating_mul(b))),
            },
            (Self::Int64(a), Self::Int64(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_mul(b))),
            },
            (Self::UInt64(a), Self::UInt64(b)) => match cfg.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::UInt64)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(a.saturating_mul(b))),
            },
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a * b)),
            (Self::Double(a), Self::Double(b)) => Ok(Self::Double(a * b)),
            _ => unreachable!("Mismatched types after conversion"),
        })
    }

    fn div(&self, rhs: &Self, _config: &EvalConfig) -> Result<Self> {
        binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => {
                    if b == 0 {
                        return Err(Error::DivisionByZero);
                    }
                    Ok(Self::Int32(a.wrapping_div(b)))
                }
                (Self::UInt32(a), Self::UInt32(b)) => {
                    if b == 0 {
                        return Err(Error::DivisionByZero);
                    }
                    Ok(Self::UInt32(a / b))
                }
                (Self::Int64(a), Self::Int64(b)) => {
                    if b == 0 {
                        return Err(Error::DivisionByZero);
                    }
                    Ok(Self::Int64(a.wrapping_div(b)))
                }
                (Self::UInt64(a), Self::UInt64(b)) => {
                    if b == 0 {
                        return Err(Error::DivisionByZero);
                    }
                    Ok(Self::UInt64(a / b))
                }
                (Self::Float(a), Self::Float(b)) => {
                    if b == 0.0 {
                        return Err(Error::DivisionByZero);
                    }
                    Ok(Self::Float(a / b))
                }
                (Self::Double(a), Self::Double(b)) => {
                    if b == 0.0 {
                        return Err(Error::DivisionByZero);
                    }
                    Ok(Self::Double(a / b))
                }
                _ => unreachable!("Mismatched types after conversion"),
            }
        })
    }

    fn modulo(&self, rhs: &Self, _config: &EvalConfig) -> Result<Self> {
        binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => {
                    if b == 0 {
                        return Err(Error::ModuloByZero);
                    }
                    Ok(Self::Int32(a.wrapping_rem(b)))
                }
                (Self::UInt32(a), Self::UInt32(b)) => {
                    if b == 0 {
                        return Err(Error::ModuloByZero);
                    }
                    Ok(Self::UInt32(a % b))
                }
                (Self::Int64(a), Self::Int64(b)) => {
                    if b == 0 {
                        return Err(Error::ModuloByZero);
                    }
                    Ok(Self::Int64(a.wrapping_rem(b)))
                }
                (Self::UInt64(a), Self::UInt64(b)) => {
                    if b == 0 {
                        return Err(Error::ModuloByZero);
                    }
                    Ok(Self::UInt64(a % b))
                }
                (Self::Float(_), Self::Float(_)) | (Self::Double(_), Self::Double(_)) => Err(
                    Error::Custom("modulo not supported for floating-point".into()),
                ),
                _ => unreachable!("Mismatched types after conversion"),
            }
        })
    }

    fn bit_and(&self, rhs: &Self) -> Self {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Int32(a & b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::UInt32(a & b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Int64(a & b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::UInt64(a & b)),
                _ => Err(Error::Custom(
                    "bitwise operations not supported for floating-point".into(),
                )),
            }
        });
        result.unwrap_or(Self::Int32(0))
    }

    fn bit_or(&self, rhs: &Self) -> Self {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Int32(a | b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::UInt32(a | b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Int64(a | b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::UInt64(a | b)),
                _ => Err(Error::Custom(
                    "bitwise operations not supported for floating-point".into(),
                )),
            }
        });
        result.unwrap_or(Self::Int32(0))
    }

    fn bit_xor(&self, rhs: &Self) -> Self {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Int32(a ^ b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::UInt32(a ^ b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Int64(a ^ b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::UInt64(a ^ b)),
                _ => Err(Error::Custom(
                    "bitwise operations not supported for floating-point".into(),
                )),
            }
        });
        result.unwrap_or(Self::Int32(0))
    }

    fn shl(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        // Left operand is promoted, right operand is used as-is
        let lhs = self.promote();
        let shift = rhs.to_u64() as u32;

        if shift > config.max_shift {
            return Err(Error::InvalidShift(i128::from(shift)));
        }

        match lhs {
            Self::Int32(v) => Ok(Self::Int32(v.wrapping_shl(shift))),
            Self::UInt32(v) => Ok(Self::UInt32(v.wrapping_shl(shift))),
            Self::Int64(v) => Ok(Self::Int64(v.wrapping_shl(shift))),
            Self::UInt64(v) => Ok(Self::UInt64(v.wrapping_shl(shift))),
            _ => Err(Error::Custom(
                "shift operations not supported for this type".into(),
            )),
        }
    }

    fn shr(&self, rhs: &Self, config: &EvalConfig) -> Result<Self> {
        // Left operand is promoted, right operand is used as-is
        let lhs = self.promote();
        let shift = rhs.to_u64() as u32;

        if shift > config.max_shift {
            return Err(Error::InvalidShift(i128::from(shift)));
        }

        match lhs {
            Self::Int32(v) => Ok(Self::Int32(v.wrapping_shr(shift))),
            Self::UInt32(v) => Ok(Self::UInt32(v.wrapping_shr(shift))),
            Self::Int64(v) => Ok(Self::Int64(v.wrapping_shr(shift))),
            Self::UInt64(v) => Ok(Self::UInt64(v.wrapping_shr(shift))),
            _ => Err(Error::Custom(
                "shift operations not supported for this type".into(),
            )),
        }
    }

    fn lt(&self, rhs: &Self) -> bool {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Bool(a < b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::Bool(a < b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Bool(a < b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::Bool(a < b)),
                (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a < b)),
                (Self::Double(a), Self::Double(b)) => Ok(Self::Bool(a < b)),
                _ => unreachable!("Mismatched types after conversion"),
            }
        });
        match result {
            Ok(Self::Bool(b)) => b,
            _ => false,
        }
    }

    fn le(&self, rhs: &Self) -> bool {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Bool(a <= b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::Bool(a <= b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Bool(a <= b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::Bool(a <= b)),
                (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a <= b)),
                (Self::Double(a), Self::Double(b)) => Ok(Self::Bool(a <= b)),
                _ => unreachable!("Mismatched types after conversion"),
            }
        });
        match result {
            Ok(Self::Bool(b)) => b,
            _ => false,
        }
    }

    fn gt(&self, rhs: &Self) -> bool {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Bool(a > b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::Bool(a > b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Bool(a > b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::Bool(a > b)),
                (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a > b)),
                (Self::Double(a), Self::Double(b)) => Ok(Self::Bool(a > b)),
                _ => unreachable!("Mismatched types after conversion"),
            }
        });
        match result {
            Ok(Self::Bool(b)) => b,
            _ => false,
        }
    }

    fn ge(&self, rhs: &Self) -> bool {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Bool(a >= b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::Bool(a >= b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Bool(a >= b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::Bool(a >= b)),
                (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a >= b)),
                (Self::Double(a), Self::Double(b)) => Ok(Self::Bool(a >= b)),
                _ => unreachable!("Mismatched types after conversion"),
            }
        });
        match result {
            Ok(Self::Bool(b)) => b,
            _ => false,
        }
    }

    fn eq(&self, rhs: &Self) -> bool {
        let result = binop_arithmetic(self, rhs, &EvalConfig::default(), |lhs, rhs, _| {
            match (lhs, rhs) {
                (Self::Int32(a), Self::Int32(b)) => Ok(Self::Bool(a == b)),
                (Self::UInt32(a), Self::UInt32(b)) => Ok(Self::Bool(a == b)),
                (Self::Int64(a), Self::Int64(b)) => Ok(Self::Bool(a == b)),
                (Self::UInt64(a), Self::UInt64(b)) => Ok(Self::Bool(a == b)),
                (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a == b)),
                (Self::Double(a), Self::Double(b)) => Ok(Self::Bool(a == b)),
                _ => unreachable!("Mismatched types after conversion"),
            }
        });
        match result {
            Ok(Self::Bool(b)) => b,
            _ => false,
        }
    }

    fn ne(&self, rhs: &Self) -> bool {
        !NumericValue::eq(self, rhs)
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

/// Function type for path resolution
type PathResolver<'a> = Box<dyn FnMut(&[String]) -> Option<Numeric> + 'a>;

/// Context for evaluating IDL expressions
pub struct IdlContext<'a> {
    /// Configuration for evaluation
    config: EvalConfig,
    /// Callback to resolve paths to values
    resolve_path: PathResolver<'a>,
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

impl EvalContext<IdlLiteral> for IdlContext<'_> {
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
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::{Binary, Expr, Op, eval};

    #[test]
    fn test_integer_promotion() {
        // Test that small types are promoted to Int32
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int16(20))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::Int32(30)));
    }

    #[test]
    fn test_unsigned_signed_conversion() {
        // UInt32 + Int32 -> UInt32
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(100))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(50))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::UInt32(150)));
    }

    #[test]
    fn test_unsigned_overflow_wrap() {
        // Test unsigned overflow with wrapping
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(u32::MAX))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(1))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::UInt32(0)));
    }

    #[test]
    fn test_floating_point_promotion() {
        // Int + Float -> Float
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(2.5))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        match result {
            Numeric::Float(v) => assert_eq!(v, 12.5),
            _ => panic!("Expected Float result"),
        }
    }

    #[test]
    fn test_shift_operations() {
        // Shifts don't do usual arithmetic conversions on the right operand
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(8))),
            op: Op::LShift,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(2))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::Int32(32)));
    }

    #[test]
    fn test_comparison_returns_bool() {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
            op: Op::Gt,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(5))),
        }));

        let mut ctx = IdlContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert!(matches!(result, Numeric::Bool(true)));
    }
}

// Casting/truncation support
impl Numeric {
    /// Cast this numeric value to the specified target type
    /// This may truncate or change the value depending on the types involved
    ///
    /// # Errors
    /// Returns an error if the value cannot be represented in the target type
    #[allow(clippy::too_many_lines)]
    pub fn cast_to(&self, target: &Numeric) -> Result<Numeric> {
        use Numeric::{
            Bool, Char, Double, Float, Int8, Int16, Int32, Int64, Octet, UInt16, UInt32, UInt64,
        };

        // First, get the numeric value as the most general type
        let as_f64 = self.to_f64();
        let as_i64 = self.to_i64();
        let as_u64 = self.to_u64();

        match target {
            Bool(_) => Ok(Bool(self.to_bool())),

            Int8(_) => {
                if self.is_floating() {
                    // Truncate float to integer
                    let truncated = as_f64.trunc() as i64;
                    if truncated < i64::from(i8::MIN) || truncated > i64::from(i8::MAX) {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for Int8"
                        )))
                    } else {
                        Ok(Int8(truncated as i8))
                    }
                } else {
                    // Integer to integer cast - check range
                    if as_i64 < i64::from(i8::MIN) || as_i64 > i64::from(i8::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_i64} out of range for Int8"
                        )))
                    } else {
                        Ok(Int8(as_i64 as i8))
                    }
                }
            }

            Octet(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc() as i64;
                    if truncated < 0 || truncated > i64::from(u8::MAX) {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for Octet"
                        )))
                    } else {
                        Ok(Octet(truncated as u8))
                    }
                } else if self.is_unsigned() {
                    if as_u64 > u64::from(u8::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_u64} out of range for Octet"
                        )))
                    } else {
                        Ok(Octet(as_u64 as u8))
                    }
                } else {
                    // Signed to unsigned - check for range
                    if as_i64 < 0 || as_i64 > i64::from(u8::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_i64} out of range for Octet"
                        )))
                    } else {
                        Ok(Octet(as_i64 as u8))
                    }
                }
            }

            Int16(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc() as i64;
                    if truncated < i64::from(i16::MIN) || truncated > i64::from(i16::MAX) {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for Int16"
                        )))
                    } else {
                        Ok(Int16(truncated as i16))
                    }
                } else {
                    // Integer to integer cast - check range
                    if as_i64 < i64::from(i16::MIN) || as_i64 > i64::from(i16::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_i64} out of range for Int16"
                        )))
                    } else {
                        Ok(Int16(as_i64 as i16))
                    }
                }
            }

            UInt16(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc() as i64;
                    if truncated < 0 || truncated > i64::from(u16::MAX) {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for UInt16"
                        )))
                    } else {
                        Ok(UInt16(truncated as u16))
                    }
                } else if self.is_unsigned() {
                    if as_u64 > u64::from(u16::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_u64} out of range for UInt16"
                        )))
                    } else {
                        Ok(UInt16(as_u64 as u16))
                    }
                } else if as_i64 < 0 || as_i64 > i64::from(u16::MAX) {
                    Err(Error::Custom(format!(
                        "Value {as_i64} out of range for UInt16"
                    )))
                } else {
                    Ok(UInt16(as_i64 as u16))
                }
            }

            Int32(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc() as i64;
                    if truncated < i64::from(i32::MIN) || truncated > i64::from(i32::MAX) {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for Int32"
                        )))
                    } else {
                        Ok(Int32(truncated as i32))
                    }
                } else {
                    // Integer to integer cast - check range
                    if as_i64 < i64::from(i32::MIN) || as_i64 > i64::from(i32::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_i64} out of range for Int32"
                        )))
                    } else {
                        Ok(Int32(as_i64 as i32))
                    }
                }
            }

            UInt32(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc() as i64;
                    if truncated < 0 || truncated > i64::from(u32::MAX) {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for UInt32"
                        )))
                    } else {
                        Ok(UInt32(truncated as u32))
                    }
                } else if self.is_unsigned() {
                    if as_u64 > u64::from(u32::MAX) {
                        Err(Error::Custom(format!(
                            "Value {as_u64} out of range for UInt32"
                        )))
                    } else {
                        Ok(UInt32(as_u64 as u32))
                    }
                } else if as_i64 < 0 || as_i64 > i64::from(u32::MAX) {
                    Err(Error::Custom(format!(
                        "Value {as_i64} out of range for UInt32"
                    )))
                } else {
                    Ok(UInt32(as_i64 as u32))
                }
            }

            Int64(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc();
                    // Check if the float value fits in i64
                    if truncated < i64::MIN as f64 || truncated > i64::MAX as f64 {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for Int64"
                        )))
                    } else {
                        Ok(Int64(truncated as i64))
                    }
                } else {
                    Ok(Int64(as_i64))
                }
            }

            UInt64(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc();
                    if truncated < 0.0 || truncated > u64::MAX as f64 {
                        Err(Error::Custom(format!(
                            "Value {truncated} out of range for UInt64"
                        )))
                    } else {
                        Ok(UInt64(truncated as u64))
                    }
                } else if self.is_unsigned() {
                    Ok(UInt64(as_u64))
                } else if as_i64 < 0 {
                    Err(Error::Custom(format!(
                        "Cannot cast negative value {as_i64} to UInt64"
                    )))
                } else {
                    Ok(UInt64(as_i64 as u64))
                }
            }

            Float(_) => Ok(Float(as_f64 as f32)),

            Double(_) => Ok(Double(as_f64)),

            Char(_) => {
                if self.is_floating() {
                    let truncated = as_f64.trunc() as u32;
                    char::from_u32(truncated).map(Char).ok_or_else(|| {
                        Error::Custom(format!("Value {truncated} is not a valid char"))
                    })
                } else {
                    let val = if self.is_unsigned() {
                        as_u64 as u32
                    } else {
                        if as_i64 < 0 {
                            return Err(Error::Custom(format!(
                                "Cannot cast negative value {as_i64} to Char"
                            )));
                        }
                        as_i64 as u32
                    };
                    char::from_u32(val)
                        .map(Char)
                        .ok_or_else(|| Error::Custom(format!("Value {val} is not a valid char")))
                }
            }
        }
    }

    /// Cast with wrapping behavior (like C-style casts)
    /// This will truncate without range checking
    #[must_use]
    pub fn cast_to_wrapping(&self, target: &Numeric) -> Numeric {
        use Numeric::{
            Bool, Char, Double, Float, Int8, Int16, Int32, Int64, Octet, UInt16, UInt32, UInt64,
        };

        let as_i64 = self.to_i64();
        let as_u64 = self.to_u64();
        let as_f64 = self.to_f64();

        match target {
            Bool(_) => Bool(self.to_bool()),
            Int8(_) => Int8(as_i64 as i8),
            Octet(_) => Octet(as_u64 as u8),
            Int16(_) => Int16(as_i64 as i16),
            UInt16(_) => UInt16(as_u64 as u16),
            Int32(_) => Int32(as_i64 as i32),
            UInt32(_) => UInt32(as_u64 as u32),
            Int64(_) => Int64(as_i64),
            UInt64(_) => UInt64(as_u64),
            Float(_) => Float(as_f64 as f32),
            Double(_) => Double(as_f64),
            Char(_) => {
                let val = as_u64 as u32;
                // If invalid char, use replacement character
                Char(char::from_u32(val).unwrap_or('\u{FFFD}'))
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod cast_tests {
    use super::*;

    #[test]
    fn test_int_to_smaller_int() {
        // Int32(300) -> Int8 should fail with range check
        let val = Numeric::Int32(300);
        let result = val.cast_to(&Numeric::Int8(0));
        assert!(result.is_err());

        // But wrapping cast should work
        let wrapped = val.cast_to_wrapping(&Numeric::Int8(0));
        assert!(matches!(wrapped, Numeric::Int8(44))); // 300 & 0xFF = 44

        // Int32(100) -> Int8 should succeed
        let val2 = Numeric::Int32(100);
        let result2 = val2.cast_to(&Numeric::Int8(0)).unwrap();
        assert!(matches!(result2, Numeric::Int8(100)));
    }

    #[test]
    fn test_float_to_int() {
        // Float(3.14) -> Int32 truncates
        let val = Numeric::Float(std::f32::consts::PI);
        let result = val.cast_to(&Numeric::Int32(0)).unwrap();
        assert!(matches!(result, Numeric::Int32(3)));

        // Float(-2.9) -> Int32 truncates toward zero
        let val2 = Numeric::Float(-2.9);
        let result2 = val2.cast_to(&Numeric::Int32(0)).unwrap();
        assert!(matches!(result2, Numeric::Int32(-2)));
    }

    #[test]
    fn test_signed_to_unsigned() {
        // Int32(-1) -> UInt32 should fail with range check
        let val = Numeric::Int32(-1);
        let result = val.cast_to(&Numeric::UInt32(0));
        assert!(result.is_err());

        // But wrapping cast should work
        let wrapped = val.cast_to_wrapping(&Numeric::UInt32(0));
        assert!(matches!(wrapped, Numeric::UInt32(4_294_967_295)));
    }

    #[test]
    fn test_large_float_to_int() {
        // Float representing value larger than i32::MAX
        let val = Numeric::Double(3e10);
        let result = val.cast_to(&Numeric::Int32(0));
        assert!(result.is_err());

        // But Int64 should work
        let result2 = val.cast_to(&Numeric::Int64(0)).unwrap();
        assert!(matches!(result2, Numeric::Int64(30_000_000_000)));
    }

    #[test]
    fn test_cast_to_bool() {
        assert!(matches!(
            Numeric::Int32(0).cast_to(&Numeric::Bool(false)).unwrap(),
            Numeric::Bool(false)
        ));
        assert!(matches!(
            Numeric::Int32(1).cast_to(&Numeric::Bool(false)).unwrap(),
            Numeric::Bool(true)
        ));
        assert!(matches!(
            Numeric::Float(0.0).cast_to(&Numeric::Bool(false)).unwrap(),
            Numeric::Bool(false)
        ));
        assert!(matches!(
            Numeric::Float(0.1).cast_to(&Numeric::Bool(false)).unwrap(),
            Numeric::Bool(true)
        ));
    }
}
