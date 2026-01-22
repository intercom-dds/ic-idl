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

//! # ic-diagnostic
//!
//! Machinery for crafting diagnostics, mapping them to source files, and
//! pretty-printing them.

mod engine;
mod format;
mod render;

use std::collections::HashMap;
use std::fmt;

use engine::LineIndex;
use format::Line;
use ic_vfs::{FileId, SourceMap, Span};

/// Different ways a diagnostic can be formatted.
#[must_use]
pub enum Style {
    /// Single-line diagnostics that only include the location of the lint and
    /// its message. Suitable for logging.
    Compact,

    /// Pretty-printed, multi-line output that highlights the span of the lint.
    /// Includes hints and notes if they were specified.
    Pretty,
}

pub use ic_cli::color::Color;

/// Represents the severity level of a diagnostic.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Disabled,
}

/// A single diagnostic intended to display lints about a particular item.
///
/// Each diagnostic should only address a single issue. If there are multiple
/// different warnings or errors for a particular item, you should instead
/// create multiple diagnostics.
#[must_use]
#[derive(Clone, Debug)]
pub struct Diag {
    msg: String,
    title: Line,
    code: Option<String>,
    help: Option<String>,
    warn: Option<String>,
    note: Option<String>,
    desc: Option<String>,
    labels: Vec<Label>,
}

impl Diag {
    pub fn error<S: Into<String>>(message: S) -> Self {
        let title = Line {
            text: "error",
            color: Color::Red,
        };
        Self::with_title(title, message.into())
    }

    pub fn warning<S: Into<String>>(message: S) -> Self {
        let title = Line {
            text: "warning",
            color: Color::Purple,
        };
        Self::with_title(title, message.into())
    }

    /// Creates a diagnostic with the specified severity level.
    /// Returns None if the level is Disabled.
    pub fn with_level<S: Into<String>>(level: Level, message: S) -> Option<Self> {
        match level {
            Level::Error => Some(Self::error(message)),
            Level::Warning => Some(Self::warning(message)),
            Level::Disabled => None,
        }
    }

    /// Sets a diagnostic code (e.g. lint name) that will be displayed with the diagnostic.
    pub fn code<S: Into<String>>(mut self, code: S) -> Self {
        self.code = Some(code.into());
        self
    }

    /// The main diagnostic message that should give a fairly short, concise
    /// description of what went wrong. Longer error messages can be added to
    /// the end of the output with [`with_note`].
    pub fn message<S: Into<String>>(mut self, msg: S) -> Self {
        self.msg = msg.into();
        self
    }

    pub fn warn<S: Into<String>>(mut self, msg: S) -> Self {
        self.warn = Some(msg.into());
        self
    }

    pub fn help<S: Into<String>>(mut self, msg: S) -> Self {
        self.help = Some(msg.into());
        self
    }

    pub fn note<S: Into<String>>(mut self, msg: S) -> Self {
        self.note = Some(msg.into());
        self
    }

    /// A label is a message that will highlight the specified span of the
    /// source code and attach a a message to it.
    pub fn label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn labels<I>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = Label>,
    {
        for label in labels {
            self = self.label(label);
        }
        self
    }

    /// An optional description that will be displayed below the diagnostic.
    /// This can be used to give a longer, more descriptive reason of what
    /// triggered the diagnostic.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.desc = Some(description.into());
        self
    }

    fn with_title(title: Line, msg: String) -> Self {
        Self {
            title,
            msg,
            code: None,
            help: None,
            warn: None,
            note: None,
            desc: None,
            labels: vec![],
        }
    }
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.title.text, self.msg)
    }
}

/// A label that points to a specific location in the source code.
///
/// Labels are used to highlight specific spans of code that are relevant
/// to the diagnostic. Multiple labels can be attached to a single diagnostic
/// to show different parts of the code that contribute to the issue.
#[must_use]
#[derive(Clone, Debug)]
pub struct Label {
    span: Span,
    msg: String,
    color: Color,
}

impl Label {
    /// Creates a new label pointing to the given span.
    pub fn new(span: impl Into<Span>) -> Self {
        Self {
            span: span.into(),
            msg: String::new(),
            color: Color::White,
        }
    }

    /// Attaches a message to this label that will be displayed alongside the span.
    pub fn message<S: Into<String>>(mut self, msg: S) -> Self {
        self.msg = msg.into();
        self
    }

    /// Sets the color used to highlight this label's span.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Creates an error diagnostic that highlights the given span.
pub fn error_span<S: Into<String>>(msg: S, label: Label) -> Diag {
    Diag::error(msg).label(label.color(Color::Red))
}

/// Creates a warning diagnostic that highlights the given span.
pub fn warn_span<S: Into<String>>(msg: S, label: Label) -> Diag {
    Diag::warning(msg).label(label.color(Color::Purple))
}

/// Creates a diagnostic with the specified level that highlights the given span.
/// Returns None if the level is Disabled.
pub fn level_span<S: Into<String>>(level: Level, msg: S, label: Label) -> Option<Diag> {
    let color = match level {
        Level::Error => Color::Red,
        Level::Warning => Color::Purple,
        Level::Disabled => return None,
    };
    Diag::with_level(level, msg).map(|d| d.label(label.color(color)))
}

pub struct DiagnosticEmitter {
    line_indices: HashMap<FileId, LineIndex>,
    max_width: Option<usize>,
}

impl DiagnosticEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            line_indices: HashMap::new(),
            max_width: None,
        }
    }

    #[must_use]
    pub fn with_max_width(max_width: usize) -> Self {
        Self {
            line_indices: HashMap::new(),
            max_width: Some(max_width),
        }
    }

    pub fn set_max_width(&mut self, max_width: Option<usize>) {
        self.max_width = max_width;
    }

    /// # Errors
    ///
    /// May fail if writing to the given buffer fails.
    pub fn emit(&mut self, f: &mut dyn fmt::Write, vfs: &SourceMap, diag: &Diag) -> fmt::Result {
        format::with_file_cached(f, vfs, diag, &mut self.line_indices, self.max_width)
    }

    /// # Errors
    ///
    /// May fail if writing to the given buffer fails.
    pub fn emit_with_source(
        &mut self,
        f: &mut dyn fmt::Write,
        name: &str,
        source: &str,
        diag: &Diag,
    ) -> fmt::Result {
        let index = LineIndex::new(source);
        format::with_source_indexed(f, name, source, diag, &index, self.max_width)
    }

    /// # Errors
    ///
    /// May fail if writing to the given buffer fails.
    pub fn emit_compact(&self, f: &mut dyn fmt::Write, filename: &str, diag: &Diag) -> fmt::Result {
        format::compact(f, filename, diag)
    }
}

impl Default for DiagnosticEmitter {
    fn default() -> Self {
        Self::new()
    }
}
