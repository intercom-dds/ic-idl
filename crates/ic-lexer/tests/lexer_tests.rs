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
use ic_lexer::token::{Base, Kind, Kw, Token};
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
fn test_all_keywords() {
    assert_eq!(kinds("any"), vec![Kind::Ident]);
    assert_eq!(kinds("module"), vec![Kind::Keyword(Kw::Module)]);
    assert_eq!(kinds("struct"), vec![Kind::Keyword(Kw::Struct)]);
    assert_eq!(kinds("const"), vec![Kind::Keyword(Kw::Const)]);
    assert_eq!(kinds("bitmask"), vec![Kind::Keyword(Kw::Bitmask)]);
    assert_eq!(kinds("bitset"), vec![Kind::Keyword(Kw::Bitset)]);
    assert_eq!(kinds("bitfield"), vec![Kind::Keyword(Kw::Bitfield)]);
    assert_eq!(kinds("enum"), vec![Kind::Keyword(Kw::Enum)]);
    assert_eq!(kinds("exception"), vec![Kind::Keyword(Kw::Exception)]);
    assert_eq!(kinds("typedef"), vec![Kind::Keyword(Kw::Typedef)]);
    assert_eq!(kinds("native"), vec![Kind::Keyword(Kw::Native)]);
    assert_eq!(kinds("fixed"), vec![Kind::Keyword(Kw::Fixed)]);
    assert_eq!(kinds("union"), vec![Kind::Keyword(Kw::Union)]);
    assert_eq!(kinds("switch"), vec![Kind::Keyword(Kw::Switch)]);
    assert_eq!(kinds("case"), vec![Kind::Keyword(Kw::Case)]);
    assert_eq!(kinds("default"), vec![Kind::Keyword(Kw::Default)]);
    assert_eq!(kinds("null"), vec![Kind::Keyword(Kw::Null)]);
    assert_eq!(kinds("valuetype"), vec![Kind::Keyword(Kw::Valuetype)]);
    assert_eq!(kinds("public"), vec![Kind::Keyword(Kw::Public)]);
    assert_eq!(kinds("private"), vec![Kind::Keyword(Kw::Private)]);
    assert_eq!(kinds("supports"), vec![Kind::Keyword(Kw::Supports)]);
    assert_eq!(kinds("factory"), vec![Kind::Keyword(Kw::Factory)]);
    assert_eq!(kinds("local"), vec![Kind::Keyword(Kw::Local)]);
    assert_eq!(kinds("interface"), vec![Kind::Keyword(Kw::Interface)]);
    assert_eq!(kinds("raises"), vec![Kind::Keyword(Kw::Raises)]);
    assert_eq!(kinds("getraises"), vec![Kind::Keyword(Kw::GetRaises)]);
    assert_eq!(kinds("setraises"), vec![Kind::Keyword(Kw::SetRaises)]);
    assert_eq!(kinds("attribute"), vec![Kind::Keyword(Kw::Attribute)]);
    assert_eq!(kinds("readonly"), vec![Kind::Keyword(Kw::ReadOnly)]);
    assert_eq!(kinds("oneway"), vec![Kind::Keyword(Kw::Oneway)]);
    assert_eq!(kinds("in"), vec![Kind::Keyword(Kw::In)]);
    assert_eq!(kinds("out"), vec![Kind::Keyword(Kw::Out)]);
    assert_eq!(kinds("inout"), vec![Kind::Keyword(Kw::InOut)]);
    assert_eq!(kinds("map"), vec![Kind::Keyword(Kw::Map)]);
    assert_eq!(kinds("sequence"), vec![Kind::Keyword(Kw::Sequence)]);
    assert_eq!(kinds("string"), vec![Kind::Keyword(Kw::String)]);
    assert_eq!(kinds("wstring"), vec![Kind::Keyword(Kw::WString)]);
    assert_eq!(kinds("unsigned"), vec![Kind::Keyword(Kw::Unsigned)]);
    assert_eq!(kinds("short"), vec![Kind::Keyword(Kw::Short)]);
    assert_eq!(kinds("long"), vec![Kind::Keyword(Kw::Long)]);
    assert_eq!(kinds("float"), vec![Kind::Keyword(Kw::Float)]);
    assert_eq!(kinds("double"), vec![Kind::Keyword(Kw::Double)]);
    assert_eq!(kinds("TRUE"), vec![Kind::Keyword(Kw::True)]);
    assert_eq!(kinds("true"), vec![Kind::Keyword(Kw::True)]);
    assert_eq!(kinds("FALSE"), vec![Kind::Keyword(Kw::False)]);
    assert_eq!(kinds("false"), vec![Kind::Keyword(Kw::False)]);
}

#[test]
fn test_operators() {
    assert_eq!(
        kinds("+ - * / %"),
        vec![
            Kind::Plus,
            Kind::Minus,
            Kind::Star,
            Kind::Slash,
            Kind::Modulo
        ]
    );
    assert_eq!(
        kinds("< > <= >= == !="),
        vec![
            Kind::Lt,
            Kind::Gt,
            Kind::LtEq,
            Kind::GtEq,
            Kind::EqEq,
            Kind::NotEq
        ]
    );
    assert_eq!(
        kinds("& | ^ ~"),
        vec![Kind::BitAnd, Kind::BitOr, Kind::BitXor, Kind::BitNot]
    );
    assert_eq!(kinds("&& ||"), vec![Kind::And, Kind::Or]);
}

#[test]
fn test_delimiters() {
    assert_eq!(
        kinds("()[]{}"),
        vec![
            Kind::LParen,
            Kind::RParen,
            Kind::LBracket,
            Kind::RBracket,
            Kind::LBrace,
            Kind::RBrace
        ]
    );
    assert_eq!(
        kinds(", . ; : ::"),
        vec![
            Kind::Comma,
            Kind::Period,
            Kind::Semi,
            Kind::Colon,
            Kind::DColon
        ]
    );
}

#[test]
fn test_float_literals() {
    assert_eq!(kinds("3.14"), vec![Kind::Float]);
    assert_eq!(kinds("0.5"), vec![Kind::Float]);
    assert_eq!(kinds("1e10"), vec![Kind::Float]);
    assert_eq!(kinds("1E10"), vec![Kind::Float]);
    assert_eq!(kinds("1.5e-10"), vec![Kind::Float]);
    assert_eq!(kinds("1.5E+10"), vec![Kind::Float]);
}

#[test]
fn test_edge_case_numbers() {
    // Edge case: leading zeros in octal
    assert_eq!(kinds("0000"), vec![Kind::Number { base: Base::Octal }]);

    // Edge case: hex without digits after 0x
    assert_eq!(
        kinds("0x"),
        vec![Kind::Number {
            base: Base::Hexadecimal
        }]
    );
    assert_eq!(
        kinds("0X"),
        vec![Kind::Number {
            base: Base::Hexadecimal
        }]
    );

    // Edge case: uppercase hex
    assert_eq!(
        kinds("0XABCDEF"),
        vec![Kind::Number {
            base: Base::Hexadecimal
        }]
    );

    // Edge case: float with just dot
    assert_eq!(kinds("1."), vec![Kind::Float]);

    // Edge case: float with e but no exponent
    assert_eq!(kinds("1e"), vec![Kind::Float]);
    assert_eq!(kinds("1E"), vec![Kind::Float]);
}

#[test]
fn test_unterminated_strings() {
    assert_eq!(
        kinds(r#""unterminated"#),
        vec![Kind::String { terminated: false }]
    );
    assert_eq!(
        kinds("\"string with\nnewline"),
        vec![Kind::String { terminated: false }, Kind::Ident]
    );
}

#[test]
fn test_escaped_characters() {
    // Escaped quote in string
    assert_eq!(
        kinds(r#""foo\"bar""#),
        vec![Kind::String { terminated: true }]
    );

    // Escaped quote in char
    assert_eq!(kinds(r"'\''"), vec![Kind::Char]);

    // Multiple escapes
    assert_eq!(kinds(r#""\\\\""#), vec![Kind::String { terminated: true }]);
}

#[test]
fn test_comments() {
    // Regular comments are stripped
    let tokens = scan("// regular comment\nfoo");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, Kind::Newline);
    assert_eq!(tokens[1].kind, Kind::Ident);

    // Doc comments are preserved
    let tokens = scan("/// doc comment\nfoo");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, Kind::Comment { trailing: false });
    assert_eq!(tokens[1].kind, Kind::Newline);
    assert_eq!(tokens[2].kind, Kind::Ident);

    // Alternative doc comment style
    let tokens = scan("//! module doc\nfoo");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, Kind::Comment { trailing: false });
}

#[test]
fn test_block_comments() {
    // Regular block comments are stripped
    let tokens = scan("/* comment */ foo");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, Kind::Ident);

    // Doc block comments are preserved
    let tokens = scan("/** doc comment */ foo");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, Kind::Comment { trailing: false });
    assert_eq!(tokens[1].kind, Kind::Ident);

    // Alternative doc block comment style
    let tokens = scan("/*! doc comment */ foo");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, Kind::Comment { trailing: false });
}

#[test]
fn test_cursor_methods() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed("foo bar baz; qux");
    let src = vfs.source(id);
    let mut cursor = Cursor::new(src.clone(), id);

    // Test peek
    assert_eq!(cursor.peek(), Some(Kind::Ident));

    // Test take_if
    assert!(cursor.take_if(Kind::Ident).is_some());
    assert!(cursor.take_if(Kind::Ident).is_some());
    assert!(cursor.take_if(Kind::Ident).is_some());
    assert!(cursor.take_if(Kind::Ident).is_none()); // Should be Semi
    assert!(cursor.take_if(Kind::Semi).is_some());

    // Test until
    let mut cursor2 = Cursor::new(src.clone(), id);
    let (tokens, _) = cursor2.until(Kind::Semi);
    assert_eq!(tokens.len(), 3); // foo bar baz

    // Test until_newline with escaped newline
    let id2 = vfs.embed("foo \\\nbar\nbaz");
    let src2 = vfs.source(id2);
    let mut cursor3 = Cursor::new(src2, id2);
    let tokens = cursor3.until_newline();
    // Should get foo, newline (from escaped newline), bar
    // The backslash is consumed but not included when it escapes a newline
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, Kind::Ident);
    assert_eq!(tokens[1].kind, Kind::Newline);
    assert_eq!(tokens[2].kind, Kind::Ident);
}

#[test]
fn test_special_tokens() {
    assert_eq!(kinds("@"), vec![Kind::At]);
    assert_eq!(kinds("#"), vec![Kind::Hash]);
    assert_eq!(kinds("?"), vec![Kind::Question]);
    assert_eq!(kinds("\\"), vec![Kind::Backslash]);
    assert_eq!(kinds("!"), vec![Kind::Not]);
    assert_eq!(kinds("!="), vec![Kind::NotEq]);
}

#[test]
fn test_mixed_content() {
    let tokens = kinds("struct Foo { long x = 42; }");
    assert_eq!(
        tokens,
        vec![
            Kind::Keyword(Kw::Struct),
            Kind::Ident,
            Kind::LBrace,
            Kind::Keyword(Kw::Long),
            Kind::Ident,
            Kind::Eq,
            Kind::Number {
                base: Base::Decimal
            },
            Kind::Semi,
            Kind::RBrace,
        ]
    );
}

#[test]
fn test_line_tracking() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed("foo\nbar\nbaz");
    let src = vfs.source(id);
    let mut cursor = Cursor::new(src, id);

    assert_eq!(cursor.line(), 1);
    cursor.next(); // foo
    assert_eq!(cursor.line(), 1);
    cursor.next(); // \n
    assert_eq!(cursor.line(), 2);
    cursor.next(); // bar
    assert_eq!(cursor.line(), 2);
    cursor.next(); // \n
    assert_eq!(cursor.line(), 3);
}

#[test]
fn test_source_of() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed("hello world");
    let src = vfs.source(id);
    let cursor = Cursor::new(src, id);
    let mut cursor2 = cursor.clone();

    let token = cursor2.next().unwrap();
    assert_eq!(cursor.source_of(token.span), "hello");
}

#[test]
fn test_trailing_comments() {
    // Test trailing comment on same line as code
    let tokens = scan("int value; /// trailing doc comment");
    assert_eq!(tokens.len(), 4); // No newline at end
    assert_eq!(tokens[0].kind, Kind::Ident); // int
    assert_eq!(tokens[1].kind, Kind::Ident); // value
    assert_eq!(tokens[2].kind, Kind::Semi);
    assert_eq!(tokens[3].kind, Kind::Comment { trailing: true }); // Trailing comment

    // Test leading comment (not trailing)
    let tokens = scan("/// leading doc comment\nint value;");
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].kind, Kind::Comment { trailing: false }); // Leading comment
    assert_eq!(tokens[1].kind, Kind::Newline);
    assert_eq!(tokens[2].kind, Kind::Ident); // int
    assert_eq!(tokens[3].kind, Kind::Ident); // value
    assert_eq!(tokens[4].kind, Kind::Semi);

    // Test trailing block comment
    let tokens = scan("int value; /** trailing block comment */");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, Kind::Ident); // int
    assert_eq!(tokens[1].kind, Kind::Ident); // value
    assert_eq!(tokens[2].kind, Kind::Semi);
    assert_eq!(tokens[3].kind, Kind::Comment { trailing: true }); // Trailing block comment

    // Test multiple items with trailing comments
    let tokens = scan("int a; /// comment a\nint b; /// comment b");
    assert_eq!(tokens.len(), 9); // No newline at end
    assert_eq!(tokens[3].kind, Kind::Comment { trailing: true }); // First trailing comment
    assert_eq!(tokens[4].kind, Kind::Newline);
    assert_eq!(tokens[8].kind, Kind::Comment { trailing: true }); // Second trailing comment

    // Test comment at beginning of line (not trailing)
    let tokens = scan("    /// indented comment\n    int value;");
    assert_eq!(tokens[0].kind, Kind::Comment { trailing: false });
}
