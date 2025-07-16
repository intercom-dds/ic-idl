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

use ic_diagnostic::{Color, Diag, Label, emit_with_source};
use ic_vfs::{FileId, Location, Span};

fn make_span(start: u32, end: u32) -> Span {
    let file_id = FileId::_do_not_use();
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn test_tab_expansion_single_tab() {
    let source = "\tint x = 42;";
    let diag = Diag::error("test error").label(
        Label::new(make_span(5, 6))
            .message("here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.rs", source, &diag).unwrap();

    // Tab should be expanded to 4 spaces, so the highlight should align with 'x'
    assert!(buf.contains("    int x = 42;"));
    assert!(buf.contains("        ^"));
}

#[test]
fn test_tab_expansion_multiple_tabs() {
    let source = "\t\tint x = 42;";
    let diag = Diag::error("test error").label(
        Label::new(make_span(6, 7))
            .message("here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.rs", source, &diag).unwrap();

    // Two tabs should be expanded to 8 spaces
    assert!(buf.contains("        int x = 42;"));
    assert!(buf.contains("            ^"));
}

#[test]
fn test_tab_expansion_mixed_spaces_and_tabs() {
    let source = " \tint x = 42;";
    let diag = Diag::error("test error").label(
        Label::new(make_span(6, 7))
            .message("here")
            .color(Color::Red),
    );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.rs", source, &diag).unwrap();

    // Space + tab should expand to align at column 4
    assert!(buf.contains("    int x = 42;"));
    assert!(buf.contains("        ^"));
}

#[test]
fn test_tab_column_calculation() {
    // Test that column numbers are calculated correctly with tabs
    let source = "\tint\tx = 42;";
    let diag = Diag::error("test error").label(
        Label::new(make_span(5, 6))
            .message("variable")
            .color(Color::Red),
    );

    let mut buf = String::new();
    emit_with_source(&mut buf, "test.rs", source, &diag).unwrap();

    // The location should show the correct column accounting for tabs
    assert!(buf.contains("test.rs:1:9")); // Column 9 (1 tab = 4 columns, 'int' = 3, 1 tab = 4 more)
}
