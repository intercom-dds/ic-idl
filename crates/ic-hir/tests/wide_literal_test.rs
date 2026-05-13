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

fn first_const_value(input: &str) -> Numeric {
    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);
    let def = result.context.definitions.get(result.order[0]);
    let DefKind::Const(const_ty) = &def.kind else {
        panic!("Expected const definition");
    };
    const_ty.value.clone()
}

#[test]
fn narrow_string_literal_lowers_to_narrow_numeric() {
    let value = first_const_value(r#"const string s = "hello";"#);
    assert!(
        matches!(value, Numeric::String(ref v) if v == "hello"),
        "got {value:?}"
    );
}

#[test]
fn wide_string_literal_lowers_to_wide_numeric() {
    let value = first_const_value(r#"const wstring s = L"hello";"#);
    assert!(
        matches!(value, Numeric::WString(ref v) if v == "hello"),
        "got {value:?}"
    );
}

#[test]
fn narrow_char_literal_lowers_to_narrow_numeric() {
    let value = first_const_value(r"const char c = 'a';");
    assert!(matches!(value, Numeric::Char('a')), "got {value:?}");
}

#[test]
fn wide_char_literal_lowers_to_wide_numeric() {
    let value = first_const_value(r"const wchar c = L'a';");
    assert!(matches!(value, Numeric::WChar('a')), "got {value:?}");
}

#[test]
fn wide_char_literal_rejected_for_char() {
    let diags = common::parse_and_expect_errors(r"const char c = L'a';");
    insta::assert_snapshot!(diags);
}

#[test]
fn narrow_char_literal_rejected_for_wchar() {
    let diags = common::parse_and_expect_errors(r"const wchar c = 'a';");
    insta::assert_snapshot!(diags);
}

#[test]
fn narrow_string_literal_rejected_for_wstring() {
    let diags = common::parse_and_expect_errors(r#"const wstring s = "narrow";"#);
    insta::assert_snapshot!(diags);
}

#[test]
fn wide_string_literal_rejected_for_string() {
    let diags = common::parse_and_expect_errors(r#"const string s = L"wide";"#);
    insta::assert_snapshot!(diags);
}

#[test]
fn integer_literal_still_casts_to_char() {
    let value = first_const_value(r"const char c = 0x41;");
    assert!(matches!(value, Numeric::Char('A')), "got {value:?}");
}

#[test]
fn integer_literal_still_casts_to_wchar() {
    let value = first_const_value(r"const wchar c = 0x41;");
    assert!(matches!(value, Numeric::WChar('A')), "got {value:?}");
}

#[test]
fn adjacent_wide_strings_concatenate() {
    let value = first_const_value(r#"const wstring s = L"foo" L"bar";"#);
    assert!(
        matches!(value, Numeric::WString(ref v) if v == "foobar"),
        "got {value:?}"
    );
}
