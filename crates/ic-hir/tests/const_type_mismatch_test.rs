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
fn test_string_to_int_assignment() {
    let input = r#"
        const string MY_STR = "foo";
        const int32 FOO = MY_STR;
    "#;

    let (result, source_map, _) = common::parse_and_resolve(input);

    // Should have an error about type mismatch
    assert!(
        !result.errors.is_empty(),
        "Expected error for string to int assignment"
    );

    // Snapshot test the error message
    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_bool_to_string_assignment() {
    let input = r"
        const boolean MY_BOOL = true;
        const string FOO = MY_BOOL;
    ";

    let (result, source_map, _) = common::parse_and_resolve(input);

    // Should have an error about type mismatch
    assert!(
        !result.errors.is_empty(),
        "Expected error for bool to string assignment"
    );

    // Snapshot test the error message
    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_valid_numeric_promotion() {
    let input = r"
        const int32 MY_INT = 100;
        const int64 BIG_INT = MY_INT;
        const double MY_DOUBLE = MY_INT;
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors - these are valid promotions
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_out_of_range_direct_literal() {
    // This tests direct literal out-of-range checking
    let input = r"
        const octet SMALL = 256;
    ";

    let (result, source_map, _) = common::parse_and_resolve(input);

    // Should have an error about value out of range
    assert!(
        !result.errors.is_empty(),
        "Expected error for out of range assignment"
    );

    // Snapshot test the error message
    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_const_to_smaller_type() {
    // This tests assigning a constant to a smaller type where the value fits
    let input = r"
        const int32 SMALL_NUM = 100;
        const octet BYTE = SMALL_NUM;
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors - 100 fits in octet
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_const_to_smaller_type_overflow() {
    // This tests assigning a constant that would wrap/overflow
    let input = r"
        const int32 BIG_NUM = 256;
        const octet SMALL = BIG_NUM;
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // In IDL/C semantics, this is allowed (wraps to 0)
    // The constant reference is preserved
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );

    // Verify SMALL has a Const reference to BIG_NUM
    for (_, def) in &result.context.definitions {
        if def.ident.name == "SMALL" {
            if let ic_hir::hir::DefKind::Const(const_ty) = &def.kind {
                assert!(
                    matches!(const_ty.value, ic_hir::hir::Numeric::Const(_)),
                    "Expected Const reference, got {:?}",
                    const_ty.value
                );
            }
        }
    }
}
