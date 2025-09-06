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

use ic_diagnostic::{Label, error_span, warn_span};

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
    ModByZero,
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
/// - Otherwise, promote to unsigned int (`UInt32`)
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
            match a_rank.cmp(&b_rank) {
                std::cmp::Ordering::Greater => {
                    if can_int_represent_all(rhs_prom, lhs_prom) {
                        // Signed can represent all unsigned values
                        lhs_prom
                    } else {
                        // Convert to unsigned of signed's rank
                        unsigned_of_rank(a_rank)
                    }
                }
                std::cmp::Ordering::Less => {
                    // Unsigned has higher rank
                    rhs_prom
                }
                std::cmp::Ordering::Equal => {
                    // Same rank: use unsigned
                    unsigned_of_rank(a_rank)
                }
            }
        }
        (false, true) => {
            match b_rank.cmp(&a_rank) {
                std::cmp::Ordering::Greater => {
                    if can_int_represent_all(lhs_prom, rhs_prom) {
                        // Signed can represent all unsigned values
                        rhs_prom
                    } else {
                        // Convert to unsigned of signed's rank
                        unsigned_of_rank(b_rank)
                    }
                }
                std::cmp::Ordering::Less => {
                    // Unsigned has higher rank
                    lhs_prom
                }
                std::cmp::Ordering::Equal => {
                    // Same rank: use unsigned
                    unsigned_of_rank(a_rank)
                }
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
    use FloatRank::{F32, F64, F128};
    match (ty, other) {
        (F128, _) | (_, F128) => F128,
        (F64, _) | (_, F64) => F64,
        _ => F32,
    }
}

fn common_type(a: &Value, b: &Value) -> Option<TyTag> {
    use Value::{Bool, Float, Int, UInt};
    match (a, b) {
        (Float(_, fa), Float(_, fb)) => Some(TyTag::Float(float_rank_for(*fa, *fb))),
        (Float(_, fr), Int(_, _) | UInt(_, _)) | (Int(_, _) | UInt(_, _), Float(_, fr)) => {
            Some(TyTag::Float(*fr))
        }
        (Int(_, ra), Int(_, rb)) => Some(TyTag::Int(usual_int_conv(*ra, *rb), true)),
        (UInt(_, ra), UInt(_, rb)) => Some(TyTag::Int(usual_int_conv(*ra, *rb), false)),
        (Int(_, ra), UInt(_, rb)) | (UInt(_, rb), Int(_, ra)) => {
            let rank = usual_int_conv(*ra, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (Bool(_), Bool(_)) => Some(TyTag::Int(INT_RANK, true)),
        (Bool(_), Int(_, rb) | UInt(_, rb)) => {
            let rank = usual_int_conv(INT_RANK, *rb);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        (Int(_, ra) | UInt(_, ra), Bool(_)) => {
            let rank = usual_int_conv(*ra, INT_RANK);
            Some(TyTag::Int(rank, is_signed(rank)))
        }
        _ => None,
    }
}

fn cast_to(value: Value, target: TyTag) -> Result<Value, EvalError> {
    use Value::{Bool, Float, Int, UInt};
    match (value, target) {
        (Int(v, _), TyTag::Int(r, sign)) => {
            if sign {
                let (min, max) = int_min_max(r);
                if v < min || v > max {
                    return Err(EvalError::RangeError);
                }
                Ok(Int(v, r))
            } else {
                // For unsigned target, wrap negative values using two's complement
                let bits = rank_bits(r);
                let mask: u128 = if bits >= 128 { !0 } else { (1u128 << bits) - 1 };
                let unsigned_val = (v as u128) & mask;
                Ok(UInt(unsigned_val, r))
            }
        }
        (UInt(v, _), TyTag::Int(r, sign)) => {
            if sign {
                // Converting unsigned to signed - check if it fits in signed range
                let max = int_min_max(r).1 as u128;
                if v > max {
                    return Err(EvalError::RangeError);
                }
                Ok(Int(v as i128, r))
            } else {
                // Converting unsigned to unsigned - apply modular reduction (wrap)
                let bits = rank_bits(r);
                let mask: u128 = if bits >= 128 { !0 } else { (1u128 << bits) - 1 };
                Ok(UInt(v & mask, r))
            }
        }
        (Int(v, _), TyTag::Float(fr)) => Ok(Float(v as f64, fr)),
        (UInt(v, _), TyTag::Float(fr)) => Ok(Float(v as f64, fr)),
        (Float(f, _), TyTag::Float(fr)) => Ok(Float(f, fr)),
        (Float(f, _), TyTag::Int(r, sign)) => {
            // Truncate float to integer
            let i = f.trunc() as i128;
            let (min, max) = int_min_max(r);
            if i < min || i > max {
                return Err(EvalError::RangeError);
            }
            if sign {
                Ok(Int(i, r))
            } else {
                Ok(UInt(i as u128, r))
            }
        }
        (Bool(b), TyTag::Int(r, _)) => Ok(Int(i128::from(b), r)),
        other => {
            // Fallback for unsupported implicit casts
            let _ = other;
            Err(EvalError::TypeMismatch)
        }
    }
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

/// Check if assigning a float literal to an integer type will lose precision and warn if so.
fn check_float_to_int_precision_loss(
    expr: &ic_syntax::Expr,
    expected_ty: &Ty,
    diagnostics: &mut super::Diagnostics,
) {
    // Check if we have a float literal
    if let ic_syntax::Expr::Literal(lit) = expr {
        if let ic_syntax::LiteralValue::Float(float_val) = &lit.value {
            // Check if target type is integer
            if let TyKind::Primitive(prim) = &expected_ty.kind {
                let is_int_type = matches!(
                    prim,
                    crate::hir::PrimitiveTy::Int8
                        | crate::hir::PrimitiveTy::UInt8
                        | crate::hir::PrimitiveTy::Int16
                        | crate::hir::PrimitiveTy::UInt16
                        | crate::hir::PrimitiveTy::Int32
                        | crate::hir::PrimitiveTy::UInt32
                        | crate::hir::PrimitiveTy::Int64
                        | crate::hir::PrimitiveTy::UInt64
                );

                if is_int_type {
                    let truncated = float_val.trunc();
                    // Check if the fractional part is non-zero
                    if (float_val - truncated).abs() > f64::EPSILON {
                        diagnostics.warnings.push(ic_diagnostic::warn_span(
                            format!(
                                "implicit conversion from 'double' to '{}' changes value from {} \
                                 to {}",
                                prim.name(),
                                float_val,
                                truncated as i64
                            ),
                            Label::new(expr.span()).message("precision loss here"),
                        ));
                    }
                }
            }
        }
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
        A::Not => return None,
    })
}

fn value_from_numeric(num: &Numeric) -> Option<Value> {
    match num {
        Numeric::Null => Some(Value::Null),
        Numeric::Bool(b) => Some(Value::Bool(*b)),
        // Treat char literals as integers for promotions (use unsigned 8-bit rank)
        Numeric::Char(c) => Some(Value::UInt(u128::from(*c as u32), IntRank::U8)),
        Numeric::Int8(v) => Some(Value::Int(i128::from(*v), IntRank::I8)),
        Numeric::Octet(v) => Some(Value::UInt(u128::from(*v), IntRank::U8)),
        Numeric::Int16(v) => Some(Value::Int(i128::from(*v), IntRank::I16)),
        Numeric::UInt16(v) => Some(Value::UInt(u128::from(*v), IntRank::U16)),
        Numeric::Int32(v) => Some(Value::Int(i128::from(*v), IntRank::I32)),
        Numeric::UInt32(v) => Some(Value::UInt(u128::from(*v), IntRank::U32)),
        Numeric::Int64(v) => Some(Value::Int(i128::from(*v), IntRank::I64)),
        Numeric::UInt64(v) => Some(Value::UInt(u128::from(*v), IntRank::U64)),
        Numeric::Float(v) => Some(Value::Float(f64::from(*v), FloatRank::F32)),
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

#[allow(clippy::unnecessary_wraps)]
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
    use PrimitiveTy::{
        Bool, Char, Float32, Float64, Float128, Int8, Int16, Int32, Int64, UInt8, UInt16, UInt32,
        UInt64, Void, WChar,
    };
    match prim {
        Int8 => Some((true, IntRank::I8)),
        UInt8 => Some((false, IntRank::U8)),
        Int16 => Some((true, IntRank::I16)),
        UInt16 => Some((false, IntRank::U16)),
        Int32 => Some((true, IntRank::I32)),
        UInt32 => Some((false, IntRank::U32)),
        Int64 => Some((true, IntRank::I64)),
        UInt64 => Some((false, IntRank::U64)),
        Bool | Char | WChar | Float32 | Float64 | Float128 | Void => None,
    }
}

fn float_rank_for_primitive(prim: PrimitiveTy) -> Option<FloatRank> {
    use PrimitiveTy::{Float32, Float64, Float128};
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
                        Value::Int(i, IntRank::I16) => u32::from(i as u16),
                        _ => return Err(EvalError::TypeMismatch),
                    };
                    if (0xD800..=0xDFFF).contains(&code) {
                        return Err(EvalError::InvalidChar);
                    }
                    // Safe: not a surrogate and within BMP
                    Ok(Value::Char(char::from_u32(code).unwrap()))
                }
                PrimitiveTy::Bool => {
                    // Handle boolean type
                    match v {
                        Value::Bool(_) => Ok(v), // Already a bool, just return it
                        _ => Err(EvalError::TypeMismatch),
                    }
                }
                _ => {
                    if let Some((signed, rank)) = rank_for_primitive(*p) {
                        cast_to(v, TyTag::Int(rank, signed))
                    } else if let Some(fr) = float_rank_for_primitive(*p) {
                        cast_to(v, TyTag::Float(fr))
                    } else {
                        // void not supported here
                        Err(EvalError::TypeMismatch)
                    }
                }
            }
        }
        TyKind::String { .. } => {
            // String types only accept string values
            match v {
                Value::String(_) => Ok(v),
                _ => Err(EvalError::TypeMismatch),
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

/// Get a human-readable name for a type.
fn get_type_name(ty: &Ty, ctx: &LoweringContext) -> String {
    match &ty.kind {
        TyKind::Primitive(p) => match p {
            PrimitiveTy::Bool => "bool",
            PrimitiveTy::Char => "char",
            PrimitiveTy::WChar => "wchar",
            PrimitiveTy::Int8 => "int8",
            PrimitiveTy::UInt8 => "uint8",
            PrimitiveTy::Int16 => "int16",
            PrimitiveTy::UInt16 => "uint16",
            PrimitiveTy::Int32 => "int32",
            PrimitiveTy::UInt32 => "uint32",
            PrimitiveTy::Int64 => "int64",
            PrimitiveTy::UInt64 => "uint64",
            PrimitiveTy::Float32 => "float",
            PrimitiveTy::Float64 => "double",
            PrimitiveTy::Float128 => "float128",
            PrimitiveTy::Void => "void",
        }
        .to_string(),
        TyKind::Adt(def_id) => {
            // Get the actual type name from the definition
            let def = ctx.context.definitions.get(*def_id);
            def.ident.name.clone()
        }
        TyKind::String { wide, .. } => if *wide { "wstring" } else { "string" }.to_string(),
        TyKind::Array { .. } => "array".to_string(),
        TyKind::Sequence { .. } => "sequence".to_string(),
        TyKind::Map { .. } => "map".to_string(),
        TyKind::Fixed => "fixed".to_string(),
        TyKind::Any => "any".to_string(),
        TyKind::Null => "null".to_string(),
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

    /// Returns a reference to the context for read-only access.
    pub fn context(&self) -> &LoweringContext {
        self.ctx
    }

    /// Returns a mutable reference to the diagnostics.
    pub fn diagnostics(&mut self) -> &mut super::Diagnostics {
        &mut self.ctx.diagnostics
    }

    /// Evaluate an expression to a HIR Numeric value (best-effort typing).
    pub fn eval_numeric(&mut self, expr: &ic_syntax::Expr) -> Option<Numeric> {
        let v = self.eval_value(expr)?;
        numeric_from_value(&v)
    }

    /// Evaluate an expression expecting a given target type (for constants declared with type).
    pub fn eval_for_type(&mut self, expr: &ic_syntax::Expr, expected_ty: &Ty) -> Option<Numeric> {
        // Handle initializer lists specially based on expected type
        if let ic_syntax::Expr::InitList(init_list) = expr {
            return self.eval_initializer_list(init_list, expected_ty, expr.span());
        }

        // Perform literal range checks before evaluation
        if let Some(lit) = extract_direct_int_literal(expr) {
            if !self.check_literal_range(lit, expected_ty, expr.span()) {
                return None;
            }
        }

        // If assigning from a path to a constant, check compatibility and optionally reuse it
        if let ic_syntax::Expr::Path(path) = expr {
            match self.try_const_path_assignment(path, expected_ty, expr.span()) {
                ConstAssignOutcome::Accepted(n) => return Some(*n),
                ConstAssignOutcome::Rejected => return None,
                ConstAssignOutcome::NotApplicable => {}
            }
        }

        // Special case: if the expected type is void, don't try to evaluate or cast.
        // Just return a dummy value and let the lint catch the invalid usage.
        if let TyKind::Primitive(PrimitiveTy::Void) = &expected_ty.kind {
            return Some(Numeric::Null);
        }

        let v = self.eval_value(expr)?;

        // Warn about precision loss when assigning float literal to integer type
        check_float_to_int_precision_loss(expr, expected_ty, &mut self.ctx.diagnostics);

        self.cast_and_convert(v, expected_ty, expr.span())
    }

    /// Evaluate an initializer list for the expected type.
    fn eval_initializer_list(
        &mut self,
        init_list: &ic_syntax::InitList,
        expected_ty: &Ty,
        span: ic_syntax::Span,
    ) -> Option<Numeric> {
        use super::initializers::InitializerEvaluator;

        match &expected_ty.kind {
            TyKind::Adt(def_id) => {
                let def = self.ctx.context.definitions.get(*def_id);
                if let DefKind::Struct(_) = &def.kind {
                    let mut init_eval = InitializerEvaluator::new(self);
                    return init_eval.eval_struct(init_list, *def_id, expected_ty);
                }
            }
            TyKind::Array { ty, len, .. } => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_array(init_list, ty, *len);
            }
            TyKind::Sequence { ty, .. } => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_sequence(init_list, ty);
            }
            TyKind::Map { key, elem, .. } => {
                let mut init_eval = InitializerEvaluator::new(self);
                return init_eval.eval_map(init_list, key, elem);
            }
            _ => {}
        }

        self.ctx.diagnostics.error(
            "initializer lists can only be used to initialize structs, arrays, sequences, or maps"
                .to_string(),
            Label::new(span).message("invalid use of initializer list"),
        );
        None
    }

    /// Check if an integer literal is within range for the expected type.
    fn check_literal_range(&mut self, lit: i128, expected_ty: &Ty, span: ic_syntax::Span) -> bool {
        let TyKind::Primitive(p) = &expected_ty.kind else {
            return true;
        };
        match p {
            PrimitiveTy::Int8 | PrimitiveTy::Int16 | PrimitiveTy::Int32 | PrimitiveTy::Int64 => {
                let rank = match p {
                    PrimitiveTy::Int8 => IntRank::I8,
                    PrimitiveTy::Int16 => IntRank::I16,
                    PrimitiveTy::Int32 => IntRank::I32,
                    PrimitiveTy::Int64 => IntRank::I64,
                    _ => unreachable!(),
                };
                let (min, max) = int_min_max(rank);
                if lit < min || lit > max {
                    let ty_name = get_type_name(expected_ty, self.ctx);
                    self.ctx.diagnostics.errors.push(error_span(
                        format!("integer literal out of range for '{ty_name}'"),
                        Label::new(span).message("out of range"),
                    ));
                    return false;
                }
            }
            PrimitiveTy::UInt8
            | PrimitiveTy::UInt16
            | PrimitiveTy::UInt32
            | PrimitiveTy::UInt64 => {
                // For unsigned targets, only reject direct positive literals
                // that exceed the target's max. Negative literals are allowed
                // (they wrap), per IDL/C integer conversion rules.
                if lit >= 0 {
                    let max_u: u128 = match p {
                        PrimitiveTy::UInt8 => u8::MAX as u128,
                        PrimitiveTy::UInt16 => u16::MAX as u128,
                        PrimitiveTy::UInt32 => u32::MAX as u128,
                        PrimitiveTy::UInt64 => u64::MAX as u128,
                        _ => 0,
                    };
                    if (lit as u128) > max_u {
                        let ty_name = get_type_name(expected_ty, self.ctx);
                        self.ctx.diagnostics.errors.push(error_span(
                            format!("integer literal out of range for '{ty_name}'"),
                            Label::new(span).message("out of range"),
                        ));
                        return false;
                    }
                }
            }
            _ => {}
        }
        true
    }

    /// Cast a value to the expected type and convert to Numeric.
    fn cast_and_convert(
        &mut self,
        v: Value,
        expected_ty: &Ty,
        span: ic_syntax::Span,
    ) -> Option<Numeric> {
        // Store value description before moving v
        let value_desc = match &v {
            Value::String(_) => "string value",
            Value::Bool(_) => "boolean value",
            Value::Char(_) => "character value",
            Value::Int(_, _) => "integer value",
            Value::UInt(_, _) => "unsigned integer value",
            Value::Float(_, _) => "floating-point value",
            Value::Null => "null value",
        };

        match cast_value_to_type(v, expected_ty) {
            Ok(v) => numeric_from_value(&v),
            Err(EvalError::RangeError) => {
                self.ctx.diagnostics.error(
                    "value out of range for target type".to_string(),
                    Label::new(span).message("out of range"),
                );
                None
            }
            Err(EvalError::InvalidChar) => {
                self.ctx.diagnostics.errors.push(error_span(
                    "invalid Unicode scalar for character type",
                    Label::new(span).message("invalid character value"),
                ));
                None
            }
            Err(_) => {
                let type_name = get_type_name(expected_ty, self.ctx);

                self.ctx.diagnostics.errors.push(error_span(
                    format!("{value_desc} cannot be assigned to type {type_name}"),
                    Label::new(span).message("incompatible types"),
                ));
                None
            }
        }
    }

    /// Evaluate an expression to a simplified Value.
    fn eval_value(&mut self, expr: &ic_syntax::Expr) -> Option<Value> {
        use ic_syntax::Expr::{Binary, Group, InitList, Literal, Path, Unary};
        match expr {
            Literal(lit) => value_from_numeric(&literal_to_numeric(&lit.value)),
            Path(path) => self.eval_path_value(path),
            Binary(bin) => self.eval_binary_value(bin, expr.span()),
            Unary(un) => self.eval_unary_value(un, expr.span()),
            Group(group) => self.eval_value(&group.expr),
            InitList(_) => {
                self.ctx.diagnostics.error(
                    "initializer lists cannot be used in arithmetic expressions".to_string(),
                    Label::new(expr.span()).message("not allowed in arithmetic context"),
                );
                None
            }
        }
    }

    /// Evaluate a path to a constant value.
    fn eval_path_value(&mut self, path: &ic_syntax::Path) -> Option<Value> {
        if let Some(def_id) = self
            .ctx
            .scopes
            .resolve_path(&self.ctx.context, self.scope, path)
        {
            // Constants, enumerators and flags are Const
            let def = self.ctx.context.definitions.get(def_id);
            if let DefKind::Const(c) = &def.kind {
                value_from_numeric(&c.value)
            } else {
                self.ctx.diagnostics.errors.push(error_span(
                    format!("`{}` is not a constant value", path_to_string(path)),
                    Label::new(path_span(path)).message("expected constant, enumerator, or flag"),
                ));
                None
            }
        } else {
            self.ctx.diagnostics.errors.push(
                error_span(
                    format!(
                        "undefined constant or enum value `{}`",
                        path_to_string(path)
                    ),
                    Label::new(path_span(path)).message("evaluation error"),
                )
                .note("check that the name is spelled correctly"),
            );
            None
        }
    }

    /// Evaluate a binary expression.
    fn eval_binary_value(
        &mut self,
        bin: &ic_syntax::Binary,
        expr_span: ic_syntax::Span,
    ) -> Option<Value> {
        let Some(op) = op_from_ast(bin.op.kind) else {
            self.ctx.diagnostics.errors.push(error_span(
                "unsupported binary operation in constant expression",
                Label::new(expr_span).message("unsupported operation"),
            ));
            return None;
        };

        // Evaluate operands
        let l = self.eval_value(&bin.lhs)?;
        let r = self.eval_value(&bin.rhs)?;

        // Check for string operands early for better error messages
        if self.check_string_operands_value(&l, &r, bin) {
            return None;
        }

        // For division/modulo by zero errors, use the RHS span if available
        let op_span = match op {
            Op::Div | Op::Mod => bin.rhs.span(),
            _ => expr_span,
        };

        self.handle_binary_result_value(eval_bin(op, l, r), expr_span, op_span, &bin.rhs)
    }

    /// Check if either operand is a string and report an error.
    fn check_string_operands_value(
        &mut self,
        l: &Value,
        r: &Value,
        bin: &ic_syntax::Binary,
    ) -> bool {
        let has_string_operand = matches!(l, Value::String(_)) || matches!(r, Value::String(_));
        if has_string_operand {
            let string_span = if matches!(l, Value::String(_)) {
                bin.lhs.span()
            } else {
                bin.rhs.span()
            };

            self.ctx.diagnostics.errors.push(
                error_span(
                    "string literals cannot be used in arithmetic expressions",
                    Label::new(string_span).message("string operand"),
                )
                .note(
                    "string literals can only be used in struct initialization or string constants",
                ),
            );
            true
        } else {
            false
        }
    }

    /// Handle the result of a binary operation evaluation.
    fn handle_binary_result_value(
        &mut self,
        result: Result<Value, EvalError>,
        expr_span: ic_syntax::Span,
        op_span: ic_syntax::Span,
        rhs: &ic_syntax::Expr,
    ) -> Option<Value> {
        match result {
            Ok(v) => Some(v),
            Err(EvalError::SignedOverflow(v)) => {
                // Signed overflow: warn and continue with wrapped result
                self.ctx.diagnostics.warnings.push(
                    warn_span(
                        "integer overflow in constant expression",
                        Label::new(expr_span).message("overflow detected"),
                    )
                    .note("consider using a larger integer type if overflow was not intended"),
                );
                Some(v)
            }
            Err(EvalError::RangeError) => {
                self.ctx.diagnostics.errors.push(error_span(
                    "value out of range for target type",
                    Label::new(expr_span).message("out of range"),
                ));
                None
            }
            Err(EvalError::InvalidChar) => {
                self.ctx.diagnostics.error(
                    "invalid Unicode scalar for character type".to_string(),
                    Label::new(expr_span).message("invalid character value"),
                );
                None
            }
            Err(EvalError::DivByZero) => {
                self.ctx.diagnostics.errors.push(error_span(
                    "division by zero in constant expression",
                    Label::new(op_span).message("division by zero"),
                ));
                None
            }
            Err(EvalError::ModByZero) => {
                self.ctx.diagnostics.errors.push(error_span(
                    "modulo by zero in constant expression",
                    Label::new(op_span).message("modulo by zero"),
                ));
                None
            }
            Err(EvalError::ShiftOutOfRange) => {
                // The old module reports this as an error, not a warning
                self.ctx.diagnostics.errors.push(error_span(
                    "invalid shift amount: shift count >= width of type or negative",
                    Label::new(rhs.span()).message("invalid shift"),
                ));
                None
            }
            Err(EvalError::TypeMismatch) => {
                self.ctx.diagnostics.error(
                    "type mismatch in constant expression".to_string(),
                    Label::new(expr_span).message("invalid operand types"),
                );
                None
            }
        }
    }

    /// Evaluate a unary expression.
    fn eval_unary_value(
        &mut self,
        un: &ic_syntax::Unary,
        expr_span: ic_syntax::Span,
    ) -> Option<Value> {
        let v = self.eval_value(&un.expr)?;
        if let Ok(v) = eval_unary(un.op.kind, v) {
            Some(v)
        } else {
            self.ctx.diagnostics.error(
                "unsupported unary operation in constant expression".to_string(),
                Label::new(expr_span).message("unsupported operation"),
            );
            None
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

    /// Evaluate an expression to a Numeric, preserving Const references for union case labels.
    pub fn eval_union_case_label(
        &mut self,
        expr: &ic_syntax::Expr,
        disc_ty: &Ty,
    ) -> Option<Numeric> {
        use ic_syntax::Expr::Path;

        // Try to evaluate using eval_for_type to get proper type checking
        if let Some(numeric) = self.eval_for_type(expr, disc_ty) {
            // Success! Now check if this is a Path expression to a constant
            if let Path(path) = expr {
                // For paths to constants, replace with Const reference
                if let Some(def_id) =
                    self.ctx
                        .scopes
                        .resolve_path(&self.ctx.context, self.scope, path)
                {
                    let def = self.ctx.context.definitions.get(def_id);
                    if let DefKind::Const(_) = &def.kind {
                        // Return a Const reference instead of the evaluated value
                        return Some(Numeric::Const(def_id));
                    }
                }
            }
            // Return the evaluated numeric value
            Some(numeric)
        } else {
            // eval_for_type already reported the error
            None
        }
    }
}

/// Outcome of attempting to assign a constant path to a target type.
enum ConstAssignOutcome {
    /// Not a constant path (or not applicable) — caller should continue with normal evaluation.
    NotApplicable,
    /// Assignment accepted; use the returned numeric (typically a Const reference).
    Accepted(Box<Numeric>),
    /// Assignment rejected and a diagnostic was emitted; caller should stop.
    Rejected,
}

impl ConstEvaluator<'_> {
    /// If `path` resolves to a constant, verify it can be assigned to `expected_ty`.
    /// Returns `Accepted(Numeric::Const`(...)) on success, Rejected on hard error,
    /// or `NotApplicable` if the path is not a constant.
    fn try_const_path_assignment(
        &mut self,
        path: &ic_syntax::Path,
        expected_ty: &Ty,
        use_span: ic_syntax::Span,
    ) -> ConstAssignOutcome {
        let Some(def_id) = self
            .ctx
            .scopes
            .resolve_path(&self.ctx.context, self.scope, path)
        else {
            return ConstAssignOutcome::NotApplicable;
        };

        let def = self.ctx.context.definitions.get(def_id);
        let DefKind::Const(c) = &def.kind else {
            return ConstAssignOutcome::NotApplicable;
        };

        if let Some(val) = value_from_numeric(&c.value) {
            match cast_value_to_type(val, expected_ty) {
                Ok(_) => ConstAssignOutcome::Accepted(Box::new(Numeric::Const(def_id))),
                Err(EvalError::RangeError) => {
                    self.ctx.diagnostics.error(
                        "value out of range for target type".to_string(),
                        Label::new(use_span).message("out of range"),
                    );
                    ConstAssignOutcome::Rejected
                }
                Err(_) => {
                    // Provide a precise error mentioning both types and declaration site
                    let from_ty = get_type_name(&c.ty, self.ctx);
                    let to_ty = get_type_name(expected_ty, self.ctx);
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
                    ConstAssignOutcome::Rejected
                }
            }
        } else {
            // Non-scalar constant — not assignable here.
            let to_ty = get_type_name(expected_ty, self.ctx);
            self.ctx.diagnostics.errors.push(
                error_span(
                    format!(
                        "constant '{}' cannot be assigned to {}",
                        def.ident.name, to_ty
                    ),
                    Label::new(use_span).message("incompatible types"),
                )
                .label(Label::new(def.ident.span).message("declared here")),
            );
            ConstAssignOutcome::Rejected
        }
    }
}

/// Try to extract a direct integer literal from an expression.
/// Handles plain integer literals, parenthesized literals, and a single leading unary '-'.
fn extract_direct_int_literal(expr: &ic_syntax::Expr) -> Option<i128> {
    use ic_syntax::Expr as E;
    match expr {
        E::Literal(lit) => match &lit.value {
            ic_syntax::LiteralValue::Int(i) => Some(*i as i128),
            _ => None,
        },
        E::Group(g) => extract_direct_int_literal(&g.expr),
        E::Unary(u) => {
            if u.op.kind == ic_syntax::OpKind::Sub {
                if let Some(v) = extract_direct_int_literal(&u.expr) {
                    return v.checked_neg();
                }
            }
            None
        }
        _ => None,
    }
}
