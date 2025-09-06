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

#![allow(clippy::cast_possible_truncation, clippy::too_many_arguments)]

use std::fmt;
use std::ops::Range;

use ic_cli::color::Colorize;
use ic_vfs::SourceMap;

use crate::{Color, Diag, Label};

/// Maximum number of lines to show before a diagnostic span
const CONTEXT_LINES_BEFORE: usize = 2;

/// Maximum number of lines to show after a diagnostic span
const CONTEXT_LINES_AFTER: usize = 2;

/// Maximum total lines to show for a single label span
const MAX_LINES_PER_SPAN: usize = 10;

/// Tab width for display purposes
const TAB_WIDTH: usize = 4;

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
    #[allow(dead_code)]
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

/// Returns the total number of lines in the source.
fn total_lines(input: &str) -> usize {
    if input.is_empty() {
        0
    } else if input.ends_with('\n') {
        input.bytes().filter(|&v| v == b'\n').count()
    } else {
        input.bytes().filter(|&v| v == b'\n').count() + 1
    }
}

/// Converts a byte offset in a buffer to the corresponding (line, column) pair.
/// Column calculation accounts for tab characters (assumes 4-space tabs).
fn line_col(input: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut last_newl = 0;
    let mut col = 1;

    for (idx, b) in input.bytes().take(offset).enumerate() {
        if b == b'\n' {
            line += 1;
            last_newl = idx + 1;
            col = 1;
        }
    }

    // Calculate visual column, accounting for tabs
    let line_start = last_newl;
    if line_start < input.len() && offset > line_start {
        for b in input[line_start..offset.min(input.len())].bytes() {
            if b == b'\t' {
                // Tab moves to next multiple of 4
                col = ((col - 1) / 4 + 1) * 4 + 1;
            } else {
                col += 1;
            }
        }
    }

    (line, col)
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

/// Expands tabs to spaces in a string
fn expand_tabs(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut col = 0;

    for ch in s.chars() {
        if ch == '\t' {
            let spaces_to_add = TAB_WIDTH - (col % TAB_WIDTH);
            for _ in 0..spaces_to_add {
                result.push(' ');
                col += 1;
            }
        } else if ch == '\n' {
            result.push(ch);
            col = 0;
        } else {
            result.push(ch);
            col += 1;
        }
    }

    result
}

/// Returns the visual column position accounting for tabs
fn visual_column(line: &str, byte_offset: usize) -> usize {
    let mut col = 0;
    let mut byte_pos = 0;

    for ch in line.chars() {
        if byte_pos >= byte_offset {
            break;
        }
        if ch == '\t' {
            col = ((col / TAB_WIDTH) + 1) * TAB_WIDTH;
        } else {
            col += 1;
        }
        byte_pos += ch.len_utf8();
    }
    col
}

/// Returns the start column of a label on the given line.
fn label_start_on_line(label: &Label, line_start_offset: u32) -> usize {
    if label.span.start.offset >= line_start_offset {
        (label.span.start.offset - line_start_offset) as usize
    } else {
        0
    }
}

/// Finds the end position of a continuous highlight for the same label.
fn find_highlight_end(col_labels: &[Vec<&Label>], start: usize, primary_label: &Label) -> usize {
    let mut end = start + 1;
    while end < col_labels.len() && !col_labels[end].is_empty() {
        if std::ptr::eq(col_labels[end][0], primary_label) {
            end += 1;
        } else {
            break;
        }
    }
    end
}

/// Returns the end column of a label on the given line.
fn label_end_on_line(
    source: &str,
    label: &Label,
    line_start_offset: u32,
    line_num: usize,
    line_len: usize,
) -> usize {
    if line_number(source, label.span.end.offset as usize) > line_num {
        line_len
    } else {
        (label.span.end.offset - line_start_offset) as usize
    }
}

struct Formatter<'a> {
    filename: Option<&'a str>,
    source: &'a str,
    chars: Charset,
}

impl Formatter<'_> {
    fn line_start_offset(&self, line_num: usize) -> usize {
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
        // Only print title and message if message is not empty
        if !diag.msg.is_empty() {
            let title = if let Some(code) = &diag.code {
                format!("{}[{}]:", diag.title.text, code)
            } else {
                format!("{}:", diag.title.text)
            };
            writeln!(
                f,
                "{} {}",
                title.fg(diag.title.color).bold(),
                diag.msg.bold(),
            )?;
        }

        if !diag.labels.is_empty() {
            self.emit_frame(f, diag)?;
        }

        if !diag.msg.is_empty() {
            // Only print notes if we printed the message
            if let Some(v) = &diag.warn {
                writeln!(f, " {} {v}", "warn:".purple().bold())?;
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
        }
        Ok(())
    }

    fn is_line_empty(&self, line_num: usize) -> bool {
        let line_start = self.line_start_offset(line_num);
        let range = line_span(self.source, line_start as u32);
        self.source[range].trim().is_empty()
    }

    fn find_context_start(&self, span_start: usize) -> usize {
        let mut context_lines_found = 0;
        let mut current_line = span_start;

        // Walk backwards looking for non-empty lines
        for line in (1..span_start).rev() {
            if !self.is_line_empty(line) {
                current_line = line;
                context_lines_found += 1;
                if context_lines_found >= CONTEXT_LINES_BEFORE {
                    break;
                }
            }
        }

        current_line
    }

    fn find_context_end(&self, span_end: usize, total_lines: usize) -> usize {
        // If no context lines after, just return the span end
        if CONTEXT_LINES_AFTER == 0 {
            return span_end;
        }

        let mut context_lines_found = 0;
        let mut current_line = span_end;

        // Walk forwards looking for non-empty lines
        for line in (span_end + 1)..=total_lines {
            if !self.is_line_empty(line) {
                current_line = line;
                context_lines_found += 1;
                if context_lines_found >= CONTEXT_LINES_AFTER {
                    break;
                }
            }
        }

        current_line
    }

    fn build_line_groups_for_labels(
        &self,
        labels: &[&Label],
        total_source_lines: usize,
    ) -> (Vec<Vec<usize>>, usize) {
        let mut line_groups = Vec::new();
        let mut seen_lines = std::collections::HashSet::new();
        let mut max_line = 0;

        for &label in labels {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);

            let mut group_lines = Vec::new();

            // Find context boundaries (skipping empty lines for context)
            let context_start = self.find_context_start(start_line);
            let context_end = self.find_context_end(end_line, total_source_lines);

            // If the span (with context) is too large, only show context around start and end
            let total_lines = context_end - context_start + 1;
            if total_lines > MAX_LINES_PER_SPAN {
                // Show context before + a few lines into the span
                let first_part_end = self
                    .find_context_end(start_line, total_source_lines)
                    .min(end_line);
                for line in context_start..=first_part_end {
                    if seen_lines.insert(line) {
                        group_lines.push(line);
                        max_line = max_line.max(line);
                    }
                }

                // Add ellipsis marker (using line number 0 as a sentinel)
                if !group_lines.is_empty() && first_part_end < end_line {
                    group_lines.push(0);
                }

                // Show last few lines of span + context after
                let last_part_start = self.find_context_start(end_line).max(first_part_end + 1);
                for line in last_part_start..=context_end {
                    if seen_lines.insert(line) {
                        group_lines.push(line);
                        max_line = max_line.max(line);
                    }
                }
            } else {
                // Show all lines including context
                for line in context_start..=context_end {
                    if seen_lines.insert(line) {
                        group_lines.push(line);
                        max_line = max_line.max(line);
                    }
                }
            }

            if !group_lines.is_empty() {
                line_groups.push(group_lines);
            }
        }

        (line_groups, max_line)
    }

    fn calculate_line_width_and_indent(max_line: usize, has_gaps: bool) -> (usize, String) {
        // Calculate line width - at least 3 if we have ellipsis
        let line_width = max_line.checked_ilog10().unwrap_or(0) as usize + 1;
        let line_width = if has_gaps {
            line_width.max(3)
        } else {
            line_width
        };

        // Calculate proper indentation based on the line width we'll actually use
        let indent = " ".repeat(line_width + 2);

        (line_width, indent)
    }

    fn emit_frame_with_labels(&self, f: &mut dyn fmt::Write, labels: &[&Label]) -> fmt::Result {
        if labels.is_empty() {
            return Ok(());
        }

        // Get the total number of lines in the source
        let total_source_lines = total_lines(self.source);

        // Build line groups
        let (line_groups, max_line) = self.build_line_groups_for_labels(labels, total_source_lines);

        // Flatten all line groups into a single sorted list, removing duplicates and ellipsis markers
        let all_lines: Vec<usize> = line_groups
            .iter()
            .flat_map(|group| group.iter())
            .filter(|&&line| line != 0) // Remove ellipsis markers
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        // Check if we'll need ellipsis anywhere
        let mut has_gaps = false;
        if all_lines.len() > 1 {
            for i in 1..all_lines.len() {
                if all_lines[i] > all_lines[i - 1] + 1 {
                    has_gaps = true;
                    break;
                }
            }
        }

        let (line_width, indent) = Self::calculate_line_width_and_indent(max_line, has_gaps);

        // Get location info from the first label
        let (first_line_number, col) = line_col(self.source, labels[0].span.start.offset as usize);

        writeln!(
            f,
            "{indent}{} {}:{first_line_number}:{col}",
            self.chars.up_right.blue().bold(),
            self.filename.unwrap_or("unknown"),
        )?;
        writeln!(f, "{indent}{}", self.chars.vertical.blue().bold())?;

        // Display lines with ellipsis for gaps
        let mut prev_line = None;
        for &line_num in &all_lines {
            // Check if we need to insert ellipsis
            if let Some(prev) = prev_line {
                if line_num > prev + 1 {
                    // There's a gap, insert ellipsis
                    writeln!(
                        f,
                        " {:>width$} {}",
                        "···".gray(),
                        self.chars.vertical.blue().bold(),
                        width = line_width
                    )?;
                }
            }

            write!(
                f,
                " {:>width$} {}",
                line_num.blue().bold(),
                self.chars.vertical.blue().bold(),
                width = line_width
            )?;

            let line_start = self.line_start_offset(line_num);
            let range = line_span(self.source, line_start as u32);
            let line_text = expand_tabs(self.source[range].trim_end());
            writeln!(f, " {line_text}")?;

            // Always use the subset method since it handles both cases
            self.emit_labels_for_line_with_subset(f, &indent, labels, line_num)?;

            prev_line = Some(line_num);
        }

        writeln!(f, "{indent}{}", self.chars.down_right.blue().bold())
    }

    fn emit_frame(&self, f: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
        if diag.labels.is_empty() {
            return Ok(());
        }

        let label_refs: Vec<&Label> = diag.labels.iter().collect();
        self.emit_frame_with_labels(f, &label_refs)
    }

    fn emit_frame_for_file(
        &self,
        f: &mut dyn fmt::Write,
        diag: &Diag,
        label_indices: &[usize],
    ) -> fmt::Result {
        if label_indices.is_empty() {
            return Ok(());
        }

        // Get only the labels for this file
        let file_labels: Vec<&Label> = label_indices.iter().map(|&idx| &diag.labels[idx]).collect();
        self.emit_frame_with_labels(f, &file_labels)
    }

    fn emit_labels_for_line_with_subset(
        &self,
        f: &mut dyn fmt::Write,
        indent: &str,
        labels: &[&Label],
        line_num: usize,
    ) -> fmt::Result {
        let labels_on_line: Vec<&Label> = labels
            .iter()
            .filter(|label| {
                let start_line = line_number(self.source, label.span.start.offset as usize);
                let end_line = line_number(self.source, label.span.end.offset as usize);
                start_line <= line_num && line_num <= end_line
            })
            .copied()
            .collect();

        if labels_on_line.is_empty() {
            return Ok(());
        }

        let line_start_offset = self.line_start_offset(line_num) as u32;
        let line_range = line_span(self.source, line_start_offset);
        let line_len = self.source[line_range.clone()].trim_end().len();

        let col_labels =
            self.build_column_label_map(&labels_on_line, line_start_offset, line_num, line_len);

        // Check if we actually have any highlights to draw
        let has_highlights = col_labels.iter().any(|labels| !labels.is_empty());

        let labels_starting_here = self.labels_starting_on_line(&labels_on_line, line_num);

        // Only emit the label line if we have highlights or messages to show
        if !has_highlights && labels_starting_here.is_empty() {
            return Ok(());
        }

        write!(f, "{indent}{} ", self.chars.vertical_dx.blue().bold())?;
        self.draw_highlights(f, &col_labels)?;

        if labels_starting_here.is_empty() {
            writeln!(f)?;
        } else {
            self.draw_label_messages(
                f,
                indent,
                &labels_starting_here,
                line_start_offset,
                &line_range,
            )?;
        }

        Ok(())
    }

    fn build_column_label_map<'a>(
        &self,
        labels_on_line: &[&'a Label],
        line_start_offset: u32,
        line_num: usize,
        _line_len: usize,
    ) -> Vec<Vec<&'a Label>> {
        // Get the actual source line to calculate visual columns
        let range = line_span(self.source, line_start_offset);
        let source_line = &self.source[range];
        let expanded_line = expand_tabs(source_line.trim_end());
        let visual_len = expanded_line.len();

        let mut col_labels: Vec<Vec<&'a Label>> = vec![Vec::new(); visual_len];

        for label in labels_on_line {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);

            // For multi-line spans, only draw highlights on first or last line
            if start_line != end_line && line_num != start_line && line_num != end_line {
                continue;
            }

            let label_start_byte = label_start_on_line(label, line_start_offset);
            let label_end_byte = label_end_on_line(
                self.source,
                label,
                line_start_offset,
                line_num,
                source_line.trim_end().len(),
            );

            // Convert byte offsets to visual columns
            let label_start = visual_column(source_line, label_start_byte);
            let label_end = visual_column(source_line, label_end_byte);

            for labels_at_col in col_labels
                .iter_mut()
                .take(label_end.min(visual_len))
                .skip(label_start)
            {
                labels_at_col.push(label);
            }
        }

        self.sort_labels_by_size(&mut col_labels, line_start_offset, line_num, visual_len);
        col_labels
    }

    fn sort_labels_by_size(
        &self,
        col_labels: &mut [Vec<&Label>],
        line_start_offset: u32,
        line_num: usize,
        line_len: usize,
    ) {
        for labels_at_col in col_labels {
            labels_at_col.sort_by_key(|label| {
                let start = label_start_on_line(label, line_start_offset);
                let end =
                    label_end_on_line(self.source, label, line_start_offset, line_num, line_len);
                end - start
            });
        }
    }

    fn draw_highlights(&self, f: &mut dyn fmt::Write, col_labels: &[Vec<&Label>]) -> fmt::Result {
        let max_highlight_pos = col_labels.iter().rposition(|labels| !labels.is_empty());

        if let Some(max_pos) = max_highlight_pos {
            let mut i = 0;
            while i <= max_pos {
                if col_labels[i].is_empty() {
                    write!(f, " ")?;
                    i += 1;
                } else {
                    let primary_label = col_labels[i][0];
                    let end = find_highlight_end(col_labels, i, primary_label);
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
        Ok(())
    }

    fn labels_starting_on_line<'a>(
        &self,
        labels_on_line: &[&'a Label],
        line_num: usize,
    ) -> Vec<&'a Label> {
        labels_on_line
            .iter()
            .filter(|l| line_number(self.source, l.span.start.offset as usize) == line_num)
            .copied()
            .collect()
    }

    fn draw_label_messages(
        &self,
        f: &mut dyn fmt::Write,
        indent: &str,
        labels_starting_here: &[&Label],
        line_start_offset: u32,
        line_range: &Range<usize>,
    ) -> fmt::Result {
        let source_line = &self.source[line_range.clone()];
        let sorted_labels =
            self.sort_labels_for_display(labels_starting_here, line_start_offset, line_range);
        let labels_left_to_right = Self::sort_labels_left_to_right(&sorted_labels);

        if let Some(first) = sorted_labels.first() {
            writeln!(f, " {}", first.msg.bold().fg(first.color))?;

            for (i, label) in sorted_labels.iter().skip(1).enumerate() {
                write!(f, "{indent}{} ", self.chars.vertical_dx.blue())?;

                let label_byte_offset = (label.span.start.offset - line_start_offset) as usize;
                let label_col = visual_column(source_line, label_byte_offset);
                self.draw_vertical_connectors(
                    f,
                    &labels_left_to_right,
                    &sorted_labels,
                    i,
                    label_col,
                    line_start_offset,
                    source_line,
                )?;

                writeln!(
                    f,
                    "{} {}",
                    self.chars.highlight_arrow.fg(label.color),
                    label.msg.bold().fg(label.color)
                )?;
            }
        }
        Ok(())
    }

    fn sort_labels_for_display<'a>(
        &self,
        labels: &[&'a Label],
        line_start_offset: u32,
        line_range: &Range<usize>,
    ) -> Vec<&'a Label> {
        let mut sorted = labels.to_vec();
        sorted.sort_by(|a, b| {
            let a_end_col = self.label_end_col(a, line_start_offset, line_range);
            let b_end_col = self.label_end_col(b, line_start_offset, line_range);

            match b_end_col.cmp(&a_end_col) {
                std::cmp::Ordering::Equal => {
                    let a_start_col = (a.span.start.offset - line_start_offset) as usize;
                    let b_start_col = (b.span.start.offset - line_start_offset) as usize;
                    b_start_col.cmp(&a_start_col)
                }
                other => other,
            }
        });
        sorted
    }

    fn label_end_col(
        &self,
        label: &Label,
        line_start_offset: u32,
        line_range: &Range<usize>,
    ) -> usize {
        let current_line = line_number(self.source, line_start_offset as usize);
        let line_len = self.source[line_range.clone()].trim_end().len();
        label_end_on_line(
            self.source,
            label,
            line_start_offset,
            current_line,
            line_len,
        )
    }

    fn sort_labels_left_to_right<'a>(labels: &[&'a Label]) -> Vec<&'a Label> {
        let mut sorted = labels.to_vec();
        sorted.sort_by_key(|v| v.span.start.offset);
        sorted
    }

    fn draw_vertical_connectors(
        &self,
        f: &mut dyn fmt::Write,
        labels_left_to_right: &[&Label],
        labels_shown: &[&Label],
        shown_index: usize,
        label_col: usize,
        line_start_offset: u32,
        source_line: &str,
    ) -> fmt::Result {
        let mut current_col = 0;

        for &other_label in labels_left_to_right {
            let other_byte_offset = (other_label.span.start.offset - line_start_offset) as usize;
            let other_col = visual_column(source_line, other_byte_offset);

            if other_col >= label_col {
                continue;
            }

            let already_shown = labels_shown[0..=shown_index]
                .iter()
                .any(|&shown| std::ptr::eq(shown, other_label));

            if !already_shown && other_col > current_col {
                write!(f, "{}", " ".repeat(other_col - current_col))?;
                write!(f, "{}", self.chars.vertical.fg(other_label.color))?;
                current_col = other_col + 1;
            }
        }

        if label_col >= current_col {
            write!(f, "{}", " ".repeat(label_col - current_col))?;
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
    // First, emit the main error message
    let title = if let Some(code) = &diag.code {
        format!("{}[{}]:", diag.title.text, code)
    } else {
        format!("{}:", diag.title.text)
    };
    writeln!(
        f,
        "{} {}",
        title.fg(diag.title.color).bold(),
        diag.msg.bold(),
    )?;

    if diag.labels.is_empty() {
        emit_notes(f, diag)?;
        return Ok(());
    }

    // Group labels by file ID
    let mut labels_by_file: std::collections::BTreeMap<ic_vfs::FileId, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (idx, label) in diag.labels.iter().enumerate() {
        labels_by_file
            .entry(label.span.start.file_id)
            .or_default()
            .push(idx);
    }

    // Process each file's labels
    for (file_id, label_indices) in labels_by_file {
        let info = vfs.file_info(file_id);
        let name = info.included_as.to_string_lossy().to_string();

        let fmt = Formatter {
            filename: Some(&name),
            source: &info.source,
            chars: Charset::unicode(),
        };

        // Emit frame header for this file
        fmt.emit_frame_for_file(f, diag, &label_indices)?;
    }

    emit_notes(f, diag)?;
    Ok(())
}

fn emit_notes(f: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
    if let Some(v) = &diag.warn {
        writeln!(f, " {} {v}", "warn:".purple().bold())?;
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

pub fn with_source(f: &mut dyn fmt::Write, name: &str, source: &str, diag: &Diag) -> fmt::Result {
    let fmt = Formatter {
        filename: Some(name),
        source,
        chars: Charset::unicode(),
    };
    fmt.report(f, diag)
}

pub fn compact(f: &mut dyn fmt::Write, filename: &str, diag: &Diag) -> fmt::Result {
    let title = if let Some(code) = &diag.code {
        format!("{}[{}]:", diag.title.text, code)
    } else {
        format!("{}:", diag.title.text)
    };
    writeln!(
        f,
        "{} {filename}: {}",
        title.fg(diag.title.color).bold(),
        diag.msg.bold(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_empty_input() {
        // Test with empty input
        assert_eq!(line_col("", 0), (1, 1));
        // Test with offset beyond empty input
        assert_eq!(line_col("", 10), (1, 1));
    }

    #[test]
    fn test_line_col_no_tabs() {
        // Test basic column calculation without tabs
        assert_eq!(line_col("hello", 0), (1, 1));
        assert_eq!(line_col("hello", 1), (1, 2));
        assert_eq!(line_col("hello", 4), (1, 5));

        // Test with content like in debug_overlap_test
        let source = "void process(struct Data { int x; int y; } data);";
        assert_eq!(line_col(source, 0), (1, 1)); // 'v'
        assert_eq!(line_col(source, 13), (1, 14)); // 's' in 'struct'
        assert_eq!(line_col(source, 27), (1, 28)); // 'i' in 'int x'
    }

    #[test]
    fn test_line_col_with_newlines() {
        let source = "line1\nline2\nline3";
        assert_eq!(line_col(source, 0), (1, 1)); // 'l' in line1
        assert_eq!(line_col(source, 5), (1, 6)); // '\n' after line1
        assert_eq!(line_col(source, 6), (2, 1)); // 'l' in line2
        assert_eq!(line_col(source, 11), (2, 6)); // '\n' after line2
        assert_eq!(line_col(source, 12), (3, 1)); // 'l' in line3
    }

    #[test]
    fn test_line_col_with_tabs() {
        // Test tab expansion
        assert_eq!(line_col("\tx", 0), (1, 1)); // '\t'
        assert_eq!(line_col("\tx", 1), (1, 5)); // 'x' after tab (col 5)

        // Test mixed spaces and tabs
        assert_eq!(line_col(" \tx", 0), (1, 1)); // ' '
        assert_eq!(line_col(" \tx", 1), (1, 2)); // '\t'
        assert_eq!(line_col(" \tx", 2), (1, 5)); // 'x' (space + tab = 4 cols)

        // Test multiple tabs
        assert_eq!(line_col("\t\tx", 0), (1, 1)); // first '\t'
        assert_eq!(line_col("\t\tx", 1), (1, 5)); // second '\t'
        assert_eq!(line_col("\t\tx", 2), (1, 9)); // 'x' after 2 tabs
    }

    #[test]
    fn test_line_col_offset_beyond_input() {
        // Test offset beyond input length
        assert_eq!(line_col("hi", 10), (1, 3)); // Should handle gracefully
        assert_eq!(line_col("a\nb", 10), (2, 2)); // Should count lines correctly
    }
}
