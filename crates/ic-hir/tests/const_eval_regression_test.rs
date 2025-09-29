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

mod common;

#[test]
fn test_bool_unsigned_bitwise_or() {
    let input = r"
        const unsigned long FLAG = 1;
        const unsigned long mask = TRUE | FLAG;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_bool_unsigned_bitwise_and() {
    let input = r"
        const unsigned long FLAG = 0xFF;
        const unsigned long mask = FALSE & FLAG;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_bool_unsigned_bitwise_xor() {
    let input = r"
        const unsigned long FLAG = 1;
        const unsigned long mask = TRUE ^ FLAG;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_bool_unsigned_arithmetic() {
    let input = r"
        const unsigned long FLAG = 10;
        const unsigned long sum = TRUE + FLAG;
        const unsigned long diff = FLAG - FALSE;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_bool_signed_arithmetic_still_works() {
    let input = r"
        const long FLAG = 10;
        const long result1 = TRUE | FLAG;
        const long result2 = TRUE + FLAG;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_nan_to_int_conversion_fails() {
    let input = r"
        const long x = (0.0 / 0.0);
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_positive_infinity_to_int_conversion_fails() {
    let input = r"
        const long x = (1.0 / 0.0);
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_negative_infinity_to_int_conversion_fails() {
    let input = r"
        const long x = (-1.0 / 0.0);
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_valid_float_to_int_conversions() {
    let input = r"
        const long x = 3.14;
        const long y = -2.7;
        const long z = 0.0;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_const_to_const_assignment_same_type() {
    let input = r"
        enum MyEnum { TWO };
        const long A = MyEnum::TWO;
        const long B = A;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_const_to_const_assignment_chained() {
    let input = r"
        enum MyEnum { VALUE };
        const long A = MyEnum::VALUE;
        const long B = A;
        const long C = B;
    ";

    let result = common::parse_and_resolve_successfully(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_const_to_const_assignment_incompatible_types() {
    let input = r#"
        const string A = "hello";
        const long B = A;
    "#;

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}
