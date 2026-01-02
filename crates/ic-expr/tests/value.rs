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

use ic_expr::{FloatRank, IntRank, Value};

#[test]
fn kind_name_int() {
    let v = Value::<()>::Int(42, IntRank::I32);
    assert_eq!(v.kind_name(), "integer value");
}

#[test]
fn kind_name_uint() {
    let v = Value::<()>::UInt(42, IntRank::U32);
    assert_eq!(v.kind_name(), "unsigned integer value");
}

#[test]
fn kind_name_float() {
    let v = Value::<()>::Float(3.5, FloatRank::F64);
    assert_eq!(v.kind_name(), "floating-point value");
}

#[test]
fn kind_name_bool() {
    let v = Value::<()>::Bool(true);
    assert_eq!(v.kind_name(), "boolean value");
}

#[test]
fn kind_name_string() {
    let v = Value::<()>::String("hello".to_string());
    assert_eq!(v.kind_name(), "string value");
}

#[test]
fn kind_name_null() {
    let v = Value::<()>::Null;
    assert_eq!(v.kind_name(), "null value");
}

#[test]
fn to_bool_int_nonzero() {
    let v = Value::<()>::Int(1, IntRank::I32);
    assert!(v.to_bool());
}

#[test]
fn to_bool_int_zero() {
    let v = Value::<()>::Int(0, IntRank::I32);
    assert!(!v.to_bool());
}

#[test]
fn to_bool_null() {
    let v = Value::<()>::Null;
    assert!(!v.to_bool());
}

#[test]
fn to_i128_from_uint() {
    let v = Value::<()>::UInt(100, IntRank::U32);
    assert_eq!(v.to_i128(), Some(100));
}

#[test]
fn to_u128_from_negative_int() {
    let v = Value::<()>::Int(-1, IntRank::I32);
    assert_eq!(v.to_u128(), None);
}

#[test]
fn to_f64_from_int() {
    let v = Value::<()>::Int(42, IntRank::I32);
    assert_eq!(v.to_f64(), Some(42.0));
}
