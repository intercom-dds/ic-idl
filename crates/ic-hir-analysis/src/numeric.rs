// Copyright 2026 KONGSBERG
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

use ic_expr::ops::{TyTag, cast_to};
use ic_expr::{FloatRank, IntRank, Value};
use ic_hir::hir::{Numeric, PrimitiveTy};

#[must_use]
pub fn integer_value(value: &Numeric) -> Option<i128> {
    match value {
        Numeric::Int8(v) => Some(i128::from(*v)),
        Numeric::UInt8(v) => Some(i128::from(*v)),
        Numeric::Int16(v) => Some(i128::from(*v)),
        Numeric::UInt16(v) => Some(i128::from(*v)),
        Numeric::Int32(v) => Some(i128::from(*v)),
        Numeric::UInt32(v) => Some(i128::from(*v)),
        Numeric::Int64(v) => Some(i128::from(*v)),
        Numeric::UInt64(v) => Some(i128::from(*v)),
        _ => None,
    }
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f32 annotation values are rounded to their target precision"
)]
pub fn coerce(value: &Numeric, primitive: PrimitiveTy) -> Option<Numeric> {
    let value = match value {
        Numeric::Float(v) => Value::<()>::Float(f64::from(*v), FloatRank::F32),
        Numeric::Double(v) => Value::Float(*v, FloatRank::F64),
        Numeric::UInt64(v) => Value::UInt(u128::from(*v), IntRank::U64),
        _ => Value::Int(integer_value(value)?, IntRank::I64),
    };

    match primitive {
        PrimitiveTy::Float32 => Some(Numeric::Float(value.to_f64()? as f32)),
        PrimitiveTy::Float64 | PrimitiveTy::Float128 => Some(Numeric::Double(value.to_f64()?)),
        _ => {
            let value = if let Value::Float(v, _) = &value {
                let signed = v.is_sign_negative();
                let rank = if signed { IntRank::I64 } else { IntRank::U64 };
                cast_to(value, TyTag::Int(rank, signed)).ok()?.to_i128()?
            } else {
                value.to_i128()?
            };

            Some(match primitive {
                PrimitiveTy::Int8 => Numeric::Int8(i8::try_from(value).ok()?),
                PrimitiveTy::UInt8 => Numeric::UInt8(u8::try_from(value).ok()?),
                PrimitiveTy::Int16 => Numeric::Int16(i16::try_from(value).ok()?),
                PrimitiveTy::UInt16 => Numeric::UInt16(u16::try_from(value).ok()?),
                PrimitiveTy::Int32 => Numeric::Int32(i32::try_from(value).ok()?),
                PrimitiveTy::UInt32 => Numeric::UInt32(u32::try_from(value).ok()?),
                PrimitiveTy::Int64 => Numeric::Int64(i64::try_from(value).ok()?),
                PrimitiveTy::UInt64 => Numeric::UInt64(u64::try_from(value).ok()?),
                _ => return None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_finite_integers() {
        assert_eq!(coerce(&Numeric::Double(f64::NAN), PrimitiveTy::Int32), None);
        assert_eq!(
            coerce(&Numeric::Double(f64::INFINITY), PrimitiveTy::Int32),
            None
        );
        assert_eq!(
            coerce(&Numeric::Double(f64::NEG_INFINITY), PrimitiveTy::UInt32),
            None
        );
    }

    #[test]
    fn truncates_before_checking_integer_range() {
        assert_eq!(
            coerce(&Numeric::Double(127.75), PrimitiveTy::Int8),
            Some(Numeric::Int8(127))
        );
        assert_eq!(
            coerce(&Numeric::Double(-128.75), PrimitiveTy::Int8),
            Some(Numeric::Int8(-128))
        );
        assert_eq!(
            coerce(&Numeric::Double(-0.75), PrimitiveTy::UInt8),
            Some(Numeric::UInt8(0))
        );
        assert_eq!(
            coerce(&Numeric::Double(255.75), PrimitiveTy::UInt8),
            Some(Numeric::UInt8(255))
        );
        assert_eq!(coerce(&Numeric::Double(256.0), PrimitiveTy::UInt8), None);
        assert_eq!(coerce(&Numeric::Double(-129.0), PrimitiveTy::Int8), None);
    }
}
