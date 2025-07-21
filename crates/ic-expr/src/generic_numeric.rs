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

use std::cmp::Ordering;

use crate::{Error, EvalConfig, NumericValue, OverflowBehavior, Result};

/// Generic numeric type for expression evaluation.
/// This provides a reusable implementation of numeric operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenericNumeric {
    Bool(bool),
    Char(char),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float(f32),
    Double(f64),
}

impl GenericNumeric {
    /// Returns the type rank for arithmetic promotion.
    /// Higher rank types are promoted to from lower rank types.
    fn type_rank(&self) -> u8 {
        match self {
            Self::Bool(_) => 0,
            Self::Char(_) => 1,
            Self::Int8(_) => 2,
            Self::UInt8(_) => 3,
            Self::Int16(_) => 4,
            Self::UInt16(_) => 5,
            Self::Int32(_) => 6,
            Self::UInt32(_) => 7,
            Self::Int64(_) => 8,
            Self::UInt64(_) => 9,
            Self::Float(_) => 10,
            Self::Double(_) => 11,
        }
    }

    /// Promotes this value to the specified type.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn promote_to(&self, target: &Self) -> Self {
        match (self, target) {
            // Already the same type
            (Self::Bool(_), Self::Bool(_))
            | (Self::Char(_), Self::Char(_))
            | (Self::Int8(_), Self::Int8(_))
            | (Self::UInt8(_), Self::UInt8(_))
            | (Self::Int16(_), Self::Int16(_))
            | (Self::UInt16(_), Self::UInt16(_))
            | (Self::Int32(_), Self::Int32(_))
            | (Self::UInt32(_), Self::UInt32(_))
            | (Self::Int64(_), Self::Int64(_))
            | (Self::UInt64(_), Self::UInt64(_))
            | (Self::Float(_), Self::Float(_))
            | (Self::Double(_), Self::Double(_)) => *self,

            // Promote to Double
            (_, Self::Double(_)) => match self {
                Self::Bool(v) => Self::Double(if *v { 1.0 } else { 0.0 }),
                Self::Char(v) => Self::Double(f64::from(*v as u8)),
                Self::Int8(v) => Self::Double(f64::from(*v)),
                Self::UInt8(v) => Self::Double(f64::from(*v)),
                Self::Int16(v) => Self::Double(f64::from(*v)),
                Self::UInt16(v) => Self::Double(f64::from(*v)),
                Self::Int32(v) => Self::Double(f64::from(*v)),
                Self::UInt32(v) => Self::Double(f64::from(*v)),
                Self::Int64(v) => Self::Double(*v as f64),
                Self::UInt64(v) => Self::Double(*v as f64),
                Self::Float(v) => Self::Double(f64::from(*v)),
                Self::Double(v) => Self::Double(*v),
            },

            // Promote to Float
            (_, Self::Float(_)) => match self {
                Self::Bool(v) => Self::Float(if *v { 1.0 } else { 0.0 }),
                Self::Char(v) => Self::Float(f32::from(*v as u8)),
                Self::Int8(v) => Self::Float(f32::from(*v)),
                Self::UInt8(v) => Self::Float(f32::from(*v)),
                Self::Int16(v) => Self::Float(f32::from(*v)),
                Self::UInt16(v) => Self::Float(f32::from(*v)),
                Self::Int32(v) => Self::Float(*v as f32),
                Self::UInt32(v) => Self::Float(*v as f32),
                Self::Int64(v) => Self::Float(*v as f32),
                Self::UInt64(v) => Self::Float(*v as f32),
                Self::Float(v) => Self::Float(*v),
                Self::Double(v) => Self::Float(*v as f32), // This would lose precision
            },

            // Integer promotions
            (_, Self::UInt64(_)) => match self {
                Self::Bool(v) => Self::UInt64(u64::from(*v)),
                Self::Char(v) => Self::UInt64(u64::from(*v as u8)),
                Self::Int8(v) => Self::UInt64(*v as u64),
                Self::UInt8(v) => Self::UInt64(u64::from(*v)),
                Self::Int16(v) => Self::UInt64(*v as u64),
                Self::UInt16(v) => Self::UInt64(u64::from(*v)),
                Self::Int32(v) => Self::UInt64(*v as u64),
                Self::UInt32(v) => Self::UInt64(u64::from(*v)),
                Self::Int64(v) => Self::UInt64(*v as u64),
                Self::UInt64(v) => Self::UInt64(*v),
                _ => unreachable!("float to int conversion"),
            },

            (_, Self::Int64(_)) => match self {
                Self::Bool(v) => Self::Int64(i64::from(*v)),
                Self::Char(v) => Self::Int64(i64::from(*v as u8)),
                Self::Int8(v) => Self::Int64(i64::from(*v)),
                Self::UInt8(v) => Self::Int64(i64::from(*v)),
                Self::Int16(v) => Self::Int64(i64::from(*v)),
                Self::UInt16(v) => Self::Int64(i64::from(*v)),
                Self::Int32(v) => Self::Int64(i64::from(*v)),
                Self::UInt32(v) => Self::Int64(i64::from(*v)),
                Self::Int64(v) => Self::Int64(*v),
                _ => unreachable!("float/uint64 to int64 conversion"),
            },

            (_, Self::UInt32(_)) => match self {
                Self::Bool(v) => Self::UInt32(u32::from(*v)),
                Self::Char(v) => Self::UInt32(u32::from(*v as u8)),
                Self::Int8(v) => Self::UInt32(*v as u32),
                Self::UInt8(v) => Self::UInt32(u32::from(*v)),
                Self::Int16(v) => Self::UInt32(*v as u32),
                Self::UInt16(v) => Self::UInt32(u32::from(*v)),
                Self::Int32(v) => Self::UInt32(*v as u32),
                Self::UInt32(v) => Self::UInt32(*v),
                _ => unreachable!("larger type to uint32 conversion"),
            },

            (_, Self::Int32(_)) => match self {
                Self::Bool(v) => Self::Int32(i32::from(*v)),
                Self::Char(v) => Self::Int32(i32::from(*v as u8)),
                Self::Int8(v) => Self::Int32(i32::from(*v)),
                Self::UInt8(v) => Self::Int32(i32::from(*v)),
                Self::Int16(v) => Self::Int32(i32::from(*v)),
                Self::UInt16(v) => Self::Int32(i32::from(*v)),
                Self::Int32(v) => Self::Int32(*v),
                _ => unreachable!("larger type to int32 conversion"),
            },

            (_, Self::UInt16(_)) => match self {
                Self::Bool(v) => Self::UInt16(u16::from(*v)),
                Self::Char(v) => Self::UInt16(u16::from(*v as u8)),
                Self::Int8(v) => Self::UInt16(*v as u16),
                Self::UInt8(v) => Self::UInt16(u16::from(*v)),
                Self::Int16(v) => Self::UInt16(*v as u16),
                Self::UInt16(v) => Self::UInt16(*v),
                _ => unreachable!("larger type to uint16 conversion"),
            },

            (_, Self::Int16(_)) => match self {
                Self::Bool(v) => Self::Int16(i16::from(*v)),
                Self::Char(v) => Self::Int16(i16::from(*v as u8)),
                Self::Int8(v) => Self::Int16(i16::from(*v)),
                Self::UInt8(v) => Self::Int16(i16::from(*v)),
                Self::Int16(v) => Self::Int16(*v),
                _ => unreachable!("larger type to int16 conversion"),
            },

            (_, Self::UInt8(_)) => match self {
                Self::Bool(v) => Self::UInt8(u8::from(*v)),
                Self::Char(v) => Self::UInt8(*v as u8),
                Self::Int8(v) => Self::UInt8(*v as u8),
                Self::UInt8(v) => Self::UInt8(*v),
                _ => unreachable!("larger type to uint8 conversion"),
            },

            (_, Self::Int8(_)) => match self {
                Self::Bool(v) => Self::Int8(i8::from(*v)),
                Self::Char(v) => Self::Int8(*v as i8),
                Self::Int8(v) => Self::Int8(*v),
                _ => unreachable!("larger type to int8 conversion"),
            },

            (_, Self::Char(_)) => match self {
                Self::Bool(v) => Self::Char(if *v { '\u{0001}' } else { '\0' }),
                Self::Char(v) => Self::Char(*v),
                _ => unreachable!("non-bool/char to char conversion"),
            },

            _ => *self, // For other conversions, keep original
        }
    }

    /// Determines the common type for arithmetic operations and promotes both values.
    fn promote_for_arithmetic(a: Self, b: Self) -> (Self, Self) {
        // If either is floating point, use floating point rules
        if matches!(a, Self::Float(_) | Self::Double(_))
            || matches!(b, Self::Float(_) | Self::Double(_))
        {
            // If either is double, both become double
            if matches!(a, Self::Double(_)) || matches!(b, Self::Double(_)) {
                let target = Self::Double(0.0);
                (a.promote_to(&target), b.promote_to(&target))
            } else {
                // Otherwise both become float
                let target = Self::Float(0.0);
                (a.promote_to(&target), b.promote_to(&target))
            }
        } else {
            // Integer promotion rules - promote to the higher ranked type
            match a.type_rank().cmp(&b.type_rank()) {
                Ordering::Less => (a.promote_to(&b), b),
                Ordering::Greater => (a, b.promote_to(&a)),
                Ordering::Equal => (a, b),
            }
        }
    }
}

impl NumericValue for GenericNumeric {
    fn from_bool(b: bool) -> Self {
        Self::Bool(b)
    }

    fn to_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Char(v) => *v != '\0',
            Self::Int8(v) => *v != 0,
            Self::UInt8(v) => *v != 0,
            Self::Int16(v) => *v != 0,
            Self::UInt16(v) => *v != 0,
            Self::Int32(v) => *v != 0,
            Self::UInt32(v) => *v != 0,
            Self::Int64(v) => *v != 0,
            Self::UInt64(v) => *v != 0,
            Self::Float(v) => *v != 0.0,
            Self::Double(v) => *v != 0.0,
        }
    }

    fn negate(&self, config: EvalConfig) -> Result<Self> {
        match self {
            Self::Bool(v) => Ok(Self::Int32(-i32::from(*v))),
            Self::Char(v) => Ok(Self::Int32(-(*v as i32))),
            Self::Int8(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int8(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int8)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int8(v.saturating_neg())),
            },
            Self::UInt8(v) => Ok(Self::Int16(-i16::from(*v))),
            Self::Int16(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int16(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int16)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int16(v.saturating_neg())),
            },
            Self::UInt16(v) => Ok(Self::Int32(-i32::from(*v))),
            Self::Int32(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int32(v.saturating_neg())),
            },
            Self::UInt32(v) => Ok(Self::Int64(-i64::from(*v))),
            Self::Int64(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(v.wrapping_neg())),
                OverflowBehavior::Error => v
                    .checked_neg()
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("negation")),
                OverflowBehavior::Saturate => Ok(Self::Int64(v.saturating_neg())),
            },
            Self::UInt64(v) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(v.wrapping_neg())),
                OverflowBehavior::Error => Err(Error::Overflow("negation of unsigned")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(0)),
            },
            Self::Float(v) => Ok(Self::Float(-v)),
            Self::Double(v) => Ok(Self::Double(-v)),
        }
    }

    fn bit_not(&self) -> Self {
        match self {
            Self::Bool(v) => Self::Bool(!v),
            Self::Char(v) => Self::Char(char::from_u32(!(*v as u32)).unwrap_or('\0')),
            Self::Int8(v) => Self::Int8(!v),
            Self::UInt8(v) => Self::UInt8(!v),
            Self::Int16(v) => Self::Int16(!v),
            Self::UInt16(v) => Self::UInt16(!v),
            Self::Int32(v) => Self::Int32(!v),
            Self::UInt32(v) => Self::UInt32(!v),
            Self::Int64(v) => Self::Int64(!v),
            Self::UInt64(v) => Self::UInt64(!v),
            Self::Float(_) | Self::Double(_) => *self,
        }
    }

    fn add(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        // Promote operands to common type
        let (lhs, rhs) = Self::promote_for_arithmetic(*self, *rhs);

        match (lhs, rhs) {
            (Self::Char(a), Self::Char(b)) => {
                let result = (a as u32).wrapping_add(b as u32);
                Ok(Self::Char(char::from_u32(result).unwrap_or('\0')))
            }
            (Self::Int8(a), Self::Int8(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int8(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::Int8)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::Int8(a.saturating_add(b))),
            },
            (Self::UInt8(a), Self::UInt8(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt8(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::UInt8)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::UInt8(a.saturating_add(b))),
            },
            (Self::Int16(a), Self::Int16(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int16(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::Int16)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::Int16(a.saturating_add(b))),
            },
            (Self::UInt16(a), Self::UInt16(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt16(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::UInt16)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::UInt16(a.saturating_add(b))),
            },
            (Self::Int32(a), Self::Int32(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_add(b))),
            },
            (Self::UInt32(a), Self::UInt32(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt32(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::UInt32)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::UInt32(a.saturating_add(b))),
            },
            (Self::Int64(a), Self::Int64(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_add(b))),
            },
            (Self::UInt64(a), Self::UInt64(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(a.wrapping_add(b))),
                OverflowBehavior::Error => a
                    .checked_add(b)
                    .map(Self::UInt64)
                    .ok_or(Error::Overflow("addition")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(a.saturating_add(b))),
            },
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a + b)),
            (Self::Double(a), Self::Double(b)) => Ok(Self::Double(a + b)),
            _ => Err(Error::Custom("type mismatch in addition".to_string())),
        }
    }

    fn sub(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        // Promote operands to common type
        let (lhs, rhs) = Self::promote_for_arithmetic(*self, *rhs);

        match (lhs, rhs) {
            (Self::Char(a), Self::Char(b)) => {
                let result = (a as u32).wrapping_sub(b as u32);
                Ok(Self::Char(char::from_u32(result).unwrap_or('\0')))
            }
            (Self::Int8(a), Self::Int8(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int8(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::Int8)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::Int8(a.saturating_sub(b))),
            },
            (Self::UInt8(a), Self::UInt8(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt8(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::UInt8)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::UInt8(a.saturating_sub(b))),
            },
            (Self::Int16(a), Self::Int16(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int16(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::Int16)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::Int16(a.saturating_sub(b))),
            },
            (Self::UInt16(a), Self::UInt16(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt16(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::UInt16)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::UInt16(a.saturating_sub(b))),
            },
            (Self::Int32(a), Self::Int32(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_sub(b))),
            },
            (Self::UInt32(a), Self::UInt32(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt32(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::UInt32)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::UInt32(a.saturating_sub(b))),
            },
            (Self::Int64(a), Self::Int64(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_sub(b))),
            },
            (Self::UInt64(a), Self::UInt64(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(a.wrapping_sub(b))),
                OverflowBehavior::Error => a
                    .checked_sub(b)
                    .map(Self::UInt64)
                    .ok_or(Error::Overflow("subtraction")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(a.saturating_sub(b))),
            },
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a - b)),
            (Self::Double(a), Self::Double(b)) => Ok(Self::Double(a - b)),
            _ => Err(Error::Custom("type mismatch in subtraction".to_string())),
        }
    }

    fn mul(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        // Promote operands to common type
        let (lhs, rhs) = Self::promote_for_arithmetic(*self, *rhs);

        match (lhs, rhs) {
            (Self::Char(a), Self::Char(b)) => {
                let result = (a as u32).wrapping_mul(b as u32);
                Ok(Self::Char(char::from_u32(result).unwrap_or('\0')))
            }
            (Self::Int8(a), Self::Int8(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int8(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::Int8)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::Int8(a.saturating_mul(b))),
            },
            (Self::UInt8(a), Self::UInt8(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt8(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::UInt8)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::UInt8(a.saturating_mul(b))),
            },
            (Self::Int16(a), Self::Int16(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int16(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::Int16)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::Int16(a.saturating_mul(b))),
            },
            (Self::UInt16(a), Self::UInt16(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt16(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::UInt16)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::UInt16(a.saturating_mul(b))),
            },
            (Self::Int32(a), Self::Int32(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::Int32)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_mul(b))),
            },
            (Self::UInt32(a), Self::UInt32(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt32(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::UInt32)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::UInt32(a.saturating_mul(b))),
            },
            (Self::Int64(a), Self::Int64(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::Int64)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_mul(b))),
            },
            (Self::UInt64(a), Self::UInt64(b)) => match config.overflow {
                OverflowBehavior::Wrap => Ok(Self::UInt64(a.wrapping_mul(b))),
                OverflowBehavior::Error => a
                    .checked_mul(b)
                    .map(Self::UInt64)
                    .ok_or(Error::Overflow("multiplication")),
                OverflowBehavior::Saturate => Ok(Self::UInt64(a.saturating_mul(b))),
            },
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a * b)),
            (Self::Double(a), Self::Double(b)) => Ok(Self::Double(a * b)),
            _ => unreachable!("promotion should ensure matching types"),
        }
    }

    fn div(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        // Promote operands to common type
        let (lhs, rhs) = Self::promote_for_arithmetic(*self, *rhs);

        match (lhs, rhs) {
            (Self::Char(a), Self::Char(b)) => {
                if b == '\0' {
                    return Err(Error::DivisionByZero);
                }
                let result = (a as u32) / (b as u32);
                Ok(Self::Char(char::from_u32(result).unwrap_or('\0')))
            }
            (Self::Int8(a), Self::Int8(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int8(a.wrapping_div(b))),
                    OverflowBehavior::Error => a
                        .checked_div(b)
                        .map(Self::Int8)
                        .ok_or(Error::Overflow("division")),
                    OverflowBehavior::Saturate => Ok(Self::Int8(a.saturating_div(b))),
                }
            }
            (Self::UInt8(a), Self::UInt8(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                Ok(Self::UInt8(a / b))
            }
            (Self::Int16(a), Self::Int16(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int16(a.wrapping_div(b))),
                    OverflowBehavior::Error => a
                        .checked_div(b)
                        .map(Self::Int16)
                        .ok_or(Error::Overflow("division")),
                    OverflowBehavior::Saturate => Ok(Self::Int16(a.saturating_div(b))),
                }
            }
            (Self::UInt16(a), Self::UInt16(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                Ok(Self::UInt16(a / b))
            }
            (Self::Int32(a), Self::Int32(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_div(b))),
                    OverflowBehavior::Error => a
                        .checked_div(b)
                        .map(Self::Int32)
                        .ok_or(Error::Overflow("division")),
                    OverflowBehavior::Saturate => Ok(Self::Int32(a.saturating_div(b))),
                }
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
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_div(b))),
                    OverflowBehavior::Error => a
                        .checked_div(b)
                        .map(Self::Int64)
                        .ok_or(Error::Overflow("division")),
                    OverflowBehavior::Saturate => Ok(Self::Int64(a.saturating_div(b))),
                }
            }
            (Self::UInt64(a), Self::UInt64(b)) => {
                if b == 0 {
                    return Err(Error::DivisionByZero);
                }
                Ok(Self::UInt64(a / b))
            }
            (Self::Float(a), Self::Float(b)) => {
                // IEEE 754: division by zero produces infinity
                Ok(Self::Float(a / b))
            }
            (Self::Double(a), Self::Double(b)) => {
                // IEEE 754: division by zero produces infinity
                Ok(Self::Double(a / b))
            }
            _ => Err(Error::Custom("type mismatch in division".to_string())),
        }
    }

    fn modulo(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        // Promote operands to common type
        let (lhs, rhs) = Self::promote_for_arithmetic(*self, *rhs);

        match (lhs, rhs) {
            (Self::Char(a), Self::Char(b)) => {
                if b == '\0' {
                    return Err(Error::ModuloByZero);
                }
                let result = (a as u32) % (b as u32);
                Ok(Self::Char(char::from_u32(result).unwrap_or('\0')))
            }
            (Self::Int8(a), Self::Int8(b)) => {
                if b == 0 {
                    return Err(Error::ModuloByZero);
                }
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int8(a.wrapping_rem(b))),
                    OverflowBehavior::Error => a
                        .checked_rem(b)
                        .map(Self::Int8)
                        .ok_or(Error::Overflow("modulo")),
                    OverflowBehavior::Saturate => Ok(Self::Int8(a.wrapping_rem(b))),
                }
            }
            (Self::UInt8(a), Self::UInt8(b)) => {
                if b == 0 {
                    return Err(Error::ModuloByZero);
                }
                Ok(Self::UInt8(a % b))
            }
            (Self::Int16(a), Self::Int16(b)) => {
                if b == 0 {
                    return Err(Error::ModuloByZero);
                }
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int16(a.wrapping_rem(b))),
                    OverflowBehavior::Error => a
                        .checked_rem(b)
                        .map(Self::Int16)
                        .ok_or(Error::Overflow("modulo")),
                    OverflowBehavior::Saturate => Ok(Self::Int16(a.wrapping_rem(b))),
                }
            }
            (Self::UInt16(a), Self::UInt16(b)) => {
                if b == 0 {
                    return Err(Error::ModuloByZero);
                }
                Ok(Self::UInt16(a % b))
            }
            (Self::Int32(a), Self::Int32(b)) => {
                if b == 0 {
                    return Err(Error::ModuloByZero);
                }
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int32(a.wrapping_rem(b))),
                    OverflowBehavior::Error => a
                        .checked_rem(b)
                        .map(Self::Int32)
                        .ok_or(Error::Overflow("modulo")),
                    OverflowBehavior::Saturate => Ok(Self::Int32(a.wrapping_rem(b))),
                }
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
                match config.overflow {
                    OverflowBehavior::Wrap => Ok(Self::Int64(a.wrapping_rem(b))),
                    OverflowBehavior::Error => a
                        .checked_rem(b)
                        .map(Self::Int64)
                        .ok_or(Error::Overflow("modulo")),
                    OverflowBehavior::Saturate => Ok(Self::Int64(a.wrapping_rem(b))),
                }
            }
            (Self::UInt64(a), Self::UInt64(b)) => {
                if b == 0 {
                    return Err(Error::ModuloByZero);
                }
                Ok(Self::UInt64(a % b))
            }
            (Self::Float(a), Self::Float(b)) => {
                // IEEE 754: modulo by zero produces NaN
                Ok(Self::Float(a % b))
            }
            (Self::Double(a), Self::Double(b)) => {
                // IEEE 754: modulo by zero produces NaN
                Ok(Self::Double(a % b))
            }
            _ => Err(Error::Custom("type mismatch in modulo".to_string())),
        }
    }

    fn bit_and(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => Self::Bool(a & b),
            (Self::Char(a), Self::Char(b)) => {
                Self::Char(char::from_u32((*a as u32) & (*b as u32)).unwrap_or('\0'))
            }
            (Self::Int8(a), Self::Int8(b)) => Self::Int8(a & b),
            (Self::UInt8(a), Self::UInt8(b)) => Self::UInt8(a & b),
            (Self::Int16(a), Self::Int16(b)) => Self::Int16(a & b),
            (Self::UInt16(a), Self::UInt16(b)) => Self::UInt16(a & b),
            (Self::Int32(a), Self::Int32(b)) => Self::Int32(a & b),
            (Self::UInt32(a), Self::UInt32(b)) => Self::UInt32(a & b),
            (Self::Int64(a), Self::Int64(b)) => Self::Int64(a & b),
            (Self::UInt64(a), Self::UInt64(b)) => Self::UInt64(a & b),
            _ => *self,
        }
    }

    fn bit_or(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => Self::Bool(a | b),
            (Self::Char(a), Self::Char(b)) => {
                Self::Char(char::from_u32((*a as u32) | (*b as u32)).unwrap_or('\0'))
            }
            (Self::Int8(a), Self::Int8(b)) => Self::Int8(a | b),
            (Self::UInt8(a), Self::UInt8(b)) => Self::UInt8(a | b),
            (Self::Int16(a), Self::Int16(b)) => Self::Int16(a | b),
            (Self::UInt16(a), Self::UInt16(b)) => Self::UInt16(a | b),
            (Self::Int32(a), Self::Int32(b)) => Self::Int32(a | b),
            (Self::UInt32(a), Self::UInt32(b)) => Self::UInt32(a | b),
            (Self::Int64(a), Self::Int64(b)) => Self::Int64(a | b),
            (Self::UInt64(a), Self::UInt64(b)) => Self::UInt64(a | b),
            _ => *self,
        }
    }

    fn bit_xor(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => Self::Bool(a ^ b),
            (Self::Char(a), Self::Char(b)) => {
                Self::Char(char::from_u32((*a as u32) ^ (*b as u32)).unwrap_or('\0'))
            }
            (Self::Int8(a), Self::Int8(b)) => Self::Int8(a ^ b),
            (Self::UInt8(a), Self::UInt8(b)) => Self::UInt8(a ^ b),
            (Self::Int16(a), Self::Int16(b)) => Self::Int16(a ^ b),
            (Self::UInt16(a), Self::UInt16(b)) => Self::UInt16(a ^ b),
            (Self::Int32(a), Self::Int32(b)) => Self::Int32(a ^ b),
            (Self::UInt32(a), Self::UInt32(b)) => Self::UInt32(a ^ b),
            (Self::Int64(a), Self::Int64(b)) => Self::Int64(a ^ b),
            (Self::UInt64(a), Self::UInt64(b)) => Self::UInt64(a ^ b),
            _ => *self,
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn shl(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        let shift = match rhs {
            Self::Int8(v) => i128::from(*v),
            Self::UInt8(v) => i128::from(*v),
            Self::Int16(v) => i128::from(*v),
            Self::UInt16(v) => i128::from(*v),
            Self::Int32(v) => i128::from(*v),
            Self::UInt32(v) => i128::from(*v),
            Self::Int64(v) => i128::from(*v),
            Self::UInt64(v) => i128::from(*v),
            _ => return Err(Error::Custom("invalid shift amount type".to_string())),
        };

        if shift < 0 || shift > i128::from(config.max_shift) {
            return Err(Error::InvalidShift(shift));
        }

        let shift = shift as u32;
        match self {
            Self::Int8(v) => Ok(Self::Int8(v.wrapping_shl(shift))),
            Self::UInt8(v) => Ok(Self::UInt8(v.wrapping_shl(shift))),
            Self::Int16(v) => Ok(Self::Int16(v.wrapping_shl(shift))),
            Self::UInt16(v) => Ok(Self::UInt16(v.wrapping_shl(shift))),
            Self::Int32(v) => Ok(Self::Int32(v.wrapping_shl(shift))),
            Self::UInt32(v) => Ok(Self::UInt32(v.wrapping_shl(shift))),
            Self::Int64(v) => Ok(Self::Int64(v.wrapping_shl(shift))),
            Self::UInt64(v) => Ok(Self::UInt64(v.wrapping_shl(shift))),
            _ => Ok(*self),
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn shr(&self, rhs: &Self, config: EvalConfig) -> Result<Self> {
        let shift = match rhs {
            Self::Int8(v) => i128::from(*v),
            Self::UInt8(v) => i128::from(*v),
            Self::Int16(v) => i128::from(*v),
            Self::UInt16(v) => i128::from(*v),
            Self::Int32(v) => i128::from(*v),
            Self::UInt32(v) => i128::from(*v),
            Self::Int64(v) => i128::from(*v),
            Self::UInt64(v) => i128::from(*v),
            _ => return Err(Error::Custom("invalid shift amount type".to_string())),
        };

        if shift < 0 || shift > i128::from(config.max_shift) {
            return Err(Error::InvalidShift(shift));
        }

        let shift = shift as u32;
        match self {
            Self::Int8(v) => Ok(Self::Int8(v.wrapping_shr(shift))),
            Self::UInt8(v) => Ok(Self::UInt8(v.wrapping_shr(shift))),
            Self::Int16(v) => Ok(Self::Int16(v.wrapping_shr(shift))),
            Self::UInt16(v) => Ok(Self::UInt16(v.wrapping_shr(shift))),
            Self::Int32(v) => Ok(Self::Int32(v.wrapping_shr(shift))),
            Self::UInt32(v) => Ok(Self::UInt32(v.wrapping_shr(shift))),
            Self::Int64(v) => Ok(Self::Int64(v.wrapping_shr(shift))),
            Self::UInt64(v) => Ok(Self::UInt64(v.wrapping_shr(shift))),
            _ => Ok(*self),
        }
    }

    fn lt(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => (!a) & b,
            (Self::Char(a), Self::Char(b)) => a < b,
            (Self::Int8(a), Self::Int8(b)) => a < b,
            (Self::UInt8(a), Self::UInt8(b)) => a < b,
            (Self::Int16(a), Self::Int16(b)) => a < b,
            (Self::UInt16(a), Self::UInt16(b)) => a < b,
            (Self::Int32(a), Self::Int32(b)) => a < b,
            (Self::UInt32(a), Self::UInt32(b)) => a < b,
            (Self::Int64(a), Self::Int64(b)) => a < b,
            (Self::UInt64(a), Self::UInt64(b)) => a < b,
            (Self::Float(a), Self::Float(b)) => a < b,
            (Self::Double(a), Self::Double(b)) => a < b,
            _ => false,
        }
    }

    fn le(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => (!a) | (a == b),
            (Self::Char(a), Self::Char(b)) => a <= b,
            (Self::Int8(a), Self::Int8(b)) => a <= b,
            (Self::UInt8(a), Self::UInt8(b)) => a <= b,
            (Self::Int16(a), Self::Int16(b)) => a <= b,
            (Self::UInt16(a), Self::UInt16(b)) => a <= b,
            (Self::Int32(a), Self::Int32(b)) => a <= b,
            (Self::UInt32(a), Self::UInt32(b)) => a <= b,
            (Self::Int64(a), Self::Int64(b)) => a <= b,
            (Self::UInt64(a), Self::UInt64(b)) => a <= b,
            (Self::Float(a), Self::Float(b)) => a <= b,
            (Self::Double(a), Self::Double(b)) => a <= b,
            _ => false,
        }
    }

    fn gt(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => *a && (!b),
            (Self::Char(a), Self::Char(b)) => a > b,
            (Self::Int8(a), Self::Int8(b)) => a > b,
            (Self::UInt8(a), Self::UInt8(b)) => a > b,
            (Self::Int16(a), Self::Int16(b)) => a > b,
            (Self::UInt16(a), Self::UInt16(b)) => a > b,
            (Self::Int32(a), Self::Int32(b)) => a > b,
            (Self::UInt32(a), Self::UInt32(b)) => a > b,
            (Self::Int64(a), Self::Int64(b)) => a > b,
            (Self::UInt64(a), Self::UInt64(b)) => a > b,
            (Self::Float(a), Self::Float(b)) => a > b,
            (Self::Double(a), Self::Double(b)) => a > b,
            _ => false,
        }
    }

    fn ge(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => *a | (a == b),
            (Self::Char(a), Self::Char(b)) => a >= b,
            (Self::Int8(a), Self::Int8(b)) => a >= b,
            (Self::UInt8(a), Self::UInt8(b)) => a >= b,
            (Self::Int16(a), Self::Int16(b)) => a >= b,
            (Self::UInt16(a), Self::UInt16(b)) => a >= b,
            (Self::Int32(a), Self::Int32(b)) => a >= b,
            (Self::UInt32(a), Self::UInt32(b)) => a >= b,
            (Self::Int64(a), Self::Int64(b)) => a >= b,
            (Self::UInt64(a), Self::UInt64(b)) => a >= b,
            (Self::Float(a), Self::Float(b)) => a >= b,
            (Self::Double(a), Self::Double(b)) => a >= b,
            _ => false,
        }
    }

    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::Int8(a), Self::Int8(b)) => a == b,
            (Self::UInt8(a), Self::UInt8(b)) => a == b,
            (Self::Int16(a), Self::Int16(b)) => a == b,
            (Self::UInt16(a), Self::UInt16(b)) => a == b,
            (Self::Int32(a), Self::Int32(b)) => a == b,
            (Self::UInt32(a), Self::UInt32(b)) => a == b,
            (Self::Int64(a), Self::Int64(b)) => a == b,
            (Self::UInt64(a), Self::UInt64(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Double(a), Self::Double(b)) => a == b,
            _ => false,
        }
    }

    fn ne(&self, rhs: &Self) -> bool {
        !NumericValue::eq(self, rhs)
    }
}
