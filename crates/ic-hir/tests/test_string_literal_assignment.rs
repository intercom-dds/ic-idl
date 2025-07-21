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
fn test_string_literal_assigned_to_string() {
    let input = r#"
        const string value = "hello world";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    // Verify the constant has the correct string value
    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind {
        match &const_ty.value {
            Numeric::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string value, got {:?}", const_ty.value),
        }
    } else {
        panic!("Expected const definition");
    }
}

#[test]
fn test_wide_string_literal_assigned_to_wstring() {
    let input = r#"
        const wstring value = "wide string";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    // Verify the constant has the correct string value
    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind {
        match &const_ty.value {
            Numeric::String(s) => assert_eq!(s, "wide string"),
            _ => panic!("Expected string value, got {:?}", const_ty.value),
        }
    } else {
        panic!("Expected const definition");
    }
}

#[test]
fn test_string_literal_with_escapes() {
    let input = r#"
        const string value = "hello\nworld\ttab";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    // Verify the constant has the correct string value with escapes
    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind {
        match &const_ty.value {
            Numeric::String(s) => assert_eq!(s, "hello\\nworld\\ttab"),
            _ => panic!("Expected string value, got {:?}", const_ty.value),
        }
    } else {
        panic!("Expected const definition");
    }
}
