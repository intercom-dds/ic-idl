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

fn single(input: &str) -> Kind {
    scan(input).first().unwrap().kind
}

#[test]
fn test_bool_lit() {
    assert_eq!(single("true"), Kind::True);
    assert_eq!(single("false"), Kind::False);
    assert!(matches!(single("fals"), Kind::Ident(_)));
    assert!(matches!(single("tru"), Kind::Ident(_)));
}

// Check that e.g. "truest" evaluates to `Ident("truest")` and not
// `(True, Ident("st"))`
#[test]
fn partial_ident() {
    assert!(matches!(single("truest"), Kind::Ident(_)));
    assert!(matches!(single("untrue"), Kind::Ident(_)));
    assert!(matches!(single("falsely"), Kind::Ident(_)));
    assert!(matches!(single("input"), Kind::Ident(_)));
    assert!(matches!(single("output"), Kind::Ident(_)));
    assert!(matches!(single("in1"), Kind::Ident(_)));
    assert!(matches!(single("out1"), Kind::Ident(_)));
    assert_eq!(single("inout"), Kind::InOut);
}

// #[test]
// fn test_integer_lit() {
//     // octal
//     assert_eq!(scan("0777"), Number::Unsigned(0o777),);
//     assert!(scan("0778").is_err());
//
//     // decimal
//     assert_eq!(scan("0"), Number::Unsigned(0));
//     assert_eq!(scan("999"), Number::Unsigned(999));
//     assert_eq!(scan("1000"), Number::Unsigned(1000));
//     assert!(scan("99F").is_err());
//
//     // hex
//     assert_eq!(scan("0x0"), Number::Unsigned(0));
//     assert_eq!(scan("0xFFF"), Number::Unsigned(0xFFF),);
//     assert!(scan("0xFG").is_err());
//
//     // separation checks
//     assert!(scan("a123").is_err());
//     assert!(scan("123a").is_err());
//     assert_eq!(scan("123 456 789"), Number::Unsigned(123),);
//     assert_eq!(scan("123;456"), Number::Unsigned(123),);
//     assert_eq!(scan("123,456"), Number::Unsigned(123),);
//     assert_eq!(scan("123]]]"), Number::Unsigned(123),);
// }

#[test]
fn test_char_lit() {
    assert_eq!(single("'a'"), Kind::Char(Some('a')));
    assert_eq!(single("'0'"), Kind::Char(Some('0')));
    assert_eq!(single("';'"), Kind::Char(Some(';')));
    assert_eq!(single("'a"), Kind::Invalid);
    assert!(matches!(single("a"), Kind::Ident(_)));
    assert_eq!(single("''"), Kind::Char(None));
    assert_eq!(single(r"'\''"), Kind::Char(Some('\'')));

    let tokens = scan("a'");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].kind, Kind::Ident(_)));
    assert_eq!(tokens[1].kind, Kind::Invalid);
}

#[test]
fn test_string_lit() {
    let input = r#""foo 'bar' baz""#;
    assert_eq!(single(input), Kind::StringLit);

    let input = r#""howdy 🤠""#;
    assert_eq!(single(input), Kind::StringLit);

    assert_eq!(single("\"foo"), Kind::Invalid);

    let tokens = scan("foo\"");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].kind, Kind::Ident(_)));
    assert_eq!(tokens[1].kind, Kind::Invalid);
}

#[test]
fn escaped_string_lit() {
    let input = scan(r#""foo \"bar\" baz""#);
    assert_eq!(input.len(), 1);
    assert_eq!(input[0].kind, Kind::StringLit);
}

// #[test]
// fn test_ident() {
//     assert_eq!(scan("foo123_456"), Token::Ident("foo123_456".to_string()),);
//     assert!(scan("123foo").is_err());
//
//     // escaped keywords
//     assert_eq!(scan("_struct"), Token::Ident("struct".to_string()),);
//     assert_eq!(scan("_string"), Token::Ident("string".to_string()),);
//     assert_eq!(scan("_foo"), Token::Ident("foo".to_string()),);
//     assert_eq!(scan("_123"), Token::Ident("_123".to_string()),);
//
//     // annotations
//     assert_eq!(scan("@foo"), Token::AnnAppl("foo".to_string()),);
//     assert_eq!(scan("@default"), Token::AnnAppl("default".to_string()),);
//     assert_eq!(
//         scan("@annotation"),
//         Token::AnnAppl("annotation".to_string()),
//     );
// }

#[test]
fn invalid_token() {
    assert_eq!(single("?"), Kind::Invalid);

    let tokens = scan("foo?bar");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0].kind, Kind::Ident(_)));
    assert_eq!(tokens[1].kind, Kind::Invalid);
    assert!(matches!(tokens[2].kind, Kind::Ident(_)));
}
