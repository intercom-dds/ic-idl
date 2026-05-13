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
        const string myStrValue = "hello world";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::String(s) | Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "hello world");
    } else {
        panic!("Expected const with string value");
    }
}

#[test]
fn test_wide_string_literal_assigned_to_wstring() {
    let input = r#"
        const wstring myWStrValue = L"wide string";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "wide string");
    } else {
        panic!("Expected const with wstring value");
    }
}

#[test]
fn test_string_literal_with_escapes() {
    let input = r#"
        const string myEscValue = "hello\nworld\ttab";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::String(s) | Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "hello\nworld\ttab");
    } else {
        panic!("Expected const with string value");
    }
}

#[test]
fn test_adjacent_string_literal_concatenation() {
    let input = r#"
        const string myValue = "hello" "world";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::String(s) | Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "helloworld");
    } else {
        panic!("Expected const with string value");
    }
}

#[test]
fn test_multiple_adjacent_string_literals() {
    let input = r#"
        const string myValue = "one" "two" "three";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::String(s) | Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "onetwothree");
    } else {
        panic!("Expected const with string value");
    }
}

#[test]
fn test_string_literal_concatenation_with_spaces() {
    let input = r#"
        const string myValue = "hello "   "world";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::String(s) | Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "hello world");
    } else {
        panic!("Expected const with string value");
    }
}

#[test]
fn test_string_literal_concatenation_with_escapes() {
    let input = r#"
        const string myValue = "hello\n" "world\t" "!";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind
        && let Numeric::String(s) | Numeric::WString(s) = &const_ty.value
    {
        assert_eq!(s, "hello\nworld\t!");
    } else {
        panic!("Expected const with string value");
    }
}
