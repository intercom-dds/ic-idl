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

use crate::Label;

pub const TAB_WIDTH: usize = 4;

pub const MAX_LINES_PER_SPAN: u32 = 10;

#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<u32>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line = self.line_starts.partition_point(|&start| start <= offset);
        let line_start = self.line_starts[line - 1];
        (line as u32, offset - line_start + 1)
    }

    pub fn line_start(&self, line: u32) -> u32 {
        self.line_starts
            .get(line as usize - 1)
            .copied()
            .unwrap_or(0)
    }

    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    pub fn visual_col(&self, source: &str, offset: u32) -> u32 {
        let line_start = self.line_starts[self.line_starts.partition_point(|&s| s <= offset) - 1];
        let line_slice = &source[line_start as usize..offset as usize];

        let mut col = 0u32;
        for ch in line_slice.chars() {
            if ch == '\t' {
                col = ((col / TAB_WIDTH as u32) + 1) * TAB_WIDTH as u32;
            } else {
                col += 1;
            }
        }
        col + 1
    }

    pub fn resolve_label(&self, source: &str, label: &Label, index: usize) -> LabelRef {
        let start_offset = label.span.start.offset;
        let end_offset = label.span.end.offset;

        let (start_line, start_col) = self.line_col(start_offset);
        let (end_line, end_col) = self.line_col(end_offset);

        let start_visual_col = self.visual_col(source, start_offset);
        let end_visual_col = self.visual_col(source, end_offset);

        LabelRef {
            label_index: index,
            start_line,
            start_col,
            end_line,
            end_col,
            start_visual_col,
            end_visual_col,
        }
    }
}

pub fn resolve_labels(index: &LineIndex, source: &str, labels: &[Label]) -> Vec<LabelRef> {
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| index.resolve_label(source, label, i))
        .collect()
}

pub fn compute_line_groups(
    labels: &[LabelRef],
    total_lines: u32,
    context_before: u32,
    context_after: u32,
    max_lines: u32,
) -> Vec<LineGroup> {
    if labels.is_empty() {
        return vec![];
    }

    let mut result: Vec<LineGroup> = vec![];

    for (idx, label) in labels.iter().enumerate() {
        let label_start = label.start_line;
        let label_end = label.end_line;
        let span_lines = label_end - label_start + 1;

        if span_lines > max_lines {
            let start_group_start = label_start.saturating_sub(context_before).max(1);
            let start_group_end = (label_start + context_after).min(total_lines);

            let end_group_start = label_end.saturating_sub(context_before).max(1);
            let end_group_end = (label_end + context_after).min(total_lines);

            if start_group_end + 1 >= end_group_start {
                let merged_start = start_group_start;
                let merged_end = end_group_end;
                merge_or_add_group(&mut result, merged_start, merged_end, idx, labels);
            } else {
                merge_or_add_group(&mut result, start_group_start, start_group_end, idx, labels);
                merge_or_add_group(&mut result, end_group_start, end_group_end, idx, labels);
            }
        } else {
            let context_start = label_start.saturating_sub(context_before).max(1);
            let context_end = (label_end + context_after).min(total_lines);
            merge_or_add_group(&mut result, context_start, context_end, idx, labels);
        }
    }

    result
}

fn merge_or_add_group(
    groups: &mut Vec<LineGroup>,
    start: u32,
    end: u32,
    label_idx: usize,
    labels: &[LabelRef],
) {
    for group in groups.iter_mut() {
        if start <= group.end_line + 1 && end >= group.start_line {
            group.start_line = group.start_line.min(start);
            group.end_line = group.end_line.max(end);
            if !group.labels.iter().any(|l| l.label_index == label_idx) {
                group.labels.push(labels[label_idx].clone());
            }
            return;
        }
    }

    groups.push(LineGroup {
        start_line: start,
        end_line: end,
        labels: vec![labels[label_idx].clone()],
        window: LineWindow::default(),
    });
}

const ELLIPSIS_LEN: u32 = 3;

pub fn compute_group_window(
    group: &LineGroup,
    source: &str,
    index: &LineIndex,
    max_width: Option<usize>,
) -> LineWindow {
    let common_indent = compute_common_indent(group, source, index);

    let Some(max_width) = max_width else {
        return LineWindow {
            common_indent,
            ..Default::default()
        };
    };

    if group.labels.is_empty() {
        return LineWindow {
            common_indent,
            ..Default::default()
        };
    }

    let mut max_line_len = 0u32;
    for line_num in group.start_line..=group.end_line {
        let line_start = index.line_start(line_num) as usize;
        let line_end = source[line_start..]
            .find('\n')
            .map_or(source.len(), |i| line_start + i);
        let line_text = &source[line_start..line_end];
        let visual_len = compute_visual_len(line_text).saturating_sub(common_indent);
        max_line_len = max_line_len.max(visual_len);
    }

    let max_width = max_width as u32;
    if max_line_len <= max_width {
        return LineWindow {
            common_indent,
            ..Default::default()
        };
    }

    let min_col = group
        .labels
        .iter()
        .map(|l| l.start_visual_col.saturating_sub(common_indent))
        .min()
        .unwrap_or(1)
        .max(1);
    let max_col = group
        .labels
        .iter()
        .map(|l| l.end_visual_col.saturating_sub(common_indent))
        .max()
        .unwrap_or(1);

    let label_span = max_col - min_col + 1;
    let will_truncate_left = min_col > 1;
    let left_ellipsis = if will_truncate_left { ELLIPSIS_LEN } else { 0 };
    let right_ellipsis = ELLIPSIS_LEN;
    let content_budget = max_width.saturating_sub(left_ellipsis + right_ellipsis);

    if content_budget == 0 {
        return LineWindow {
            start_col: min_col,
            end_col: min_col,
            truncate_left: will_truncate_left,
            truncate_right: true,
            common_indent,
        };
    }

    if label_span >= content_budget {
        return LineWindow {
            start_col: min_col,
            end_col: min_col + content_budget - 1,
            truncate_left: will_truncate_left,
            truncate_right: true,
            common_indent,
        };
    }

    let padding = content_budget - label_span;
    let left_pad = padding / 2;
    let start_col = min_col.saturating_sub(left_pad).max(1);

    let truncate_left = start_col > 1;
    let actual_left_ellipsis = if truncate_left { ELLIPSIS_LEN } else { 0 };
    let actual_content_budget = max_width.saturating_sub(actual_left_ellipsis + right_ellipsis);
    if actual_content_budget == 0 {
        return LineWindow {
            start_col,
            end_col: start_col,
            truncate_left,
            truncate_right: true,
            common_indent,
        };
    }
    let end_col = start_col + actual_content_budget - 1;
    let truncate_right = end_col < max_line_len;

    let final_end = if truncate_right {
        end_col
    } else {
        end_col + right_ellipsis
    };

    LineWindow {
        start_col,
        end_col: final_end.min(max_line_len),
        truncate_left,
        truncate_right,
        common_indent,
    }
}

fn compute_visual_len(line: &str) -> u32 {
    let mut len = 0u32;
    for ch in line.chars() {
        if ch == '\t' {
            len = ((len / TAB_WIDTH as u32) + 1) * TAB_WIDTH as u32;
        } else {
            len += 1;
        }
    }
    len
}

fn compute_line_indent(line: &str) -> u32 {
    let mut indent = 0u32;
    for ch in line.chars() {
        match ch {
            ' ' => indent += 1,
            '\t' => indent = ((indent / TAB_WIDTH as u32) + 1) * TAB_WIDTH as u32,
            _ => break,
        }
    }
    indent
}

fn compute_common_indent(group: &LineGroup, source: &str, index: &LineIndex) -> u32 {
    let mut min_indent: Option<u32> = None;

    for line_num in group.start_line..=group.end_line {
        let line_start = index.line_start(line_num) as usize;
        let line_end = source[line_start..]
            .find('\n')
            .map_or(source.len(), |i| line_start + i);
        let line_text = &source[line_start..line_end];

        if line_text.trim().is_empty() {
            continue;
        }

        let indent = compute_line_indent(line_text);
        min_indent = Some(min_indent.map_or(indent, |m| m.min(indent)));
    }

    min_indent.unwrap_or(0)
}

pub fn compute_frame_layout(
    index: &LineIndex,
    source: &str,
    labels: &[Label],
    max_width: Option<usize>,
    context_lines: u32,
) -> FrameLayout {
    let label_refs = resolve_labels(index, source, labels);
    let total_lines = index.line_count();

    let mut groups = compute_line_groups(
        &label_refs,
        total_lines,
        context_lines,
        context_lines,
        MAX_LINES_PER_SPAN,
    );

    let max_line = groups.iter().map(|g| g.end_line).max().unwrap_or(1);
    let gutter_width = (max_line.checked_ilog10().unwrap_or(0) + 1) as usize;

    let content_width = max_width.map(|w| w.saturating_sub(gutter_width + 5));

    for group in &mut groups {
        trim_blank_context(group, source, index);
        group.window = compute_group_window(group, source, index, content_width);
    }

    FrameLayout {
        line_groups: groups,
        gutter_width,
    }
}

fn trim_blank_context(group: &mut LineGroup, source: &str, index: &LineIndex) {
    if group.labels.is_empty() {
        return;
    }

    let first_label_line = group.labels.iter().map(|l| l.start_line).min().unwrap();
    let last_label_line = group.labels.iter().map(|l| l.end_line).max().unwrap();

    while group.start_line < first_label_line {
        if !is_blank_line(source, index, group.start_line) {
            break;
        }
        group.start_line += 1;
    }

    while group.end_line > last_label_line {
        if !is_blank_line(source, index, group.end_line) {
            break;
        }
        group.end_line -= 1;
    }
}

fn is_blank_line(source: &str, index: &LineIndex, line_num: u32) -> bool {
    let line_start = index.line_start(line_num) as usize;
    let line_end = source[line_start..]
        .find('\n')
        .map_or(source.len(), |i| line_start + i);
    source[line_start..line_end].trim().is_empty()
}

pub fn expand_tabs(s: &str) -> String {
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

pub fn visual_column(line: &str, byte_offset: usize) -> usize {
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

pub fn apply_window(line: &str, window: &LineWindow) -> String {
    let chars: Vec<char> = line.chars().collect();
    let total_len = chars.len();

    let common_indent = window.common_indent as usize;
    let after_indent: String = if common_indent < total_len {
        chars[common_indent..].iter().collect()
    } else {
        String::new()
    };

    if window.start_col == 0 && window.end_col == 0 {
        return after_indent;
    }

    let chars: Vec<char> = after_indent.chars().collect();
    let total_len = chars.len();

    let start_col = window.start_col.saturating_sub(1) as usize;
    let end_col = window.end_col as usize;

    if start_col >= total_len {
        return String::new();
    }

    let actual_end = end_col.min(total_len);
    let visible: String = chars[start_col..actual_end].iter().collect();

    let mut result = String::new();

    if window.truncate_left {
        result.push_str("...");
    }

    result.push_str(&visible);

    if window.truncate_right && actual_end < total_len {
        result.push_str("...");
    }

    result
}

#[derive(Debug, Clone)]
pub struct LabelRef {
    pub label_index: usize,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub start_visual_col: u32,
    pub end_visual_col: u32,
}

#[derive(Debug, Clone, Default)]
pub struct LineWindow {
    pub start_col: u32,
    pub end_col: u32,
    pub truncate_left: bool,
    pub truncate_right: bool,
    pub common_indent: u32,
}

#[derive(Debug, Clone)]
pub struct LineGroup {
    pub start_line: u32,
    pub end_line: u32,
    pub labels: Vec<LabelRef>,
    pub window: LineWindow,
}

#[derive(Debug)]
pub struct FrameLayout {
    pub line_groups: Vec<LineGroup>,
    pub gutter_width: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index_empty() {
        let idx = LineIndex::new("");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.line_col(0), (1, 1));
    }

    #[test]
    fn test_line_index_single_line() {
        let idx = LineIndex::new("hello");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.line_col(0), (1, 1));
        assert_eq!(idx.line_col(4), (1, 5));
    }

    #[test]
    fn test_line_index_multiple_lines() {
        let idx = LineIndex::new("line1\nline2\nline3");
        assert_eq!(idx.line_count(), 3);
        assert_eq!(idx.line_col(0), (1, 1));
        assert_eq!(idx.line_col(5), (1, 6));
        assert_eq!(idx.line_col(6), (2, 1));
        assert_eq!(idx.line_col(12), (3, 1));
    }

    #[test]
    fn test_line_start() {
        let idx = LineIndex::new("abc\ndef\nghi");
        assert_eq!(idx.line_start(1), 0);
        assert_eq!(idx.line_start(2), 4);
        assert_eq!(idx.line_start(3), 8);
    }

    #[test]
    fn test_visual_col_no_tabs() {
        let source = "hello";
        let idx = LineIndex::new(source);
        assert_eq!(idx.visual_col(source, 0), 1);
        assert_eq!(idx.visual_col(source, 4), 5);
    }

    #[test]
    fn test_visual_col_with_tabs() {
        let source = "\tx";
        let idx = LineIndex::new(source);
        assert_eq!(idx.visual_col(source, 0), 1);
        assert_eq!(idx.visual_col(source, 1), 5);
    }

    #[test]
    fn test_visual_col_mixed() {
        let source = " \tx";
        let idx = LineIndex::new(source);
        assert_eq!(idx.visual_col(source, 0), 1);
        assert_eq!(idx.visual_col(source, 1), 2);
        assert_eq!(idx.visual_col(source, 2), 5);
    }

    #[test]
    fn test_compute_line_groups_single_label() {
        let labels = vec![LabelRef {
            label_index: 0,
            start_line: 5,
            start_col: 1,
            end_line: 5,
            end_col: 10,
            start_visual_col: 1,
            end_visual_col: 10,
        }];

        let groups = compute_line_groups(&labels, 20, 2, 2, 10);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].start_line, 3);
        assert_eq!(groups[0].end_line, 7);
    }

    #[test]
    fn test_compute_line_groups_merge_adjacent() {
        let labels = vec![
            LabelRef {
                label_index: 0,
                start_line: 5,
                start_col: 1,
                end_line: 5,
                end_col: 10,
                start_visual_col: 1,
                end_visual_col: 10,
            },
            LabelRef {
                label_index: 1,
                start_line: 8,
                start_col: 1,
                end_line: 8,
                end_col: 10,
                start_visual_col: 1,
                end_visual_col: 10,
            },
        ];

        let groups = compute_line_groups(&labels, 20, 2, 2, 10);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].start_line, 3);
        assert_eq!(groups[0].end_line, 10);
        assert_eq!(groups[0].labels.len(), 2);
    }
}
