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

use ic_syntax::{Expr, Item, LiteralValue};

fn parse_char_literal(input: &str) -> Option<char> {
    let full_input = format!("const char c = {input};");
    let result = ic_parse::from_str(&full_input);

    if !result.errors.is_empty() {
        return None;
    }

    // Extract the character value from the parsed AST
    for item in &result.tree {
        if let Item::ConstValue(c) = item
            && let Expr::Literal(lit) = &c.value
            && let LiteralValue::Char(ch) = &lit.value
        {
            return Some(*ch);
        }
    }
    None
}

#[test]
fn test_simple_chars() {
    assert_eq!(parse_char_literal("'a'"), Some('a'));
    assert_eq!(parse_char_literal("'Z'"), Some('Z'));
    assert_eq!(parse_char_literal("'0'"), Some('0'));
    assert_eq!(parse_char_literal("'9'"), Some('9'));
    assert_eq!(parse_char_literal("' '"), Some(' '));
    assert_eq!(parse_char_literal("'!'"), Some('!'));
}

#[test]
fn test_escape_sequences() {
    assert_eq!(parse_char_literal(r"'\n'"), Some('\n'));
    assert_eq!(parse_char_literal(r"'\t'"), Some('\t'));
    assert_eq!(parse_char_literal(r"'\r'"), Some('\r'));
    assert_eq!(parse_char_literal(r"'\0'"), Some('\0'));
    assert_eq!(parse_char_literal(r"'\\'"), Some('\\'));
    assert_eq!(parse_char_literal(r"'\''"), Some('\''));
    assert_eq!(parse_char_literal(r#"'\"'"#), Some('"'));
    assert_eq!(parse_char_literal(r"'\b'"), Some('\u{0008}'));
    assert_eq!(parse_char_literal(r"'\f'"), Some('\u{000C}'));
    assert_eq!(parse_char_literal(r"'\v'"), Some('\u{000B}'));
}

#[test]
fn test_hex_escape_sequences() {
    assert_eq!(parse_char_literal(r"'\x41'"), Some('A'));
    assert_eq!(parse_char_literal(r"'\x42'"), Some('B'));
    assert_eq!(parse_char_literal(r"'\x00'"), Some('\0'));
    assert_eq!(parse_char_literal(r"'\xFF'"), Some('\u{00FF}'));
    assert_eq!(parse_char_literal(r"'\x20'"), Some(' '));
    assert_eq!(parse_char_literal(r"'\x7E'"), Some('~'));
}

#[test]
fn test_empty_char_literal() {
    // Empty char literal is parsed as '\0' (default char value)
    assert_eq!(parse_char_literal("''"), Some('\0'));
}

#[test]
fn test_invalid_char_literals() {
    // Multi-character literals should fail parsing
    assert_eq!(parse_char_literal("'ab'"), None);
    assert_eq!(parse_char_literal("'abc'"), None);

    // Invalid escape sequences
    assert_eq!(parse_char_literal(r"'\x'"), None);
    assert_eq!(parse_char_literal(r"'\xG'"), None);
    assert_eq!(parse_char_literal(r"'\x1'"), None);
}
