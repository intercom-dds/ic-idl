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

use std::rc::Rc;

use ic_vfs::{FileId, Location, Span};

use crate::iter::{OwnedChars, EOF};
use crate::token::{Base, Kind, Kw, Token};

#[must_use]
#[derive(Clone, Debug)]
pub struct Cursor {
    chars: OwnedChars,
    file_id: FileId,
}

impl Cursor {
    pub fn new(source: Rc<str>, file_id: FileId) -> Self {
        let chars = OwnedChars::from(source);
        Cursor { chars, file_id }
    }

    fn span_since(&self, start: u32) -> Span {
        Span {
            start: Location::new(start, self.file_id),
            end: Location::new(self.chars.index(), self.file_id),
        }
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> Span {
        let start = self.chars.index();
        loop {
            let c = self.chars.peek();
            if c == EOF || !predicate(c) {
                break;
            }
            self.chars.next();
        }
        self.span_since(start)
    }

    fn ident(&mut self, start: u32) -> Kind {
        self.eat_while(is_ident);
        let span = self.span_since(start);
        let src = &self.chars.as_str()[span.range()];
        Kw::from_str(src).map_or(Kind::Ident, Kind::Keyword)
    }

    fn number(&mut self, leading: char) -> Kind {
        self.eat_while(|v| v.is_ascii_digit());

        match self.chars.peek() {
            '.' | 'e' | 'E' => {
                _ = self.chars.next();
                self.eat_while(|v| v.is_ascii_digit());
                Kind::Float
            }
            'x' | 'X' => {
                _ = self.chars.next();
                self.eat_while(|v| v.is_ascii_hexdigit());
                Kind::Number {
                    base: Base::Hexadecimal,
                }
            }
            _ => Kind::Number {
                base: if leading == '0' {
                    Base::Octal
                } else {
                    Base::Decimal
                },
            },
        }
    }

    fn string_lit(&mut self) -> Kind {
        let mut terminated = false;
        while let Some(c) = self.chars.next() {
            match c {
                '\\' => {
                    // TODO: should be escape newlines in string literals?
                    if self.chars.peek() == '"' {
                        _ = self.chars.next();
                    }
                }
                '\n' => break,
                '"' => {
                    terminated = true;
                    break;
                }
                _ => (),
            }
        }
        Kind::String { terminated }
    }

    fn char_lit(&mut self) -> Kind {
        if let Some(v) = self.chars.next() {
            if v == '\'' {
                return Kind::Char;
            }
            if v == '\\' && self.chars.peek() == '\'' {
                self.chars.next();
            }
            if self.chars.peek() == '\'' {
                self.chars.next();
                return Kind::Char;
            }
        }
        Kind::Unknown
    }

    // Code comments (`//`) are stripped from the output, but documentation
    // comments (`///`) are not.
    //
    // Returns true if this was a documentation-style comment.
    fn comment(&mut self) -> bool {
        // Consume the leading '/'
        _ = self.chars.next();

        let is_doc = self.chars.peek() == '/';
        _ = self.until_peek(Kind::Newline);
        is_doc
    }

    // Returns true if this was a documentation-style comment.
    fn block_comment(&mut self) -> bool {
        // Consume the leading '/'
        _ = self.chars.next();

        let is_doc = matches!(self.chars.peek(), '*' | '!');
        loop {
            match self.chars.next() {
                Some('*') => {
                    if self.chars.next() == Some('/') {
                        break;
                    }
                }
                None => panic!("unterminated"),
                _ => (),
            }
        }
        is_doc
    }

    fn peek_or(&mut self, c: char, a: Kind, b: Kind) -> Kind {
        if self.chars.peek() == c {
            _ = self.chars.next();
            a
        } else {
            b
        }
    }

    /// Advances the iterator until it finds a token with the specified `kind`.
    pub fn until(&mut self, kind: Kind) -> (Vec<Token>, Span) {
        let mut tokens = vec![];
        let start = self.chars.index();
        while let Some(tok) = self.next() {
            if tok.kind == kind {
                break;
            }
            tokens.push(tok);
        }
        (tokens, self.span_since(start))
    }

    /// Advances the iterator until the next, peeked token is equal to the
    /// specified `kind`.
    #[must_use]
    pub fn until_peek(&mut self, kind: Kind) -> Span {
        let start = self.chars.index();
        while let Some(tok) = self.peek() {
            if tok == kind {
                break;
            }
            self.next();
        }
        self.span_since(start)
    }

    /// Consumes all token until a newline is encountered, but unlike
    /// `Cursor::until`, this accounts for escaped newlines.
    pub fn until_newline(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(tok) = self.next() {
            match tok.kind {
                Kind::Backslash => {
                    // Don't include the bachslash in the macro definition if
                    // it was used to escape a newline.
                    if let Some(next) = self.next() {
                        // An escaped newline followed by a non-escaped newline
                        // counts as an empty macro definition.
                        if next.kind != Kind::Newline {
                            tokens.push(tok);
                        }
                        tokens.push(next);
                    } else {
                        tokens.push(tok);
                    }
                }
                Kind::Newline => break,
                _ => tokens.push(tok),
            }
        }
        tokens
    }

    /// Advances the underlying iterator and yields the next token.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Token> {
        loop {
            let start = self.chars.index();
            let kind = match self.chars.next()? {
                '#' => Kind::Hash,
                ',' => Kind::Comma,
                '.' => Kind::Period,
                ';' => Kind::Semi,
                '{' => Kind::LBrace,
                '}' => Kind::RBrace,
                '[' => Kind::LBracket,
                ']' => Kind::RBracket,
                '(' => Kind::LParen,
                ')' => Kind::RParen,
                '+' => Kind::Plus,
                '-' => Kind::Minus,
                '*' => Kind::Star,
                '%' => Kind::Modulo,
                '?' => Kind::Question,
                '\n' => Kind::Newline,
                '\\' => Kind::Backslash,
                '~' => Kind::BitNot,
                '^' => Kind::BitXor,
                '&' => self.peek_or('&', Kind::And, Kind::BitAnd),
                '|' => self.peek_or('|', Kind::Or, Kind::BitOr),
                '=' => self.peek_or('=', Kind::EqEq, Kind::Eq),
                ':' => self.peek_or(':', Kind::DColon, Kind::Colon),
                '!' => self.peek_or('=', Kind::NotEq, Kind::Unknown),
                '>' => self.peek_or('=', Kind::GtEq, Kind::Gt),
                '<' => self.peek_or('=', Kind::LtEq, Kind::Lt),
                '"' => self.string_lit(),
                '\'' => self.char_lit(),

                '/' => match self.chars.peek() {
                    '/' => {
                        if self.comment() {
                            Kind::Comment
                        } else {
                            continue;
                        }
                    }
                    '*' => {
                        if self.block_comment() {
                            Kind::Comment
                        } else {
                            continue;
                        }
                    }
                    _ => Kind::Slash,
                },

                c if c.is_ascii_digit() => self.number(c),
                c if is_ident(c) => self.ident(start),
                c if c.is_whitespace() => continue,
                _ => Kind::Unknown,
            };

            let span = self.span_since(start);
            break Some(Token { kind, span });
        }
    }

    /// Advances if the iterator if the next, peeked token corresponds is of
    /// type `kind`.
    pub fn take_if(&mut self, kind: Kind) -> Option<Token> {
        if self.peek()? == kind {
            self.next()
        } else {
            None
        }
    }

    /// Returns the source of th given span.
    ///
    /// # Panics
    ///
    /// Panics if the given span does not belong to this cursor.
    #[must_use]
    pub fn source_of(&self, span: Span) -> &str {
        assert_eq!(self.file_id, span.start.file_id, "FileId mismatch");
        &self.chars.as_str()[span.range()]
    }

    /// Returns the ID of the cursor's file.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Returns the current line of the cursor.
    #[must_use]
    pub fn line(&self) -> u32 {
        self.chars.line()
    }

    /// Peeks the next token by cloning the underlying iterator. This is
    /// necessary as we cannot advance the underlying iterator, but we need
    /// `N` lookup to properly parse expressions.
    #[must_use]
    pub fn peek(&self) -> Option<Kind> {
        self.clone().next().map(|v| v.kind)
    }
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '@'
}

#[cfg(test)]
mod tests {
    use ic_vfs::SourceMap;

    use super::*;

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

    fn single(input: &str) -> Kind {
        scan(input).first().copied().unwrap().kind
    }

    #[test]
    fn test_integer_lit() {
        // octal
        assert_eq!(single("0"), Kind::Number { base: Base::Octal });
        assert_eq!(single("0777"), Kind::Number { base: Base::Octal });

        // decimal
        assert_eq!(
            single("999"),
            Kind::Number {
                base: Base::Decimal
            }
        );
        assert_eq!(
            single("1000"),
            Kind::Number {
                base: Base::Decimal
            }
        );

        // hex
        assert_eq!(
            single("0x0"),
            Kind::Number {
                base: Base::Hexadecimal
            }
        );
        assert_eq!(
            single("0xFFF"),
            Kind::Number {
                base: Base::Hexadecimal
            }
        );

        // separation checks
        assert_eq!(scan("a123").len(), 1);
        assert_eq!(scan("123a").len(), 2);
        assert_eq!(scan("123 456 789").len(), 3);
        assert_eq!(scan("123;456").len(), 3);
        assert_eq!(scan("123]]]").len(), 4);
    }

    #[test]
    fn test_char_lit() {
        assert_eq!(single("'a'"), Kind::Char);
        assert_eq!(single("'0'"), Kind::Char);
        assert_eq!(single("';'"), Kind::Char);
        assert_eq!(single("'a"), Kind::Unknown);
        assert_eq!(single("a"), Kind::Ident);
        assert_eq!(single("''"), Kind::Char);

        let escaped = scan(r"'\''");
        assert_eq!(escaped.len(), 1);
        assert_eq!(escaped[0].kind, Kind::Char);

        let tokens = scan("a'");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, Kind::Ident);
        assert_eq!(tokens[1].kind, Kind::Unknown);
    }

    #[test]
    fn test_string_lit() {
        let input = r#""foo 'bar' baz""#;
        assert_eq!(single(input), Kind::String { terminated: true });

        let input = r#""howdy 🤠""#;
        assert_eq!(single(input), Kind::String { terminated: true });
    }

    #[test]
    fn escaped_string_lit() {
        let input = scan(r#""foo \"bar\" baz""#);
        assert_eq!(input.len(), 1);
        assert_eq!(input[0].kind, Kind::String { terminated: true });
    }

    #[test]
    fn invalid_token() {
        assert_eq!(single("§"), Kind::Unknown);

        let tokens = scan("foo§bar");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, Kind::Ident);
        assert_eq!(tokens[1].kind, Kind::Unknown);
        assert_eq!(tokens[2].kind, Kind::Ident);
    }

    #[test]
    fn weird_utf8() {
        let tokens = scan(
            r#"
            Ā Á Ă À
            ā á ă à
            Ǖ Ǘ Ǚ Ǜ
            ǖ ǘ ǚ ǜ
            Ĉ ĉ Ĝ ĝ Ĥ ĥ
            Ĵ ĵ Ŝ ŝ Ŭ ŭ
            Я не говорю по русски
            𠜎𠜱𠝹𠱓 𠱸𠲖𠳏 𠳕𠴕𠵼𠵿𠸎𠸏𠹷 𠺝𠺢𠻗𠻹𠻺𠼭 𠼮𠽌𠾴𠾼𠿪𡁜𡁯𡁵𡁶𡁻𡃁𡃉𡇙𢃇𢞵𢫕
            𢭃 𢯊 𢱑 𢱕 𢳂 𢴈 𢵌 𢵧 𢺳 𣲷 𤓓 𤶸 𤷪 𥄫 𦉘 𦟌 𦧲 𦧺 𧨾 𨅝 𨈇 𨋢 𨳊 𨳍 𨳒 𩶘
            אני לא לומד עברית
            𓂝𓃀𓅡𓄿𓌂 𓋴𓅓𓏏𓇏𓇌𓀀
            我们刚才从
            图书馆来了
            我們剛才從
            圖書館來了
            øæå"#,
        );
        assert!(
            tokens
                .iter()
                .all(|v| matches!(v.kind, Kind::Ident | Kind::Newline))
        )
    }
}
