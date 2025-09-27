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

#![allow(clippy::cast_possible_truncation)]

use std::fmt;

use chumsky::Stream;
use ic_lexer::token::{Base, Kw};
use ic_macros::DiscHash;
use ic_preproc::TokenIter;
use ic_syntax::Span;

/// All tokens recognized by the lexer.
#[derive(Clone, Debug, PartialEq, DiscHash)]
pub enum Kind {
    /// A valid UAX#31 identifier.
    Ident(String),

    /// An IDL keyword
    Keyword(Kw),

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
    Less,

    /// `>`
    Greater,

    /// `~`
    BitNot,

    /// `&`
    BitAnd,

    /// `|`
    BitOr,

    /// `^`
    BitXor,

    /// `+`
    Plus,

    /// `-`
    Minus,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `%`
    Modulo,

    /// Octal number, e.g. `0123`.
    Octal(u64),

    /// Decimal number.
    Decimal(u64),

    /// Hexadecimal number.
    Hex(u64),

    /// Floating-point literal
    Float(f64),

    /// String literal. Handles escaped quotes.
    StringLit(String),

    /// Any single UTF-8 character surrounded by single quotes.
    Char(char),

    // Preserve documentation comments
    // The bool indicates if this is a trailing comment (on same line as code)
    Comment(String, bool),

    Eoi,

    /// Fallback for invalid tokens
    Invalid,

    /// String literal that was not properly terminated
    UnterminatedString,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Keyword(v) => write!(f, "{v}"),
            Kind::Colon => write!(f, "`:`"),
            Kind::DColon => write!(f, "`::`"),
            Kind::Eq => write!(f, "`=`"),
            Kind::Semi => write!(f, "`;`"),
            Kind::Comma => write!(f, "`,`"),
            Kind::Period => write!(f, "`.`"),
            Kind::Less => write!(f, "`<`"),
            Kind::Greater => write!(f, "`>`"),
            Kind::LBrace => write!(f, "`{{`"),
            Kind::RBrace => write!(f, "`}}`"),
            Kind::LParen => write!(f, "`(`"),
            Kind::RParen => write!(f, "`)`"),
            Kind::LBracket => write!(f, "`[`"),
            Kind::RBracket => write!(f, "`]`"),
            Kind::BitNot => write!(f, "`~`"),
            Kind::BitAnd => write!(f, "`&`"),
            Kind::BitOr => write!(f, "`|`"),
            Kind::BitXor => write!(f, "`^`"),
            Kind::Plus => write!(f, "`+`"),
            Kind::Minus => write!(f, "`-`"),
            Kind::Star => write!(f, "`*`"),
            Kind::Slash => write!(f, "`/`"),
            Kind::Modulo => write!(f, "`%`"),
            Kind::Char(v) => write!(f, "'{v}'"),
            Kind::Octal(_) | Kind::Decimal(_) | Kind::Hex(_) => write!(f, "number"),
            Kind::Comment(_, _) => write!(f, "comment"),
            Kind::Invalid => write!(f, "invalid identifier"),
            Kind::Ident(_) => write!(f, "identifier"),
            Kind::At => write!(f, "annotation"),
            Kind::Float(_) => write!(f, "floating-point number"),
            Kind::StringLit(_) => write!(f, "string"),
            Kind::Eoi => write!(f, "end of input"),
            Kind::UnterminatedString => write!(f, "unterminated string literal"),
        }
    }
}

impl Eq for Kind {}

/// A lexed token. Contains the span of the token and its kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub span: Span,
    pub kind: Kind,
}

// Constructs a stream of input tokens. Unlike [`Iterator`], a stream supports
// backtracking and some other required features.
// #[must_use]
// pub fn stream(input: &str) -> Stream<'_, Kind, Span, impl Iterator<Item = (Kind, Span)> + '_> {
//     // Remove trailing whitespace so we can correctly scope errors about
//     // missing semicolons at the end of the file.
//     let input = input.trim_end();
//     let lexer = lexer(input);
//     let end = Span {
//         start: input.len() as u32,
//         end: input.len() as u32 + 1,
//     };
//
//     Stream::from_iter(end, lexer.map(move |tok| (tok.kind, tok.span)))
// }

// Constructs an iterator that lazily lexes the input.
//
// Lexing is infallible: any invalid tokens or characters will be mapped to a
// `Kind::Invalid` token.
// pub fn lexer(input: &str) -> impl Iterator<Item = Token> + '_ {
//     // If push comes to shove, we create an invalid token that spans from the
//     // current position until the next delimiter. This makes lexing infallible,
//     // and lets us better handle the error during parsing.
//     Kind::lexer(input).spanned().map(|(token, span)| Token {
//         kind: token.unwrap_or(Kind::Invalid),
//         span: Span {
//             start: span.start as u32,
//             end: span.end as u32,
//         },
//     })
// }

impl From<ic_preproc::Token> for Token {
    fn from(value: ic_preproc::Token) -> Self {
        let kind = match value.kind {
            ic_preproc::Kind::Keyword(v) => Kind::Keyword(v),
            ic_preproc::Kind::Ident => Kind::Ident(String::new()),
            ic_preproc::Kind::Comment { trailing } => Kind::Comment(String::new(), trailing),
            ic_preproc::Kind::String { .. } => Kind::StringLit(String::new()),
            ic_preproc::Kind::Char => Kind::Char('\0'),
            ic_preproc::Kind::At => Kind::At,
            ic_preproc::Kind::Comma => Kind::Comma,
            ic_preproc::Kind::Period => Kind::Period,
            ic_preproc::Kind::Colon => Kind::Colon,
            ic_preproc::Kind::DColon => Kind::DColon,
            ic_preproc::Kind::Semi => Kind::Semi,
            ic_preproc::Kind::Eq => Kind::Eq,
            ic_preproc::Kind::LBrace => Kind::LBrace,
            ic_preproc::Kind::RBrace => Kind::RBrace,
            ic_preproc::Kind::LParen => Kind::LParen,
            ic_preproc::Kind::RParen => Kind::RParen,
            ic_preproc::Kind::LBracket => Kind::LBracket,
            ic_preproc::Kind::RBracket => Kind::RBracket,
            ic_preproc::Kind::Lt => Kind::Less,
            ic_preproc::Kind::Gt => Kind::Greater,
            ic_preproc::Kind::BitNot => Kind::BitNot,
            ic_preproc::Kind::BitAnd => Kind::BitAnd,
            ic_preproc::Kind::BitOr => Kind::BitOr,
            ic_preproc::Kind::BitXor => Kind::BitXor,
            ic_preproc::Kind::Plus => Kind::Plus,
            ic_preproc::Kind::Minus => Kind::Minus,
            ic_preproc::Kind::Star => Kind::Star,
            ic_preproc::Kind::Slash => Kind::Slash,
            ic_preproc::Kind::Modulo => Kind::Modulo,
            ic_preproc::Kind::Number { .. } => Kind::Decimal(0),
            ic_preproc::Kind::Float => Kind::Float(0.0),
            ic_preproc::Kind::Eoi => Kind::Eoi,
            _ => Kind::Invalid, // Invalid includes newlines and backslashes handled by preprocessor
        };

        Self {
            span: value.span,
            kind,
        }
    }
}

#[must_use]
pub fn from_iter<I>(iter: I) -> Stream<'static, Kind, Span, impl Iterator<Item = (Kind, Span)>>
where
    I: IntoIterator<Item = ic_preproc::Token>,
{
    let iter = iter.into_iter().filter_map(|v| {
        if v.kind == ic_preproc::Kind::Newline {
            None
        } else {
            Some(Token::from(v))
        }
    });

    let end = Span::default();
    Stream::from_iter(end, iter.map(move |tok| (tok.kind, tok.span)))
}

fn kind_number(base: Base, value: u64) -> Kind {
    match base {
        Base::Octal => Kind::Octal(value),
        Base::Decimal => Kind::Decimal(value),
        Base::Hexadecimal => Kind::Hex(value),
    }
}

/// Creates a token iterator from the preprocessor output
pub fn create_token_iterator<'a, S>(
    mut iter: TokenIter<'a, S>,
    ignore_comments: bool,
) -> impl Iterator<Item = Token> + 'a
where
    S: std::borrow::BorrowMut<ic_preproc::State> + 'a,
{
    std::iter::from_fn(move || {
        loop {
            let next = iter.next()?;
            match next.kind {
                ic_preproc::Kind::Newline => (),
                ic_preproc::Kind::Ident => {
                    let ident = iter.source_of(next.span).to_string();
                    let ident = if let Some(v) = ident.strip_prefix('_') {
                        v.to_string()
                    } else {
                        ident
                    };

                    break Some(Token {
                        kind: Kind::Ident(ident),
                        span: next.span,
                    });
                }
                ic_preproc::Kind::Number { base } => {
                    let src = iter.source_of(next.span);
                    let src = match base {
                        Base::Hexadecimal => &src[2..],
                        _ => src,
                    };

                    let kind = u64::from_str_radix(src, base as u32)
                        .ok()
                        .map_or(Kind::Invalid, |v| kind_number(base, v));

                    break Some(Token {
                        kind,
                        span: next.span,
                    });
                }
                ic_preproc::Kind::Float => {
                    let src = iter.source_of(next.span);
                    let kind = src.parse::<f64>().map_or(Kind::Invalid, Kind::Float);
                    break Some(Token {
                        kind,
                        span: next.span,
                    });
                }
                ic_preproc::Kind::Comment { trailing } => {
                    if ignore_comments {
                        continue;
                    }
                    let comment = iter.source_of(next.span).to_string();
                    break Some(Token {
                        kind: Kind::Comment(comment, trailing),
                        span: next.span,
                    });
                }
                ic_preproc::Kind::String { terminated } => {
                    if !terminated {
                        // For unterminated strings, return a special token type
                        // This allows the parser to give a more specific error message
                        break Some(Token {
                            kind: Kind::UnterminatedString,
                            span: next.span,
                        });
                    }

                    let source = iter.source_of(next.span);
                    #[allow(clippy::map_unwrap_or)]
                    let str = source
                        .strip_prefix('"')
                        .and_then(|v| v.strip_suffix('"'))
                        .map(std::string::ToString::to_string)
                        .unwrap_or_else(|| {
                            // Handle malformed string - return the content without quotes if possible
                            source.strip_prefix('"').unwrap_or(source).to_string()
                        });

                    break Some(Token {
                        kind: Kind::StringLit(str),
                        span: next.span,
                    });
                }
                ic_preproc::Kind::Char => {
                    let source = iter.source_of(next.span);
                    let ch = ic_expr::c_adapter::parse_character(source)
                        .ok()
                        .or_else(|| {
                            // If parsing fails, try to extract a single character
                            let content = source.trim_start_matches('\'').trim_end_matches('\'');
                            content.chars().next()
                        })
                        .unwrap_or_default();

                    break Some(Token {
                        kind: Kind::Char(ch),
                        span: next.span,
                    });
                }
                _ => break Some(Token::from(next)),
            }
        }
    })
}

/// # Panics
///
/// Panics if the provided iterator yields invalid tokens.
#[must_use]
pub fn from_cursor<'a, S>(
    iter: TokenIter<'a, S>,
    ignore_comments: bool,
) -> Stream<'static, Kind, Span, impl Iterator<Item = (Kind, Span)> + 'a>
where
    S: std::borrow::BorrowMut<ic_preproc::State> + 'a,
{
    let token_iter = create_token_iterator(iter, ignore_comments);
    let end = Span::default();
    Stream::from_iter(end, token_iter.map(move |tok| (tok.kind, tok.span)))
}
