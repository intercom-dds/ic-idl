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

use std::iter::Peekable;
use std::ops::Range;
use std::rc::Rc;

use ic_vfs::FileId;

use crate::iter::OwnedChars;

const EOF: char = '\0';

#[derive(Copy, Clone, Debug)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
    pub file_id: FileId,
}

impl SourceSpan {
    pub fn range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: SourceSpan,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Directive {
    If,
    Ifdef,
    Ifndef,
    Elif,
    Else,
    Endif,
    Include,
    Define,
    Undef,
    Line,
    Warning,
    Error,
    Pragma,
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Kind {
    /// Any valid UAX#31 identifier
    Ident,

    /// A documentation-style comment
    Comment,

    /// Octal, decimal or hexadecimal number
    Number,

    /// Floating-point literal
    Float,

    /// String literal
    String,

    /// Single UTF-8 character literal
    Char,

    /// `#`
    Hash,

    /// `@`
    At,

    /// `,`
    Comma,

    /// `.`
    Period,

    /// `:`
    Colon,

    /// `::`
    DColon,

    /// `:`
    Semi,

    /// `=`
    Eq,

    /// `==`
    EqEq,

    /// `!=`
    NotEq,

    /// `{`
    LBrace,

    /// `}`
    RBrace,

    /// `(`
    LParen,

    /// `)`
    RParen,

    /// `[`
    LBracket,

    /// `]`
    RBracket,

    /// `<`
    Lt,

    /// `>`
    Gt,

    /// `<=`
    LtEq,

    /// `>=`
    GtEq,

    /// `~`
    BitNot,

    /// `&`
    BitAnd,

    /// `|`
    BitOr,

    /// `^`
    BitXor,

    /// `!`
    Not,

    /// `&&`
    And,

    /// `||`
    Or,

    /// `+`
    Plus,

    /// `-`
    Minus,

    /// `~`
    Tilde,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `%`
    Modulo,

    /// `?`
    Question,

    /// `\n`
    Newline,

    /// `\`
    Backslash,

    /// Fallback for invalid tokens
    Unknown,
}

struct CharIdx<I: Iterator<Item = char>> {
    iter: Peekable<I>,
    index: u32,
}

impl<I> CharIdx<I>
where
    I: Iterator<Item = char>,
{
    fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }
}

impl<I> Iterator for CharIdx<I>
where
    I: Iterator<Item = char>,
{
    type Item = (u32, char);

    fn next(&mut self) -> Option<Self::Item> {
        let next = (self.index, self.iter.next()?);
        self.index += next.1.len_utf8() as u32;
        Some(next)
    }
}

#[derive(Debug)]
pub struct Cursor {
    pub cursor: Peekable<OwnedChars>,
    pub index: u32,
    pub file_id: FileId,
}

impl Cursor {
    pub fn new(source: Rc<str>, file_id: FileId) -> Self {
        let chars = OwnedChars::from(source);
        Cursor {
            cursor: chars.peekable(),
            index: 0,
            file_id,
        }
    }

    // TODO: remove this and hide it. it creates too many possible bugs.
    fn next_tok(&mut self) -> Option<char> {
        let c = self.cursor.next()?;
        self.index += c.len_utf8() as u32;
        Some(c)
    }

    fn span_since(&mut self, start: u32) -> SourceSpan {
        SourceSpan {
            start,
            end: self.index,
            file_id: self.file_id,
        }
    }

    fn peek_char(&mut self) -> char {
        self.cursor.peek().copied().unwrap_or(EOF)
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> SourceSpan {
        let start = self.index;
        while let Some(c) = self.cursor.peek() {
            if predicate(*c) {
                self.next_tok();
            } else {
                break;
            }
        }

        SourceSpan {
            start,
            end: self.index,
            file_id: self.file_id,
        }
    }

    fn ident(&mut self) -> Token {
        let span = self.eat_while(is_ident);

        Token {
            kind: Kind::Ident,
            span,
        }
    }

    fn number(&mut self) -> Token {
        let start = self.index;
        self.eat_while(|v| v.is_numeric());

        if let Some('.' | 'e' | 'E') = self.cursor.peek() {
            self.eat_while(|v| v.is_numeric());
        }

        let span = self.span_since(start);
        Token {
            kind: Kind::Number,
            span,
        }
    }

    // TODO: literals can be escaped. Should we care? Support it + add pedantic
    // lint maybe? don't think it's allowed in standard IDL, but also not sure
    // if the standard says anything about it.
    //
    // yes, we do care. but we still may need a lint.
    fn string_lit(&mut self) -> Token {
        // let span = self.eat_while(|v| v != '"');
        let start = self.index;

        // TODO: handle unterminated
        while let Some(c) = self.next_tok() {
            match c {
                '\\' => {
                    // TODO: newline??
                    if self.peek_char() == '"' {
                        _ = self.next_tok();
                    }
                }
                '\n' => {
                    // TODO: propagate that this was not terminated.
                    break;
                }
                '"' => break,
                _ => (),
            }
        }

        let span = self.span_since(start);
        Token {
            kind: Kind::String,
            span,
        }
    }

    // Code comments (`//`) are stripped from the output, but documentation
    // comments (`///`) are not.
    fn comment(&mut self) -> (Token, bool) {
        let is_doc = self.peek_char() == '/';
        let start = self.index;
        // self.until_peek(Kind::Newline);
        // TODO: this fails with floating points
        // let tokens = self.until(Kind::Newline);
        // fixed now, but we need to peek, not consume, so self.until_peek
        // until_peek consumes more characters than it should.
        while self.peek_char() != '\n' {
            self.next_tok();
        }

        let span = self.span_since(start);
        (
            Token {
                kind: Kind::Comment,
                span,
            },
            is_doc,
        )
    }

    // TODO: multi-line doc comments should... you guessed it... be stripped,
    // but be replaced by the correct amount of newlines.
    fn block_comment(&mut self) -> (Token, bool) {
        let is_doc = matches!(self.peek_char(), '*' | '!');
        let start = self.index;

        loop {
            match self.next_tok() {
                Some('*') => {
                    if self.next_tok().unwrap_or(EOF) == '/' {
                        break;
                    }
                }
                None => panic!("unterminated"),
                _ => (),
            }
        }

        let span = self.span_since(start);
        (
            Token {
                kind: Kind::Comment,
                span,
            },
            is_doc,
        )
    }

    fn peek_or(&mut self, c: char, a: Kind, b: Kind) -> Kind {
        if self.cursor.peek().copied() == Some(c) {
            _ = self.next_tok();
            a
        } else {
            b
        }
    }

    // TODO: should this stop at newlines? we require so for `dir_include`.
    pub fn until(&mut self, kind: Kind) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(tok) = self.next() {
            if tok.kind == kind {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    pub fn until_peek(&mut self, kind: Kind) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(tok) = self.peek() {
            if tok == kind {
                break;
            }
            tokens.push(self.next().unwrap());
        }
        tokens
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

    pub fn next(&mut self) -> Option<Token> {
        loop {
            let start = self.index;
            let kind = match self.next_tok()? {
                '#' => Kind::Hash,
                '@' => Kind::At,
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

                '"' => {
                    self.string_lit();
                    Kind::String
                }

                '/' => match self.peek_char() {
                    '/' => {
                        _ = self.next_tok();
                        if self.comment().1 {
                            Kind::Comment
                        } else {
                            continue;
                        }
                    }
                    '*' => {
                        _ = self.next_tok();
                        if self.block_comment().1 {
                            Kind::Comment
                        } else {
                            continue;
                        }
                    }
                    _ => Kind::Slash,
                },

                c if c.is_numeric() => {
                    self.number();
                    Kind::Number
                }

                c if is_ident(c) => {
                    self.ident();
                    Kind::Ident
                }

                c if c.is_whitespace() => continue,

                v => {
                    println!("unknown: {v}");
                    Kind::Unknown
                }
            };

            let span = self.span_since(start);
            debug_assert_ne!(kind, Kind::Unknown, "unknown character encountered");
            break Some(Token { kind, span });
        }
    }

    /// Be careful using this, as it will skip whitespace and thus advance the
    /// inner iterator.
    //
    // TODO: remove this. it was a temporary hack.
    pub fn peek(&mut self) -> Option<Kind> {
        loop {
            let kind = match self.next_tok()? {
                '#' => Kind::Hash,
                '@' => Kind::At,
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

                // TODO: the problem here is that we're already operating on a
                // peeked token, so the peek_or function won't work.
                //
                // The best we can do is probably add a `get()` function that
                // caches the current token. So we can use "next" here instead,
                // and then just rely on `get()` so we don't lose the value.
                '&' => self.peek_or('&', Kind::And, Kind::BitAnd),
                '|' => self.peek_or('|', Kind::Or, Kind::BitOr),
                '=' => self.peek_or('=', Kind::EqEq, Kind::Eq),
                ':' => self.peek_or(':', Kind::DColon, Kind::Colon),
                '!' => self.peek_or('=', Kind::NotEq, Kind::Unknown),
                '>' => self.peek_or('=', Kind::GtEq, Kind::Gt),
                '<' => self.peek_or('=', Kind::LtEq, Kind::Lt),

                '"' => {
                    self.string_lit();
                    Kind::String
                }

                '/' => match self.peek_char() {
                    '/' => {
                        _ = self.next_tok();
                        if self.comment().1 {
                            Kind::Comment
                        } else {
                            continue;
                        }
                    }
                    '*' => {
                        _ = self.next_tok();
                        if self.block_comment().1 {
                            Kind::Comment
                        } else {
                            continue;
                        }
                    }
                    _ => Kind::Slash,
                },

                c if c.is_numeric() => {
                    self.number();
                    Kind::Number
                }

                c if is_ident(c) => {
                    self.ident();
                    Kind::Ident
                }

                c if c.is_whitespace() => continue,

                v => {
                    println!("unknown: {v}");
                    Kind::Unknown
                }
            };
            break Some(kind);
        }
    }
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
