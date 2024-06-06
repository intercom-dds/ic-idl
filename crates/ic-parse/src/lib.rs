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

//! # ic-parse
//!
//! An IDL 4.2-compliant parser.
//!
//! This crate contains the code for the lexer and parser. The parser produces
//! an AST which is a pure transcription of the source code. For a higher-level
//! parse tree where types have been resolved, see [`ic-hir`], which can be
//! constructed from the syntax tree.
//!
//! The output of the parser is not guaranteed to be valid IDL, neither
//! syntactically nor semantically. The parser follows a relaxed version of the IDL
//! grammar for the sake of ease of parsing, and instead relies on traversing the
//! AST afterwards to detect deviations and syntax errors.
//!
//! ## Building blocks
//!
//! The following building blocks are supported:
//!
//!  - Core Data Types
//!  - Any
//!  - Interfaces - Basic
//!  - Interfaces - Full
//!  - Value Types
//!  - Extended Data Types
//!  - Anonymous Types<sup>[1]</sup>
//!  - Annotations
//!
//! <sup>[1]</sup> Anonymous structs, unions and enumerators are not supported.
//!
//! ### Extensions
//!
//! The parser supports the following extensions to the IDL grammar:
//!
//!  - Scoped enums.
//!  - Default values for complex types, through annotations or constants.
//!  - Values of enumerators and bitmask flags can be assigned using an assignment
//!    expression.
//!  - Complex types may be used as keys in maps.
//!  - Lowercase boolean literals (i.e. `true`, `false`) are accepted.
//!  - Empty prototypes are allowed.
//!  - The `in` keyword may be omitted for in-parameters in prototypes.
//!
//! While the parser will always accept the extensions, [`ic-lint`] has an
//! optional set of pedantic lints that can trigger a warning or error if these
//! extensions are used.
//!
//! [`ic-lint`]: ../ic_lint/index.html
//! [`ic-hir`]: ../ic_hir/index.html

use chumsky::error::{Simple, SimpleReason};
use chumsky::{Parser, Stream};
use ic_alloc::interner::Interner;
use ic_syntax::Definition;
use lexer::{Kind, Span, Token};

pub mod lexer;
pub mod parser;
pub mod source;

#[derive(Debug)]
pub struct ParseResult {
    interner: Interner,
    pub tree: Vec<Definition>,
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
    let ast = parser::specification().parse(tokens);

    if let Ok(tree) = ast {
        Ok(ParseResult {
            tree,
            interner: Interner::default(),
        })
    } else {
        Err(anyhow::anyhow!("parsing failed"))
    }
}

/// Constructs an AST from the given token iterator.
///
/// # Errors
pub fn from_iter<I>(iter: I) -> Result<ParseResult, Vec<Simple<Kind>>>
where
    I: IntoIterator<Item = Token>,
{
    let tokens = iter.into_iter();
    let stream = Stream::from_iter(Span::default(), tokens.map(move |tok| (tok.kind, tok.span)));
    let ast = parser::specification().parse(stream)?;

    Ok(ParseResult {
        interner: Interner::default(),
        tree: ast,
    })
}
