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
//!  - A `null` keyword that lets you define "empty" union members.
//!
//! While the parser will always accept the extensions, [`ic-lint`] has an
//! optional set of pedantic lints that can trigger a warning or error if these
//! extensions are used.
//!
//! [`ic-lint`]: ../ic_lint/index.html
//! [`ic-hir`]: ../ic_hir/index.html

use std::path::Path;

use chumsky::error::{Simple, SimpleReason};
use chumsky::{Parser, Stream};
use ic_preproc::ProcArgs;
use ic_syntax::{Item, Span};
pub use ic_vfs::SourceMap;
use ic_vfs::{FileId, Include};
use lexer::{Kind, Token};

pub mod lexer;

mod parser;

#[derive(Debug)]
pub struct ParseResult {
    pub tree: Vec<Item>,
}

#[derive(Clone, Debug)]
pub enum Reason {
    /// An unexpected input was found.
    Unexpected,

    /// An unclosed delimiter was found.
    Unclosed {
        /// The span of the unclosed delimiter.
        span: Span,

        /// The unclosed delimiter.
        delimiter: Kind,
    },

    /// An error with a custom message occurred.
    Custom(String),
}

impl From<SimpleReason<Kind, Span>> for Reason {
    fn from(value: SimpleReason<Kind, Span>) -> Self {
        match value {
            SimpleReason::Unexpected => Self::Unexpected,
            SimpleReason::Unclosed { span, delimiter } => Self::Unclosed { span, delimiter },
            SimpleReason::Custom(v) => Self::Custom(v),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    pub found: Option<Kind>,
    pub expected: Option<Vec<Kind>>,
    pub reason: Reason,
    pub label: Option<&'static str>,
    pub span: Span,
}

impl From<Simple<Kind, Span>> for Error {
    fn from(value: Simple<Kind, Span>) -> Self {
        Self {
            found: value.found().cloned(),
            expected: value.expected().cloned().collect(),
            reason: Reason::from(value.reason().clone()),
            label: value.label(),
            span: value.span(),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: syntax error: expected {:?}, found {:?}",
            self.span, self.found, self.expected,
        )
    }
}

/// Constructs an AST from the given source code.
///
/// # Errors
///
/// # Panics
#[must_use]
pub fn from_str(input: &str) -> (ParseResult, Vec<Error>) {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(input);
    let args = ProcArgs::default();
    from_file(file_id, args, &mut vfs)
}

/// Parses the specified file and constructs an AST.
///
/// # Errors
///
/// # Panics
#[must_use]
pub fn from_path(
    path: &Path,
    args: ProcArgs,
    vfs: &mut SourceMap,
) -> std::io::Result<(ParseResult, Vec<Error>)> {
    let (file_id, _) = vfs.open(path, Include::Static)?;
    Ok(from_file(file_id, args, vfs))
}

/// Parses the specified file and constructs an AST.
///
/// # Errors
///
/// # Panics
#[must_use]
pub fn from_file(
    file_id: FileId,
    args: ProcArgs,
    vfs: &mut SourceMap,
) -> (ParseResult, Vec<Error>) {
    let skip = args.get_skip_comments();
    let iter = ic_preproc::preprocess(file_id, args, vfs);
    let tokens = lexer::from_cursor(iter, skip);
    let (tree, errors) = parser::specification().parse_recovery(tokens);
    let tree = tree.unwrap_or_default();
    let errors = errors.into_iter().map(Error::from).collect();

    (ParseResult { tree }, errors)
}

/// Constructs an AST from the given token iterator.
///
/// # Errors
///
/// If the given input contains IDL that is not syntactically valid, a
/// non-exhausitve list of parse errors will be returned that contains the
/// cause of each error and its span.
pub fn from_iter<I>(iter: I) -> Result<ParseResult, Vec<Error>>
where
    I: IntoIterator<Item = Token>,
{
    let tokens = iter.into_iter();
    let stream = Stream::from_iter(Span::default(), tokens.map(move |tok| (tok.kind, tok.span)));
    let tree = parser::specification()
        .parse(stream)
        .map_err(|v| v.into_iter().map(Error::from).collect::<Vec<_>>())?;

    Ok(ParseResult { tree })
}
