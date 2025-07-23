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

// Test type checking phase

mod common;

#[test]
fn test_string_assigned_to_int() {
    let input = r#"
        const string MY_STR = "foo";
        const int32 FOO = MY_STR;
    "#;

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_int_overflow() {
    let input = r"
        const int8 SMALL = 256;  // Too large for int8
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_valid_constants() {
    let input = r"
        const int32 FOO = 42;
        const boolean FLAG = TRUE;
        const double PI = 3.15;
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Verify we have the expected constants
    assert_eq!(result.order.len(), 3);
}

#[test]
#[ignore = "annotations not yet handled, can't set bit_bound"]
fn test_enum_value_overflow() {
    let input = r"
        enum SmallEnum {
            A = 100,
            B = 200  // Too large for int8
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have an overflow error
    assert!(
        !result.errors.is_empty(),
        "Expected overflow error for enum value 200 in int8, but got no errors"
    );
}

#[test]
fn test_union_case_type_mismatch() {
    let input = r#"
        union MyUnion switch (int32) {
            case "string":  // String literal for int32 discriminator
                string s;
            case 1:
                int32 i;
        };
    "#;

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}
