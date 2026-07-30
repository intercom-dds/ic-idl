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
fn test_horizontal_clamping() {
    ic_cli::color::set_color_override(ColorMode::Never);

    let source = "x".repeat(200);
    let mut map = SourceMap::default();
    let file_id = map.embed(&source);
    let span = make_span(file_id, 150, 160);

    let diag =
        Diag::error("value too long").label(Label::new(span).message("here").color(Color::Red));

    let mut buf = String::new();
    let mut emitter = DiagnosticEmitter::new();
    emitter.set_max_width(Some(80));
    emitter
        .emit_with_source(&mut buf, "test.idl", &source, &diag)
        .unwrap();

    assert!(
        buf.contains("..."),
        "Expected truncation markers in output:\n{buf}"
    );
}
