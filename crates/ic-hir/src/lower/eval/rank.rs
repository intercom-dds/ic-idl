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

//! Integer and floating-point type ranking and promotion rules.
//!
//! Implements C-style integer promotions and usual arithmetic conversions
//! for constant expression evaluation.

use super::Value;

/// Integer rank categories for promotions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntRank {
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
pub enum FloatRank {
    F32,
    F64,
    F128,
}

/// Type tag for common type computation.
#[derive(Clone, Copy, Debug)]
pub enum TyTag {
    Int(IntRank, bool),
    Float(FloatRank),
}

/// IDL long (default integer type for promotions)
const INT_RANK: IntRank = IntRank::I32;

pub fn rank_bits(r: IntRank) -> u32 {
    match r {
        IntRank::I8 | IntRank::U8 => 8,
        IntRank::I16 | IntRank::U16 => 16,
        IntRank::I32 | IntRank::U32 => 32,
        IntRank::I64 | IntRank::U64 => 64,
    }
}

/// Get a bit mask for the given integer rank (signed representation).
pub fn rank_mask_signed(r: IntRank) -> i128 {
    let bits = rank_bits(r);
    if bits >= 64 {
        i64::MAX as i128
    } else {
        (1i128 << bits) - 1
    }
}

/// Get a bit mask for the given integer rank (unsigned representation).
pub fn rank_mask_unsigned(r: IntRank) -> u128 {
    let bits = rank_bits(r);
    if bits >= 128 { !0 } else { (1u128 << bits) - 1 }
}

pub fn is_signed(r: IntRank) -> bool {
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

pub fn int_min_max(r: IntRank) -> (i128, i128) {
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
pub fn promote_integer(r: IntRank) -> IntRank {
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
pub fn usual_int_conv(lhs: IntRank, rhs: IntRank) -> IntRank {
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
        (signed_is_lhs @ true, false) | (signed_is_lhs @ false, true) => {
            let (signed_rank, signed_rank_ord, unsigned_rank, unsigned_rank_ord) = if signed_is_lhs
            {
                (lhs_prom, a_rank, rhs_prom, b_rank)
            } else {
                (rhs_prom, b_rank, lhs_prom, a_rank)
            };

            match signed_rank_ord.cmp(&unsigned_rank_ord) {
                std::cmp::Ordering::Greater => {
                    if can_int_represent_all(unsigned_rank, signed_rank) {
                        signed_rank
                    } else {
                        unsigned_of_rank(signed_rank_ord)
                    }
                }
                std::cmp::Ordering::Less => unsigned_rank,
                std::cmp::Ordering::Equal => unsigned_of_rank(signed_rank_ord),
            }
        }
    }
}

pub fn float_rank_for(ty: FloatRank, other: FloatRank) -> FloatRank {
    use FloatRank::{F32, F64, F128};
    match (ty, other) {
        (F128, _) | (_, F128) => F128,
        (F64, _) | (_, F64) => F64,
        _ => F32,
    }
}

/// Compute the common type for two values according to C promotion rules.
pub fn common_type(a: &Value, b: &Value) -> Option<TyTag> {
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
