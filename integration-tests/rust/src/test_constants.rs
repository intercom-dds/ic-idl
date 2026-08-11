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

use crate::{constant_types, large_integer_types};

#[test]
fn int_constant() {
    assert_eq!(constant_types::INT_CONST, 42);
}

#[test]
fn uint_constant() {
    assert_eq!(constant_types::UINT_CONST, 100);
}

#[test]
fn short_constant() {
    assert_eq!(constant_types::SHORT_CONST, -10);
}

#[test]
fn longlong_constant() {
    assert_eq!(constant_types::LONGLONG_CONST, 9999999999i64);
}

#[test]
fn double_constant() {
    assert_approx!(constant_types::DOUBLE_CONST, 3.14159, f64::EPSILON);
}

#[test]
fn float_constant() {
    assert_approx!(constant_types::FLOAT_CONST, 2.5f32, f32::EPSILON);
}

#[test]
fn string_constant() {
    assert_eq!(constant_types::STRING_CONST, "hello world");
}

#[test]
fn bool_constants() {
    assert_eq!(constant_types::BOOL_TRUE, true);
    assert_eq!(constant_types::BOOL_FALSE, false);
}

#[test]
fn octet_constant() {
    assert_eq!(constant_types::OCTET_CONST, 255u8);
}

#[test]
fn constant_chain() {
    assert_eq!(constant_types::CHAIN_1, 10);
    assert_eq!(constant_types::CHAIN_2, 10);
    assert_eq!(constant_types::CHAIN_3, 10);
    assert_eq!(constant_types::CHAIN_4, 10);
    assert_eq!(constant_types::CHAIN_5, 10);
    assert_eq!(constant_types::CHAIN_2, constant_types::CHAIN_1);
    assert_eq!(constant_types::CHAIN_3, constant_types::CHAIN_2);
    assert_eq!(constant_types::CHAIN_4, constant_types::CHAIN_3);
    assert_eq!(constant_types::CHAIN_5, constant_types::CHAIN_4);
}

#[test]
fn arithmetic_chain() {
    assert_eq!(constant_types::ARITH_BASE, 100);
    assert_eq!(constant_types::ARITH_DOUBLED, 200);
    assert_eq!(constant_types::ARITH_QUADRUPLED, 400);
    assert_eq!(constant_types::ARITH_OCTUPLED, 800);
    assert_eq!(
        constant_types::ARITH_DOUBLED,
        constant_types::ARITH_BASE * 2
    );
    assert_eq!(
        constant_types::ARITH_QUADRUPLED,
        constant_types::ARITH_BASE * 4
    );
    assert_eq!(
        constant_types::ARITH_OCTUPLED,
        constant_types::ARITH_BASE * 8
    );
}

#[test]
fn mixed_arithmetic_chain() {
    assert_eq!(constant_types::MATH_1, 5);
    assert_eq!(constant_types::MATH_2, 15);
    assert_eq!(constant_types::MATH_3, 30);
    assert_eq!(constant_types::MATH_4, 25);
    assert_eq!(constant_types::MATH_5, 5);
}

#[test]
fn negation() {
    assert_eq!(constant_types::NEGATIVE, -50);
    assert_eq!(constant_types::NEGATED, 50);
    assert_eq!(constant_types::DOUBLE_NEGATED, -50);
    assert_eq!(constant_types::NEGATED, -constant_types::NEGATIVE);
    assert_eq!(constant_types::DOUBLE_NEGATED, -constant_types::NEGATED);
}

#[test]
fn bitwise_operations() {
    assert_eq!(constant_types::BITS_A, 0x0F);
    assert_eq!(constant_types::BITS_B, 0xF0);
    assert_eq!(constant_types::BITS_OR, 0xFF);
    assert_eq!(constant_types::BITS_AND, 255);
    assert_eq!(constant_types::BITS_XOR, 240);
    assert_eq!(constant_types::BITS_SHIFT_LEFT, 16);
    assert_eq!(constant_types::BITS_SHIFT_RIGHT, 16);
    assert_eq!(
        constant_types::BITS_OR,
        (constant_types::BITS_A | constant_types::BITS_B)
    );
}

#[test]
fn float_chain() {
    assert_approx!(constant_types::FLOAT_A, 1.0, f64::EPSILON);
    assert_approx!(constant_types::FLOAT_B, 1.5, f64::EPSILON);
    assert_approx!(constant_types::FLOAT_C, 3.0, f64::EPSILON);
    assert_approx!(constant_types::FLOAT_D, 0.75, f64::EPSILON);
}

#[test]
fn enum_constant_reference() {
    assert_eq!(
        constant_types::PRIORITY_VALUE,
        constant_types::Priority::High
    );
    assert_eq!(constant_types::PRIORITY_VALUE as i32, 100);
    assert_eq!(
        constant_types::PRIORITY_CHAIN,
        constant_types::PRIORITY_VALUE
    );
}

#[test]
fn parenthesized_expressions() {
    assert_eq!(constant_types::PAREN_A, 30);
    assert_eq!(constant_types::PAREN_B, 20);
    assert_eq!(constant_types::PAREN_C, 25);
}

#[test]
fn modulo_operations() {
    assert_eq!(constant_types::MOD_A, 2);
    assert_eq!(constant_types::MOD_B, 2);
}

#[test]
fn octet_limits() {
    assert_eq!(large_integer_types::OCTET_MAX, u8::MAX);
    assert_eq!(large_integer_types::OCTET_MIN, u8::MIN);
}

#[test]
fn short_limits() {
    assert_eq!(large_integer_types::SHORT_MAX, i16::MAX);
    assert_eq!(large_integer_types::SHORT_MIN, i16::MIN);
}

#[test]
fn ushort_limits() {
    assert_eq!(large_integer_types::USHORT_MAX, u16::MAX);
    assert_eq!(large_integer_types::USHORT_MIN, u16::MIN);
}

#[test]
fn long_limits() {
    assert_eq!(large_integer_types::IDL_LONG_MAX, i32::MAX);
    assert_eq!(large_integer_types::IDL_LONG_MIN, i32::MIN);
}

#[test]
fn ulong_limits() {
    assert_eq!(large_integer_types::IDL_ULONG_MAX, u32::MAX);
    assert_eq!(large_integer_types::IDL_ULONG_MIN, u32::MIN);
}

#[test]
fn longlong_limits() {
    assert_eq!(large_integer_types::LONGLONG_MAX, i64::MAX);
    assert_eq!(large_integer_types::LONGLONG_MIN, i64::MIN);
}

#[test]
fn ulonglong_limits() {
    assert_eq!(large_integer_types::ULONGLONG_MAX, u64::MAX);
    assert_eq!(large_integer_types::ULONGLONG_MIN, u64::MIN);
}

#[test]
fn hex_literals() {
    assert_eq!(large_integer_types::HEX_DEADBEEF, 0xDEADBEEF);
    assert_eq!(large_integer_types::HEX_FFFFFFFF, 0xFFFFFFFF);
    assert_eq!(large_integer_types::HEX_64_BIT, 0x123456789ABCDEF0);
}

#[test]
fn octal_literals() {
    assert_eq!(large_integer_types::OCTAL_777, 511);
    assert_eq!(large_integer_types::OCTAL_777, 0o0777);
}

#[test]
fn large_int_struct() {
    let fields = large_integer_types::LargeIntFields {
        big_signed: 9223372036854775807,
        big_unsigned: 18446744073709551615,
    };
    assert_eq!(fields.big_signed, 9223372036854775807);
    assert_eq!(fields.big_unsigned, 18446744073709551615);
    assert_eq!(fields.big_signed, large_integer_types::LONGLONG_MAX);
    assert_eq!(fields.big_unsigned, large_integer_types::ULONGLONG_MAX);

    let default_fields = large_integer_types::LargeIntFields::new();
    assert_eq!(default_fields.big_signed, 0);
    assert_eq!(default_fields.big_unsigned, 0);

    let copy_fields = fields;
    assert_eq!(copy_fields, fields);
    assert_eq!(copy_fields.big_signed, fields.big_signed);
    assert_eq!(copy_fields.big_unsigned, fields.big_unsigned);
}

#[test]
fn derived_constants() {
    assert_eq!(large_integer_types::LONG_MAX_MINUS_ONE, 2147483646);
    assert_eq!(
        large_integer_types::LONG_MAX_MINUS_ONE,
        large_integer_types::IDL_LONG_MAX - 1
    );
    assert_eq!(
        large_integer_types::LONGLONG_MAX_MINUS_ONE,
        9223372036854775806
    );
    assert_eq!(
        large_integer_types::LONGLONG_MAX_MINUS_ONE,
        large_integer_types::LONGLONG_MAX - 1
    );
}
