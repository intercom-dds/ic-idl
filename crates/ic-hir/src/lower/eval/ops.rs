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

//! Arithmetic, bitwise, and logical operator implementations.

use super::cast::cast_to;
use super::rank::{IntRank, TyTag, common_type, int_min_max, rank_bits};
use super::{EvalError, Op, Value};

// Helper to return a SignedOverflow while carrying a wrapped result
fn signed_overflow(v: Value) -> Result<Value, EvalError> {
    Err(EvalError::SignedOverflow(v))
}

/// Helper function to handle signed integer overflow with proper wrapping
fn handle_signed_overflow<F>(
    x: i128,
    y: i128,
    r: IntRank,
    op: F,
    wrapping_op: fn(u128, u128) -> u128,
) -> Result<Value, EvalError>
where
    F: FnOnce(i128, i128) -> Option<i128>,
{
    let (min, max) = int_min_max(r);
    match op(x, y) {
        Some(v) if v >= min && v <= max => Ok(Value::Int(v, r)),
        _ => {
            // Overflow occurred, wrap according to the rank's bit width
            let bits = rank_bits(r);
            let mask = if bits == 64 {
                u64::MAX as i128
            } else {
                (1i128 << bits) - 1
            };
            let unsigned_result = wrapping_op(x as u128, y as u128) & (mask as u128);
            let wrapped = if unsigned_result > (max as u128) {
                // Wrapped to negative
                (unsigned_result as i128) - ((mask + 1) as i128)
            } else {
                unsigned_result as i128
            };
            signed_overflow(Value::Int(wrapped, r))
        }
    }
}

// Per-class operation implementations after casting to common type
fn add_int(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(y, _)) => {
            handle_signed_overflow(x, y, rank, i128::checked_add, u128::wrapping_add)
        }
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x.wrapping_add(y), rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn add_float(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Float(x, rank), Value::Float(y, _)) => Ok(Value::Float(x + y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn sub_int(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(y, _)) => {
            handle_signed_overflow(x, y, rank, i128::checked_sub, u128::wrapping_sub)
        }
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x.wrapping_sub(y), rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn sub_float(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Float(x, rank), Value::Float(y, _)) => Ok(Value::Float(x - y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn mul_int(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(y, _)) => {
            handle_signed_overflow(x, y, rank, i128::checked_mul, u128::wrapping_mul)
        }
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x.wrapping_mul(y), rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn mul_float(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Float(x, rank), Value::Float(y, _)) => Ok(Value::Float(x * y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn div_int(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(_, _), Value::Int(0, _)) | (Value::UInt(_, _), Value::UInt(0, _)) => {
            Err(EvalError::DivByZero)
        }
        (Value::Int(x, rank), Value::Int(y, _)) => {
            // Detect MIN / -1 overflow and warn; result wraps to MIN
            let (min, _max) = int_min_max(rank);
            if y == -1 && x == min {
                signed_overflow(Value::Int(x, rank))
            } else {
                Ok(Value::Int(x / y, rank))
            }
        }
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x / y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn div_float(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Float(x, rank), Value::Float(y, _)) => Ok(Value::Float(x / y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn mod_int(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(_, _), Value::Int(0, _)) | (Value::UInt(_, _), Value::UInt(0, _)) => {
            Err(EvalError::ModByZero)
        }
        (Value::Int(x, rank), Value::Int(y, _)) => {
            if y == -1 {
                Ok(Value::Int(0, rank))
            } else {
                Ok(Value::Int(x % y, rank))
            }
        }
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x % y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn bit_and(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(y, _)) => Ok(Value::Int(x & y, rank)),
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x & y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn bit_or(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(y, _)) => Ok(Value::Int(x | y, rank)),
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x | y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn bit_xor(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(y, _)) => Ok(Value::Int(x ^ y, rank)),
        (Value::UInt(x, rank), Value::UInt(y, _)) => Ok(Value::UInt(x ^ y, rank)),
        _ => Err(EvalError::TypeMismatch),
    }
}

/// Validate and convert shift amount to u32
fn validate_shift_amount(shift: i128, signed: bool, rank: IntRank) -> Result<u32, EvalError> {
    if signed && shift < 0 {
        return Err(EvalError::ShiftOutOfRange);
    }
    let s = shift as u32;
    if s >= rank_bits(rank) {
        return Err(EvalError::ShiftOutOfRange);
    }
    Ok(s)
}

fn shl(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(shift, _)) => {
            let amount = validate_shift_amount(shift, true, rank)?;
            match x.checked_shl(amount) {
                Some(val) => Ok(Value::Int(val, rank)),
                None => signed_overflow(Value::Int(x.wrapping_shl(amount), rank)),
            }
        }
        (Value::UInt(x, rank), Value::Int(shift, _)) => {
            let amount = validate_shift_amount(shift, true, rank)?;
            Ok(Value::UInt(x.wrapping_shl(amount), rank))
        }
        (Value::UInt(x, rank), Value::UInt(shift, _)) => {
            let amount = validate_shift_amount(shift as i128, false, rank)?;
            Ok(Value::UInt(x.wrapping_shl(amount), rank))
        }
        _ => Err(EvalError::TypeMismatch),
    }
}

fn shr(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(x, rank), Value::Int(shift, _)) => {
            let amount = validate_shift_amount(shift, true, rank)?;
            Ok(Value::Int(x >> amount, rank))
        }
        (Value::UInt(x, rank), Value::Int(shift, _)) => {
            let amount = validate_shift_amount(shift, true, rank)?;
            Ok(Value::UInt(x >> amount, rank))
        }
        (Value::UInt(x, rank), Value::UInt(shift, _)) => {
            let amount = validate_shift_amount(shift as i128, false, rank)?;
            Ok(Value::UInt(x >> amount, rank))
        }
        _ => Err(EvalError::TypeMismatch),
    }
}

fn impl_for(op: Op, tag: TyTag) -> fn(Value, Value) -> Result<Value, EvalError> {
    match (op, tag) {
        (Op::Add, TyTag::Float(_)) => add_float,
        (Op::Sub, TyTag::Int(_, _)) => sub_int,
        (Op::Sub, TyTag::Float(_)) => sub_float,
        (Op::Mul, TyTag::Int(_, _)) => mul_int,
        (Op::Mul, TyTag::Float(_)) => mul_float,
        (Op::Div, TyTag::Int(_, _)) => div_int,
        (Op::Div, TyTag::Float(_)) => div_float,
        (Op::Mod, TyTag::Int(_, _)) => mod_int,
        (Op::BitAnd, TyTag::Int(_, _)) => bit_and,
        (Op::BitOr, TyTag::Int(_, _)) => bit_or,
        (Op::Xor, TyTag::Int(_, _)) => bit_xor,
        (Op::Shl, TyTag::Int(_, _)) => shl,
        (Op::Shr, TyTag::Int(_, _)) => shr,
        _ => add_int,
    }
}

pub(super) fn op_from_ast(op: ic_syntax::OpKind) -> Option<Op> {
    use ic_syntax::OpKind as A;
    Some(match op {
        A::Add => Op::Add,
        A::Sub => Op::Sub,
        A::Multiply => Op::Mul,
        A::Divide => Op::Div,
        A::Modulo => Op::Mod,
        A::And => Op::BitAnd,
        A::Or => Op::BitOr,
        A::Xor => Op::Xor,
        A::Lshift => Op::Shl,
        A::Rshift => Op::Shr,
        A::Not => return None,
    })
}

pub(super) fn eval_bin(op: Op, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    let Some(tag) = common_type(&lhs, &rhs) else {
        return Err(EvalError::TypeMismatch);
    };
    let l = cast_to(lhs, tag)?;
    let r = cast_to(rhs, tag)?;
    let f = impl_for(op, tag);
    f(l, r)
}

pub(super) fn eval_unary(op: ic_syntax::OpKind, val: Value) -> Result<Value, EvalError> {
    use ic_syntax::OpKind as A;
    match (op, val) {
        (A::Sub, Value::Int(i, r)) => match i.checked_neg() {
            Some(v) => Ok(Value::Int(v, r)),
            None => signed_overflow(Value::Int(i.wrapping_neg(), r)),
        },
        (A::Sub, Value::UInt(u, r)) => {
            // -u for unsigned: compute in signed domain, then wrap back to unsigned
            let i = u as i128;
            let neg = i.wrapping_neg();
            // Wrap back to unsigned without warnings
            let bits = rank_bits(r);
            let mask = if bits == 64 {
                u64::MAX as u128
            } else {
                (1u128 << bits) - 1
            };
            let unsigned_val = (neg as u128) & mask;
            Ok(Value::UInt(unsigned_val, r))
        }
        (A::Sub, Value::Float(f, r)) => Ok(Value::Float(-f, r)),
        (A::Not, Value::Int(i, r)) => Ok(Value::Int(!i, r)),
        (A::Not, Value::UInt(u, r)) => Ok(Value::UInt(!u, r)),
        (A::Add, v) => Ok(v),
        _ => Err(EvalError::TypeMismatch),
    }
}
