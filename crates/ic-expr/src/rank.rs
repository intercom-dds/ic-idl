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

const DEFAULT_INT_RANK: IntRank = IntRank::I32;

/// Integer rank for type promotions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

/// Floating-point rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatRank {
    F32,
    F64,
    F128,
}

/// Get the bit width for an integer rank.
#[must_use]
pub const fn rank_bits(r: IntRank) -> u32 {
    match r {
        IntRank::I8 | IntRank::U8 => 8,
        IntRank::I16 | IntRank::U16 => 16,
        IntRank::I32 | IntRank::U32 => 32,
        IntRank::I64 | IntRank::U64 => 64,
    }
}

/// Get a bit mask for the given integer rank.
#[must_use]
pub const fn rank_mask(r: IntRank) -> u128 {
    let bits = rank_bits(r);
    if bits >= 128 { !0 } else { (1u128 << bits) - 1 }
}

/// Check if an integer rank is signed.
#[must_use]
pub const fn is_signed(r: IntRank) -> bool {
    matches!(r, IntRank::I8 | IntRank::I16 | IntRank::I32 | IntRank::I64)
}

/// Get the min/max values for an integer rank.
#[must_use]
pub const fn int_bounds(r: IntRank) -> (i128, i128) {
    let bits = rank_bits(r);
    if is_signed(r) {
        let max = (1_i128 << (bits - 1)) - 1;
        let min = -1_i128 - max;
        (min, max)
    } else {
        (0, (1_i128 << bits) - 1)
    }
}

fn rank_ord(r: IntRank) -> u32 {
    match r {
        IntRank::I8 | IntRank::U8 => 0,
        IntRank::I16 | IntRank::U16 => 1,
        IntRank::I32 | IntRank::U32 => 2,
        IntRank::I64 | IntRank::U64 => 3,
    }
}

fn can_represent_all(r: IntRank, target: IntRank) -> bool {
    let (min, max) = int_bounds(r);
    let (tmin, tmax) = int_bounds(target);
    min >= tmin && max <= tmax
}

/// Integer promotion (C standard 6.3.1.1).
///
/// Types smaller than int are promoted to int or unsigned int.
#[must_use]
pub fn promote_int(r: IntRank) -> IntRank {
    if rank_bits(r) < rank_bits(DEFAULT_INT_RANK) {
        if can_represent_all(r, DEFAULT_INT_RANK) {
            DEFAULT_INT_RANK
        } else {
            IntRank::U32
        }
    } else {
        r
    }
}

fn unsigned_of_rank(ord: u32) -> IntRank {
    match ord {
        0 => IntRank::U8,
        1 => IntRank::U16,
        2 => IntRank::U32,
        _ => IntRank::U64,
    }
}

/// Usual arithmetic conversion (C standard 6.3.1.8).
///
/// Computes the common type for a binary operation.
#[must_use]
pub fn common_int_rank(lhs: IntRank, rhs: IntRank) -> IntRank {
    let lhs = promote_int(lhs);
    let rhs = promote_int(rhs);

    if lhs == rhs {
        return lhs;
    }

    let a_ord = rank_ord(lhs);
    let b_ord = rank_ord(rhs);

    match (is_signed(lhs), is_signed(rhs)) {
        // Both signed or both unsigned: use larger rank
        (true, true) | (false, false) => {
            if a_ord >= b_ord {
                lhs
            } else {
                rhs
            }
        }
        // Mixed signedness
        (true, false) => mixed_sign_conv(lhs, a_ord, rhs, b_ord),
        (false, true) => mixed_sign_conv(rhs, b_ord, lhs, a_ord),
    }
}

fn mixed_sign_conv(signed: IntRank, s_ord: u32, unsigned: IntRank, u_ord: u32) -> IntRank {
    match s_ord.cmp(&u_ord) {
        std::cmp::Ordering::Greater => {
            if can_represent_all(unsigned, signed) {
                signed
            } else {
                unsigned_of_rank(s_ord)
            }
        }
        std::cmp::Ordering::Less => unsigned,
        std::cmp::Ordering::Equal => unsigned_of_rank(s_ord),
    }
}

/// Get the larger float rank.
#[must_use]
pub const fn common_float_rank(lhs: FloatRank, rhs: FloatRank) -> FloatRank {
    match (lhs, rhs) {
        (FloatRank::F128, _) | (_, FloatRank::F128) => FloatRank::F128,
        (FloatRank::F64, _) | (_, FloatRank::F64) => FloatRank::F64,
        _ => FloatRank::F32,
    }
}
