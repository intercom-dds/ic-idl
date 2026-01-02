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

use ic_expr::ops::{ArithError, TyTag, cast_to, cast_to_int};
use ic_expr::{IntRank, Value};

#[test]
fn cast_signed_in_range() {
    let v = Value::<()>::Int(100, IntRank::I32);
    let result = cast_to(v, TyTag::Int(IntRank::I8, true)).unwrap();
    assert!(matches!(result, Value::Int(100, IntRank::I8)));
}

#[test]
fn cast_signed_out_of_range() {
    let v = Value::<()>::Int(256, IntRank::I32);
    let result = cast_to(v, TyTag::Int(IntRank::I8, true));
    assert!(matches!(result, Err(ArithError::RangeError)));
}

#[test]
fn cast_unsigned_wraps() {
    let v = Value::<()>::Int(256, IntRank::I32);
    let result = cast_to(v, TyTag::Int(IntRank::U8, false)).unwrap();
    assert!(matches!(result, Value::UInt(0, IntRank::U8)));
}

#[test]
fn cast_negative_to_unsigned_wraps() {
    let v = Value::<()>::Int(-1, IntRank::I32);
    let result = cast_to(v, TyTag::Int(IntRank::U8, false)).unwrap();
    assert!(matches!(result, Value::UInt(255, IntRank::U8)));
}

#[test]
fn cast_to_int_strict_rejects_overflow() {
    let v = Value::<()>::Int(256, IntRank::I32);
    let result = cast_to_int(v, IntRank::U8, false, true);
    assert!(matches!(result, Err(ArithError::RangeError)));
}

#[test]
fn cast_to_int_strict_allows_negative() {
    let v = Value::<()>::Int(-1, IntRank::I32);
    let result = cast_to_int(v, IntRank::U8, false, true);
    assert!(matches!(result, Err(ArithError::RangeError)));
}

#[test]
fn cast_to_int_non_strict_allows_overflow() {
    let v = Value::<()>::Int(256, IntRank::I32);
    let result = cast_to_int(v, IntRank::U8, false, false).unwrap();
    assert!(matches!(result, Value::UInt(0, IntRank::U8)));
}

#[test]
fn cast_to_int_non_strict_allows_negative() {
    let v = Value::<()>::Int(-1, IntRank::I32);
    let result = cast_to_int(v, IntRank::U8, false, false).unwrap();
    assert!(matches!(result, Value::UInt(255, IntRank::U8)));
}
