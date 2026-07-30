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

use ic_cli::color::ColorMode;
use ic_diagnostic::{Color, Diag, DiagnosticEmitter, Label};
use ic_vfs::{FileId, Location, SourceMap, Span};

fn make_span(file_id: FileId, start: u32, end: u32) -> Span {
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn single_tab() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = "\tint x = 42;";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("test error").label(
        Label::new(make_span(file_id, 5, 6))
            .message("here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}

#[test]
fn multiple_tabs() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = "\t\tint x = 42;";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("test error").label(
        Label::new(make_span(file_id, 6, 7))
            .message("here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}

#[test]
fn mixed_spaces_and_tabs() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = " \tint x = 42;";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("test error").label(
        Label::new(make_span(file_id, 6, 7))
            .message("here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}

#[test]
fn tabs_between_tokens() {
    ic_cli::color::set_color_override(ColorMode::Never);
    // Test that column numbers are calculated correctly with tabs
    let source = "\tint\tx = 42;";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("test error").label(
        Label::new(make_span(file_id, 5, 6))
            .message("variable")
            .color(Color::Red),
    );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}

#[test]
fn tab_at_end_of_line() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = "int x = 42;\t";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("trailing tab").label(
        Label::new(make_span(file_id, 11, 12))
            .message("tab here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}

#[test]
fn multiple_errors_with_tabs() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = "\tint\tx\t=\t42;";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("multiple tabs")
        .label(
            Label::new(make_span(file_id, 1, 4))
                .message("type")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, 5, 6))
                .message("variable")
                .color(Color::Yellow),
        )
        .label(
            Label::new(make_span(file_id, 9, 11))
                .message("value")
                .color(Color::Blue),
        );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}

#[test]
fn tab_in_multiline_error() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = "struct Foo {\n\tint x;\n\tfloat y;\n}";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);
    let diag = Diag::error("struct fields")
        .label(
            Label::new(make_span(file_id, 13, 19))
                .message("first field")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(file_id, 21, 29))
                .message("second field")
                .color(Color::Yellow),
        );

    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.rs", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);
}
