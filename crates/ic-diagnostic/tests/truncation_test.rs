// Copyright 2025 KONGSBERG
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

use ic_cli::color::ColorMode;
use ic_diagnostic::{Color, Diag, Label, emit_with_source};
use ic_vfs::{FileId, Location, Span};

fn make_span(file_id: FileId, start: u32, end: u32) -> Span {
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn test_large_span_truncation() {
    ic_cli::color::set_color_override(ColorMode::Never);
    // Create a source with many empty lines between two labels
    let mut lines = vec!["struct Foo {".to_string()];
    lines.push("    @optional".to_string());
    lines.push("    @key".to_string());

    // Add 30 empty lines
    for _ in 0..30 {
        lines.push(String::new());
    }

    lines.push("    string value;".to_string());
    lines.push("};".to_string());

    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Calculate positions for the annotations
    let optional_start = source.find("@optional").unwrap() as u32;
    let optional_end = optional_start + "@optional".len() as u32;

    let key_start = source.find("@key").unwrap() as u32;
    let key_end = key_start + "@key".len() as u32;

    let value_start = source.find("string value").unwrap() as u32;
    let value_pos = value_start + "string ".len() as u32;
    let value_end = value_pos + "value".len() as u32;

    // Create the diagnostic
    let diag = Diag::error("struct member `value` cannot be both @optional and @key")
        .label(
            Label::new(make_span(file_id, optional_start, optional_end))
                .message("@optional annotation here")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, key_start, key_end))
                .message("@key annotation here")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, value_pos, value_end))
                .message("conflicting annotations on struct member")
                .color(Color::Red),
        )
        .help("remove either @optional or @key");

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_multiple_truncated_spans() {
    ic_cli::color::set_color_override(ColorMode::Never);
    // Test with multiple spans that each need truncation
    let mut lines = vec!["interface Test {".to_string()];

    // First error location
    lines.push("    void method1(".to_string());
    lines.push("        in long a,".to_string());

    // Add 20 empty lines
    for _ in 0..20 {
        lines.push(String::new());
    }

    lines.push("        in long a".to_string());
    lines.push("    );".to_string());

    // Add more empty lines
    for _ in 0..15 {
        lines.push(String::new());
    }

    // Second error location
    lines.push("    void method2(".to_string());
    lines.push("        in string name,".to_string());

    // Add 25 empty lines
    for _ in 0..25 {
        lines.push(String::new());
    }

    lines.push("        in string name".to_string());
    lines.push("    );".to_string());
    lines.push("};".to_string());

    let source = lines.join("\n");
    let file_id = FileId::_do_not_use();

    // Find positions for first duplicate parameter
    let first_a = source.find("in long a,").unwrap() as u32;
    let first_a_start = first_a + "in long ".len() as u32;
    let first_a_end = first_a_start + 1;

    let second_a_line = "        in long a";
    let second_a_pos = source.find(second_a_line).unwrap() as u32;
    let second_a_start = second_a_pos + "        in long ".len() as u32;
    let second_a_end = second_a_start + 1;

    // Create diagnostic for duplicate parameters
    let diag = Diag::error("duplicate parameter names")
        .label(
            Label::new(make_span(file_id, first_a_start, first_a_end))
                .message("first defined here")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, second_a_start, second_a_end))
                .message("duplicate parameter")
                .color(Color::Red),
        )
        .note("parameter names must be unique within a method");

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", &source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}

#[test]
fn test_no_truncation_needed() {
    ic_cli::color::set_color_override(ColorMode::Never);
    // Test case where spans are close enough that no truncation is needed
    let source = r"struct Point {
    @optional
    @key
    long x;
    
    @optional
    @key
    long y;
};";

    let file_id = FileId::_do_not_use();

    // Find positions for the first field annotations
    let first_optional = source.find("@optional").unwrap() as u32;
    let first_key = source.find("@key").unwrap() as u32;
    let x_pos = source.find("long x").unwrap() as u32 + "long ".len() as u32;

    // Create diagnostic
    let diag = Diag::error("struct member `x` cannot be both @optional and @key")
        .label(
            Label::new(make_span(file_id, first_optional, first_optional + 9))
                .message("@optional annotation here")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, first_key, first_key + 4))
                .message("@key annotation here")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, x_pos, x_pos + 1))
                .message("conflicting annotations on struct member")
                .color(Color::Red),
        );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();

    insta::assert_snapshot!(buf);
}
