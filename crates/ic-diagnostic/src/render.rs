// Copyright 2026 KONGSBERG
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

#![allow(clippy::cast_possible_truncation)]

use std::fmt;

use ic_cli::color::Colorize as _;

use crate::engine::{
    FrameLayout, LabelRef, LineGroup, LineIndex, LineWindow, apply_window, expand_tabs,
    visual_column,
};
use crate::{Diag, Label};

#[derive(Debug, Clone, Copy, Default)]
struct WindowOffset {
    col_offset: usize,
    left_pad: usize,
    common_indent: usize,
}

impl WindowOffset {
    fn from_window(window: &LineWindow) -> Self {
        let col_offset = if window.start_col > 0 {
            (window.start_col - 1) as usize
        } else {
            0
        };
        let left_pad = if window.truncate_left { 3 } else { 0 };
        Self {
            col_offset,
            left_pad,
            common_indent: window.common_indent as usize,
        }
    }
}

struct ConnectorContext<'a> {
    labels_left_to_right: &'a [&'a LabelRef],
    labels_shown: &'a [&'a LabelRef],
    shown_index: usize,
    labels: &'a [Label],
}

#[derive(Debug)]
pub struct Charset {
    pub up_right: &'static str,
    pub down_right: &'static str,
    pub vertical: &'static str,
    pub vertical_dx: &'static str,
    pub highlight: &'static str,
    pub highlight_arrow: &'static str,
}

impl Charset {
    #[allow(dead_code)]
    pub fn ascii() -> Self {
        Self {
            up_right: "==>",
            down_right: "---",
            vertical: "|",
            vertical_dx: "+",
            highlight: "^",
            highlight_arrow: "`~~",
        }
    }

    pub fn unicode() -> Self {
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

pub struct Renderer<'a, W: fmt::Write + ?Sized> {
    writer: &'a mut W,
    index: &'a LineIndex,
    source: &'a str,
    charset: Charset,
    gutter_width: usize,
    filename: Option<&'a str>,
}

impl<'a, W: fmt::Write + ?Sized> Renderer<'a, W> {
    pub fn new(
        writer: &'a mut W,
        index: &'a LineIndex,
        source: &'a str,
        gutter_width: usize,
        filename: Option<&'a str>,
    ) -> Self {
        Self {
            writer,
            index,
            source,
            charset: Charset::unicode(),
            gutter_width,
            filename,
        }
    }

    pub fn render_diagnostic(&mut self, diag: &Diag, layout: &FrameLayout) -> fmt::Result {
        self.render_header(diag)?;

        if !diag.labels.is_empty() && !layout.line_groups.is_empty() {
            self.render_frame(layout, &diag.labels)?;
        }

        self.render_footer(diag)
    }

    pub fn render_header(&mut self, diag: &Diag) -> fmt::Result {
        if diag.msg.is_empty() {
            return Ok(());
        }

        let title = if let Some(code) = &diag.code {
            format!("{}[{}]:", diag.title.text, code)
        } else {
            format!("{}:", diag.title.text)
        };

        writeln!(
            self.writer,
            "{} {}",
            title.fg(diag.title.color).bold(),
            diag.msg.bold(),
        )
    }

    pub fn render_frame(&mut self, layout: &FrameLayout, labels: &[Label]) -> fmt::Result {
        if layout.line_groups.is_empty() {
            return Ok(());
        }

        let indent = " ".repeat(self.gutter_width + 2);

        let first_label = &labels[0];
        let (first_line, _) = self.index.line_col(first_label.span.start.offset);
        let visual_col = self
            .index
            .visual_col(self.source, first_label.span.start.offset);

        writeln!(
            self.writer,
            "{indent}{} {}:{}:{}",
            self.charset.up_right.blue().bold(),
            self.filename.unwrap_or("unknown"),
            first_line,
            visual_col,
        )?;
        writeln!(
            self.writer,
            "{indent}{}",
            self.charset.vertical.blue().bold()
        )?;

        let mut prev_line: Option<u32> = None;
        for group in &layout.line_groups {
            self.render_line_group(group, labels, &mut prev_line, &indent)?;
        }

        writeln!(
            self.writer,
            "{indent}{}",
            self.charset.down_right.blue().bold()
        )
    }

    pub fn render_line_group(
        &mut self,
        group: &LineGroup,
        labels: &[Label],
        prev_line: &mut Option<u32>,
        indent: &str,
    ) -> fmt::Result {
        for line_num in group.start_line..=group.end_line {
            if let Some(prev) = *prev_line
                && line_num > prev + 1
            {
                self.render_truncation_marker()?;
            }

            self.render_source_line(line_num, &group.window, &group.labels, labels)?;
            self.render_labels_for_line(line_num, indent, &group.window, &group.labels, labels)?;

            *prev_line = Some(line_num);
        }
        Ok(())
    }

    pub fn render_source_line(
        &mut self,
        line_num: u32,
        window: &LineWindow,
        _label_refs: &[LabelRef],
        _labels: &[Label],
    ) -> fmt::Result {
        let line_start = self.index.line_start(line_num) as usize;
        let line_end = self.source[line_start..]
            .find('\n')
            .map_or(self.source.len(), |i| line_start + i);
        let line_text = &self.source[line_start..line_end];
        let expanded = expand_tabs(line_text.trim_end());

        let display_text = apply_window(&expanded, window);

        write!(
            self.writer,
            " {} {}",
            format!("{:>width$}", line_num, width = self.gutter_width)
                .blue()
                .bold(),
            self.charset.vertical.blue().bold()
        )?;
        writeln!(self.writer, " {display_text}")
    }

    pub fn render_truncation_marker(&mut self) -> fmt::Result {
        writeln!(
            self.writer,
            " {} {}",
            format!("{:>width$}", "···", width = self.gutter_width).gray(),
            self.charset.vertical.blue().bold()
        )
    }

    pub fn render_footer(&mut self, diag: &Diag) -> fmt::Result {
        if diag.msg.is_empty() {
            return Ok(());
        }

        if let Some(v) = &diag.warn {
            writeln!(self.writer, " {} {v}", "warn:".purple().bold())?;
        }
        if let Some(v) = &diag.help {
            writeln!(self.writer, " {} {v}", "help:".cyan().bold())?;
        }
        if let Some(v) = &diag.note {
            writeln!(self.writer, " {} {v}", "note:".gray().bold())?;
        }
        if let Some(v) = &diag.desc {
            writeln!(self.writer, "\n{v}")?;
        }
        Ok(())
    }

    fn render_labels_for_line(
        &mut self,
        line_num: u32,
        indent: &str,
        window: &LineWindow,
        label_refs: &[LabelRef],
        labels: &[Label],
    ) -> fmt::Result {
        let labels_on_line: Vec<&LabelRef> = label_refs
            .iter()
            .filter(|lr| lr.start_line <= line_num && line_num <= lr.end_line)
            .collect();

        if labels_on_line.is_empty() {
            return Ok(());
        }

        let line_start = self.index.line_start(line_num) as usize;
        let line_end = self.source[line_start..]
            .find('\n')
            .map_or(self.source.len(), |i| line_start + i);
        let source_line = &self.source[line_start..line_end];
        let expanded_line = expand_tabs(source_line.trim_end());
        let visual_len = expanded_line.len();

        let col_labels = Self::build_column_label_map(
            &labels_on_line,
            line_num,
            visual_len,
            source_line,
            labels,
        );

        let has_highlights = col_labels.iter().any(|l| !l.is_empty());
        let labels_starting_here: Vec<&LabelRef> = labels_on_line
            .iter()
            .filter(|lr| lr.start_line == line_num)
            .copied()
            .collect();

        if !has_highlights && labels_starting_here.is_empty() {
            return Ok(());
        }

        write!(
            self.writer,
            "{indent}{} ",
            self.charset.vertical_dx.blue().bold()
        )?;

        let win_offset = WindowOffset::from_window(window);

        self.draw_highlights_windowed(&col_labels, labels, win_offset)?;

        if labels_starting_here.is_empty() {
            writeln!(self.writer)?;
        } else {
            self.draw_label_messages(
                indent,
                &labels_starting_here,
                source_line,
                labels,
                win_offset,
            )?;
        }

        Ok(())
    }

    fn build_column_label_map(
        labels_on_line: &[&LabelRef],
        line_num: u32,
        visual_len: usize,
        source_line: &str,
        labels: &[Label],
    ) -> Vec<Vec<usize>> {
        let mut col_labels: Vec<Vec<usize>> = vec![Vec::new(); visual_len];

        for &label_ref in labels_on_line {
            let start_line = label_ref.start_line;
            let end_line = label_ref.end_line;

            if start_line != end_line && line_num != start_line && line_num != end_line {
                continue;
            }

            let label_start_byte = if start_line == line_num {
                label_ref.start_col - 1
            } else {
                0
            };

            let label_end_byte = if end_line == line_num {
                label_ref.end_col - 1
            } else {
                source_line.trim_end().len() as u32
            };

            let label_start = visual_column(source_line, label_start_byte as usize);
            let label_end = visual_column(source_line, label_end_byte as usize);

            for labels_at_col in col_labels
                .iter_mut()
                .take(label_end.min(visual_len))
                .skip(label_start)
            {
                labels_at_col.push(label_ref.label_index);
            }
        }

        Self::sort_labels_by_size(&mut col_labels, labels);
        col_labels
    }

    fn sort_labels_by_size(col_labels: &mut [Vec<usize>], labels: &[Label]) {
        for labels_at_col in col_labels {
            labels_at_col
                .sort_by_key(|&idx| labels[idx].span.end.offset - labels[idx].span.start.offset);
        }
    }

    fn draw_highlights_windowed(
        &mut self,
        col_labels: &[Vec<usize>],
        labels: &[Label],
        win: WindowOffset,
    ) -> fmt::Result {
        let max_highlight_pos = col_labels.iter().rposition(|l| !l.is_empty());

        if win.left_pad > 0 {
            write!(self.writer, "{}", " ".repeat(win.left_pad))?;
        }

        if let Some(max_pos) = max_highlight_pos {
            let start = win.common_indent + win.col_offset;
            if start > max_pos {
                return Ok(());
            }

            let mut i = start;
            while i <= max_pos {
                if col_labels[i].is_empty() {
                    write!(self.writer, " ")?;
                    i += 1;
                } else {
                    let primary_idx = col_labels[i][0];
                    let primary_label = &labels[primary_idx];
                    let mut end = i + 1;
                    while end < col_labels.len()
                        && !col_labels[end].is_empty()
                        && col_labels[end][0] == primary_idx
                    {
                        end += 1;
                    }
                    let highlight_len = end - i;

                    write!(
                        self.writer,
                        "{}",
                        self.charset
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

    fn draw_label_messages(
        &mut self,
        indent: &str,
        labels_starting_here: &[&LabelRef],
        source_line: &str,
        labels: &[Label],
        win: WindowOffset,
    ) -> fmt::Result {
        let mut sorted_labels = labels_starting_here.to_vec();
        sorted_labels.sort_by(|a, b| {
            let a_end = a.end_visual_col;
            let b_end = b.end_visual_col;

            match b_end.cmp(&a_end) {
                std::cmp::Ordering::Equal => b.start_visual_col.cmp(&a.start_visual_col),
                other => other,
            }
        });

        let mut labels_left_to_right = sorted_labels.clone();
        labels_left_to_right.sort_by_key(|lr| lr.start_visual_col);

        if let Some(first) = sorted_labels.first() {
            let label = &labels[first.label_index];
            writeln!(self.writer, " {}", label.msg.bold().fg(label.color))?;

            for (i, &label_ref) in sorted_labels.iter().skip(1).enumerate() {
                let label = &labels[label_ref.label_index];
                write!(self.writer, "{indent}{} ", self.charset.vertical_dx.blue())?;

                let label_col = visual_column(source_line, (label_ref.start_col - 1) as usize);
                let ctx = ConnectorContext {
                    labels_left_to_right: &labels_left_to_right,
                    labels_shown: &sorted_labels,
                    shown_index: i,
                    labels,
                };
                self.draw_vertical_connectors(&ctx, label_col, source_line, win)?;

                writeln!(
                    self.writer,
                    "{} {}",
                    self.charset.highlight_arrow.fg(label.color),
                    label.msg.bold().fg(label.color)
                )?;
            }
        }
        Ok(())
    }

    fn draw_vertical_connectors(
        &mut self,
        ctx: &ConnectorContext<'_>,
        label_col: usize,
        source_line: &str,
        win: WindowOffset,
    ) -> fmt::Result {
        if win.left_pad > 0 {
            write!(self.writer, "{}", " ".repeat(win.left_pad))?;
        }

        let total_offset = win.common_indent + win.col_offset;
        let adjusted_label_col = label_col.saturating_sub(total_offset);
        let mut current_col = 0;

        for &other_lr in ctx.labels_left_to_right {
            let other_col = visual_column(source_line, (other_lr.start_col - 1) as usize);
            let adjusted_other_col = other_col.saturating_sub(total_offset);

            if other_col < total_offset || adjusted_other_col >= adjusted_label_col {
                continue;
            }

            let already_shown = ctx.labels_shown[0..=ctx.shown_index]
                .iter()
                .any(|&shown| shown.label_index == other_lr.label_index);

            if !already_shown && adjusted_other_col > current_col {
                write!(
                    self.writer,
                    "{}",
                    " ".repeat(adjusted_other_col - current_col)
                )?;
                let other_label = &ctx.labels[other_lr.label_index];
                write!(
                    self.writer,
                    "{}",
                    self.charset.vertical.fg(other_label.color)
                )?;
                current_col = adjusted_other_col + 1;
            }
        }

        if adjusted_label_col >= current_col {
            write!(
                self.writer,
                "{}",
                " ".repeat(adjusted_label_col - current_col)
            )?;
        }

        Ok(())
    }
}

pub fn render_diagnostic(
    writer: &mut dyn fmt::Write,
    index: &LineIndex,
    source: &str,
    diag: &Diag,
    filename: Option<&str>,
    max_width: Option<usize>,
) -> fmt::Result {
    use crate::engine::compute_frame_layout;

    let layout = compute_frame_layout(index, source, &diag.labels, max_width);
    let mut renderer = Renderer::new(writer, index, source, layout.gutter_width, filename);
    renderer.render_diagnostic(diag, &layout)
}

pub fn render_header(writer: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
    if diag.msg.is_empty() {
        return Ok(());
    }

    let title = if let Some(code) = &diag.code {
        format!("{}[{}]:", diag.title.text, code)
    } else {
        format!("{}:", diag.title.text)
    };

    writeln!(
        writer,
        "{} {}",
        title.fg(diag.title.color).bold(),
        diag.msg.bold(),
    )
}

pub fn render_footer(writer: &mut dyn fmt::Write, diag: &Diag) -> fmt::Result {
    if diag.msg.is_empty() {
        return Ok(());
    }

    if let Some(v) = &diag.warn {
        writeln!(writer, " {} {v}", "warn:".purple().bold())?;
    }
    if let Some(v) = &diag.help {
        writeln!(writer, " {} {v}", "help:".cyan().bold())?;
    }
    if let Some(v) = &diag.note {
        writeln!(writer, " {} {v}", "note:".gray().bold())?;
    }
    if let Some(v) = &diag.desc {
        writeln!(writer, "\n{v}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tabs() {
        assert_eq!(expand_tabs("hello"), "hello");
        assert_eq!(expand_tabs("\tx"), "    x");
        assert_eq!(expand_tabs(" \tx"), "    x");
        assert_eq!(expand_tabs("ab\tx"), "ab  x");
    }

    #[test]
    fn test_visual_column() {
        assert_eq!(visual_column("hello", 0), 0);
        assert_eq!(visual_column("hello", 3), 3);
        assert_eq!(visual_column("\tx", 0), 0);
        assert_eq!(visual_column("\tx", 1), 4);
    }
}
