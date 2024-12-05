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

#[cfg(test)]
mod tests;

use std::fmt;

use chumsky::chain::Chain;
use chumsky::Stream;
use ic_alloc::inline_str::InlineStr;
use logos::{Lexer, Logos, Source};

macro_rules! tokens {
    ($(
        $(#[$meta:meta])*
        $var:ident $(= $func:ident($val:expr))?,
    )*) => {
        #[derive(Logos, Copy, Clone, Debug, PartialEq, Eq, Hash)]
        #[logos(skip r"[ \t\n\f]+")]
        #[logos(subpattern digits = "[0-9][_0-9]*")]
        pub enum Kind {
            $(
                $(#[$meta])*
                $(#[$func($val)])*
                $var,
            )*
        }
    };
}

#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(subpattern digits = "[0-9][_0-9]*")]
pub enum Kind {
    #[token("annotation")]
    Annotation,

    #[token("module")]
    Module,

    #[token("struct")]
    Struct,

    #[token("const")]
    Const,

    #[token("bitmask")]
    Bitmask,

    #[token("bitset")]
    Bitset,

    #[token("bitfield")]
    Bitfield,

    #[token("exception")]
    Exception,

    #[token("union")]
    Union,

    #[token("switch")]
    Switch,

    #[token("default")]
    Default,

    #[token("null")]
    Null,

    #[token("local")]
    Local,

    #[token("interface")]
    Interface,

    #[token("raises")]
    Raises,

    #[token("getraises")]
    GetRaises,

    #[token("setraises")]
    SetRaises,

    #[token("attribute")]
    Attribute,

    #[token("readonly")]
    ReadOnly,

    #[token("in")]
    In,

    #[token("out")]
    Out,

    #[token("inout")]
    InOut,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(";")]
    Semi,

    #[token("=")]
    Eq,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[regex("true|TRUE")]
    True,

    #[regex("false|FALSE")]
    False,

    #[regex("0[1-9]+")]
    Octal,

    #[regex("[1-9][0-9]*")]
    Decimal,

    #[regex("0[xX][a-fA-F0-9]+")]
    Hex,

    #[regex(r"(?&digits)(?:[eE](?&digits)|\.(?&digits)(?:[eE](?&digits))?)")]
    Float,

    /// String literal. Handles escaped quotes.
    #[regex(r#""(?:[^"]|\\")*""#)]
    String,

    /// A valid UAX#31 identifier.
    #[regex(r#"[\p{XID_Start}_]\p{XID_Continue}*"#)]
    Ident,

    /// Any single UTF-8 character surrounded by single quotes.
    #[regex(r"'(?:\\.|[^\\'])?'", to_char)]
    Char(Option<char>),

    /// Fallback for invalid tokens
    Invalid,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Annotation => write!(f, "annotation"),
            Kind::Struct => write!(f, "struct"),
            // Kind::Enum => write!(f, "enum"),
            Kind::Bitmask => write!(f, "bitmask"),
            Kind::Exception => write!(f, "exception"),
            Kind::Const => write!(f, "const"),
            Kind::Module => write!(f, "module"),
            Kind::Semi => write!(f, "`;`"),
            // Kind::Typedef => write!(f, "typedef"),
            // Kind::Union => write!(f, "union"),
            // Kind::Switch => write!(f, "switch"),
            // Kind::Case => write!(f, "case"),
            // Kind::Default => write!(f, "default"),
            // Kind::Null => write!(f, "null"),
            // Kind::Local => write!(f, "local"),
            // Kind::Interface => write!(f, "interface"),
            // Kind::In => write!(f, "in"),
            // Kind::Out => write!(f, "out"),
            // Kind::Inout => write!(f, "inout"),
            // Kind::Raises => write!(f, "raises"),
            // Kind::GetRaises => write!(f, "getraises"),
            // Kind::SetRaises => write!(f, "setraises"),
            // Kind::Attribute => write!(f, "attribute"),
            // Kind::ReadOnly => write!(f, "readonly"),
            // Kind::Valuetype => write!(f, "valuetype"),
            // Kind::Public => write!(f, "public"),
            // Kind::Private => write!(f, "private"),
            // Kind::Bitset => write!(f, "bitset"),
            // Kind::Bitfield => write!(f, "bitfield"),
            // Kind::Sequence => write!(f, "sequence"),
            // Kind::Map => write!(f, "map"),
            _ => write!(f, "unknown"),
        }
    }
}

// Empty character literals are permitted during parsing and instead gets
// checked later during the linting stage.
fn to_char(lex: &mut Lexer<Kind>) -> Option<char> {
    match lex.slice().len() {
        // Empty character literal
        2 => None,

        // Single UTF-8 character literal
        3 => lex.slice().chars().nth(1),

        // An escaped single-quote character
        4 => Some('\''),

        // The regex will never match more than 4 characters
        _ => unreachable!(),
    }
}

/// file from which it was read.
pub type Span = std::ops::Range<usize>;

/// A lexed token. Contains the span of the token and its kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub span: Span,
    pub kind: Kind,
}

/// Constructs a stream of input tokens. Unlike [`Iterator`], a stream supports
/// backtracking and some other required features.
#[must_use]
#[allow(clippy::range_plus_one)]
pub fn stream(input: &str) -> Stream<'_, Kind, Span, impl Iterator<Item = (Kind, Span)> + '_> {
    let lexer = lexer(input);
    let len = input.len();
    Stream::from_iter(len..len + 1, lexer.map(move |tok| (tok.kind, tok.span)))
}

/// Constructs an iterator that lazily lexes the input.
///
/// Lexing is infallible: any invalid tokens or characters will be mapped to an
/// `Kind::Invalid` token.
pub fn lexer(input: &str) -> impl Iterator<Item = Token> + '_ {
    // If push comes to shove, we create an invalid token that spans from the
    // current position until the next delimiter. This makes lexing infallible,
    // and lets us better handle the error during parsing.
    Kind::lexer(input).spanned().map(|(token, span)| Token {
        kind: token.unwrap_or(Kind::Invalid),
        span,
    })
}

/// Exhaustively tokenizes the entire input string, returning a list of all
/// lexed tokens.
#[must_use]
pub fn scan(input: &str) -> Vec<Token> {
    lexer(input).collect()
}
