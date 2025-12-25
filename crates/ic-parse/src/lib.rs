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

use std::ops::Deref;
use std::path::Path;

use chumsky::error::Rich;
use chumsky::prelude::*;
use ic_preproc::ProcArgs;
use ic_syntax::{Item, Span};
pub use ic_vfs::SourceMap;
use ic_vfs::{FileId, Include};
use lexer::{Kind, Token};

pub mod lexer;

mod comment_attacher;
mod parser;

#[derive(Debug)]
pub struct ParseResult {
    pub tree: Vec<Item>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Error>,
    /// Map of spans to their macro expansion context
    pub expansion_info: std::collections::HashMap<Span, ic_preproc::ExpansionInfo>,
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

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unexpected => write!(f, "unexpected input"),
            Self::Unclosed { delimiter, .. } => write!(f, "unclosed delimiter {delimiter:?}"),
            Self::Custom(msg) => write!(f, "{msg}"),
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

impl Error {
    fn from_rich(value: Rich<'_, Kind, Span>) -> Self {
        let found = value.found().cloned();
        let span = *value.span();

        // Extract expected tokens from the error
        let expected: Vec<Kind> = value
            .expected()
            .filter_map(|e| match e {
                chumsky::error::RichPattern::Token(t) => Some(t.deref().clone()),
                _ => None,
            })
            .collect();

        let expected = if expected.is_empty() {
            None
        } else {
            Some(expected)
        };

        // Determine the reason
        let reason = match value.into_reason() {
            chumsky::error::RichReason::ExpectedFound { .. } => Reason::Unexpected,
            chumsky::error::RichReason::Custom(msg) => Reason::Custom(msg),
        };

        Self {
            found,
            expected,
            reason,
            label: None,
            span,
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
pub fn from_str(input: &str) -> ParseResult {
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
pub fn from_path(path: &Path, args: ProcArgs, vfs: &mut SourceMap) -> std::io::Result<ParseResult> {
    let (file_id, _) = vfs.open(path, Include::Static)?;
    Ok(from_file(file_id, args, vfs))
}

/// Parses the specified file and constructs an AST.
///
/// # Errors
///
/// # Panics
#[must_use]
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

pub fn from_file(file_id: FileId, args: ProcArgs, vfs: &mut SourceMap) -> ParseResult {
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, vfs);

    // Create token iterator using the existing function in lexer.rs
    let token_iter = lexer::create_token_iterator(iter, false);

    // Collect comments while filtering them out of the token stream
    let mut comments = Vec::new();
    let tokens: Vec<(Kind, Span)> = token_iter
        .filter_map(|tok| match &tok.kind {
            Kind::Comment(text, trailing) => {
                // Store comment and filter it out
                comments.push(comment_attacher::Comment {
                    span: tok.span,
                    text: text.clone(),
                    is_trailing: *trailing,
                });
                None
            }
            _ => Some((tok.kind, tok.span)),
        })
        .collect();

    // Parse with recovery
    let eoi_span = tokens.last().map_or(Span::default(), |(_, s)| *s);
    let input = parser::make_input(tokens.as_slice(), eoi_span);
    let (tree, parse_errors) = parser::specification().parse(input).into_output_errors();

    let mut tree = tree.unwrap_or_default();

    // Attach collected comments to the AST
    let mut attacher = comment_attacher::CommentAttacher::new(comments);
    tree = attacher.attach(tree);

    // Collect parser errors
    let mut errors: Vec<Error> = parse_errors.into_iter().map(Error::from_rich).collect();

    // Process preprocessor errors and warnings
    errors.extend(process_preprocessor_errors(state.errors(), vfs));
    let warnings = process_preprocessor_warnings(state.warnings(), vfs);

    ParseResult {
        tree,
        errors,
        warnings,
        expansion_info: state
            .expansion_info
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect(),
    }
}

/// Constructs an AST from the given token iterator.
///
/// # Errors
///
/// If the given input contains IDL that is not syntactically valid, a
/// non-exhausitve list of parse errors will be returned that contains the
/// cause of each error and its span.
pub fn from_iter<I>(iter: I) -> ParseResult
where
    I: IntoIterator<Item = Token>,
{
    let tokens: Vec<(Kind, Span)> = iter.into_iter().map(|tok| (tok.kind, tok.span)).collect();

    let eoi_span = tokens.last().map_or(Span::default(), |(_, s)| *s);
    let input = parser::make_input(tokens.as_slice(), eoi_span);
    let (tree, parse_errors) = parser::specification().parse(input).into_output_errors();

    ParseResult {
        tree: tree.unwrap_or_default(),
        errors: parse_errors.into_iter().map(Error::from_rich).collect(),
        warnings: Vec::new(),
        expansion_info: std::collections::HashMap::default(),
    }
}
