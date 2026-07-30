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
use ic_diagnostic::{Color, Diag, Label};
use ic_vfs::{FileId, Location, SourceMap, Span};

fn make_span(file_id: FileId, start: u32, end: u32) -> Span {
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn test_multiline_span() {
    ic_cli::color::set_color_override(ColorMode::Never);
    // Test with a multi-line span
    let source = r"
interface MyInterface {
    void myMethod(
        int param1,
        string param2
    );
}";
    let mut map = SourceMap::default();
    let file_id = map.embed(source);

    // Create a diagnostic that spans multiple lines (from "myMethod" to the closing paren)
    let diag = Diag::error("method spans multiple lines").label(
        Label::new(make_span(file_id, 35, 87)) // This should span from "myMethod" to ")"
            .message("this method definition spans multiple lines")
            .color(Color::Red),
    );

    let mut buf = String::new();
    ic_diagnostic::DiagnosticEmitter::new()
        .emit_with_source(&mut buf, "test.idl", source, &diag)
        .unwrap();
    insta::assert_snapshot!(buf);

    // Test with multiple labels on different lines
    let diag2 = Diag::error("multiple parameters on different lines")
        .label(
            Label::new(make_span(file_id, 53, 63)) // "int param1"
                .message("first parameter")
                .color(Color::Yellow),
        )
        .label(
            Label::new(make_span(file_id, 73, 86)) // "string param2"
                .message("second parameter")
                .color(Color::Blue),
        );

    let mut buf2 = String::new();
    ic_diagnostic::DiagnosticEmitter::new()
        .emit_with_source(&mut buf2, "test.idl", source, &diag2)
        .unwrap();
    insta::assert_snapshot!("multiple_labels_different_lines", buf2);

    // Test showing current behavior - only first line is highlighted
    let diag3 = Diag::error("large multi-line block").label(
        Label::new(make_span(file_id, 23, 92)) // From opening brace to closing brace
            .message("entire method block")
            .color(Color::Green),
    );

    let mut buf3 = String::new();
    ic_diagnostic::DiagnosticEmitter::new()
        .emit_with_source(&mut buf3, "test.idl", source, &diag3)
        .unwrap();
    insta::assert_snapshot!("large_multiline_block", buf3);
}
