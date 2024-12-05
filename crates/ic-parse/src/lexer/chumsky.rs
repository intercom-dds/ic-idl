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

use chumsky::prelude::*;
use chumsky::text::{Character, TextParser};
use chumsky::{Error, Parser as _};

// use crate::syntax::Span;

type Span = std::ops::Range<usize>;

// Workaround until trait aliases are stabilized
pub trait Lexer<T>: chumsky::Parser<char, T, Error = Simple<char>> + Clone {}

// Blanket impl because we really just want an alias
impl<T, U: chumsky::Parser<char, T, Error = Simple<char>> + Clone> Lexer<T> for U {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // Types and related keywords
    Annotation,
    Struct,
    Enum,
    Bitmask,
    Exception,
    Const,
    Module,
    Typedef,

    // Union
    Union,
    Switch,
    Case,
    Default,
    Null,

    // Interface
    Local,
    Interface,
    In,
    Out,
    Inout,
    Raises,
    GetRaises,
    SetRaises,
    Attribute,
    ReadOnly,

    // Valuetype
    Valuetype,
    Public,
    Private,
    Factory,

    // Bitset
    Bitset,
    Bitfield,

    // CORBA-isms
    Any,
    Void,
    Object,

    // Collections
    Sequence,
    Map,

    // Other keywords
    True,
    False,

    // Identifiers
    Ctrl(char),
    Ident(String),

    // TODO: remove
    AnnAppl(String),
    Literal(Literal),

    // Fallback for unrecognized tokens
    Invalid,
}

// TODO: don't parse numbers here, just collect the spans.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Number {
    // Signed(isize),
    Unsigned(usize),
    // Float(f64),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Literal {
    Boolean(bool),
    Char(char),
    String(String),
    Integer(Number),
}

fn separator() -> impl Lexer<()> {
    choice((
        end(),
        filter(|c: &char| c.is_ascii_punctuation() || c.is_whitespace()).ignored(),
    ))
}

/// Handles case-insensitive keywords.
fn keyword(kw: &'static str) -> impl Lexer<()> {
    text::ident().try_map(move |s: String, span| {
        s.eq_ignore_ascii_case(kw)
            .then_some(())
            .ok_or_else(|| Simple::expected_input_found(span, None, None))
    })
}

fn idl_keyword(ident: String) -> Token {
    match ident.as_str() {
        "annotation" => Token::Annotation,
        "struct" => Token::Struct,
        "enum" => Token::Enum,
        "bitmask" => Token::Bitmask,
        "exception" => Token::Exception,
        "const" => Token::Const,
        "module" => Token::Module,
        "typedef" => Token::Typedef,
        "union" => Token::Union,
        "switch" => Token::Switch,
        "case" => Token::Case,
        "default" => Token::Default,
        "null" => Token::Null,
        "local" => Token::Local,
        "interface" => Token::Interface,
        "in" => Token::In,
        "out" => Token::Out,
        "inout" => Token::Inout,
        "raises" => Token::Raises,
        "getraises" => Token::GetRaises,
        "setraises" => Token::SetRaises,
        "attribute" => Token::Attribute,
        "readonly" => Token::ReadOnly,
        "valuetype" => Token::Valuetype,
        "public" => Token::Public,
        "private" => Token::Private,
        "factory" => Token::Factory,
        "bitset" => Token::Bitset,
        "bitfield" => Token::Bitfield,
        "any" => Token::Any,
        "void" => Token::Void,
        "object" => Token::Object,
        "sequence" => Token::Sequence,
        "map" => Token::Map,
        _ => Token::Ident(ident),
    }
}

fn ident() -> impl Lexer<Token> {
    choice((
        just('_')
            .ignore_then(text::ident())
            .map(|ident: String| Token::Ident(ident))
            .labelled("identifier"),
        just('@')
            .ignore_then(text::ident())
            .map(|ident: String| Token::AnnAppl(ident))
            .labelled("annotation"),
        text::ident()
            .then_ignore(separator())
            .map(|ident: String| idl_keyword(ident))
            .labelled("identifier"),
    ))
}

fn integer_lit() -> impl Lexer<Number> {
    let hex = just("0x")
        .ignore_then(text::int(16))
        .map(|v: String| usize::from_str_radix(&v, 16).unwrap())
        .labelled("hexadecimal number");

    let oct = just('0')
        .ignore_then(text::int(8))
        .map(|v: String| usize::from_str_radix(&v, 8).unwrap())
        .labelled("octal number");

    let dec = text::int(10)
        .map(|v: String| v.parse().unwrap())
        .labelled("number");

    choice((hex, oct, dec))
        .map(Number::Unsigned)
        .then_ignore(separator())
}

fn bool_lit() -> impl Lexer<bool> {
    keyword("true")
        .to(true)
        .or(keyword("false").to(false))
        // .then_ignore(separator())
        .labelled("boolean")
}

fn char_lit() -> impl Lexer<char> {
    just('\'')
        .ignore_then(filter(|v: &char| v.is_ascii()))
        .then_ignore(just('\''))
        .labelled("character")
}

fn string_lit() -> impl Lexer<String> {
    just('"')
        .ignore_then(none_of('"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .labelled("string")
}

fn literal() -> impl Lexer<Literal> {
    choice((
        bool_lit().map(Literal::Boolean),
        char_lit().map(Literal::Char),
        // integer_lit().map(Literal::Integer),
        string_lit().map(Literal::String),
    ))
}

fn comment() -> impl Lexer<String> {
    let line = just("//")
        .ignore_then(take_until(text::newline()))
        .padded()
        .map(|(v, _)| v)
        .collect();

    let block = just("/*")
        .ignore_then(take_until(just("*/")))
        .padded()
        .map(|(v, _)| v)
        .collect();

    line.or(block).labelled("comment")
}

fn token() -> impl Lexer<Token> {
    let ctrl = one_of("()[]{}<>;,+-/*=").map(Token::Ctrl);
    let lit = literal().map(Token::Literal);
    choice((ctrl, lit, ident()))
}

#[must_use]
pub fn lexer() -> impl Lexer<Vec<(Token, Span)>> {
    // If push comes to shove, we create an invalid token that spans from the
    // current position until the next delimiter. This makes lexing infallible,
    // and lets us better handle the error during parsing.
    let invalid = any().to(Token::Invalid);

    let token = token()
        .or(invalid)
        .map_with_span(|t, span| (t, span))
        .padded_by(comment().repeated())
        .padded();

    token.repeated().then_ignore(end())
}

/// Exhaustively tokenizes the entire input string, returning a list of all
/// lexed tokens.
///
/// # Errors
///
///
// TODO: it might be a good idea to make this infallible and instead add an
// `Invalid` token that just matches whatever so we can recover.
pub fn scan(input: &str) -> Result<Vec<(Token, Span)>, Vec<Simple<char>>> {
    lexer().parse(input)
}
