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

//! Type casting and conversion between Value, Numeric, and HIR types.

use ic_diagnostic::Label;

use super::rank::{FloatRank, IntRank, TyTag, int_min_max, rank_bits};
use super::{EvalError, Value};
use crate::hir::{Numeric, PrimitiveTy, Ty, TyKind};

/// Cast a value to a target type tag, performing necessary conversions.
pub(super) fn cast_to(value: Value, target: TyTag) -> Result<Value, EvalError> {
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
        (Bool(b), TyTag::Int(r, sign)) => {
            if sign {
                Ok(Int(i128::from(b), r))
            } else {
                Ok(UInt(u128::from(b), r))
            }
        }

        // Const values should be resolved before calling this function
        (Value::Const(_), _) => Err(EvalError::TypeMismatch),
        other => {
            // Fallback for unsupported implicit casts
            let _ = other;
            Err(EvalError::TypeMismatch)
        }
    }
}

/// Cast a value to a specific HIR type.
pub(super) fn cast_value_to_type(v: Value, ty: &Ty) -> Result<Value, EvalError> {
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
        TyKind::Any => {
            // 'any' type accepts any value
            Ok(v)
        }
        // For non-primitive types (enums/bitmasks/etc), we rely on callers to interpret
        _ => Ok(v),
    }
}

/// Convert a HIR Numeric to an evaluation Value.
pub(super) fn value_from_numeric(num: &Numeric) -> Option<Value> {
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
        Numeric::Const(def_id) => Some(Value::Const(*def_id)),
        Numeric::Array { .. }
        | Numeric::Sequence { .. }
        | Numeric::Map { .. }
        | Numeric::Struct { .. }
        | Numeric::Union { .. } => None,
    }
}

/// Convert an evaluation Value to a HIR Numeric.
#[allow(clippy::unnecessary_wraps)]
pub(super) fn numeric_from_value(v: &Value) -> Option<Numeric> {
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
        Value::Const(def_id) => Some(Numeric::Const(*def_id)),
    }
}

/// Map a primitive type to its integer rank.
pub fn rank_for_primitive(prim: PrimitiveTy) -> Option<(bool, IntRank)> {
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

/// Map a primitive type to its floating-point rank.
pub fn float_rank_for_primitive(prim: PrimitiveTy) -> Option<FloatRank> {
    use PrimitiveTy::{Float32, Float64, Float128};
    Some(match prim {
        Float32 => FloatRank::F32,
        Float64 => FloatRank::F64,
        Float128 => FloatRank::F128,
        _ => return None,
    })
}

/// Get a human-readable name for a type.
pub fn get_type_name(ty: &Ty, ctx: &super::super::LoweringContext) -> String {
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

/// Check if assigning a float literal to an integer type will lose precision and warn if so.
pub fn check_float_to_int_precision_loss(
    expr: &ic_syntax::Expr,
    expected_ty: &Ty,
    diagnostics: &mut super::super::Diagnostics,
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
