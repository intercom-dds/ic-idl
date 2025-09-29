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

//! Test that constant evaluation failures produce appropriate diagnostics.

mod common;

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn test_const_eval_failure_with_diagnostic() {
    // Test a case that should fail evaluation but still produce a diagnostic
    let input = r"
        const long x = 1 / 0;  // Division by zero
    ";

    let (result, _, _warnings) = common::parse_and_resolve(input);

    // Should have an error about division by zero
    assert!(
        !result.errors.is_empty(),
        "Expected error for division by zero"
    );
    assert!(
        result.errors.iter().any(|e| e.to_string().contains("zero")),
        "Expected division by zero error"
    );
}

#[test]
fn test_const_eval_empty_sequence() {
    // Test that empty sequences work correctly
    let input = r"
        struct Foo {
            long value;
        };
        
        const sequence<Foo> x = {};
    ";

    let (result, _, _warnings) = common::parse_and_resolve(input);

    // Should work without errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );

    // Let's verify the constant has a proper value
    let const_def_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            matches!(&def.kind, DefKind::Const(_)) && def.ident.name == "x"
        })
        .expect("constant x not found");

    let const_def = result.context.definitions.get(*const_def_id);
    if let DefKind::Const(const_ty) = &const_def.kind {
        match &const_ty.value {
            Numeric::Sequence { values, .. } => {
                assert_eq!(values.len(), 0, "Empty sequence should have 0 values");
            }
            _ => panic!("Expected sequence value, got {:?}", const_ty.value),
        }
    }
}

#[test]
fn test_const_eval_failure_undefined_ref() {
    let input = r"
        // Reference to undefined constant
        const long x = UNDEFINED_CONSTANT;
    ";

    let (result, _, _warnings) = common::parse_and_resolve(input);

    // Should have an error about undefined reference
    assert!(
        !result.errors.is_empty(),
        "Expected error for undefined reference"
    );

    // The constant should still be created but with appropriate error handling
    let const_def_id = result.order.iter().find(|&&def_id| {
        let def = result.context.definitions.get(def_id);
        matches!(&def.kind, DefKind::Const(_)) && def.ident.name == "x"
    });

    assert!(
        const_def_id.is_some(),
        "Constant should still be created even with eval failure"
    );
}

#[test]
fn test_const_eval_failure_type_mismatch() {
    let input = r#"
        // String literal for numeric type
        const long x = "hello";
    "#;

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_struct_init_forward_ref() {
    let input = r"
        struct Bar;  // Forward declaration
        
        const Bar x = { .value = 42 };
        
        struct Bar {
            long value;
        };
    ";

    let (result, _, _warnings) = common::parse_and_resolve(input);

    // Currently, evaluation happens before forward references are resolved, so this fails
    // and we should get a diagnostic
    assert!(
        !result.errors.is_empty(),
        "Expected error for forward declaration"
    );

    let const_def_id = result.order.iter().find(|&&def_id| {
        let def = result.context.definitions.get(def_id);
        matches!(&def.kind, DefKind::Const(_)) && def.ident.name == "x"
    });

    if let Some(&const_def_id) = const_def_id {
        let const_def = result.context.definitions.get(const_def_id);
        if let DefKind::Const(const_ty) = &const_def.kind {
            match &const_ty.value {
                Numeric::Null => {} // Expected - evaluation failed due to forward ref
                _ => panic!(
                    "Expected Null value due to forward ref, got {:?}",
                    const_ty.value
                ),
            }
        }
    }
}
