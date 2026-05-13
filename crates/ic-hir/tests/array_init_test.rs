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

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn test_array_init_basic() {
    let input = r"
        const int32 VALUES[3] = { 1, 2, 3 };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    let values = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "VALUES")
        .expect("VALUES constant not found");

    match &values.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Array { values, .. } => {
                assert_eq!(values.len(), 3);
                match (values.first(), values.get(1), values.get(2)) {
                    (
                        Some(&Numeric::Int32(1)),
                        Some(&Numeric::Int32(2)),
                        Some(&Numeric::Int32(3)),
                    ) => {}
                    _ => panic!("Expected [1, 2, 3]"),
                }
            }
            _ => panic!("Expected array initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_array_init_wrong_count() {
    let input = r"
        const int32 VALUES[3] = { 1, 2 };
    ";

    let (result, _, output) = common::parse_and_resolve(input);

    // Should have an error about array size mismatch
    assert!(
        !result.errors.is_empty(),
        "Expected error for array size mismatch"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(output);
}

#[test]
fn test_sequence_init() {
    let input = r#"
        const sequence<string> NAMES = { "Alice", "Bob", "Charlie" };
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    let names = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "NAMES")
        .expect("NAMES constant not found");

    match &names.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Sequence { values, .. } => {
                assert_eq!(values.len(), 3);
                match (values.first(), values.get(1), values.get(2)) {
                    (
                        Some(Numeric::String(s1) | Numeric::WString(s1)),
                        Some(Numeric::String(s2) | Numeric::WString(s2)),
                        Some(Numeric::String(s3) | Numeric::WString(s3)),
                    ) => {
                        assert_eq!(s1, "Alice");
                        assert_eq!(s2, "Bob");
                        assert_eq!(s3, "Charlie");
                    }
                    _ => panic!("Expected string values"),
                }
            }
            _ => panic!("Expected sequence initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_map_init() {
    let input = r#"
        const map<string, int32> AGE_MAP = { 
            { "Alice", 30 }, 
            { "Bob", 25 }, 
            { "Charlie", 35 } 
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    let age_map = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "AGE_MAP")
        .expect("AGE_MAP constant not found");

    match &age_map.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Map {
                entries: values, ..
            } => {
                assert_eq!(values.len(), 3);
                // Check first pair
                match values.first() {
                    Some((Numeric::String(key) | Numeric::WString(key), Numeric::Int32(value))) => {
                        assert_eq!(key, "Alice");
                        assert_eq!(*value, 30);
                    }
                    _ => panic!("Expected (string, int32) pair"),
                }
            }
            _ => panic!("Expected map initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_map_init_missing_pair() {
    let input = r#"
        const map<string, int32> AGE_MAP = {
            { "Alice" },  // Missing value
            { "Bob", 25 }
        };
    "#;

    let (result, _, output) = common::parse_and_resolve(input);

    // Should have an error about missing value in map pair
    assert!(
        !result.errors.is_empty(),
        "Expected error for map pair with missing value"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(output);
}

#[test]
fn test_array_const_to_const_assignment() {
    let input = r"
        const int32 ARRAY1[3] = { 1, 2, 3 };
        const int32 ARRAY2[3] = ARRAY1;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(
        result.errors.is_empty(),
        "Array constant assignment should succeed when dimensions match"
    );

    let array2 = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ARRAY2")
        .expect("ARRAY2 constant not found");

    match &array2.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Const(def_id) => {
                let array1 = result.context.definitions.get(*def_id);
                assert_eq!(array1.ident.name, "ARRAY1");
            }
            _ => panic!("Expected const reference"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_array_const_to_const_assignment_dimension_mismatch() {
    let input = r"
        const int32 ARRAY1[3] = { 1, 2, 3 };
        const int32 ARRAY2[4] = ARRAY1;
    ";

    let (result, _, output) = common::parse_and_resolve(input);
    assert!(
        !result.errors.is_empty(),
        "Array constant assignment should fail when dimensions don't match"
    );

    insta::assert_snapshot!(output);
}
