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
use ic_vfs::{FileId, Location, Span};

fn make_span(start: u32, end: u32) -> Span {
    let file_id = FileId::_do_not_use();
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn test_overlap_colors() {
    ic_cli::color::set_color_override(ColorMode::Never);
    let source = "void process(struct Data { int x; int y; } data);";

    // Create spans with specific colors
    let diag = Diag::error("nested structures")
        .label(
            Label::new(make_span(13, 43))
                .message("struct (should be blue)")
                .color(Color::Blue),
        )
        .label(
            Label::new(make_span(27, 33))
                .message("int x (should be yellow)")
                .color(Color::Yellow),
        )
        .label(
            Label::new(make_span(34, 40))
                .message("int y (should be green)")
                .color(Color::Green),
        );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    insta::assert_snapshot!(buf);
}
