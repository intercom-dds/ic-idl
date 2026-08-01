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

use ic_cli::color::ColorMode;
use ic_diagnostic::{Color, Diag, DiagnosticEmitter, Label, Level, json_message};
use ic_vfs::{FileId, Location, SourceMap, Span};

fn make_span(file_id: FileId, start: u32, end: u32) -> Span {
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

fn emit(map: &SourceMap, diag: &Diag) -> String {
    let mut buf = String::new();
    DiagnosticEmitter::new()
        .emit_json(&mut buf, map, diag)
        .unwrap();
    buf
}

#[test]
fn json_single_label() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut map = SourceMap::default();
    let file_id = map.embed_with_name("a.idl", "module foo {\n  struct Bar }\n");

    let diag = Diag::error("unexpected `}`, expected `{`").label(
        Label::new(make_span(file_id, 26, 27))
            .message("unexpected `}`")
            .color(Color::Red),
    );

    insta::assert_snapshot!(emit(&map, &diag));
}

#[test]
fn json_no_labels() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let map = SourceMap::default();
    let diag = Diag::warning("no input files");

    insta::assert_snapshot!(emit(&map, &diag));
}

#[test]
fn json_all_optional_fields() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut map = SourceMap::default();
    let file_id = map.embed_with_name("a.idl", "struct Bar {\n  int32 x;\n};\n");

    let diag = Diag::error("duplicate member `x`")
        .code("duplicate-name")
        .help("rename one of the members")
        .note("names must be unique within a struct")
        .description("IDL requires member names to be unique in their scope")
        .label(Label::new(make_span(file_id, 15, 23)).message("first defined here"));

    insta::assert_snapshot!(emit(&map, &diag));
}

#[test]
fn json_labels_in_two_files() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut map = SourceMap::default();
    let first = map.embed_with_name("a.idl", "struct Bar {\n};\n");
    let second = map.embed_with_name("b.idl", "struct Bar {\n};\n");

    let diag = Diag::error("duplicate definition of `Bar`")
        .label(Label::new(make_span(first, 7, 10)).message("first definition"))
        .label(Label::new(make_span(second, 7, 10)).message("redefined here"));

    insta::assert_snapshot!(emit(&map, &diag));
}

#[test]
fn json_label_without_message() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut map = SourceMap::default();
    let file_id = map.embed_with_name("a.idl", "struct Bar {\n};\n");

    let diag = Diag::error("bad struct").label(Label::new(make_span(file_id, 0, 6)));

    insta::assert_snapshot!(emit(&map, &diag));
}

#[test]
fn json_escapes_special_characters() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let map = SourceMap::default();
    let diag = Diag::error("quote \" backslash \\ tab \t newline \n control \u{1} unicode \u{e5}");

    insta::assert_snapshot!(emit(&map, &diag));
}

#[test]
fn json_message_without_source() {
    ic_cli::color::set_color_override(ColorMode::Never);
    insta::assert_snapshot!(json_message(Level::Error, "no such file: missing.idl"));
}

#[test]
fn json_message_warning_level() {
    ic_cli::color::set_color_override(ColorMode::Never);
    insta::assert_snapshot!(json_message(Level::Warning, "unknown warning 'bogus'"));
}

#[test]
fn json_message_disabled_level_is_empty() {
    assert_eq!(json_message(Level::Disabled, "ignored"), "");
}
