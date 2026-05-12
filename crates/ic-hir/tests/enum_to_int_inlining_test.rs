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

//! Regression tests for IC-283: enum values assigned to non-enum numeric
//! consts should be inlined as numeric literals so downstream backends never
//! see an enum reference where they expect an integer or float.

mod common;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefKind, Numeric};

fn find_const_value(result: &ResolvedGraph, name: &str) -> Numeric {
    for (_, def) in &result.context.definitions {
        if def.ident.name == name
            && let DefKind::Const(const_ty) = &def.kind
        {
            return const_ty.value.clone();
        }
    }
    panic!("const '{name}' not found");
}

#[test]
fn enum_inlined_into_long_const() {
    let input = r"
        enum Color { RED };
        const long MY_COLOR = RED;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(find_const_value(&result, "MY_COLOR"), Numeric::Int32(0));
}

#[test]
fn enum_inlined_into_octet_const() {
    let input = r"
        enum E { A = 1, B = 2 };
        const octet O = B;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(find_const_value(&result, "O"), Numeric::UInt8(2));
}

#[test]
fn enum_inlined_into_int8_const() {
    let input = r"
        enum E { NEG = -3 };
        const int8 X = NEG;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(find_const_value(&result, "X"), Numeric::Int8(-3));
}

#[test]
fn enum_inlined_into_float_const() {
    let input = r"
        enum E { VAL = 7 };
        const float F = VAL;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(find_const_value(&result, "F"), Numeric::Float(7.0));
}

#[test]
fn enum_inlined_into_double_const() {
    let input = r"
        enum E { VAL = 42 };
        const double D = VAL;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(find_const_value(&result, "D"), Numeric::Double(42.0));
}

#[test]
fn same_enum_const_preserves_reference() {
    let input = r"
        enum Color { RED, GREEN };
        const Color C = GREEN;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(
        matches!(find_const_value(&result, "C"), Numeric::Const(_)),
        "same-enum assignment must preserve enum reference"
    );
}

#[test]
fn chained_enum_const_inlined_into_int() {
    let input = r"
        enum Color { RED, GREEN, BLUE };
        const Color VIA = BLUE;
        const int32 N = VIA;
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(find_const_value(&result, "N"), Numeric::Int32(2));
}

#[test]
fn enum_value_out_of_range_for_target_is_rejected() {
    let input = r"
        enum E { BIG = 300 };
        const int8 X = BIG;
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}
