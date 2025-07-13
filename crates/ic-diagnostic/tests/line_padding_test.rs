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

#![allow(clippy::cast_possible_truncation)]

use ic_diagnostic::{Diag, Label, emit_with_source};
use ic_vfs::{FileId, Location, Span};

fn make_span(file_id: FileId, start: u32, end: u32) -> Span {
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn test_line_padding_two_digits() {
    let source = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10";
    let file_id = FileId::_do_not_use();

    // Create labels on lines 1 and 10 to test 1-digit vs 2-digit padding
    let diag = Diag::error("test error message")
        .label(Label::new(make_span(file_id, 0, 6)).message("on line 1"))
        .label(Label::new(make_span(file_id, 63, 70)).message("on line 10"));

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_line_padding_three_digits() {
    // Create a source with 100+ lines
    let mut lines = Vec::new();
    let mut offsets = Vec::new();
    let mut current_offset = 0;

    for i in 1..=105 {
        let line = format!("line {i}");
        offsets.push((i, current_offset, current_offset + line.len()));
        current_offset += line.len() + 1; // +1 for newline
        lines.push(line);
    }
    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Get exact offsets for lines 1, 50, and 100
    let (_, start_1, end_1) = offsets[0];
    let (_, start_50, end_50) = offsets[49];
    let (_, start_100, end_100) = offsets[99];

    // Create labels on lines 1, 50, and 100 to test different digit counts
    let diag = Diag::error("multiple labels with different line number widths")
        .label(Label::new(make_span(file_id, start_1 as u32, end_1 as u32)).message("on line 1"))
        .label(Label::new(make_span(file_id, start_50 as u32, end_50 as u32)).message("on line 50"))
        .label(
            Label::new(make_span(file_id, start_100 as u32, end_100 as u32)).message("on line 100"),
        );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_line_padding_with_multiline_spans() {
    // Test that padding works correctly with multi-line spans
    let mut lines = Vec::new();
    let mut offsets = Vec::new();
    let mut current_offset = 0;

    for i in 1..=150 {
        let line = format!("line {i}");
        offsets.push((i, current_offset, current_offset + line.len()));
        current_offset += line.len() + 1; // +1 for newline
        lines.push(line);
    }
    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Get exact offsets - span from line 5 to line 8, and line 145
    let (_, start_5, _) = offsets[4];
    let (_, _, end_8) = offsets[7];
    let (_, start_145, end_145) = offsets[144];

    // Create a multi-line span from line 5-8 and another label on line 145
    let diag = Diag::error("multi-line span with distant label")
        .label(
            Label::new(make_span(file_id, start_5 as u32, end_8 as u32)).message("spans lines 5-8"),
        )
        .label(
            Label::new(make_span(file_id, start_145 as u32, end_145 as u32)).message("on line 145"),
        );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_line_padding_with_ellipsis() {
    // Test padding with large spans that trigger ellipsis
    let mut lines = Vec::new();
    let mut offsets = Vec::new();
    let mut current_offset = 0;

    for i in 1..=200 {
        let line = format!("line {i}");
        offsets.push((i, current_offset, current_offset + line.len()));
        current_offset += line.len() + 1; // +1 for newline
        lines.push(line);
    }
    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Get exact offsets for a very large span from line 10 to line 190
    let (_, start_10, _) = offsets[9];
    let (_, _, end_190) = offsets[189];

    // Create a very large span from line 10 to line 190
    let diag = Diag::error("large span with ellipsis").label(
        Label::new(make_span(file_id, start_10 as u32, end_190 as u32)).message("huge span"),
    );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_line_padding_four_digits() {
    // Create a source with 10000+ lines
    let mut lines = Vec::new();
    let mut offsets = Vec::new();
    let mut current_offset = 0;

    for i in 1..=10005 {
        let line = format!("line {i}");
        offsets.push((i, current_offset, current_offset + line.len()));
        current_offset += line.len() + 1; // +1 for newline
        lines.push(line);
    }
    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Get exact offsets for lines 1, 99, 999, and 9999
    let (_, start_1, end_1) = offsets[0];
    let (_, start_99, end_99) = offsets[98];
    let (_, start_999, end_999) = offsets[998];
    let (_, start_9999, end_9999) = offsets[9998];

    // Create labels on lines with different digit counts
    let diag = Diag::error("labels with up to 4-digit line numbers")
        .label(Label::new(make_span(file_id, start_1 as u32, end_1 as u32)).message("on line 1"))
        .label(Label::new(make_span(file_id, start_99 as u32, end_99 as u32)).message("on line 99"))
        .label(
            Label::new(make_span(file_id, start_999 as u32, end_999 as u32)).message("on line 999"),
        )
        .label(
            Label::new(make_span(file_id, start_9999 as u32, end_9999 as u32))
                .message("on line 9999"),
        );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_line_padding_mixed_extreme() {
    // Test with a mix of single digit and 5-digit line numbers to ensure padding works
    let mut lines = Vec::new();
    let mut offsets = Vec::new();
    let mut current_offset = 0;

    for i in 1..=10000 {
        let line = format!("line {i}");
        offsets.push((i, current_offset, current_offset + line.len()));
        current_offset += line.len() + 1; // +1 for newline
        lines.push(line);
    }
    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Get exact offsets for lines 5 and 10000
    let (_, start_5, end_5) = offsets[4];
    let (_, start_10000, end_10000) = offsets[9999];

    // Create labels on lines 5 and 10000 - the most extreme case
    let diag = Diag::error("extreme line number difference")
        .label(Label::new(make_span(file_id, start_5 as u32, end_5 as u32)).message("on line 5"))
        .label(
            Label::new(make_span(file_id, start_10000 as u32, end_10000 as u32))
                .message("on line 10000"),
        );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}
