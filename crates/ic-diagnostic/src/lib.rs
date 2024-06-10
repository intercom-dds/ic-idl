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

#![allow(clippy::needless_pass_by_value, unused, dead_code)]

//! # ic-diagnostic
//!
//! Machinery for crafting diagnostics, mapping them to source files, and
//! pretty-printing them.

use std::fmt::Write;
use std::ops::Range;

/// Different ways a diagnostic can be formatted.
pub enum Style {
    /// Single-line diagnostics that only include the location of the lint and
    /// its message. Suitable for logging.
    Short,

    /// Pretty-printed, multi-line output that highlights the span of the lint.
    /// Includes hints and notes if they were specified.
    Pretty,
}

pub enum Color {
    Red,
    Yellow,
    Blue,
}

/// A single diagnostic intended to display lints about a particular item.
#[must_use]
#[derive(Default, Debug)]
pub struct Diag {
    msg: String,
    note: Option<String>,
}

impl Diag {
    pub fn new() -> Self {
        Self {
            msg: String::new(),
            note: None,
        }
    }

    /// The main diagnostic message that should give a fairly short, concise
    /// description of what went wrong. Longer error messages can be added to
    /// the end of the output with [`with_note`].
    pub fn message<S: ToString>(mut self, msg: S) -> Self {
        self.msg = msg.to_string();
        self
    }

    /// A label is a message that will highlight the specified span of the
    /// source code and attach a a message to it.
    pub fn label(mut self, label: Label) -> Self {
        self
    }

    /// An optional description that will be displayed below the diagnostic.
    /// This can be used to give a longer, more descriptive reason of what
    /// triggered the diagnostic.
    pub fn description<T, S>(mut self, _title: S, note: S) -> Self
    where
        T: ToString,
        S: ToString,
    {
        self.note = Some(note.to_string());
        self
    }
}

// One label => text + line to span
#[must_use]
pub struct Label {
    span: Range<usize>,
    msg: Option<String>,
    color: Option<Color>,
}

impl Label {
    pub fn new(span: Range<usize>) -> Self {
        Self {
            span,
            msg: None,
            color: None,
        }
    }

    pub fn message<S: ToString>(mut self, msg: S) -> Self {
        self.msg = Some(msg.to_string());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Creates an error diagnostic that highlights the given span.
pub fn error_span<S: ToString>(msg: S, label: Label) -> Diag {
    Diag::new().message(msg).label(label.color(Color::Red))
}

/// Creates a warning diagnostic that highlights the given span.
pub fn warn_span<S: ToString>(msg: S, label: Label) -> Diag {
    Diag::new().message(msg).label(label.color(Color::Yellow))
}

struct Formatter {}

impl Formatter {}

// TODO: include file name and input here? so we're agnostic of SourceMap
// TODO: there may be multiple files, though...
pub fn emit_diagnostic<W: Write>(_w: &mut W, _lint: &Diag) {}
