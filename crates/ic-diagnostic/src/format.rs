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

#![allow(dead_code)]

use std::fmt;
use std::ops::Range;

use ic_cli::color::Colorize;

use crate::{Color, Diag, Label};

#[derive(Debug)]
struct Charset {
    up_right: &'static str,
    down_right: &'static str,
    vertical: &'static str,
    vertical_dx: &'static str,
    highlight: &'static str,
    highlight_arrow: &'static str,
}

impl Charset {
    fn ascii() -> Self {
        Self {
            up_right: "==>",
            down_right: "---",
            vertical: "|",
            vertical_dx: "+",
            highlight: "^",
            highlight_arrow: "`~~",
        }
    }

    fn unicode() -> Self {
        Self {
            up_right: "┌",
            down_right: "└──",
            vertical: "│",
            vertical_dx: "·",
            highlight: "^",
            highlight_arrow: "└─",
        }
    }
}

#[derive(Debug)]
pub struct Line {
    pub text: &'static str,
    pub color: Color,
}

/// Converts a byte offset in a buffer to the corresponding line number.
fn line_number(input: &str, offset: usize) -> usize {
    input.bytes().take(offset).filter(|&v| v == b'\n').count() + 1
}

/// Converts a byte offset in a buffer to the corresponding (line, column) pair.
fn line_col(input: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut last_newl = 1;

    for (idx, b) in input.bytes().take(offset).enumerate() {
        if b == b'\n' {
            line += 1;
            last_newl = idx;
        }
    }
    (line, offset.checked_sub(last_newl).unwrap_or(1))
}

/// Returns the span of the line in which the given byte offset exists.
fn line_span(input: &str, offset: usize) -> Range<usize> {
    let start = input
        .bytes()
        .enumerate()
        .take(offset)
        .rfind(|(_, v)| *v == b'\n')
        .map_or(0, |v| v.0 + 1);

    let end = input
        .trim_end()
        .bytes()
        .enumerate()
        .skip(offset)
        .find(|(_, v)| *v == b'\n')
        .map_or(input.len(), |v| v.0);

    Range { start, end }
}

/// If the given span contains a newline, it will truncate the span to the
/// first line.
fn line_or_span(input: &str, span: &Range<usize>) -> Range<usize> {
    let end = input[span.start..span.end]
        .find('\n')
        .map_or(span.end, |v| span.start + v);

    Range {
        start: span.start,
        end,
    }
}

struct Formatter<'a> {
    filename: Option<&'a str>,
    source: &'a str,
    chars: Charset,
}

impl<'a> Formatter<'a> {
    fn report(self, f: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
        // Header
        let title = format!("{}:", diag.title.text);
        writeln!(
            f,
            "{} {}",
            title.fg(diag.title.color).bold(),
            diag.msg.bold(),
        )?;

        // The core of the diagnostic
        if !diag.labels.is_empty() {
            self.emit_frame(f, diag)?;
        }

        // Emit warnings/hints/notes if specified
        if let Some(v) = &diag.warn {
            writeln!(f, " {} {v}", "warn:".yellow().bold())?;
        }
        if let Some(v) = &diag.help {
            writeln!(f, " {} {v}", "help:".cyan().bold())?;
        }
        if let Some(v) = &diag.note {
            writeln!(f, " {} {v}", "note:".gray().bold())?;
        }
        if let Some(v) = &diag.desc {
            writeln!(f, "\n{v}")?;
        }
        Ok(())
    }

    fn emit_frame(&self, f: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
        // Determine the necessary indentation based on length of the linenu
        let (line_number, col) =
            line_col(self.source, diag.labels.first().map_or(0, |v| v.span.start));
        let indent = line_number.checked_ilog10().unwrap_or(0) as usize + 3;
        let indent = " ".repeat(indent);

        // Start the diagnostic frame by emitting the file name
        writeln!(
            f,
            "{indent}{} {}:{line_number}:{col}",
            self.chars.up_right.blue().bold(),
            self.filename.unwrap_or("unknown"),
        )?;
        writeln!(f, "{indent}{}", self.chars.vertical.blue().bold())?;

        // Fill in the line number
        write!(
            f,
            " {} {}",
            line_number.blue().bold(),
            self.chars.vertical.blue().bold(),
        )?;

        // Embed the origin of the diagnostic
        let range = line_span(self.source, diag.labels.first().map_or(0, |v| v.span.start));
        writeln!(f, " {}", self.source[range].trim_end())?;

        // Draw all labels
        self.emit_labels(f, &indent, &diag.labels)?;

        // Finish the frame
        writeln!(f, "{indent}{}", self.chars.down_right.blue().bold())
    }

    /// Highlights the relevant portion of the line. A highlight is a sequence
    /// of `^` characters.
    fn emit_highlight(&self, f: &mut dyn fmt::Write, ordered: &[&Label]) -> fmt::Result {
        let last_idx = ordered.first().map_or(0, |v| v.span.start);
        let mut last_idx = line_span(self.source, last_idx).start;

        for label in ordered {
            // Determine the indentation needed to reach the highlighted region
            let indent = " ".repeat(label.span.start.saturating_sub(last_idx));

            // Emit the highlight. If the label spans multiple lines, we
            // should only emit highlights for the first line.
            let line = line_or_span(self.source, &label.span);
            let len = self.source[line].chars().count();

            write!(
                f,
                "{indent}{}",
                self.chars.highlight.repeat(len).bold().fg(label.color),
            )?;
            last_idx = label.span.end;
        }
        Ok(())
    }

    fn emit_labels(&self, f: &mut dyn fmt::Write, indent: &str, labels: &[Label]) -> fmt::Result {
        // Order the labels from left to right, as determined by their
        // starting position.
        let mut ordered: Vec<_> = labels.iter().collect();
        ordered.sort_unstable_by_key(|v| v.span.start);
        write!(f, "{indent}{} ", self.chars.vertical_dx.blue().bold())?;

        // To properly format multiple labels, we start with the right-most
        // label and work our way towards the left.
        let mut iter = ordered.iter();

        // Iterate once over all labels and highlight the relevant portions
        self.emit_highlight(f, &ordered)?;

        // Emit the message of the last, right-most diagnostic
        if let Some(last) = iter.next_back() {
            writeln!(f, " {}", last.msg.bold().fg(last.color))?;
        }

        while let Some(label) = iter.next_back() {
            // Draw the line of the frame
            write!(f, "{indent}{} ", self.chars.vertical_dx.blue())?;

            // Iterate over all remaining labels, drawing a vertical line
            // at the appropriate location.
            let last_idx = ordered.first().map_or(0, |v| v.span.start);
            let mut last_idx = line_span(self.source, last_idx).start;

            for rem in iter.clone() {
                let padding = " ".repeat(rem.span.start - last_idx);
                last_idx = rem.span.start + 1;
                write!(f, "{padding}{}", self.chars.vertical.fg(rem.color))?;
            }

            // Emit the message of the current label
            let padding = " ".repeat(label.span.start - last_idx);
            write!(
                f,
                "{padding}{} ",
                self.chars.highlight_arrow.fg(label.color),
            )?;
            writeln!(f, "{}", label.msg.bold().fg(label.color))?;
        }
        Ok(())
    }
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = Formatter {
            filename: None,
            source: "",
            chars: Charset::unicode(),
        };
        fmt.report(f, self)
    }
}

pub fn with_source(f: &mut dyn fmt::Write, name: &str, source: &str, diag: &Diag) -> fmt::Result {
    let fmt = Formatter {
        filename: Some(name),
        source,
        chars: Charset::unicode(),
    };
    fmt.report(f, diag)
}

pub fn compact(f: &mut dyn fmt::Write, filename: &str, diag: &Diag) -> fmt::Result {
    let title = format!("{}:", diag.title.text);
    writeln!(
        f,
        "{} {filename}: {}",
        title.fg(diag.title.color).bold(),
        diag.msg.bold(),
    )
}
