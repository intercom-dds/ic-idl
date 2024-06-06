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
use ic_alloc::interner::{Interner, SymbolId};
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
#[logos(skip r"//[^@][^\r\n]*")]
#[logos(subpattern digits = "[0-9][_0-9]*")]
#[logos(subpattern ident = r#"[\p{XID_Start}_]\p{XID_Continue}*"#)]
#[logos(extras = Context)]
pub enum Kind {
    #[token("any")]
    Any,

    #[token("@annotation")]
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

    #[token("enum")]
    Enum,

    #[token("exception")]
    Exception,

    #[token("typedef")]
    Typedef,

    #[token("native")]
    Native,

    #[token("fixed")]
    Fixed,

    #[token("union")]
    Union,

    #[token("switch")]
    Switch,

    #[token("case")]
    Case,

    #[token("default")]
    Default,

    #[token("null")]
    Null,

    #[token("valuetype")]
    Valuetype,

    #[token("public")]
    Public,

    #[token("private")]
    Private,

    #[token("supports")]
    Supports,

    #[token("factory")]
    Factory,

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

    #[token("oneway")]
    Oneway,

    #[token("in")]
    In,

    #[token("out")]
    Out,

    #[token("inout")]
    InOut,

    #[token("map")]
    Map,

    #[token("sequence")]
    Sequence,

    #[token("string")]
    String,

    #[token("wstring")]
    WString,

    /// `,`
    #[token(",")]
    Comma,

    /// `:`
    #[token(":")]
    Colon,

    /// `::`
    #[token("::")]
    DColon,

    /// `:`
    #[token(";")]
    Semi,

    /// `=`
    #[token("=")]
    Eq,

    /// `{`
    #[token("{")]
    LBrace,

    /// `}`
    #[token("}")]
    RBrace,

    /// `(`
    #[token("(")]
    LParen,

    /// `)`
    #[token(")")]
    RParen,

    /// `[`
    #[token("[")]
    LBracket,

    /// `]`
    #[token("]")]
    RBracket,

    /// `<`
    #[token("<")]
    Less,

    /// `>`
    #[token(">")]
    Greater,

    /// `&`
    #[token("&")]
    BitAnd,

    /// `|`
    #[token("|")]
    BitOr,

    /// `^`
    #[token("^")]
    BitXor,

    /// `+`
    #[token("+")]
    Plus,

    /// `-`
    #[token("-")]
    Minus,

    /// `~`
    #[token("~")]
    Tilde,

    /// `*`
    #[token("*")]
    Star,

    /// `/`
    #[token("/")]
    Slash,

    /// `%`
    #[token("%")]
    Modulo,

    /// `true`
    #[regex("true|TRUE")]
    True,

    /// `false`
    #[regex("false|FALSE")]
    False,

    /// Octal number, e.g. `0123`.
    #[regex("0[1-9]+")]
    Octal,

    /// Decimal number.
    #[regex("0|([1-9][0-9]*)")]
    Decimal,

    /// Hexadecimal number.
    #[regex("0[xX][a-fA-F0-9]+")]
    Hex,

    /// Fpoating-point literal
    #[regex(r"(?&digits)(?:[eE](?&digits)|\.(?&digits)(?:[eE](?&digits))?)")]
    Float,

    /// String literal. Handles escaped quotes.
    #[regex(r#"L?"(?:[^"]|\\")*""#)]
    StringLit,

    /// Applied annotation made up of a valid UAX#31 identifier
    #[regex(r#"(@|//@)(?&ident)"#)]
    AnnotationAppl,

    /// A valid UAX#31 identifier.
    #[regex("(?&ident)", to_interned)]
    Ident(SymbolId),

    /// Any single UTF-8 character surrounded by single quotes.
    #[regex(r"L?'(?:\\.|[^\\'])?'", to_char)]
    Char(Option<char>),

    // Preserve documentation comments
    #[regex(r"//[/!][^\r\n]*", priority = 7)]
    Comment,

    /// Fallback for invalid tokens
    Invalid,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Annotation => write!(f, "annotation"),
            Kind::Struct => write!(f, "struct"),
            Kind::Enum => write!(f, "enum"),
            Kind::Bitmask => write!(f, "bitmask"),
            Kind::Exception => write!(f, "exception"),
            Kind::Const => write!(f, "const"),
            Kind::Module => write!(f, "module"),
            Kind::Ident(_) => write!(f, "identifier"),
            Kind::Case => write!(f, "case label"),
            Kind::Default => write!(f, "default label"),
            Kind::Any => write!(f, "any"),
            Kind::Bitset => write!(f, "bitset"),
            Kind::Bitfield => write!(f, "bitfield"),
            Kind::Typedef => write!(f, "typedef"),
            Kind::Native => write!(f, "native"),
            Kind::Fixed => write!(f, "fixed"),
            Kind::Union => write!(f, "union"),
            Kind::Switch => write!(f, "switch"),
            Kind::Null => write!(f, "null"),
            Kind::Valuetype => write!(f, "valuetype"),
            Kind::Public => write!(f, "public"),
            Kind::Private => write!(f, "private"),
            Kind::Supports => write!(f, "supports"),
            Kind::Factory => write!(f, "factory"),
            Kind::Local => write!(f, "local"),
            Kind::Interface => write!(f, "interface"),
            Kind::Raises => write!(f, "raises"),
            Kind::GetRaises => write!(f, "getraises"),
            Kind::SetRaises => write!(f, "setraises"),
            Kind::Attribute => write!(f, "attribute"),
            Kind::ReadOnly => write!(f, "readonly"),
            Kind::Oneway => write!(f, "oneway"),
            Kind::Float => write!(f, "floating-point number"),
            Kind::StringLit => write!(f, "string literal"),
            Kind::AnnotationAppl => write!(f, "applied annotation"),
            Kind::In => write!(f, "in"),
            Kind::Out => write!(f, "out"),
            Kind::InOut => write!(f, "inout"),
            Kind::Sequence => write!(f, "sequence"),
            Kind::String => write!(f, "string"),
            Kind::WString => write!(f, "wstring"),
            Kind::Map => write!(f, "map"),
            Kind::Colon => write!(f, "`:`"),
            Kind::DColon => write!(f, "`::`"),
            Kind::Eq => write!(f, "`=`"),
            Kind::Semi => write!(f, "`;`"),
            Kind::Comma => write!(f, "`,`"),
            Kind::Less => write!(f, "`<`"),
            Kind::Greater => write!(f, "`>`"),
            Kind::LBrace => write!(f, "`{{`"),
            Kind::RBrace => write!(f, "`}}`"),
            Kind::LParen => write!(f, "`(`"),
            Kind::RParen => write!(f, "`)`"),
            Kind::LBracket => write!(f, "`[`"),
            Kind::RBracket => write!(f, "`]`"),
            Kind::True => write!(f, "`TRUE`"),
            Kind::False => write!(f, "`FALSE`"),
            Kind::BitAnd => write!(f, "`&`"),
            Kind::BitOr => write!(f, "`|`"),
            Kind::BitXor => write!(f, "`^`"),
            Kind::Plus => write!(f, "`+`"),
            Kind::Minus => write!(f, "`-`"),
            Kind::Tilde => write!(f, "`~`"),
            Kind::Star => write!(f, "`*`"),
            Kind::Slash => write!(f, "`/`"),
            Kind::Modulo => write!(f, "`%`"),
            Kind::Char(v) => write!(f, "'{}'", v.unwrap_or_default()),
            Kind::Octal | Kind::Decimal | Kind::Hex => write!(f, "number"),
            Kind::Comment => write!(f, "comment"),
            Kind::Invalid => write!(f, "invalid identifier"),
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

// Stores a lexed slice in the string interner.
fn to_interned(lex: &mut Lexer<Kind>) -> SymbolId {
    let slice = lex.slice();
    lex.extras.interner.insert(slice)
}

/// Context used by the lexer to store additional information.
#[derive(Default)]
pub struct Context {
    interner: Interner,
}

/// Byte offset to a token.
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
    // Remove trailing whitespace so we can correctly scope errors about
    // missing semicolons at the end of the file.
    let input = input.trim_end();
    let lexer = lexer(input);
    let len = input.len();

    Stream::from_iter(len..len + 1, lexer.map(move |tok| (tok.kind, tok.span)))
}

/// Constructs an iterator that lazily lexes the input.
///
/// Lexing is infallible: any invalid tokens or characters will be mapped to a
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
