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

#![allow(dead_code, clippy::cast_possible_truncation)]

use std::fmt;
use std::ops::Range;

use ic_cli::color::Colorize;
use ic_vfs::{SourceMap, Span};

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

#[derive(Clone, Debug)]
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
fn line_span(input: &str, offset: u32) -> Range<usize> {
    let start = input
        .bytes()
        .enumerate()
        .take(offset as usize)
        .rfind(|(_, v)| *v == b'\n')
        .map_or(0, |v| v.0 + 1);

    let end = input
        .trim_end()
        .bytes()
        .enumerate()
        .skip(offset as usize)
        .find(|(_, v)| *v == b'\n')
        .map_or(input.len(), |v| v.0);

    Range { start, end }
}

/// If the given span contains a newline, it will truncate the span to the
/// first line.
fn line_or_span(input: &str, span: &Span) -> Range<usize> {
    let end = input[span.start.offset as usize..span.end.offset as usize]
        .find('\n')
        .map_or(span.end.offset as usize, |v| span.start.offset as usize + v);

    Range {
        start: span.start.offset as usize,
        end,
    }
}

struct Formatter<'a> {
    filename: Option<&'a str>,
    source: &'a str,
    chars: Charset,
}

impl Formatter<'_> {
    fn get_line_start_offset(&self, line_num: usize) -> usize {
        let mut line_start = 0;
        let mut current_line = 1;
        for (idx, b) in self.source.bytes().enumerate() {
            if current_line == line_num {
                line_start = idx;
                break;
            }
            if b == b'\n' {
                current_line += 1;
            }
        }
        line_start
    }

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
        let (first_line_number, col) = line_col(
            self.source,
            diag.labels
                .first()
                .map_or(0, |v| v.span.start.offset as usize),
        );

        // Find the range of lines that contain labels
        let mut min_line = first_line_number;
        let mut max_line = first_line_number;

        for label in &diag.labels {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);
            min_line = min_line.min(start_line);
            max_line = max_line.max(end_line);
        }

        let indent = max_line.checked_ilog10().unwrap_or(0) as usize + 3;
        let indent = " ".repeat(indent);

        // Start the diagnostic frame by emitting the file name
        writeln!(
            f,
            "{indent}{} {}:{first_line_number}:{col}",
            self.chars.up_right.blue().bold(),
            self.filename.unwrap_or("unknown"),
        )?;
        writeln!(f, "{indent}{}", self.chars.vertical.blue().bold())?;

        // Emit all lines that contain labels
        for line_num in min_line..=max_line {
            // Fill in the line number
            write!(
                f,
                " {} {}",
                line_num.blue().bold(),
                self.chars.vertical.blue().bold(),
            )?;

            // Find the byte offset of the start of this line
            let line_start = self.get_line_start_offset(line_num);

            // Embed the line content
            let range = line_span(self.source, line_start as u32);
            writeln!(f, " {}", self.source[range].trim_end())?;

            // Draw labels for this line
            self.emit_labels_for_line(f, &indent, &diag.labels, line_num)?;
        }

        // Finish the frame
        writeln!(f, "{indent}{}", self.chars.down_right.blue().bold())
    }

    #[allow(clippy::too_many_lines)]
    fn emit_labels_for_line(
        &self,
        f: &mut dyn fmt::Write,
        indent: &str,
        labels: &[Label],
        line_num: usize,
    ) -> fmt::Result {
        // Find labels that affect this line
        let mut labels_on_line: Vec<&Label> = Vec::new();
        for label in labels {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);
            if start_line <= line_num && line_num <= end_line {
                labels_on_line.push(label);
            }
        }

        if labels_on_line.is_empty() {
            return Ok(());
        }

        // Sort labels by their start position
        labels_on_line.sort_unstable_by_key(|v| v.span.start.offset);

        // Get line boundaries
        let line_start_offset = self.get_line_start_offset(line_num) as u32;
        let line_range = line_span(self.source, line_start_offset);

        // Draw the highlight line
        write!(f, "{indent}{} ", self.chars.vertical_dx.blue().bold())?;

        // For overlapping spans, we need to handle them specially
        // Build a map of which labels cover each column
        let line_text = &self.source[line_range.clone()];
        let line_len = line_text.trim_end().len();

        // Track which labels cover each column position (can be multiple)
        let mut col_labels: Vec<Vec<&Label>> = vec![Vec::new(); line_len];

        // Process all labels and track which ones cover each position
        for label in &labels_on_line {
            let label_start_on_line = if label.span.start.offset >= line_start_offset {
                (label.span.start.offset - line_start_offset) as usize
            } else {
                0
            };

            let label_end_on_line =
                if line_number(self.source, label.span.end.offset as usize) > line_num {
                    line_len
                } else {
                    (label.span.end.offset - line_start_offset) as usize
                };

            // Mark columns covered by this label
            for labels_at_col in col_labels
                .iter_mut()
                .take(label_end_on_line.min(line_len))
                .skip(label_start_on_line)
            {
                labels_at_col.push(label);
            }
        }

        // Sort labels at each position by size (smaller first) for priority
        for labels_at_col in &mut col_labels {
            labels_at_col.sort_by_key(|label| {
                let start = if label.span.start.offset >= line_start_offset {
                    (label.span.start.offset - line_start_offset) as usize
                } else {
                    0
                };
                let end = if line_number(self.source, label.span.end.offset as usize) > line_num {
                    line_len
                } else {
                    (label.span.end.offset - line_start_offset) as usize
                };
                // Sort by span size (smaller first) for display priority
                end - start
            });
        }

        // Find the rightmost position that has a highlight
        let mut max_highlight_pos = None;
        for (i, labels_at_col) in col_labels.iter().enumerate() {
            if !labels_at_col.is_empty() {
                max_highlight_pos = Some(i);
            }
        }

        // Now draw the highlights up to the last highlighted position
        if let Some(max_pos) = max_highlight_pos {
            let mut i = 0;

            while i <= max_pos {
                if col_labels[i].is_empty() {
                    // No label here, just space
                    write!(f, " ")?;
                    i += 1;
                } else {
                    // Get the highest priority label (first in sorted list)
                    let primary_label = col_labels[i][0];

                    // Find the end of this continuous highlight with same primary label
                    let mut end = i + 1;
                    while end < line_len && !col_labels[end].is_empty() {
                        if std::ptr::eq(col_labels[end][0], primary_label) {
                            end += 1;
                        } else {
                            break;
                        }
                    }

                    // Always use the same highlight character, color will differentiate

                    // Draw the highlight
                    let highlight_len = end - i;
                    write!(
                        f,
                        "{}",
                        self.chars
                            .highlight
                            .repeat(highlight_len)
                            .bold()
                            .fg(primary_label.color),
                    )?;
                    i = end;
                }
            }
        }

        // After highlights, we should be at highlights_end_pos
        // Don't write extra spaces to line_len, just continue from where we are

        // Get labels that start on this line for message display
        let mut labels_starting_here: Vec<&Label> = labels_on_line
            .iter()
            .filter(|l| line_number(self.source, l.span.start.offset as usize) == line_num)
            .copied()
            .collect();

        if labels_starting_here.is_empty() {
            // No labels start on this line, just the highlights
            writeln!(f)?;
        } else {
            // For message display order, we want to show labels from right to left
            // based on where their highlights end on this line
            labels_starting_here.sort_by(|a, b| {
                // Calculate where each label's highlight ends on this line
                let a_end_col = if line_number(self.source, a.span.end.offset as usize) > line_num {
                    // Continues to next line, use end of current line
                    let line_text = &self.source[line_range.clone()];
                    line_text.trim_end().len()
                } else {
                    (a.span.end.offset - line_start_offset) as usize
                };

                let b_end_col = if line_number(self.source, b.span.end.offset as usize) > line_num {
                    let line_text = &self.source[line_range.clone()];
                    line_text.trim_end().len()
                } else {
                    (b.span.end.offset - line_start_offset) as usize
                };

                // Sort by end column on this line (rightmost first)
                match b_end_col.cmp(&a_end_col) {
                    std::cmp::Ordering::Equal => {
                        // If they end at the same place, smaller span first
                        let a_start_col = (a.span.start.offset - line_start_offset) as usize;
                        let b_start_col = (b.span.start.offset - line_start_offset) as usize;
                        b_start_col.cmp(&a_start_col)
                    }
                    other => other,
                }
            });

            // Keep a copy sorted left to right by start position for vertical line calculation
            let mut labels_left_to_right = labels_starting_here.clone();
            labels_left_to_right.sort_by_key(|v| v.span.start.offset);

            // Display first message inline after the highlights
            if let Some(first) = labels_starting_here.first() {
                // Add a space and the message on the same line
                writeln!(f, " {}", first.msg.bold().fg(first.color))?;

                // Display remaining messages with vertical connectors
                for (i, label) in labels_starting_here.iter().skip(1).enumerate() {
                    write!(f, "{indent}{} ", self.chars.vertical_dx.blue())?;

                    // Calculate column position of this label
                    let label_col = (label.span.start.offset - line_start_offset) as usize;

                    let mut current_col = 0;

                    // Draw vertical lines for labels that haven't been shown yet
                    // We need to draw lines for all labels to the left of the current one
                    // that haven't been displayed yet
                    for &other_label in &labels_left_to_right {
                        let other_col =
                            (other_label.span.start.offset - line_start_offset) as usize;

                        // Skip if this is at or after the current label position
                        if other_col >= label_col {
                            continue;
                        }

                        // Check if we've already shown this label
                        let already_shown = labels_starting_here[0..=i]
                            .iter()
                            .any(|&shown| std::ptr::eq(shown, other_label));

                        if !already_shown {
                            // Add padding to reach this column
                            if other_col > current_col {
                                write!(f, "{}", " ".repeat(other_col - current_col))?;
                                write!(f, "{}", self.chars.vertical.fg(other_label.color))?;
                                current_col = other_col + 1;
                            }
                        }
                    }

                    // Draw padding to reach the arrow position
                    if label_col >= current_col {
                        write!(f, "{}", " ".repeat(label_col - current_col))?;
                    }

                    // Draw arrow and message
                    writeln!(
                        f,
                        "{} {}",
                        self.chars.highlight_arrow.fg(label.color),
                        label.msg.bold().fg(label.color)
                    )?;
                }
            }
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

pub fn with_file(f: &mut dyn fmt::Write, vfs: &SourceMap, diag: &Diag) -> fmt::Result {
    // TODO; we assume all labels come form the same file -- this is not necessarily accurate.
    if let Some(label) = diag.labels.first() {
        let info = vfs.file_info(label.span.start.file_id);
        let name = info.included_as.to_string_lossy().to_string();
        with_source(f, &name, &info.source, diag)
    } else {
        Ok(())
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
