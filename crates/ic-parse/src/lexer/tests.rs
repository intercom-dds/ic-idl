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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use super::*;

#[test]
fn test_bool_lit() {
    assert!(bool_lit().parse("true").unwrap());
    assert!(!bool_lit().parse("false").unwrap());
    assert!(bool_lit().parse("fals").is_err());
    assert!(bool_lit().parse("tru").is_err());
}

#[test]
fn test_integer_lit() {
    // octal
    assert_eq!(
        integer_lit().parse("0777").unwrap(),
        Number::Unsigned(0o777),
    );
    assert!(integer_lit().parse("0778").is_err());

    // decimal
    assert_eq!(integer_lit().parse("0").unwrap(), Number::Unsigned(0));
    assert_eq!(integer_lit().parse("999").unwrap(), Number::Unsigned(999));
    assert_eq!(integer_lit().parse("1000").unwrap(), Number::Unsigned(1000));
    assert!(integer_lit().parse("99F").is_err());

    // hex
    assert_eq!(integer_lit().parse("0x0").unwrap(), Number::Unsigned(0));
    assert_eq!(
        integer_lit().parse("0xFFF").unwrap(),
        Number::Unsigned(0xFFF),
    );
    assert!(integer_lit().parse("0xFG").is_err());

    // separation checks
    assert!(integer_lit().parse("a123").is_err());
    assert!(integer_lit().parse("123a").is_err());
    assert_eq!(
        integer_lit().parse("123 456 789").unwrap(),
        Number::Unsigned(123),
    );
    assert_eq!(
        integer_lit().parse("123;456").unwrap(),
        Number::Unsigned(123),
    );
    assert_eq!(
        integer_lit().parse("123,456").unwrap(),
        Number::Unsigned(123),
    );
    assert_eq!(
        integer_lit().parse("123]]]").unwrap(),
        Number::Unsigned(123),
    );
}

#[test]
fn test_char_lit() {
    assert_eq!(char_lit().parse("'a'").unwrap(), 'a');
    assert_eq!(char_lit().parse("'0'").unwrap(), '0');
    assert_eq!(char_lit().parse("';'").unwrap(), ';');
    assert!(char_lit().parse("'a").is_err());
    assert!(char_lit().parse("a").is_err());
    assert!(char_lit().parse("a'").is_err());
    assert!(char_lit().parse("''").is_err());
}

#[test]
fn test_string_lit() {
    let input = "foo 'bar' baz";
    assert_eq!(string_lit().parse(format!("\"{input}\"")).unwrap(), input);

    let input = "howdy 🤠";
    assert_eq!(string_lit().parse(format!("\"{input}\"")).unwrap(), input);

    assert!(string_lit().parse("\"foo").is_err());
    assert!(string_lit().parse("foo\"").is_err());
    assert!(string_lit().parse("foo").is_err());
}

#[test]
#[ignore]
fn escaped_string_lit() {
    let input = "foo \"bar\" baz";
    assert_eq!(string_lit().parse(format!("\"{input}\"")).unwrap(), input);
}

#[test]
fn test_ident() {
    assert_eq!(
        token().parse("foo123_456").unwrap(),
        Token::Ident("foo123_456".to_string()),
    );
    assert!(token().parse("123foo").is_err());

    // escaped keywords
    assert_eq!(
        token().parse("_struct").unwrap(),
        Token::Ident("struct".to_string()),
    );
    assert_eq!(
        token().parse("_string").unwrap(),
        Token::Ident("string".to_string()),
    );
    assert_eq!(
        token().parse("_foo").unwrap(),
        Token::Ident("foo".to_string()),
    );
    assert_eq!(
        token().parse("_123").unwrap(),
        Token::Ident("_123".to_string()),
    );

    // annotations
    assert_eq!(
        token().parse("@foo").unwrap(),
        Token::AnnAppl("foo".to_string()),
    );
    assert_eq!(
        token().parse("@default").unwrap(),
        Token::AnnAppl("default".to_string()),
    );
    assert_eq!(
        token().parse("@annotation").unwrap(),
        Token::AnnAppl("annotation".to_string()),
    );
}

// Check that e.g. "truest" evaluates to `Ident("truest")` and not
// `(Bool("true"), Ident("st"))`
#[test]
fn partial_ident() {
    assert_eq!(
        token().parse("truest").unwrap(),
        Token::Ident("truest".to_string()),
    );
    assert_eq!(
        token().parse("untrue").unwrap(),
        Token::Ident("untrue".to_string()),
    );
    assert_eq!(
        token().parse("falsely").unwrap(),
        Token::Ident("falsely".to_string()),
    );
    assert_eq!(
        token().parse("input").unwrap(),
        Token::Ident("input".to_string()),
    );
    assert_eq!(
        token().parse("output").unwrap(),
        Token::Ident("output".to_string()),
    );
    assert_eq!(token().parse("inout").unwrap(), Token::Inout);
}

#[test]
fn const_dcl() {
    let tokens = scan("const boolean FOO = true;").unwrap();
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens.last().unwrap().0, Token::Ctrl(';'));
}
