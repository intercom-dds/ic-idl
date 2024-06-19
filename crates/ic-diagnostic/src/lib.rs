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

//! # ic-diagnostic
//!
//! Machinery for crafting diagnostics, mapping them to source files, and
//! pretty-printing them.

mod format;

use std::fmt;
use std::ops::Range;

use format::Line;

/// Different ways a diagnostic can be formatted.
pub enum Style {
    /// Single-line diagnostics that only include the location of the lint and
    /// its message. Suitable for logging.
    Compact,

    /// Pretty-printed, multi-line output that highlights the span of the lint.
    /// Includes hints and notes if they were specified.
    Pretty,
}

pub type Color = ic_cli::color::Color;

/// A single diagnostic intended to display lints about a particular item.
///
/// Each diagnostic should only address a single issue. If there are multiple
/// different warnings or errors for a particular item, you should instead
/// create multiple diagnostics.
#[must_use]
#[derive(Debug)]
pub struct Diag {
    msg: String,
    title: Line,
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
            color: Color::Yellow,
        };
        Self::with_title(title, message.into())
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
            help: None,
            warn: None,
            note: None,
            desc: None,
            labels: vec![],
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct Label {
    span: Range<usize>,
    msg: String,
    color: Color,
}

impl Label {
    pub fn new(span: impl Into<Range<usize>>) -> Self {
        Self {
            span: span.into(),
            msg: String::new(),
            color: Color::White,
        }
    }

    pub fn message<S: Into<String>>(mut self, msg: S) -> Self {
        self.msg = msg.into();
        self
    }

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
    Diag::warning(msg).label(label.color(Color::Yellow))
}

/// # Errors
///
/// May fail if writing to the given buffer fails.
pub fn emit_diagnostic(f: &mut dyn fmt::Write, source: &str, diag: &Diag) -> fmt::Result {
    format::with_source(f, source, diag)
}

/// A compact representation of the diagnostic. Only includes the origin of the
/// diagnostic; it does not include any of the labels, nor does it display any
/// parts of the source code.
///
/// # Errors
///
/// May fail if writing to the given buffer fails.
pub fn emit_compact(f: &mut dyn fmt::Write, filename: &str, diag: &Diag) -> fmt::Result {
    format::compact(f, filename, diag)
}
