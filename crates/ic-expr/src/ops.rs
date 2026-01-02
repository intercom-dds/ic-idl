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

use crate::rank::{
    FloatRank, IntRank, common_float_rank, common_int_rank, int_bounds, is_signed, rank_bits,
    rank_mask,
};
use crate::value::Value;

/// Arithmetic operation error.
#[derive(Clone, Debug)]
pub enum ArithError<R> {
    /// Signed overflow occurred; contains the wrapped result.
    SignedOverflow(Value<R>),

    /// Division by zero.
    DivByZero,

    /// Modulo by zero.
    ModByZero,

    /// Shift amount out of range; contains the invalid shift amount.
    ShiftOutOfRange(i128),

    /// Type mismatch (e.g., bitwise op on float).
    TypeMismatch,

    /// Cannot operate on reference directly.
    UnresolvedRef(R),

    /// Value out of range for target type.
    RangeError,

    /// Invalid floating-point value (NaN or infinity) in conversion.
    InvalidFloat,

    /// Invalid unary operator for expression.
    InvalidUnaryOp,

    /// Custom error message (for context-specific errors).
    Custom(String),

    /// Invalid character value (e.g., surrogate for wchar).
    InvalidChar,
}

/// Type tag for common type computation.
#[derive(Clone, Copy, Debug)]
pub enum TyTag {
    Int(IntRank, bool), // rank, signed
    Float(FloatRank),
}

/// Compute the common type for two values.
pub fn common_type<R>(a: &Value<R>, b: &Value<R>) -> Option<TyTag> {
    match (a, b) {
        (Value::Float(_, fa), Value::Float(_, fb)) => {
            Some(TyTag::Float(common_float_rank(*fa, *fb)))
        }
        (Value::Float(_, fr), Value::Int(..) | Value::UInt(..))
        | (Value::Int(..) | Value::UInt(..), Value::Float(_, fr)) => Some(TyTag::Float(*fr)),
        (Value::Int(_, ra), Value::Int(_, rb)) => Some(TyTag::Int(common_int_rank(*ra, *rb), true)),
        (Value::UInt(_, ra), Value::UInt(_, rb)) => {
            Some(TyTag::Int(common_int_rank(*ra, *rb), false))
        }
        (Value::Int(_, ra), Value::UInt(_, rb)) | (Value::UInt(_, rb), Value::Int(_, ra)) => {
            let rank = common_int_rank(*ra, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (Value::Bool(_), Value::Bool(_)) => Some(TyTag::Int(IntRank::I32, true)),
        (Value::Bool(_), Value::Int(_, rb) | Value::UInt(_, rb))
        | (Value::Int(_, rb) | Value::UInt(_, rb), Value::Bool(_)) => {
            let rank = common_int_rank(IntRank::I32, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        _ => None,
    }
}

/// Cast a value to the target type tag with range checking.
///
/// # Errors
///
/// Returns an error if the cast is invalid or the value is out of range.
pub fn cast_to<R: Clone>(v: Value<R>, tag: TyTag) -> Result<Value<R>, ArithError<R>> {
    match (v, tag) {
        (Value::Int(val, _), TyTag::Int(r, sign)) => {
            if sign {
                let (min, max) = int_bounds(r);
                if val < min || val > max {
                    return Err(ArithError::RangeError);
                }
                Ok(Value::Int(val, r))
            } else {
                // For unsigned target, wrap using two's complement
                let mask = rank_mask(r);
                let unsigned_val = (val as u128) & mask;
                Ok(Value::UInt(unsigned_val, r))
            }
        }
        (Value::UInt(val, _), TyTag::Int(r, sign)) => {
            if sign {
                // Converting unsigned to signed, check if it fits
                let max = int_bounds(r).1 as u128;
                if val > max {
                    return Err(ArithError::RangeError);
                }
                Ok(Value::Int(val as i128, r))
            } else {
                // Unsigned to unsigned, apply modular reduction
                let mask = rank_mask(r);
                Ok(Value::UInt(val & mask, r))
            }
        }
        (Value::Float(f, _), TyTag::Int(r, sign)) => {
            if !f.is_finite() {
                return Err(ArithError::InvalidFloat);
            }
            let truncated = f.trunc();
            if truncated < (i128::MIN as f64) || truncated > (i128::MAX as f64) {
                return Err(ArithError::RangeError);
            }
            let i = truncated as i128;
            let (min, max) = int_bounds(r);
            if i < min || i > max {
                return Err(ArithError::RangeError);
            }
            if sign {
                Ok(Value::Int(i, r))
            } else {
                Ok(Value::UInt(i as u128, r))
            }
        }
        (Value::Bool(b), TyTag::Int(r, sign)) => {
            if sign {
                Ok(Value::Int(i128::from(b), r))
            } else {
                Ok(Value::UInt(u128::from(b), r))
            }
        }
        (Value::Char(c), TyTag::Int(r, sign)) => {
            let val = i128::from(c as u32);
            if sign {
                Ok(Value::Int(val, r))
            } else {
                Ok(Value::UInt(val as u128, r))
            }
        }
        (Value::Int(v, _), TyTag::Float(fr)) => Ok(Value::Float(v as f64, fr)),
        (Value::UInt(v, _), TyTag::Float(fr)) => Ok(Value::Float(v as f64, fr)),
        (Value::Float(f, _), TyTag::Float(fr)) => Ok(Value::Float(f, fr)),
        (Value::Bool(b), TyTag::Float(fr)) => Ok(Value::Float(if b { 1.0 } else { 0.0 }, fr)),
        (Value::Ref(r), _) => Err(ArithError::UnresolvedRef(r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

/// Cast a value to an integer type with range checking.
///
/// # Errors
///
/// Returns `RangeError` if the value is out of range for the target type.
pub fn cast_to_int<R: Clone>(
    v: Value<R>,
    rank: IntRank,
    signed: bool,
    check_unsigned_range: bool,
) -> Result<Value<R>, ArithError<R>> {
    let (min, max) = int_bounds(rank);
    if signed {
        match &v {
            Value::Int(val, _) if (*val < min || *val > max) => return Err(ArithError::RangeError),
            Value::UInt(val, _) if (*val > max as u128) => return Err(ArithError::RangeError),
            _ => {}
        }
    } else if check_unsigned_range {
        match &v {
            Value::Int(val, _) if (*val < 0 || *val > max) => return Err(ArithError::RangeError),
            Value::UInt(val, _) if (*val > max as u128) => return Err(ArithError::RangeError),
            _ => {}
        }
    }
    cast_to(v, TyTag::Int(rank, signed))
}

// Helper for signed overflow
fn signed_overflow<R>(v: Value<R>) -> Result<Value<R>, ArithError<R>> {
    Err(ArithError::SignedOverflow(v))
}

/// Mask an unsigned value to the width of the given rank.
#[inline]
fn mask_unsigned(val: u128, r: IntRank) -> u128 {
    val & rank_mask(r)
}

fn handle_signed_overflow<R, F>(
    x: i128,
    y: i128,
    r: IntRank,
    op: F,
    wrapping_op: fn(u128, u128) -> u128,
) -> Result<Value<R>, ArithError<R>>
where
    F: FnOnce(i128, i128) -> Option<i128>,
{
    let (min, max) = int_bounds(r);
    match op(x, y) {
        Some(v) if v >= min && v <= max => Ok(Value::Int(v, r)),
        _ => {
            let mask = rank_mask(r) as i128;
            let unsigned_result = wrapping_op(x as u128, y as u128) & (mask as u128);
            let wrapped = if unsigned_result > (max as u128) {
                (unsigned_result as i128) - (mask + 1)
            } else {
                unsigned_result as i128
            };
            signed_overflow(Value::Int(wrapped, r))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
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

/// Evaluate a binary operation.
///
/// # Errors
///
/// Returns an error for type mismatches, division/modulo by zero, signed
/// overflow, shift out of range, or unresolved references.
pub fn eval_bin<R: Clone>(
    op: BinOp,
    lhs: Value<R>,
    rhs: Value<R>,
) -> Result<Value<R>, ArithError<R>> {
    let Some(tag) = common_type(&lhs, &rhs) else {
        return Err(ArithError::TypeMismatch);
    };
    let l = cast_to(lhs, tag)?;
    let r = cast_to(rhs, tag)?;

    match op {
        BinOp::Add => add(l, r),
        BinOp::Sub => sub(l, r),
        BinOp::Mul => mul(l, r),
        BinOp::Div => div(l, r),
        BinOp::Mod => modulo(l, r),
        BinOp::BitAnd => bit_and(l, r),
        BinOp::BitOr => bit_or(l, r),
        BinOp::Xor => bit_xor(l, r),
        BinOp::Shl => shl(l, r),
        BinOp::Shr => shr(l, r),
    }
}

/// Unary operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    BitNot,
    Plus,
}

/// Evaluate a unary operation.
///
/// # Errors
///
/// Returns an error for type mismatches, signed overflow, or unresolved
/// references.
pub fn eval_unary<R: Clone>(op: UnaryOp, val: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match op {
        UnaryOp::Plus => Ok(val),
        UnaryOp::Neg => neg(val),
        UnaryOp::BitNot => bit_not(val),
    }
}

fn add<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(y, _)) => {
            handle_signed_overflow(x, y, r, i128::checked_add, u128::wrapping_add)
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => {
            Ok(Value::UInt(mask_unsigned(x.wrapping_add(y), r), r))
        }
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x + y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn sub<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(y, _)) => {
            handle_signed_overflow(x, y, r, i128::checked_sub, u128::wrapping_sub)
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => {
            Ok(Value::UInt(mask_unsigned(x.wrapping_sub(y), r), r))
        }
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x - y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn mul<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(y, _)) => {
            handle_signed_overflow(x, y, r, i128::checked_mul, u128::wrapping_mul)
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => {
            Ok(Value::UInt(mask_unsigned(x.wrapping_mul(y), r), r))
        }
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x * y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn div<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(_, _), Value::Int(0, _)) | (Value::UInt(_, _), Value::UInt(0, _)) => {
            Err(ArithError::DivByZero)
        }
        (Value::Int(x, r), Value::Int(y, _)) => {
            let (min, _) = int_bounds(r);
            if y == -1 && x == min {
                signed_overflow(Value::Int(x, r))
            } else {
                Ok(Value::Int(x / y, r))
            }
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x / y, r)),
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x / y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn modulo<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(_, _), Value::Int(0, _)) | (Value::UInt(_, _), Value::UInt(0, _)) => {
            Err(ArithError::ModByZero)
        }
        (Value::Int(x, r), Value::Int(y, _)) => {
            if y == -1 {
                Ok(Value::Int(0, r))
            } else {
                Ok(Value::Int(x % y, r))
            }
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x % y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn bit_and<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(y, _)) => Ok(Value::Int(x & y, r)),
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x & y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn bit_or<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(y, _)) => Ok(Value::Int(x | y, r)),
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x | y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn bit_xor<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(y, _)) => Ok(Value::Int(x ^ y, r)),
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x ^ y, r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn validate_shift<R>(shift: i128, _signed: bool, rank: IntRank) -> Result<u32, ArithError<R>> {
    if shift < 0 {
        return Err(ArithError::ShiftOutOfRange(shift));
    }
    let bits = i128::from(rank_bits(rank));
    if shift >= bits {
        return Err(ArithError::ShiftOutOfRange(shift));
    }
    Ok(shift as u32)
}

fn shl<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(shift, _)) => {
            let amount = validate_shift(shift, true, r)?;
            Ok(Value::Int(x.wrapping_shl(amount), r))
        }
        (Value::UInt(x, r), Value::Int(shift, _)) => {
            let amount = validate_shift(shift, true, r)?;
            Ok(Value::UInt(mask_unsigned(x.wrapping_shl(amount), r), r))
        }
        (Value::UInt(x, r), Value::UInt(shift, _)) => {
            let amount = validate_shift(shift as i128, false, r)?;
            Ok(Value::UInt(mask_unsigned(x.wrapping_shl(amount), r), r))
        }
        _ => Err(ArithError::TypeMismatch),
    }
}

fn shr<R>(lhs: Value<R>, rhs: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match (lhs, rhs) {
        (Value::Int(x, r), Value::Int(shift, _)) => {
            let amount = validate_shift(shift, true, r)?;
            Ok(Value::Int(x >> amount, r))
        }
        (Value::UInt(x, r), Value::Int(shift, _)) => {
            let amount = validate_shift(shift, true, r)?;
            let masked = mask_unsigned(x, r);
            Ok(Value::UInt(masked >> amount, r))
        }
        (Value::UInt(x, r), Value::UInt(shift, _)) => {
            let amount = validate_shift(shift as i128, false, r)?;
            let masked = mask_unsigned(x, r);
            Ok(Value::UInt(masked >> amount, r))
        }
        _ => Err(ArithError::TypeMismatch),
    }
}

fn neg<R: Clone>(val: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match val {
        Value::Int(i, r) => match i.checked_neg() {
            Some(v) => Ok(Value::Int(v, r)),
            None => signed_overflow(Value::Int(i.wrapping_neg(), r)),
        },
        Value::UInt(u, r) => {
            let i = u as i128;
            let neg = i.wrapping_neg();
            let mask = rank_mask(r);
            let unsigned_val = (neg as u128) & mask;
            Ok(Value::UInt(unsigned_val, r))
        }
        Value::Float(f, r) => Ok(Value::Float(-f, r)),
        Value::Ref(r) => Err(ArithError::UnresolvedRef(r)),
        _ => Err(ArithError::TypeMismatch),
    }
}

fn bit_not<R: Clone>(val: Value<R>) -> Result<Value<R>, ArithError<R>> {
    match val {
        Value::Int(i, r) => {
            let mask = rank_mask(r);
            let bits = rank_bits(r);
            let masked = (!i as u128) & mask;
            let sign_bit = 1u128 << (bits - 1);
            let result = if masked & sign_bit != 0 {
                (masked | !mask) as i128
            } else {
                masked as i128
            };
            Ok(Value::Int(result, r))
        }
        Value::UInt(u, r) => Ok(Value::UInt(mask_unsigned(!u, r), r)),
        Value::Ref(r) => Err(ArithError::UnresolvedRef(r)),
        _ => Err(ArithError::TypeMismatch),
    }
}
