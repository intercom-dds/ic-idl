// Copyright 2024 KONGSBERG
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
        const int32 NUMS[3] = {1, 2, 3};
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);

    let nums = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "NUMS")
        .expect("NUMS constant not found");

    match &nums.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Array { values, .. } => {
                assert_eq!(values.len(), 3);
                match (values[0].clone(), values[1].clone(), values[2].clone()) {
                    (Numeric::Int32(1), Numeric::Int32(2), Numeric::Int32(3)) => {}
                    _ => panic!("Expected array values 1, 2, 3"),
                }
            }
            _ => panic!("Expected array initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_sequence_init() {
    let input = r#"
        const sequence<string> NAMES = {"Alice", "Bob", "Charlie"};
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);

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
                let expected = ["Alice", "Bob", "Charlie"];
                for (i, value) in values.iter().enumerate() {
                    match value {
                        Numeric::String(s) | Numeric::WString(s) => assert_eq!(s, expected[i]),
                        _ => panic!("Expected string value"),
                    }
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
        const map<string, int32> AGES = {
            {"Alice", 30},
            {"Bob", 25},
            {"Charlie", 35}
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);

    let ages = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "AGES")
        .expect("AGES constant not found");

    match &ages.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Map {
                entries: values, ..
            } => {
                assert_eq!(values.len(), 3);

                let expected = [("Alice", 30), ("Bob", 25), ("Charlie", 35)];
                for (i, (key, value)) in values.iter().enumerate() {
                    match key {
                        Numeric::String(k) | Numeric::WString(k) => assert_eq!(k, expected[i].0),
                        _ => panic!("Expected string key"),
                    }
                    match value {
                        Numeric::Int32(v) => assert_eq!(*v, expected[i].1),
                        _ => panic!("Expected int32 value"),
                    }
                }
            }
            _ => panic!("Expected map initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_nested_collections() {
    let input = r"
        const int32 MATRIX[2][3] = {{1, 2, 3}, {4, 5, 6}};
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);

    let matrix = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "MATRIX")
        .expect("MATRIX constant not found");

    match &matrix.1.kind {
        DefKind::Const(const_ty) => {
            match &const_ty.value {
                Numeric::Array { values, .. } => {
                    assert_eq!(values.len(), 2);

                    // Check first row
                    match &values[0] {
                        Numeric::Array { values: row1, .. } => {
                            assert_eq!(row1.len(), 3);
                            match (row1[0].clone(), row1[1].clone(), row1[2].clone()) {
                                (Numeric::Int32(1), Numeric::Int32(2), Numeric::Int32(3)) => {}
                                _ => panic!("Expected first row values 1, 2, 3"),
                            }
                        }
                        _ => panic!("Expected nested array"),
                    }

                    // Check second row
                    match &values[1] {
                        Numeric::Array { values: row2, .. } => {
                            assert_eq!(row2.len(), 3);
                            match (row2[0].clone(), row2[1].clone(), row2[2].clone()) {
                                (Numeric::Int32(4), Numeric::Int32(5), Numeric::Int32(6)) => {}
                                _ => panic!("Expected second row values 4, 5, 6"),
                            }
                        }
                        _ => panic!("Expected nested array"),
                    }
                }
                _ => panic!("Expected array initialization"),
            }
        }
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_array_size_mismatch_error() {
    let input = r"
        const int32 NUMS[3] = {1, 2};  // Too few elements
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have an error about array size mismatch
    assert!(
        !result.errors.is_empty(),
        "Expected error for array size mismatch"
    );
}

#[test]
fn test_empty_collections() {
    let input = r"
        const sequence<string> EMPTY_SEQ = {};
        const map<int32, string> EMPTY_MAP = {};
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);

    // Check empty sequence
    let empty_seq = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "EMPTY_SEQ")
        .expect("EMPTY_SEQ not found");

    match &empty_seq.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Sequence { values, .. } => {
                assert_eq!(values.len(), 0);
            }
            _ => panic!("Expected sequence"),
        },
        _ => panic!("Expected constant"),
    }

    // Check empty map
    let empty_map = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "EMPTY_MAP")
        .expect("EMPTY_MAP not found");

    match &empty_map.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Map {
                entries: values, ..
            } => {
                assert_eq!(values.len(), 0);
            }
            _ => panic!("Expected map"),
        },
        _ => panic!("Expected constant"),
    }
}
