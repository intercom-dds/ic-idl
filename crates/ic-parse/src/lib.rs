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
    clippy::unnecessary_wraps,
    clippy::missing_errors_doc
)]

//! # ic-parse
//!
//! An IDL 4.2-compliant parser.
//!
//! This is a hand-written recursive descent parser with first-class annotation support.

//! # ic-parse
//!
//! An IDL 4.2-compliant parser.
//!
//! This crate contains the code for hand-written recursive descent parser. The
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

use std::collections::HashMap;
use std::path::Path;

pub use error::{Error, Expected, Reason};
pub use ic_lexer as lexer;
use ic_lexer::token::Kind;
use ic_preproc::ProcArgs;
use ic_syntax::{AnnotationAppl, Item, Span};
pub use ic_vfs::SourceMap;
use ic_vfs::{FileId, Include};
use parser::Parser;

/// Result of parsing an IDL file.
#[derive(Debug)]
pub struct ParseResult {
    pub tree: Vec<Item>,
    pub errors: Vec<Error>,
    /// Annotations that couldn't be attached to any construct.
    pub orphaned_annotations: Vec<AnnotationAppl>,
    /// Warnings from the preprocessor
    pub preproc_warnings: Vec<Error>,

    /// Map of spans to their macro expansion context
    pub expansion_info: HashMap<Span, ic_preproc::ExpansionInfo>,
}

/// Constructs an AST from the given source code.
#[must_use]
pub fn from_str(input: &str) -> ParseResult {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(input);
    from_file(file_id, ProcArgs::default(), &mut vfs)
}

/// Parses the specified file path and constructs an AST.
pub fn from_path(path: &Path, args: ProcArgs, vfs: &mut SourceMap) -> std::io::Result<ParseResult> {
    let (file_id, _) = vfs.open(path, Include::Static)?;
    Ok(from_file(file_id, args, vfs))
}

/// Parses the specified file and constructs an AST.
#[must_use]
pub fn from_file(file_id: FileId, args: ProcArgs, vfs: &mut SourceMap) -> ParseResult {
    use ic_preproc::Token;

    // Run preprocessor and collect tokens
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, vfs);

    let tokens: Vec<Token> = iter
        .filter(|t| !matches!(t.kind, Kind::Newline))
        .collect();

    // Parse
    let parser = Parser::new(tokens, vfs);
    let (tree, parse_errors, orphaned_annotations) = parser.parse();

    // Convert internal errors to public Error type
    let mut errors: Vec<Error> = parse_errors;

    // Process preprocessor errors
    errors.extend(process_preprocessor_errors(state.errors(), vfs));

    // Convert preprocessor warnings
    let preproc_warnings = process_preprocessor_warnings(state.warnings(), vfs);

    ParseResult {
        tree,
        errors,
        orphaned_annotations,
        preproc_warnings,
        expansion_info: state
            .expansion_info
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect(),
    }
}

/// Constructs an AST from the given token iterator.
pub fn from_iter<I>(iter: I) -> ParseResult
where
    I: IntoIterator<Item = ic_lexer::token::Token>,
{
    // Create a minimal source map for parsing
    let vfs = SourceMap::default();

    let tokens: Vec<_> = iter.into_iter().collect();

    let parser = Parser::new(tokens, &vfs);
    let (tree, errors, orphaned_annotations) = parser.parse();

    ParseResult {
        tree,
        errors,
        orphaned_annotations,
        preproc_warnings: Vec::new(),
        expansion_info: HashMap::default(),
    }
}

fn process_preprocessor_errors(errors: &[ic_preproc::Error], vfs: &SourceMap) -> Vec<Error> {
    let mut result = Vec::new();

    for error in errors {
        match error {
            ic_preproc::Error::Note { span, tokens } => {
                let message = if tokens.is_empty() {
                    "#error directive".to_string()
                } else {
                    let token_text = tokens
                        .iter()
                        .map(|t| &vfs.source_str(t.span.start.file_id)[t.span.range()])
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("#error directive: {token_text}")
                };
                result.push(Error {
                    found: None,
                    expected: None,
                    reason: Reason::Custom(message),
                    label: None,
                    span: *span,
                });
            }
            ic_preproc::Error::Syntax { message, span }
            | ic_preproc::Error::Expr { message, span } => {
                result.push(Error {
                    found: None,
                    expected: None,
                    reason: Reason::Custom((*message).to_string()),
                    label: None,
                    span: *span,
                });
            }
            ic_preproc::Error::Extraneous { .. } => {}
        }
    }

    result
}

fn process_preprocessor_warnings(warnings: &[ic_preproc::Error], vfs: &SourceMap) -> Vec<Error> {
    let mut result = Vec::new();

    for warning in warnings {
        match warning {
            ic_preproc::Error::Note { span, tokens } => {
                let message = if tokens.is_empty() {
                    "#warning directive".to_string()
                } else {
                    let token_text = tokens
                        .iter()
                        .map(|t| &vfs.source_str(t.span.start.file_id)[t.span.range()])
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("#warning directive: {token_text}")
                };
                result.push(Error {
                    found: None,
                    expected: None,
                    reason: Reason::Custom(message),
                    label: Some("preprocessor warning"),
                    span: *span,
                });
            }
            ic_preproc::Error::Extraneous {
                directive, span, ..
            } => {
                result.push(Error {
                    found: None,
                    expected: None,
                    reason: Reason::Custom(format!("extra tokens after #{directive} directive")),
                    label: Some("preprocessor warning"),
                    span: *span,
                });
            }
            ic_preproc::Error::Syntax { message, span }
            | ic_preproc::Error::Expr { message, span } => {
                result.push(Error {
                    found: None,
                    expected: None,
                    reason: Reason::Custom((*message).to_string()),
                    label: Some("preprocessor warning"),
                    span: *span,
                });
            }
        }
    }

    result
}
