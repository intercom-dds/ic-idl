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

use crate::fast_lookup::{get_single_char_token, is_ascii_whitespace, is_special_char};
use crate::iter::{EOF, OwnedChars};
use crate::token::{Base, Kind, Kw, Token};

/// ASCII digit lookup table for faster checking
const ASCII_DIGIT: [bool; 128] = {
    let mut table = [false; 128];
    let mut i = b'0' as usize;
    while i <= b'9' as usize {
        table[i] = true;
        i += 1;
    }
    table
};

/// ASCII hex digit lookup table for faster checking
const ASCII_HEX_DIGIT: [bool; 128] = {
    let mut table = [false; 128];
    let mut i = b'0' as usize;
    while i <= b'9' as usize {
        table[i] = true;
        i += 1;
    }
    let mut i = b'A' as usize;
    while i <= b'F' as usize {
        table[i] = true;
        i += 1;
    }
    let mut i = b'a' as usize;
    while i <= b'f' as usize {
        table[i] = true;
        i += 1;
    }
    table
};

/// A lexical cursor that tokenizes IDL source code.
///
/// The cursor maintains the current position in the source and provides
/// methods to extract tokens and navigate through the input.
#[must_use]
#[derive(Clone, Debug)]
pub struct Cursor {
    chars: OwnedChars,
    file_id: FileId,
    /// Tracks if non-whitespace tokens have been emitted on the current line
    has_content_on_line: bool,
}

impl Cursor {
    /// Creates a new cursor for the given source code.
    pub fn new(source: Rc<str>, file_id: FileId) -> Self {
        let chars = OwnedChars::from(source);
        Cursor {
            chars,
            file_id,
            has_content_on_line: false,
        }
    }

    #[inline]
    fn span_since(&self, start: u32) -> Span {
        let end = self.chars.index();
        Span {
            start: Location::new(start, self.file_id),
            end: Location::new(end, self.file_id),
        }
    }

    #[inline]
    #[allow(dead_code)]
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

    // Specialized eat_while for common cases - avoids function call overhead
    #[inline]
    fn eat_while_ascii_digit(&mut self) {
        loop {
            let c = self.chars.peek();
            if c == EOF || (c as u32) >= 128 || !ASCII_DIGIT[c as usize] {
                break;
            }
            self.chars.next();
        }
    }

    #[inline]
    fn eat_while_ascii_hexdigit(&mut self) {
        loop {
            let c = self.chars.peek();
            if c == EOF || (c as u32) >= 128 || !ASCII_HEX_DIGIT[c as usize] {
                break;
            }
            self.chars.next();
        }
    }

    #[inline]
    fn eat_while_ident(&mut self) {
        loop {
            let c = self.chars.peek();
            if c == EOF || !is_ident(c) {
                break;
            }
            self.chars.next();
        }
    }

    #[inline]
    fn ident(&mut self, start: u32) -> Kind {
        self.eat_while_ident();
        let span = self.span_since(start);
        let src = &self.chars.as_str()[span.range()];
        Kw::from_str(src).map_or(Kind::Ident, Kind::Keyword)
    }

    #[inline]
    fn number(&mut self, leading: char) -> Kind {
        // Handle hex numbers (must start with 0)
        if leading == '0' {
            match self.chars.peek() {
                'x' | 'X' => {
                    _ = self.chars.next(); // consume 'x' or 'X'
                    self.eat_while_ascii_hexdigit();
                    return Kind::Number {
                        base: Base::Hexadecimal,
                    };
                }
                _ => {
                    // Could be octal or just '0'
                }
            }
        }

        // Consume all digits
        self.eat_while_ascii_digit();

        // Check for float indicators
        match self.chars.peek() {
            '.' => {
                _ = self.chars.next(); // consume '.'
                self.eat_while_ascii_digit();
                // Check for exponent
                if matches!(self.chars.peek(), 'e' | 'E') {
                    _ = self.chars.next(); // consume 'e' or 'E'
                    // Handle optional sign
                    if matches!(self.chars.peek(), '+' | '-') {
                        _ = self.chars.next();
                    }
                    self.eat_while_ascii_digit();
                }
                Kind::Float
            }
            'e' | 'E' => {
                _ = self.chars.next(); // consume 'e' or 'E'
                // Handle optional sign
                if matches!(self.chars.peek(), '+' | '-') {
                    _ = self.chars.next();
                }
                self.eat_while_ascii_digit();
                Kind::Float
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

    #[inline]
    fn string_lit(&mut self) -> Kind {
        // Fast path for strings without escapes (common case)
        let mut escape_seen = false;
        loop {
            let Some(c) = self.chars.next() else {
                return Kind::String { terminated: false };
            };

            if escape_seen {
                escape_seen = false;
                continue;
            }

            match c {
                '"' => return Kind::String { terminated: true },
                '\n' => return Kind::String { terminated: false },
                '\\' => escape_seen = true,
                _ => {}
            }
        }
    }

    #[inline]
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

    // `@annotation` is special because it's a keyword that consists of
    // non-alphanumeric characters.
    #[inline]
    fn annotation(&mut self) -> Kind {
        if let Some(v) = self.clone().next() {
            if v.kind == Kind::Ident && self.source_of(v.span) == "annotation" {
                _ = self.next();
                return Kind::Keyword(Kw::Annotation);
            }
        }
        Kind::At
    }

    // Code comments (`//`) are stripped from the output, but documentation
    // comments (`///`) are not.
    //
    // Returns (is_doc, is_trailing) tuple
    #[inline]
    fn comment(&mut self) -> (bool, bool) {
        // Check if this is a trailing comment (on same line as code)
        let is_trailing = self.has_content_on_line;

        // Consume the leading '/'
        _ = self.chars.next();

        let is_doc = matches!(self.chars.peek(), '/' | '!');
        _ = self.until_peek(Kind::Newline);
        (is_doc, is_trailing)
    }

    // Returns (is_doc, is_trailing) tuple
    #[inline]
    fn block_comment(&mut self) -> (bool, bool) {
        // Check if this is a trailing comment (on same line as code)
        let is_trailing = self.has_content_on_line;

        // Consume the leading '/'
        _ = self.chars.next();

        // Check if this might be a doc comment
        let first_char = self.chars.peek();
        let mut is_doc = false;

        if first_char == '!' {
            // /*! style doc comment
            is_doc = true;
        } else if first_char == '*' {
            // Could be /** style doc comment
            // We need to check if there's actual content after /**
            let mut chars_clone = self.chars.clone();
            chars_clone.next(); // Skip the first *

            // Skip any additional stars
            while chars_clone.peek() == '*' {
                chars_clone.next();
            }

            // Now check what comes after the stars
            let next_char = chars_clone.peek();
            // It's a doc comment only if there's something other than / after the stars
            // This makes /** text */ a doc comment but /**/, /***/, etc. regular comments
            is_doc = next_char != '/' && next_char != EOF;
        }

        let mut prev_was_star = false;
        loop {
            match self.chars.next() {
                Some('/') if prev_was_star => break,
                Some('*') => prev_was_star = true,
                Some(_) => prev_was_star = false,
                None => break, // Unterminated block comment
            }
        }
        (is_doc, is_trailing)
    }

    #[inline]
    fn peek_or(&mut self, c: char, a: Kind, b: Kind) -> Kind {
        if self.chars.peek() == c {
            _ = self.chars.next();
            a
        } else {
            b
        }
    }

    /// Advances the iterator until it finds a token with the specified `kind`.
    ///
    /// Returns all tokens consumed before finding the target kind and the span
    /// covering all consumed tokens.
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
            let c = self.chars.next()?;

            // Fast path for common single-character tokens
            if let Some(kind) = get_single_char_token(c) {
                // Reset line state on newline
                self.has_content_on_line = kind != Kind::Newline;
                return Some(Token {
                    kind,
                    span: self.span_since(start),
                });
            }

            // Handle whitespace early to avoid further checks
            if is_ascii_whitespace(c) || (c as u32 >= 128 && c.is_whitespace()) {
                // Note: newlines are handled in single-char tokens above
                continue;
            }

            // Check for digits early (common case)
            if (c as u32) < 128 && ASCII_DIGIT[c as usize] {
                self.has_content_on_line = true;
                let kind = self.number(c);
                return Some(Token {
                    kind,
                    span: self.span_since(start),
                });
            }

            // Check for identifiers (common case)
            if is_ident(c) {
                self.has_content_on_line = true;
                let kind = self.ident(start);
                return Some(Token {
                    kind,
                    span: self.span_since(start),
                });
            }

            // Handle special characters that need lookahead
            let kind = if is_special_char(c) {
                match c {
                    '&' => self.peek_or('&', Kind::And, Kind::BitAnd),
                    '|' => self.peek_or('|', Kind::Or, Kind::BitOr),
                    '=' => self.peek_or('=', Kind::EqEq, Kind::Eq),
                    ':' => self.peek_or(':', Kind::DColon, Kind::Colon),
                    '!' => self.peek_or('=', Kind::NotEq, Kind::Not),
                    '>' => match self.chars.peek() {
                        '=' => {
                            self.chars.next();
                            Kind::GtEq
                        }
                        '>' => {
                            self.chars.next();
                            Kind::RShift
                        }
                        _ => Kind::Gt,
                    },
                    '<' => match self.chars.peek() {
                        '=' => {
                            self.chars.next();
                            Kind::LtEq
                        }
                        '<' => {
                            self.chars.next();
                            Kind::LShift
                        }
                        _ => Kind::Lt,
                    },
                    '"' => self.string_lit(),
                    '\'' => self.char_lit(),
                    '@' => self.annotation(),
                    '/' => match self.chars.peek() {
                        '/' => {
                            let (is_doc, is_trailing) = self.comment();
                            if is_doc {
                                Kind::Comment {
                                    trailing: is_trailing,
                                }
                            } else {
                                continue;
                            }
                        }
                        '*' => {
                            let (is_doc, is_trailing) = self.block_comment();
                            if is_doc {
                                Kind::Comment {
                                    trailing: is_trailing,
                                }
                            } else {
                                continue;
                            }
                        }
                        _ => {
                            self.has_content_on_line = true;
                            Kind::Slash
                        }
                    },
                    _ => Kind::Unknown,
                }
            } else {
                Kind::Unknown
            };

            // Mark that we've seen content on this line (unless it's a comment)
            if !matches!(kind, Kind::Comment { .. }) {
                self.has_content_on_line = true;
            }

            return Some(Token {
                kind,
                span: self.span_since(start),
            });
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

    /// Returns the source of the given span.
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

/// ASCII identifier character lookup table
const ASCII_IDENT: [bool; 128] = {
    let mut table = [false; 128];
    // Digits
    let mut i = b'0' as usize;
    while i <= b'9' as usize {
        table[i] = true;
        i += 1;
    }
    // Uppercase letters
    let mut i = b'A' as usize;
    while i <= b'Z' as usize {
        table[i] = true;
        i += 1;
    }
    // Lowercase letters
    let mut i = b'a' as usize;
    while i <= b'z' as usize {
        table[i] = true;
        i += 1;
    }
    // Underscore
    table[b'_' as usize] = true;
    table
};

/// Returns true if the character can appear in an identifier.
#[inline]
fn is_ident(c: char) -> bool {
    // Fast path for ASCII (most common)
    let code = c as u32;
    if code < 128 {
        ASCII_IDENT[code as usize]
    } else {
        // Slower path for Unicode
        c.is_alphanumeric()
    }
}

/// Returns true if the character can start an identifier.
#[allow(dead_code)]
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
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
            r"
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
            øæå",
        );
        assert!(
            tokens
                .iter()
                .all(|v| matches!(v.kind, Kind::Ident | Kind::Newline))
        );
    }

    #[test]
    fn annotation_kw() {
        let tokens = scan("@annotation");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, Kind::Keyword(Kw::Annotation));

        let tokens = scan("@      annotation");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, Kind::Keyword(Kw::Annotation));

        let tokens = scan("@foo");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, Kind::At);
        assert_eq!(tokens[1].kind, Kind::Ident);
    }
}
