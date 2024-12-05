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

#![allow(unused, dead_code)]

use chumsky::error::{Simple, SimpleReason};
use chumsky::{Parser, Stream};
use ic_alloc::interner::Interner;
use lexer::{Kind, Span, Token};

pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod visit;

#[derive(Debug)]
pub struct ParseResult {
    interner: Interner,
    pub tree: Vec<syntax::Definition>,
}

#[derive(Clone, Debug)]
pub struct Error<I, S> {
    pub reason: Reason<I, S>,
    pub span: Span,
}

// impl<I, S> From<Simple<I, S>> for Error<I, S> {
//     fn from(value: Simple<I, S>) -> Self {
//         Self {
//             reason: value.reason(),
//             span: value.span(),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub enum Reason<I, S> {
    /// An unexpected input was found.
    Unexpected,

    /// An unclosed delimiter was found.
    Unclosed {
        /// The span of the unclosed delimiter.
        span: S,
        /// The unclosed delimiter.
        delimiter: I,
    },

    /// An error with a custom message occurred.
    Custom(String),
}

impl<I, S> From<SimpleReason<I, S>> for Reason<I, S> {
    fn from(value: SimpleReason<I, S>) -> Self {
        match value {
            SimpleReason::Unexpected => Self::Unexpected,
            SimpleReason::Unclosed { span, delimiter } => Self::Unclosed { span, delimiter },
            SimpleReason::Custom(v) => Self::Custom(v),
        }
    }
}

/// Constructs an AST from the given source code.
///
/// # Errors
///
/// # Panics
pub fn from_str(input: &str) -> anyhow::Result<ParseResult> {
    let mut tokens = lexer::stream(input);
    {
        let all: Vec<_> = tokens.fetch_tokens().collect();
        dbg!(&all);
    }

    let ast = parser::specification().parse(tokens).unwrap();
    dbg!(ast);
    todo!()
    // if let Some(ast) = ast {
    //     Ok(ParseResult {
    //         interner: Interner::default(),
    //         tree: ast,
    //     })
    // } else {
    //     Err(anyhow::anyhow!("parse error"))
    // }
}

// Constructs an AST from the given tokens.
//
// # Errors
// pub fn from_tokens(input: &str, tokens: &[Token]) -> anyhow::Result<Vec<syntax::Definition>> {
//     let len = input.len();
//     let stream = Stream::from_iter(len..len + 1, tokens.iter());
//     from_stream(stream)
// }

// Constructs an AST from the token stream.
//
// # Errors
// pub fn from_stream<'a, K, S, I>(
//     _tokens: Stream<'a, K, S, I>,
// ) -> anyhow::Result<Vec<syntax::Definition>>
// where
//     S: chumsky::Span + 'a,
//     I: Iterator<Item = (Kind, Span)> + 'a,
// {
//     Ok(vec![])
// }

// Constructs an AST from the given token iterator.
// pub fn from_iter<I>(iter: I) -> Result<Vec<Definition>>
// where
//     I: IntoIterator<Item = Token>,
// {
//     let tokens = iter.into_iter();
//     // let ast = parser::specification().parse(stream);
//     Ok(vec![])
// }
