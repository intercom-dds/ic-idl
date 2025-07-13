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

#![allow(clippy::cast_possible_truncation)]

use std::fmt;
use std::ops::Range;

use ic_cli::color::Colorize;
use ic_vfs::SourceMap;

use crate::{Color, Diag, Label};

/// Maximum number of lines to show before and after in a diagnostic span
const CONTEXT_LINES: usize = 2;
/// Maximum total lines to show for a single label span
const MAX_LINES_PER_SPAN: usize = 10;

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
            let title = format!("{}:", diag.title.text);
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

    fn emit_frame(&self, f: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
        if diag.labels.is_empty() {
            return Ok(());
        }

        // Calculate the maximum line number for proper indentation
        let mut max_line = 0;
        for label in &diag.labels {
            let end_line = line_number(self.source, label.span.end.offset as usize);
            max_line = max_line.max(end_line);
        }

        let indent = max_line.checked_ilog10().unwrap_or(0) as usize + 3;
        let indent = " ".repeat(indent);

        // Get location info from the first label
        let (first_line_number, col) = line_col(
            self.source,
            diag.labels.first().unwrap().span.start.offset as usize,
        );

        writeln!(
            f,
            "{indent}{} {}:{first_line_number}:{col}",
            self.chars.up_right.blue().bold(),
            self.filename.unwrap_or("unknown"),
        )?;
        writeln!(f, "{indent}{}", self.chars.vertical.blue().bold())?;

        // Collect line ranges for each label, preserving order
        let mut line_groups = Vec::new();
        let mut seen_lines = std::collections::HashSet::new();
        
        for label in &diag.labels {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);
            
            let mut group_lines = Vec::new();
            
            // If the span is too large, only show context around start and end
            let total_lines = end_line - start_line + 1;
            if total_lines > MAX_LINES_PER_SPAN {
                // Show first few lines
                for line in start_line..=start_line.saturating_add(CONTEXT_LINES) {
                    if line <= end_line && seen_lines.insert(line) {
                        group_lines.push(line);
                    }
                }
                
                // Add ellipsis marker (using line number 0 as a sentinel)
                group_lines.push(0);
                
                // Show last few lines
                for line in end_line.saturating_sub(CONTEXT_LINES)..=end_line {
                    if line > start_line.saturating_add(CONTEXT_LINES) && seen_lines.insert(line) {
                        group_lines.push(line);
                    }
                }
            } else {
                // Show all lines for small spans
                for line in start_line..=end_line {
                    if seen_lines.insert(line) {
                        group_lines.push(line);
                    }
                }
            }
            
            if !group_lines.is_empty() {
                line_groups.push(group_lines);
            }
        }

        // Display line groups in order
        for group in line_groups {
            for &line_num in &group {
                if line_num == 0 {
                    // Print ellipsis for skipped lines
                    writeln!(f, "{indent}{} {}", self.chars.vertical_dx.blue().bold(), "[...]")?;
                    continue;
                }
                
                write!(
                    f,
                    " {} {}",
                    line_num.blue().bold(),
                    self.chars.vertical.blue().bold(),
                )?;

                let line_start = self.line_start_offset(line_num);
                let range = line_span(self.source, line_start as u32);
                writeln!(f, " {}", self.source[range].trim_end())?;

                self.emit_labels_for_line(f, &indent, &diag.labels, line_num)?;
            }
        }

        writeln!(f, "{indent}{}", self.chars.down_right.blue().bold())
    }
    
    fn emit_frame_for_file(
        &self, 
        f: &mut dyn fmt::Write, 
        diag: &Diag, 
        label_indices: &[usize]
    ) -> fmt::Result {
        if label_indices.is_empty() {
            return Ok(());
        }

        // Get only the labels for this file
        let file_labels: Vec<&Label> = label_indices
            .iter()
            .map(|&idx| &diag.labels[idx])
            .collect();

        // Calculate the maximum line number for proper indentation
        let mut max_line = 0;
        for label in &file_labels {
            let end_line = line_number(self.source, label.span.end.offset as usize);
            max_line = max_line.max(end_line);
        }

        let indent = max_line.checked_ilog10().unwrap_or(0) as usize + 3;
        let indent = " ".repeat(indent);

        // Get location info from the first label
        let (first_line_number, col) = line_col(
            self.source,
            file_labels[0].span.start.offset as usize,
        );

        writeln!(
            f,
            "{indent}{} {}:{first_line_number}:{col}",
            self.chars.up_right.blue().bold(),
            self.filename.unwrap_or("unknown"),
        )?;
        writeln!(f, "{indent}{}", self.chars.vertical.blue().bold())?;

        // Collect line ranges for each label, preserving order
        let mut line_groups = Vec::new();
        let mut seen_lines = std::collections::HashSet::new();
        
        for &label in &file_labels {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);
            
            let mut group_lines = Vec::new();
            
            // If the span is too large, only show context around start and end
            let total_lines = end_line - start_line + 1;
            if total_lines > MAX_LINES_PER_SPAN {
                // Show first few lines
                for line in start_line..=start_line.saturating_add(CONTEXT_LINES) {
                    if line <= end_line && seen_lines.insert(line) {
                        group_lines.push(line);
                    }
                }
                
                // Add ellipsis marker (using line number 0 as a sentinel)
                group_lines.push(0);
                
                // Show last few lines
                for line in end_line.saturating_sub(CONTEXT_LINES)..=end_line {
                    if line > start_line.saturating_add(CONTEXT_LINES) && seen_lines.insert(line) {
                        group_lines.push(line);
                    }
                }
            } else {
                // Show all lines for small spans
                for line in start_line..=end_line {
                    if seen_lines.insert(line) {
                        group_lines.push(line);
                    }
                }
            }
            
            if !group_lines.is_empty() {
                line_groups.push(group_lines);
            }
        }

        // Display line groups in order
        for group in line_groups {
            for &line_num in &group {
                if line_num == 0 {
                    // Print ellipsis for skipped lines
                    writeln!(f, "{indent}{} {}", self.chars.vertical_dx.blue().bold(), "[...]")?;
                    continue;
                }
                
                write!(
                    f,
                    " {} {}",
                    line_num.blue().bold(),
                    self.chars.vertical.blue().bold(),
                )?;

                let line_start = self.line_start_offset(line_num);
                let range = line_span(self.source, line_start as u32);
                writeln!(f, " {}", self.source[range].trim_end())?;

                self.emit_labels_for_line_with_subset(f, &indent, &file_labels, line_num)?;
            }
        }

        writeln!(f, "{indent}{}", self.chars.down_right.blue().bold())
    }

    fn emit_labels_for_line(
        &self,
        f: &mut dyn fmt::Write,
        indent: &str,
        labels: &[Label],
        line_num: usize,
    ) -> fmt::Result {
        let labels_on_line = self.labels_on_line(labels, line_num);
        if labels_on_line.is_empty() {
            return Ok(());
        }

        let line_start_offset = self.line_start_offset(line_num) as u32;
        let line_range = line_span(self.source, line_start_offset);
        let line_len = self.source[line_range.clone()].trim_end().len();

        write!(f, "{indent}{} ", self.chars.vertical_dx.blue().bold())?;

        let col_labels =
            self.build_column_label_map(&labels_on_line, line_start_offset, line_num, line_len);
        self.draw_highlights(f, &col_labels)?;

        let labels_starting_here = self.labels_starting_on_line(&labels_on_line, line_num);
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

        write!(f, "{indent}{} ", self.chars.vertical_dx.blue().bold())?;

        let col_labels =
            self.build_column_label_map(&labels_on_line, line_start_offset, line_num, line_len);
        self.draw_highlights(f, &col_labels)?;

        let labels_starting_here = self.labels_starting_on_line(&labels_on_line, line_num);
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

    fn labels_on_line<'a>(&self, labels: &'a [Label], line_num: usize) -> Vec<&'a Label> {
        labels
            .iter()
            .filter(|label| {
                let start_line = line_number(self.source, label.span.start.offset as usize);
                let end_line = line_number(self.source, label.span.end.offset as usize);
                start_line <= line_num && line_num <= end_line
            })
            .collect()
    }

    fn build_column_label_map<'a>(
        &self,
        labels_on_line: &[&'a Label],
        line_start_offset: u32,
        line_num: usize,
        line_len: usize,
    ) -> Vec<Vec<&'a Label>> {
        let mut col_labels: Vec<Vec<&'a Label>> = vec![Vec::new(); line_len];

        for label in labels_on_line {
            let start_line = line_number(self.source, label.span.start.offset as usize);
            let end_line = line_number(self.source, label.span.end.offset as usize);
            
            // For multi-line spans, only draw highlights on first or last line
            if start_line != end_line && line_num != start_line && line_num != end_line {
                continue;
            }
            
            let label_start = label_start_on_line(label, line_start_offset);
            let label_end =
                label_end_on_line(self.source, label, line_start_offset, line_num, line_len);

            for labels_at_col in col_labels
                .iter_mut()
                .take(label_end.min(line_len))
                .skip(label_start)
            {
                labels_at_col.push(label);
            }
        }

        self.sort_labels_by_size(&mut col_labels, line_start_offset, line_num, line_len);
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
        let sorted_labels =
            self.sort_labels_for_display(labels_starting_here, line_start_offset, line_range);
        let labels_left_to_right = Self::sort_labels_left_to_right(&sorted_labels);

        if let Some(first) = sorted_labels.first() {
            writeln!(f, " {}", first.msg.bold().fg(first.color))?;

            for (i, label) in sorted_labels.iter().skip(1).enumerate() {
                write!(f, "{indent}{} ", self.chars.vertical_dx.blue())?;

                let label_col = (label.span.start.offset - line_start_offset) as usize;
                self.draw_vertical_connectors(
                    f,
                    &labels_left_to_right,
                    &sorted_labels,
                    i,
                    label_col,
                    line_start_offset,
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
    ) -> fmt::Result {
        let mut current_col = 0;

        for &other_label in labels_left_to_right {
            let other_col = (other_label.span.start.offset - line_start_offset) as usize;

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
    let title = format!("{}:", diag.title.text);
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
    let mut labels_by_file: std::collections::BTreeMap<ic_vfs::FileId, Vec<usize>> = std::collections::BTreeMap::new();
    for (idx, label) in diag.labels.iter().enumerate() {
        labels_by_file.entry(label.span.start.file_id).or_default().push(idx);
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
    let title = format!("{}:", diag.title.text);
    writeln!(
        f,
        "{} {filename}: {}",
        title.fg(diag.title.color).bold(),
        diag.msg.bold(),
    )
}
