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
fn test_struct_init_basic() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
        };
        
        const Point ORIGIN = { x = 0, y = 0 };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Find the constant
    let origin = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ORIGIN")
        .expect("ORIGIN constant not found");

    match &origin.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Struct { fields, .. } => {
                assert_eq!(fields.len(), 2);
                // Fields are stored in struct member declaration order (x, y)
                match (&fields[0], &fields[1]) {
                    (Numeric::Int32(0), Numeric::Int32(0)) => {}
                    _ => panic!("Expected both fields to be 0"),
                }
            }
            _ => panic!("Expected struct initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_struct_init_with_strings() {
    let input = r#"
        struct Config {
            string name;
            int32 port;
            boolean enabled;
        };
        
        const Config DEFAULT_CONFIG = { 
            name= "localhost", 
            port= 8080,
            enabled= true
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    let config = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "DEFAULT_CONFIG")
        .expect("DEFAULT_CONFIG constant not found");

    match &config.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Struct { fields, .. } => {
                assert_eq!(fields.len(), 3);
                // Fields are in struct member declaration order: name, port, enabled
                match &fields[0] {
                    Numeric::String(s) | Numeric::WString(s) => assert_eq!(s, "localhost"),
                    _ => panic!("Expected string for name field"),
                }
                match &fields[1] {
                    Numeric::Int32(8080) => {}
                    _ => panic!("Expected 8080 for port field"),
                }
                match &fields[2] {
                    Numeric::Bool(true) => {}
                    _ => panic!("Expected true for enabled field"),
                }
            }
            _ => panic!("Expected struct initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_struct_init_positional() {
    let input = r"
        struct Vec3 {
            float x;
            float y;
            float z;
        };
        
        const Vec3 UNIT_X = { 1.0, 0.0, 0.0 };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    let unit_x = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "UNIT_X")
        .expect("UNIT_X constant not found");

    match &unit_x.1.kind {
        DefKind::Const(const_ty) => {
            match &const_ty.value {
                Numeric::Struct { fields, .. } => {
                    assert_eq!(fields.len(), 3);
                    // Fields are in struct member declaration order: x, y, z
                    match (&fields[0], &fields[1], &fields[2]) {
                        (Numeric::Float(x), Numeric::Float(y), Numeric::Float(z)) => {
                            assert!((x - 1.0).abs() < f32::EPSILON);
                            assert!(y.abs() < f32::EPSILON);
                            assert!(z.abs() < f32::EPSILON);
                        }
                        _ => panic!("Expected float values"),
                    }
                }
                _ => panic!("Expected struct initialization"),
            }
        }
        _ => panic!("Expected constant definition"),
    }
}

#[test]
#[ignore = "Field order validation not yet implemented"]
fn test_struct_init_field_order_error() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
        };
        
        const Point BAD = { y= 1, x= 2 };  // Wrong order
    ";

    let (result, _, output) = common::parse_and_resolve(input);

    // Should have an error about field order
    assert!(
        !result.errors.is_empty(),
        "Expected error for out-of-order struct fields"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(output);
}

#[test]
fn test_struct_init_missing_field_error() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
        };
        
        const Point INCOMPLETE = { x= 1 };  // Missing y
    ";

    let (result, _, output) = common::parse_and_resolve(input);

    // Should have an error about missing field
    assert!(
        !result.errors.is_empty(),
        "Expected error for missing struct field"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(output);
}

#[test]
fn test_struct_init_extra_field_error() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
        };
        
        const Point EXTRA = { x= 1, y= 2, z= 3 };  // Extra field z
    ";

    let (result, _, output) = common::parse_and_resolve(input);

    // Should have an error about extra field
    assert!(
        !result.errors.is_empty(),
        "Expected error for extra struct field"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(output);
}

#[test]
fn test_struct_init_duplicate_field_error() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
        };
        
        const Point DUP = { x = 1, y = 2, x = 3 };
    ";

    let (result, _, output) = common::parse_and_resolve(input);

    assert!(
        !result.errors.is_empty(),
        "Expected error for duplicate struct field"
    );

    insta::assert_snapshot!(output);
}

#[test]
fn test_struct_init_bad_field_value_error() {
    let input = r"
        struct Point {
            uint8 x;
            uint8 y;
        };

        const Point BAD = {
            x = 1,
            y = 256
        };
    ";

    let (result, _, output) = common::parse_and_resolve(input);
    assert_eq!(result.errors.len(), 1, "{output}");
    insta::assert_snapshot!(output);
}
