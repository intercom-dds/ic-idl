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

use std::fmt;

use ic_lexer::token::{Kind, Kw};
use ic_vfs::Span;

/// The reason for a parse error.
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

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected => write!(f, "unexpected input"),
            Self::Unclosed { delimiter, .. } => write!(f, "unclosed delimiter {delimiter:?}"),
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

/// A parse error.
#[derive(Clone, Debug)]
pub struct Error {
    /// The token that was found (if any).
    pub found: Option<Kind>,

    /// What was expected at this position.
    pub expected: Option<Vec<Kind>>,

    /// The reason for the error.
    pub reason: Reason,

    /// Optional label for the span in diagnostics.
    pub label: Option<&'static str>,

    /// The span where the error occurred.
    pub span: Span,
}

impl Error {
    /// Creates a new parse error from expected tokens.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(span: Span, found: Option<Kind>, expected: Vec<Expected>) -> Self {
        // Convert Expected to Kind where possible
        let expected_kinds: Vec<Kind> = expected
            .iter()
            .filter_map(|e| match e {
                Expected::Token(k) => Some(*k),
                Expected::Keyword(kw) => Some(Kind::Keyword(*kw)),
                _ => None,
            })
            .collect();

        // Check for a custom message
        let reason = expected
            .iter()
            .find_map(|e| {
                if let Expected::Message(msg) = e {
                    Some(Reason::Custom((*msg).to_string()))
                } else {
                    None
                }
            })
            .unwrap_or(Reason::Unexpected);

        Self {
            found,
            expected: if expected_kinds.is_empty() {
                None
            } else {
                Some(expected_kinds)
            },
            reason,
            label: None,
            span,
        }
    }

    /// Creates an error with a custom message.
    #[must_use]
    pub fn custom(span: Span, message: impl Into<String>) -> Self {
        Self {
            found: None,
            expected: None,
            reason: Reason::Custom(message.into()),
            label: None,
            span,
        }
    }

    /// Sets the label for the span in diagnostics.
    #[must_use]
    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: syntax error: expected {:?}, found {:?}",
            self.span, self.found, self.expected,
        )
    }
}

impl std::error::Error for Error {}

/// Describes what was expected at a parse position (internal use).
#[derive(Clone, Debug)]
pub enum Expected {
    /// A specific token kind.
    Token(Kind),

    /// A specific keyword.
    Keyword(Kw),

    /// A description of what was expected (e.g., "identifier", "type").
    Desc(&'static str),

    /// A complete error message (replaces the default "expected X, found Y" format).
    Message(&'static str),
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(kind) => write!(f, "{kind:?}"),
            Self::Keyword(kw) => write!(f, "`{kw}`"),
            Self::Desc(desc) => write!(f, "{desc}"),
            Self::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<Kind> for Expected {
    fn from(kind: Kind) -> Self {
        Self::Token(kind)
    }
}

impl From<Kw> for Expected {
    fn from(kw: Kw) -> Self {
        Self::Keyword(kw)
    }
}

impl From<&'static str> for Expected {
    fn from(desc: &'static str) -> Self {
        Self::Desc(desc)
    }
}

/// Result type alias for parser operations.
pub type Result<T> = std::result::Result<T, ParseError>;

/// Internal parse error type used during parsing.
pub type ParseError = Error;
