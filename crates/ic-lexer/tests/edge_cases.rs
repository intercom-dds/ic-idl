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

use ic_lexer::cursor::Cursor;
use ic_lexer::token::{Base, Kind, Token};
use ic_vfs::SourceMap;

fn scan(input: &str) -> Vec<Token> {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(input);
    let src = vfs.source(id);
    let mut cursor = Cursor::new(src, id);

    let mut tokens = vec![];
    while let Some(t) = cursor.next() {
        tokens.push(t);
    }
    tokens
}

fn kinds(input: &str) -> Vec<Kind> {
    scan(input).into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_float_with_signs() {
    assert_eq!(kinds("1.5e+10"), vec![Kind::Float]);
    assert_eq!(kinds("1.5e-10"), vec![Kind::Float]);
    assert_eq!(kinds("1.5E+10"), vec![Kind::Float]);
    assert_eq!(kinds("1.5E-10"), vec![Kind::Float]);
    assert_eq!(kinds("2e+5"), vec![Kind::Float]);
    assert_eq!(kinds("2e-5"), vec![Kind::Float]);
    assert_eq!(kinds("2E+5"), vec![Kind::Float]);
    assert_eq!(kinds("2E-5"), vec![Kind::Float]);
}

#[test]
fn test_hex_only_after_zero() {
    // Only 0x should be hex
    assert_eq!(
        kinds("0x123"),
        vec![Kind::Number {
            base: Base::Hexadecimal
        }]
    );
    assert_eq!(
        kinds("0X123"),
        vec![Kind::Number {
            base: Base::Hexadecimal
        }]
    );

    // 1x, 2x, etc. should not be hex
    assert_eq!(
        kinds("1x123"),
        vec![
            Kind::Number {
                base: Base::Decimal
            },
            Kind::Ident
        ]
    );
    assert_eq!(
        kinds("2X456"),
        vec![
            Kind::Number {
                base: Base::Decimal
            },
            Kind::Ident
        ]
    );
    assert_eq!(
        kinds("9xABC"),
        vec![
            Kind::Number {
                base: Base::Decimal
            },
            Kind::Ident
        ]
    );
}

#[test]
fn test_string_escape_sequences() {
    // Test various escape sequences
    assert_eq!(
        kinds(r#""foo\nbar""#),
        vec![Kind::String { terminated: true }]
    );
    assert_eq!(
        kinds(r#""foo\tbar""#),
        vec![Kind::String { terminated: true }]
    );
    assert_eq!(
        kinds(r#""foo\\bar""#),
        vec![Kind::String { terminated: true }]
    );
    assert_eq!(
        kinds(r#""foo\'bar""#),
        vec![Kind::String { terminated: true }]
    );
    assert_eq!(
        kinds(r#""foo\rbar""#),
        vec![Kind::String { terminated: true }]
    );

    // Test escaped backslash at end
    assert_eq!(kinds(r#""foo\\""#), vec![Kind::String { terminated: true }]);
}

#[test]
fn test_unterminated_block_comment() {
    // Should not panic on unterminated block comment
    let tokens = kinds("/* unterminated block comment");
    assert_eq!(tokens.len(), 0); // Comment is consumed but not emitted

    let tokens = kinds("/** unterminated doc comment");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Kind::Comment { trailing: false });
}

#[test]
fn test_nested_escapes() {
    // Multiple consecutive escapes
    assert_eq!(
        kinds(r#""\\\\\\\\""#),
        vec![Kind::String { terminated: true }]
    );

    // Escape at end of string
    assert_eq!(
        kinds(r#""test\""#),
        vec![Kind::String { terminated: false }]
    ); // Escapes the closing quote
}

#[test]
fn test_edge_case_identifiers() {
    // Identifiers with numbers
    assert_eq!(kinds("foo123"), vec![Kind::Ident]);
    assert_eq!(kinds("_foo"), vec![Kind::Ident]);
    assert_eq!(kinds("_123"), vec![Kind::Ident]);
    assert_eq!(kinds("__"), vec![Kind::Ident]);

    // Not identifiers
    assert_eq!(
        kinds("123foo"),
        vec![
            Kind::Number {
                base: Base::Decimal
            },
            Kind::Ident
        ]
    );
}

#[test]
fn test_consecutive_operators() {
    assert_eq!(kinds("++"), vec![Kind::Plus, Kind::Plus]);
    assert_eq!(kinds("--"), vec![Kind::Minus, Kind::Minus]);
    assert_eq!(kinds("**"), vec![Kind::Star, Kind::Star]);
    assert_eq!(kinds("///"), vec![Kind::Comment { trailing: false }]); // Doc comment
    assert_eq!(kinds("////"), vec![Kind::Comment { trailing: false }]); // Still doc comment
}

#[test]
fn test_whitespace_handling() {
    // Various whitespace should be consumed
    assert_eq!(kinds("  \t  \r  foo  \t  "), vec![Kind::Ident]);

    // Newlines are tokens
    assert_eq!(
        kinds("foo\nbar"),
        vec![Kind::Ident, Kind::Newline, Kind::Ident]
    );
    assert_eq!(
        kinds("foo\r\nbar"),
        vec![Kind::Ident, Kind::Newline, Kind::Ident]
    );
}

#[test]
fn test_octal_edge_cases() {
    // Valid octals
    assert_eq!(kinds("0"), vec![Kind::Number { base: Base::Octal }]);
    assert_eq!(kinds("0777"), vec![Kind::Number { base: Base::Octal }]);
    assert_eq!(kinds("0123"), vec![Kind::Number { base: Base::Octal }]);

    // Invalid octal digits should still be parsed as octal
    // (error handling would be done at a higher level)
    assert_eq!(kinds("0999"), vec![Kind::Number { base: Base::Octal }]);
}

#[test]
fn test_empty_char_literal() {
    assert_eq!(kinds("''"), vec![Kind::Char]);
}

#[test]
fn test_multiline_strings() {
    // String interrupted by newline - newline terminates the string
    let tokens = kinds("\"foo\nbar\"");
    // Should be: unterminated string, identifier, unterminated string (orphaned quote)
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Kind::String { terminated: false });
    assert_eq!(tokens[1], Kind::Ident); // "bar"
    assert_eq!(tokens[2], Kind::String { terminated: false }); // orphaned closing quote
}

#[test]
fn test_special_float_cases() {
    // Just a dot after number
    assert_eq!(kinds("123."), vec![Kind::Float]);

    // Multiple dots
    assert_eq!(
        kinds("1.2.3"),
        vec![
            Kind::Float,
            Kind::Period,
            Kind::Number {
                base: Base::Decimal
            }
        ]
    );

    // No digits after e
    assert_eq!(kinds("1e"), vec![Kind::Float]);
    assert_eq!(kinds("1e+"), vec![Kind::Float]);
    assert_eq!(kinds("1e-"), vec![Kind::Float]);
}

#[test]
fn test_many_slashes_in_comment() {
    // Regression test: many slashes in a line comment should not cause infinite loop.
    // The comment body contains characters that look like comment starters.
    let input = "///////////////////////////////////////////////";
    let tokens = kinds(input);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Kind::Comment { trailing: false });

    // Even more slashes
    let input = "//".to_string() + &"/".repeat(1000);
    let tokens = kinds(&input);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Kind::Comment { trailing: false });

    // Mixed content after slashes
    let input = "/// some text /// more slashes /// end";
    let tokens = kinds(input);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Kind::Comment { trailing: false });
}
