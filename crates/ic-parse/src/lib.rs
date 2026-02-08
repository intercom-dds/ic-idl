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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![allow(
    clippy::result_large_err,
    clippy::missing_errors_doc,
    clippy::unused_self
)]

//! # ic-parse
//!
//! An IDL 4.2-compliant parser.
//!
//! This crate contains the code for a hand-written recursive descent parser. The
//! parser produces an AST which is a pure transcription of the source code.
//! For a higher-level parse tree where types have been resolved, see
//! [`ic-hir`], which can be constructed from the syntax tree.
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
//!  - Anonymous Types<sup>\[1\]</sup>
//!  - Annotations
//!
//! <sup>\[1\]</sup> Anonymous structs, unions and enumerators are not supported.
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

mod annotation;
mod decl;
mod error;
mod expr;
mod parser;
mod types;

pub use error::{Error, Expected, Reason};
use ic_lexer::cursor::Cursor;
use ic_lexer::stream::Stream;
use ic_lexer::token::{Kind, Token};
use ic_syntax::{AnnotationAppl, Item};
use ic_vfs::FileId;
pub use ic_vfs::SourceMap;
use parser::Parser;
use tracing::debug;

/// Result of parsing an IDL file.
#[derive(Debug)]
pub struct ParseResult {
    pub tree: Vec<Item>,
    pub errors: Vec<Error>,
    pub orphaned_annotations: Vec<AnnotationAppl>,
}

/// Parses source code from a string.
#[must_use]
pub fn from_str(input: &str) -> ParseResult {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(input);
    from_file(file_id, &vfs)
}

/// Parses source from a file that has already been loaded into the
/// `SourceMap`.
///
/// This function lexes the source and parses the resulting tokens. It does
/// *not* run the preprocessor - callers should preprocess first if needed.
#[must_use]
pub fn from_file(file_id: FileId, vfs: &SourceMap) -> ParseResult {
    let source = vfs.source(file_id);
    let cursor = Cursor::new(source, file_id);
    from_iter(cursor, vfs)
}

/// Parses from a token iterator.
pub fn from_iter<I>(tokens: I, vfs: &SourceMap) -> ParseResult
where
    I: IntoIterator<Item = Token>,
{
    let stream = Stream::new(
        tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, Kind::Newline)),
    );
    let parser = Parser::new(stream, vfs);
    let (tree, errors, orphaned_annotations) = parser.parse();
    debug!(items = tree.len(), errors = errors.len(), "parsed");

    ParseResult {
        tree,
        errors,
        orphaned_annotations,
    }
}
