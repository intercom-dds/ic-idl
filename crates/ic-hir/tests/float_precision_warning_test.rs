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
fn test_float_literal_precision_loss_warning() {
    let idl = r"
        const int32 LOSES_PRECISION = 1.5;
        const uint32 ALSO_LOSES = 3.7;
        const int64 TINY_LOSS = 1.00000001;
    ";

    let diagnostics = common::compile_idl_with_warnings(idl);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_no_warning_for_whole_floats() {
    let idl = r"
        const int32 WHOLE_NUMBER = 1.0;
        const uint32 ANOTHER_WHOLE = 42.0;
        const int64 BIG_WHOLE = 1000000.0;
        const int8 SMALL_WHOLE = 127.0;
    ";

    let result = common::parse_and_resolve_successfully(idl);
    assert_eq!(
        result.warnings.len(),
        0,
        "Should have no warnings for whole number floats"
    );
}

#[test]
fn test_no_warning_for_float_expressions() {
    let idl = r"
        const double PI = 3.14159;
        const int32 TRUNCATED_FROM_CONST = PI;  // No warning - not a literal
    ";

    let result = common::parse_and_resolve_successfully(idl);
    assert_eq!(
        result.warnings.len(),
        0,
        "Should have no warnings for float values that are part of expressions"
    );
}

#[test]
fn test_warning_for_negative_floats() {
    let idl = r"
        const int32 NEGATIVE_LOSS = -1.5;
        const int64 NEGATIVE_TINY = -0.1;
    ";

    let diagnostics = common::compile_idl_with_warnings(idl);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_no_warning_for_float_to_float() {
    let idl = r"
        const float FLOAT_VALUE = 1.5;
        const double DOUBLE_VALUE = 3.14159;
    ";

    let result = common::parse_and_resolve_successfully(idl);
    assert_eq!(
        result.warnings.len(),
        0,
        "Should have no warnings when assigning float literals to float types"
    );
}

#[test]
fn test_warning_only_for_direct_literals() {
    let idl = r"
        const double PI = 3.14159;
        const int32 TRUNCATED_PI = PI;  // No warning - not a literal
        const int32 LITERAL_PI = 3.14159;  // Warning - direct literal
    ";

    let diagnostics = common::compile_idl_with_warnings(idl);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_all_integer_types() {
    let idl = r"
        const int8 I8_LOSS = 1.5;
        const uint8 U8_LOSS = 2.3;
        const int16 I16_LOSS = 3.7;
        const uint16 U16_LOSS = 4.9;
        const int32 I32_LOSS = 5.1;
        const uint32 U32_LOSS = 6.6;
        const int64 I64_LOSS = 7.8;
        const uint64 U64_LOSS = 8.2;
    ";

    let diagnostics = common::compile_idl_with_warnings(idl);
    insta::assert_snapshot!(diagnostics);
}
