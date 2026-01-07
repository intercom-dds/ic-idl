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
use ic_lexer::token::{Kind, Token};
use ic_vfs::SourceMap;

fn scan(input: &str) -> Vec<Token> {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(input);
    let src = vfs.source(id);
    let mut cursor = Cursor::new(src, id);

    let mut tokens = vec![];
    while let Some(t) = cursor.advance() {
        tokens.push(t);
    }
    tokens
}

fn kinds(input: &str) -> Vec<Kind> {
    scan(input).into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_invalid_utf8_recovery() {
    // Various invalid/unusual characters should produce Unknown tokens
    // but lexing should continue
    assert_eq!(
        kinds("a § b"),
        vec![Kind::Ident, Kind::Unknown, Kind::Ident]
    );
    assert_eq!(
        kinds("x • y"),
        vec![Kind::Ident, Kind::Unknown, Kind::Ident]
    );
    assert_eq!(
        kinds("foo ¿ bar"),
        vec![Kind::Ident, Kind::Unknown, Kind::Ident]
    );
}

#[test]
fn test_unclosed_constructs() {
    // Unclosed string
    assert_eq!(
        kinds(r#""hello world"#),
        vec![Kind::String { terminated: false }]
    );

    // Unclosed char with multiple characters
    assert_eq!(kinds("'abc"), vec![Kind::Unknown, Kind::Ident]); // ' is unknown, abc is ident
    assert_eq!(
        kinds("'ab'"),
        vec![Kind::Unknown, Kind::Ident, Kind::Unknown]
    ); // 'a is unknown, b is ident, ' is unknown

    // Unclosed block comments are emitted for error reporting
    let tokens = scan("/* comment");
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].kind,
        Kind::Comment {
            trailing: false,
            terminated: false
        }
    );

    let tokens = scan("/** doc comment");
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].kind,
        Kind::Comment {
            trailing: false,
            terminated: false
        }
    );
}

#[test]
fn test_consecutive_invalid_tokens() {
    // Multiple invalid characters in a row
    assert_eq!(
        kinds("§§§"),
        vec![Kind::Unknown, Kind::Unknown, Kind::Unknown]
    );

    // Mixed valid and invalid
    assert_eq!(
        kinds("a§b§c"),
        vec![
            Kind::Ident,
            Kind::Unknown,
            Kind::Ident,
            Kind::Unknown,
            Kind::Ident
        ]
    );
}

#[test]
fn test_lexer_continues_after_errors() {
    // Lexer should continue working after encountering errors
    let tokens = kinds(
        r#"
        struct Foo {
            § invalid
            long x;
            "unterminated string
            float y;
            'z  // invalid char
            double z;
        }
    "#,
    );

    // Check that we still get valid tokens after errors
    assert!(tokens.contains(&Kind::Keyword(ic_lexer::token::Kw::Struct)));
    assert!(tokens.contains(&Kind::Keyword(ic_lexer::token::Kw::Long)));
    assert!(tokens.contains(&Kind::Keyword(ic_lexer::token::Kw::Float)));
    assert!(tokens.contains(&Kind::Keyword(ic_lexer::token::Kw::Double)));
    assert!(tokens.contains(&Kind::Unknown)); // From §
    assert!(tokens.contains(&Kind::String { terminated: false })); // From unterminated string
}

#[test]
fn test_edge_case_operators() {
    assert_eq!(kinds("!"), vec![Kind::Not]);
    assert_eq!(kinds("! ="), vec![Kind::Not, Kind::Eq]);
    assert_eq!(kinds("!="), vec![Kind::NotEq]);
}

#[test]
fn test_span_accuracy_after_errors() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed("foo § bar");
    let src = vfs.source(id);
    let cursor = Cursor::new(src.clone(), id);

    let tokens = scan("foo § bar");
    assert_eq!(tokens.len(), 3);

    // Check that spans are correct
    assert_eq!(cursor.source_of(tokens[0].span), "foo");
    assert_eq!(cursor.source_of(tokens[1].span), "§");
    assert_eq!(cursor.source_of(tokens[2].span), "bar");
}
