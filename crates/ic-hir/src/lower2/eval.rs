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

use ic_diagnostic::Label;

use super::LoweringContext;
use super::utils::{literal_to_numeric, path_span, path_to_string};
use crate::hir::{DefKind, Numeric, PrimitiveTy, Ty, TyKind};
use crate::scope::ScopeId;

/// Integer rank categories for promotions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IntRank {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

/// Floating-point widths we care about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FloatRank {
    F32,
    F64,
    F128,
}

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
    DivByZero,
    TypeMismatch,
    ShiftOutOfRange,
}

// Helper to return a SignedOverflow while carrying a wrapped result
fn signed_overflow(v: Value) -> Result<Value, EvalError> {
    Err(EvalError::SignedOverflow(v))
}

fn rank_bits(r: IntRank) -> u32 {
    match r {
        IntRank::I8 | IntRank::U8 => 8,
        IntRank::I16 | IntRank::U16 => 16,
        IntRank::I32 | IntRank::U32 => 32,
        IntRank::I64 | IntRank::U64 => 64,
    }
}

fn is_signed(r: IntRank) -> bool {
    matches!(r, IntRank::I8 | IntRank::I16 | IntRank::I32 | IntRank::I64)
}

fn rank_ord(r: IntRank) -> u32 {
    match r {
        IntRank::I8 | IntRank::U8 => 0,
        IntRank::I16 | IntRank::U16 => 1,
        IntRank::I32 | IntRank::U32 => 2,
        IntRank::I64 | IntRank::U64 => 3,
    }
}

// IDL long
const INT_RANK: IntRank = IntRank::I32;

fn int_min_max(r: IntRank) -> (i128, i128) {
    let bits = rank_bits(r);
    if is_signed(r) {
        let max = (1_i128 << (bits - 1)) - 1;
        let min = -1_i128 - max;
        (min, max)
    } else {
        (0, (1_i128 << bits) - 1)
    }
}

fn can_int_represent_all(r: IntRank, int_r: IntRank) -> bool {
    let (min, max) = int_min_max(r);
    let (imin, imax) = int_min_max(int_r);
    min >= imin && max <= imax
}

/// Integer promotions (C standard 6.3.1.1)
///
/// Values of types smaller than int are promoted when used in expressions:
/// - If int (Int32) can represent all values of the original type, promote to int
/// - Otherwise, promote to unsigned int (UInt32)
/// - Types already int-sized or larger are unchanged
fn promote_integer(r: IntRank) -> IntRank {
    if rank_bits(r) < rank_bits(INT_RANK) {
        if can_int_represent_all(r, INT_RANK) {
            // int8/uint8/int16 → int32
            INT_RANK
        } else {
            // uint16 → uint32 (when int32 can't hold all values)
            IntRank::U32
        }
    } else {
        // Already int-sized or larger
        r
    }
}

fn unsigned_of_rank(rank_ord_val: u32) -> IntRank {
    match rank_ord_val {
        0 => IntRank::U8,
        1 => IntRank::U16,
        2 => IntRank::U32,
        _ => IntRank::U64,
    }
}

/// Usual arithmetic conversions (C standard 6.3.1.8)
///
/// When two operands have different types, they are converted to a common type:
/// 1. If both operands have the same type after promotion, no further conversion
/// 2. If both are signed or both unsigned, the smaller rank converts to larger
/// 3. If the unsigned operand has rank >= signed operand, signed converts to unsigned
/// 4. If the signed type can represent all values of the unsigned type, unsigned converts to signed
/// 5. Otherwise, both convert to the unsigned type corresponding to the signed type's rank
fn usual_int_conv(lhs: IntRank, rhs: IntRank) -> IntRank {
    let lhs_prom = promote_integer(lhs);
    let rhs_prom = promote_integer(rhs);
    if lhs_prom == rhs_prom {
        return lhs_prom;
    }

    let a_rank = rank_ord(lhs_prom);
    let b_rank = rank_ord(rhs_prom);
    match (is_signed(lhs_prom), is_signed(rhs_prom)) {
        // Both signed or both unsigned: use the larger rank
        (true, true) | (false, false) => {
            if a_rank >= b_rank {
                lhs_prom
            } else {
                rhs_prom
            }
        }
        // Mixed signedness: follow C rules
        (true, false) => {
            if a_rank > b_rank {
                if can_int_represent_all(rhs_prom, lhs_prom) {
                    // Signed can represent all unsigned values
                    lhs_prom
                } else {
                    // Convert to unsigned of signed's rank
                    unsigned_of_rank(a_rank)
                }
            } else if a_rank < b_rank {
                // Unsigned has higher rank
                rhs_prom
            } else {
                // Same rank: use unsigned
                unsigned_of_rank(a_rank)
            }
        }
        (false, true) => {
            if b_rank > a_rank {
                if can_int_represent_all(lhs_prom, rhs_prom) {
                    // Signed can represent all unsigned values
                    rhs_prom
                } else {
                    // Convert to unsigned of signed's rank
                    unsigned_of_rank(b_rank)
                }
            } else if b_rank < a_rank {
                // Unsigned has higher rank
                lhs_prom
            } else {
                // Same rank: use unsigned
                unsigned_of_rank(a_rank)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TyTag {
    Int(IntRank, bool),
    Float(FloatRank),
}

fn float_rank_for(ty: FloatRank, other: FloatRank) -> FloatRank {
    use FloatRank::*;
    match (ty, other) {
        (F128, _) | (_, F128) => F128,
        (F64, _) | (_, F64) => F64,
        _ => F32,
    }
}

fn common_type(a: &Value, b: &Value) -> Option<TyTag> {
    use Value::*;
    match (a, b) {
        (Float(_, fa), Float(_, fb)) => Some(TyTag::Float(float_rank_for(*fa, *fb))),
        (Float(_, fr), Int(_, _))
        | (Float(_, fr), UInt(_, _))
        | (Int(_, _), Float(_, fr))
        | (UInt(_, _), Float(_, fr)) => Some(TyTag::Float(*fr)),
        (Int(_, ra), Int(_, rb)) => Some(TyTag::Int(usual_int_conv(*ra, *rb), true)),
        (UInt(_, ra), UInt(_, rb)) => Some(TyTag::Int(usual_int_conv(*ra, *rb), false)),
        (Int(_, ra), UInt(_, rb)) | (UInt(_, rb), Int(_, ra)) => {
            let rank = usual_int_conv(*ra, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (Bool(_), Bool(_)) => Some(TyTag::Int(INT_RANK, true)),
        (Bool(_), Int(_, rb)) => {
            let rank = usual_int_conv(INT_RANK, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (Int(_, ra), Bool(_)) => {
            let rank = usual_int_conv(*ra, INT_RANK);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (Bool(_), UInt(_, rb)) => {
            let rank = usual_int_conv(INT_RANK, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (UInt(_, ra), Bool(_)) => {
            let rank = usual_int_conv(*ra, INT_RANK);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        _ => None,
    }
}

fn cast_to(value: Value, target: TyTag) -> Result<Value, EvalError> {
    use Value::*;
    match (value, target) {
        (Int(v, _), TyTag::Int(r, sign)) => {
            let (min, max) = int_min_max(r);
            if v < min || v > max {
                return Err(EvalError::RangeError);
            }
            if sign {
                Ok(Int(v, r))
            } else {
                Ok(UInt(v as u128, r))
            }
        }
        (UInt(v, _), TyTag::Int(r, sign)) => {
            let max = int_min_max(r).1 as u128;
            if v > max {
                return Err(EvalError::RangeError);
            }
            if sign {
                Ok(Int(v as i128, r))
            } else {
                Ok(UInt(v, r))
            }
        }
        (Int(v, _), TyTag::Float(fr)) => Ok(Float(v as f64, fr)),
        (UInt(v, _), TyTag::Float(fr)) => Ok(Float(v as f64, fr)),
        (Float(f, _), TyTag::Float(fr)) => Ok(Float(f, fr)),
        (Bool(b), TyTag::Int(r, _)) => Ok(Int(if b { 1 } else { 0 }, r)),
        other => {
            // Fallback for unsupported implicit casts
            let _ = other;
            Err(EvalError::TypeMismatch)
        }
    }
}

// Per-class operation implementations after casting to common type
fn add_int(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(y, _)) => match x.checked_add(y) {
            Some(v) => Ok(Value::Int(v, r)),
            None => signed_overflow(Value::Int(x.wrapping_add(y), r)),
        },
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x.wrapping_add(y), r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn add_float(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x + y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn sub_int(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(y, _)) => match x.checked_sub(y) {
            Some(v) => Ok(Value::Int(v, r)),
            None => signed_overflow(Value::Int(x.wrapping_sub(y), r)),
        },
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x.wrapping_sub(y), r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn sub_float(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x - y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn mul_int(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(y, _)) => match x.checked_mul(y) {
            Some(v) => Ok(Value::Int(v, r)),
            None => signed_overflow(Value::Int(x.wrapping_mul(y), r)),
        },
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x.wrapping_mul(y), r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn mul_float(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x * y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn div_int(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(_, _), Value::Int(0, _)) | (Value::UInt(_, _), Value::UInt(0, _)) => {
            Err(EvalError::DivByZero)
        }
        (Value::Int(x, r), Value::Int(y, _)) => {
            // Detect MIN / -1 overflow and warn; result wraps to MIN
            let (min, _max) = int_min_max(r);
            if y == -1 && x == min {
                signed_overflow(Value::Int(x, r))
            } else {
                Ok(Value::Int(x / y, r))
            }
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x / y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn div_float(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Float(x, r), Value::Float(y, _)) => Ok(Value::Float(x / y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn mod_int(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(_, _), Value::Int(0, _)) | (Value::UInt(_, _), Value::UInt(0, _)) => {
            Err(EvalError::DivByZero)
        }
        (Value::Int(x, r), Value::Int(y, _)) => {
            if y == -1 {
                Ok(Value::Int(0, r))
            } else {
                Ok(Value::Int(x % y, r))
            }
        }
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x % y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn bit_and(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(y, _)) => Ok(Value::Int(x & y, r)),
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x & y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn bit_or(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(y, _)) => Ok(Value::Int(x | y, r)),
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x | y, r)),
        _ => Err(EvalError::TypeMismatch),
    }
}

fn bit_xor(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(y, _)) => Ok(Value::Int(x ^ y, r)),
        (Value::UInt(x, r), Value::UInt(y, _)) => Ok(Value::UInt(x ^ y, r)),
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

fn shl(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(shift, _)) => {
            let s = validate_shift_amount(shift, true, r)?;
            match x.checked_shl(s) {
                Some(v) => Ok(Value::Int(v, r)),
                None => signed_overflow(Value::Int(x.wrapping_shl(s), r)),
            }
        }
        (Value::UInt(x, r), Value::Int(shift, _)) => {
            let s = validate_shift_amount(shift, true, r)?;
            Ok(Value::UInt(x.wrapping_shl(s), r))
        }
        (Value::UInt(x, r), Value::UInt(shift, _)) => {
            let s = validate_shift_amount(shift as i128, false, r)?;
            Ok(Value::UInt(x.wrapping_shl(s), r))
        }
        _ => Err(EvalError::TypeMismatch),
    }
}

fn shr(a: Value, b: Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Int(x, r), Value::Int(shift, _)) => {
            let s = validate_shift_amount(shift, true, r)?;
            Ok(Value::Int(x >> s, r))
        }
        (Value::UInt(x, r), Value::Int(shift, _)) => {
            let s = validate_shift_amount(shift, true, r)?;
            Ok(Value::UInt(x >> s, r))
        }
        (Value::UInt(x, r), Value::UInt(shift, _)) => {
            let s = validate_shift_amount(shift as i128, false, r)?;
            Ok(Value::UInt(x >> s, r))
        }
        _ => Err(EvalError::TypeMismatch),
    }
}

fn impl_for(op: Op, tag: TyTag) -> fn(Value, Value) -> Result<Value, EvalError> {
    match (op, tag) {
        (Op::Add, TyTag::Int(_, _)) => add_int,
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
        // Default to int for unsupported combo (should be caught earlier)
        _ => add_int,
    }
}

fn op_from_ast(op: ic_syntax::OpKind) -> Option<Op> {
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
        _ => return None,
    })
}

fn value_from_numeric(num: &Numeric) -> Option<Value> {
    match num {
        Numeric::Null => Some(Value::Null),
        Numeric::Bool(b) => Some(Value::Bool(*b)),
        // Treat char literals as integers for promotions (use unsigned 8-bit rank)
        Numeric::Char(c) => Some(Value::UInt((*c as u32) as u128, IntRank::U8)),
        Numeric::Int8(v) => Some(Value::Int(*v as i128, IntRank::I8)),
        Numeric::Octet(v) => Some(Value::UInt(*v as u128, IntRank::U8)),
        Numeric::Int16(v) => Some(Value::Int(*v as i128, IntRank::I16)),
        Numeric::UInt16(v) => Some(Value::UInt(*v as u128, IntRank::U16)),
        Numeric::Int32(v) => Some(Value::Int(*v as i128, IntRank::I32)),
        Numeric::UInt32(v) => Some(Value::UInt(*v as u128, IntRank::U32)),
        Numeric::Int64(v) => Some(Value::Int(*v as i128, IntRank::I64)),
        Numeric::UInt64(v) => Some(Value::UInt(*v as u128, IntRank::U64)),
        Numeric::Float(v) => Some(Value::Float(*v as f64, FloatRank::F32)),
        Numeric::Double(v) => Some(Value::Float(*v, FloatRank::F64)),
        Numeric::String(s) => Some(Value::String(s.clone())),
        Numeric::Const(_)
        | Numeric::Array { .. }
        | Numeric::Sequence { .. }
        | Numeric::Map { .. }
        | Numeric::Struct { .. }
        | Numeric::Union { .. } => None,
    }
}

fn numeric_from_value(v: &Value) -> Option<Numeric> {
    match v {
        Value::Null => Some(Numeric::Null),
        Value::Bool(b) => Some(Numeric::Bool(*b)),
        Value::Char(c) => Some(Numeric::Char(*c)),
        Value::Int(i, r) => Some(match r {
            IntRank::I8 => Numeric::Int8(*i as i8),
            IntRank::I16 => Numeric::Int16(*i as i16),
            IntRank::I32 => Numeric::Int32(*i as i32),
            IntRank::I64 => Numeric::Int64(*i as i64),
            IntRank::U8 => Numeric::Octet(*i as u8),
            IntRank::U16 => Numeric::UInt16(*i as u16),
            IntRank::U32 => Numeric::UInt32(*i as u32),
            IntRank::U64 => Numeric::UInt64(*i as u64),
        }),
        Value::UInt(u, r) => Some(match r {
            IntRank::I8 => Numeric::Int8(*u as i8),
            IntRank::I16 => Numeric::Int16(*u as i16),
            IntRank::I32 => Numeric::Int32(*u as i32),
            IntRank::I64 => Numeric::Int64(*u as i64),
            IntRank::U8 => Numeric::Octet(*u as u8),
            IntRank::U16 => Numeric::UInt16(*u as u16),
            IntRank::U32 => Numeric::UInt32(*u as u32),
            IntRank::U64 => Numeric::UInt64(*u as u64),
        }),
        Value::Float(f, fr) => Some(match fr {
            FloatRank::F32 => Numeric::Float(*f as f32),
            _ => Numeric::Double(*f),
        }),
        Value::String(s) => Some(Numeric::String(s.clone())),
    }
}

fn rank_for_primitive(prim: PrimitiveTy) -> Option<(bool, IntRank)> {
    use PrimitiveTy::*;
    Some(match prim {
        Bool => return None,
        Char | WChar => return None,
        Int8 => (true, IntRank::I8),
        UInt8 => (false, IntRank::U8),
        Int16 => (true, IntRank::I16),
        UInt16 => (false, IntRank::U16),
        Int32 => (true, IntRank::I32),
        UInt32 => (false, IntRank::U32),
        Int64 => (true, IntRank::I64),
        UInt64 => (false, IntRank::U64),
        Float32 => return None,
        Float64 => return None,
        Float128 => return None,
        Void => return None,
    })
}

fn float_rank_for_primitive(prim: PrimitiveTy) -> Option<FloatRank> {
    use PrimitiveTy::*;
    Some(match prim {
        Float32 => FloatRank::F32,
        Float64 => FloatRank::F64,
        Float128 => FloatRank::F128,
        _ => return None,
    })
}

fn cast_value_to_type(v: Value, ty: &Ty) -> Result<Value, EvalError> {
    match &ty.kind {
        TyKind::Primitive(p) => {
            match *p {
                PrimitiveTy::Char => {
                    // Cast to unsigned 8-bit, then to char
                    let vv = cast_to(v, TyTag::Int(IntRank::U8, false))?;
                    match vv {
                        Value::UInt(u, IntRank::U8) => Ok(Value::Char((u as u8) as char)),
                        Value::Int(i, IntRank::I8) => Ok(Value::Char((i as u8) as char)),
                        _ => Err(EvalError::TypeMismatch),
                    }
                }
                PrimitiveTy::WChar => {
                    // Cast to unsigned 16-bit, then validate Unicode scalar (reject surrogates)
                    let vv = cast_to(v, TyTag::Int(IntRank::U16, false))?;
                    let code = match vv {
                        Value::UInt(u, IntRank::U16) => u as u32,
                        Value::Int(i, IntRank::I16) => (i as u16) as u32,
                        _ => return Err(EvalError::TypeMismatch),
                    };
                    if (0xD800..=0xDFFF).contains(&code) {
                        return Err(EvalError::InvalidChar);
                    }
                    // Safe: not a surrogate and within BMP
                    Ok(Value::Char(char::from_u32(code).unwrap()))
                }
                _ => {
                    if let Some((signed, rank)) = rank_for_primitive(*p) {
                        cast_to(v, TyTag::Int(rank, signed))
                    } else if let Some(fr) = float_rank_for_primitive(*p) {
                        cast_to(v, TyTag::Float(fr))
                    } else {
                        // bool/void not supported here
                        Err(EvalError::TypeMismatch)
                    }
                }
            }
        }
        // For non-primitive types (enums/bitmasks/etc), we rely on callers to interpret
        _ => Ok(v),
    }
}

fn eval_bin(op: Op, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    let Some(tag) = common_type(&lhs, &rhs) else {
        return Err(EvalError::TypeMismatch);
    };
    let l = cast_to(lhs, tag)?;
    let r = cast_to(rhs, tag)?;
    let f = impl_for(op, tag);
    f(l, r)
}

fn eval_unary(op: ic_syntax::OpKind, val: Value) -> Result<Value, EvalError> {
    use ic_syntax::OpKind as A;
    match (op, val) {
        (A::Sub, Value::Int(i, r)) => match i.checked_neg() {
            Some(v) => Ok(Value::Int(v, r)),
            None => signed_overflow(Value::Int(i.wrapping_neg(), r)),
        },
        (A::Sub, Value::UInt(u, r)) => {
            // -u for unsigned: apply in signed domain of same rank, warn on overflow
            let signed = match r {
                IntRank::U8 => IntRank::I8,
                IntRank::U16 => IntRank::I16,
                IntRank::U32 => IntRank::I32,
                IntRank::U64 => IntRank::I64,
                _ => IntRank::I32,
            };
            let i = u as i128;
            match i.checked_neg() {
                Some(v) => Ok(Value::Int(v, signed)),
                None => signed_overflow(Value::Int(i.wrapping_neg(), signed)),
            }
        }
        (A::Not, Value::Int(i, r)) => Ok(Value::Int(!i, r)),
        (A::Not, Value::UInt(u, r)) => Ok(Value::UInt(!u, r)),
        (A::Add, v) => Ok(v),
        _ => Err(EvalError::TypeMismatch),
    }
}

/// Table-driven constant evaluator with promotions.
pub struct ConstEvaluator<'a> {
    ctx: &'a mut LoweringContext,
    scope: ScopeId,
}

impl<'a> ConstEvaluator<'a> {
    pub fn new(ctx: &'a mut LoweringContext, scope: ScopeId) -> Self {
        Self { ctx, scope }
    }

    /// Evaluate an expression to a HIR Numeric value (best-effort typing).
    pub fn eval_numeric(&mut self, expr: &ic_syntax::Expr) -> Option<Numeric> {
        let v = self.eval_value(expr)?;
        numeric_from_value(&v)
    }

    /// Evaluate an expression expecting a given target type (for constants declared with type).
    pub fn eval_for_type(&mut self, expr: &ic_syntax::Expr, expected_ty: &Ty) -> Option<Numeric> {
        let v = self.eval_value(expr)?;
        match cast_value_to_type(v, expected_ty) {
            Ok(v) => numeric_from_value(&v),
            Err(EvalError::RangeError) => {
                self.ctx.diagnostics.error(
                    "value out of range for target type".to_string(),
                    Label::new(expr.span()).message("out of range"),
                );
                None
            }
            Err(EvalError::InvalidChar) => {
                self.ctx.diagnostics.error(
                    "invalid Unicode scalar for character type".to_string(),
                    Label::new(expr.span()).message("invalid character value"),
                );
                None
            }
            Err(_) => {
                self.ctx.diagnostics.error(
                    "cannot convert constant to expected type".to_string(),
                    Label::new(expr.span()).message("type mismatch"),
                );
                None
            }
        }
    }

    /// Evaluate an expression to a simplified Value.
    fn eval_value(&mut self, expr: &ic_syntax::Expr) -> Option<Value> {
        use ic_syntax::Expr::*;
        match expr {
            Literal(lit) => value_from_numeric(&literal_to_numeric(&lit.value)),
            Path(path) => {
                match self
                    .ctx
                    .scopes
                    .resolve_path(&self.ctx.context, self.scope, path)
                {
                    Some(def_id) => {
                        // Constants, enumerators and flags are Const
                        let def = self.ctx.context.definitions.get(def_id);
                        match &def.kind {
                            DefKind::Const(c) => value_from_numeric(&c.value),
                            _ => {
                                self.ctx.diagnostics.error(
                                    format!("`{}` is not a constant value", path_to_string(path)),
                                    Label::new(path_span(path))
                                        .message("expected constant, enumerator, or flag"),
                                );
                                None
                            }
                        }
                    }
                    None => {
                        self.ctx.diagnostics.error(
                            format!("constant `{}` not found", path_to_string(path)),
                            Label::new(path_span(path)).message("must be declared before use"),
                        );
                        None
                    }
                }
            }
            Binary(bin) => {
                let op = match op_from_ast(bin.op.kind) {
                    Some(o) => o,
                    None => {
                        self.ctx.diagnostics.error(
                            "unsupported binary operation in constant expression".to_string(),
                            Label::new(expr.span()).message("unsupported operation"),
                        );
                        return None;
                    }
                };

                // Evaluate operands and track if the RHS is a division/modulo operation
                let l = self.eval_value(&bin.lhs)?;
                let r = self.eval_value(&bin.rhs)?;

                // For division/modulo by zero errors, use the RHS span if available
                let error_span = match op {
                    Op::Div | Op::Mod => bin.rhs.span(),
                    _ => expr.span(),
                };

                match eval_bin(op, l, r) {
                    Ok(v) => Some(v),
                    Err(EvalError::SignedOverflow(v)) => {
                        // Signed overflow: warn and continue with wrapped result
                        self.ctx.diagnostics.warn(
                            "signed overflow in constant expression".to_string(),
                            Label::new(expr.span()).message("overflow (wrapped)"),
                        );
                        Some(v)
                    }
                    Err(EvalError::RangeError) => {
                        self.ctx.diagnostics.error(
                            "value out of range for target type".to_string(),
                            Label::new(expr.span()).message("out of range"),
                        );
                        None
                    }
                    Err(EvalError::InvalidChar) => {
                        self.ctx.diagnostics.error(
                            "invalid Unicode scalar for character type".to_string(),
                            Label::new(expr.span()).message("invalid character value"),
                        );
                        None
                    }
                    Err(EvalError::DivByZero) => {
                        self.ctx.diagnostics.error(
                            "division by zero in constant expression".to_string(),
                            Label::new(error_span).message("division by zero"),
                        );
                        None
                    }
                    Err(EvalError::ShiftOutOfRange) => {
                        // Match C behavior: warn but continue with masked shift
                        self.ctx.diagnostics.warn(
                            "shift count >= width of type or negative".to_string(),
                            Label::new(bin.rhs.span()).message("shift count overflow"),
                        );
                        None
                    }
                    Err(EvalError::TypeMismatch) => {
                        self.ctx.diagnostics.error(
                            "type mismatch in constant expression".to_string(),
                            Label::new(expr.span()).message("invalid operand types"),
                        );
                        None
                    }
                }
            }
            Unary(un) => {
                let v = self.eval_value(&un.expr)?;
                match eval_unary(un.op.kind, v) {
                    Ok(v) => Some(v),
                    Err(_) => {
                        self.ctx.diagnostics.error(
                            "unsupported unary operation in constant expression".to_string(),
                            Label::new(expr.span()).message("unsupported operation"),
                        );
                        None
                    }
                }
            }
            _ => {
                self.ctx.diagnostics.error(
                    "complex expressions not yet supported in constants".to_string(),
                    Label::new(expr.span()).message("unsupported expression"),
                );
                None
            }
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
}
