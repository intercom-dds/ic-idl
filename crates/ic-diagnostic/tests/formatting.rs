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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_diagnostic::{Color, Diag, Label};

#[test]
fn diag() {
    let diag = Diag::error("unexpected identifier `def`")
        .label(Label::new(4..7).message("222").color(Color::Red))
        .label(Label::new(0..3).message("111").color(Color::Green))
        .label(Label::new(8..11).message("333").color(Color::Cyan))
        .label(Label::new(12..15).message("444").color(Color::Yellow))
        .warn("def is deprecated")
        .help("use `bar` instead")
        .note("foobar")
        .description(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
             \nincididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis \
             \nnostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. \
             \nDuis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu \
             \nfugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt \
             \ninculpa qui officia deserunt mollit anim id est laborum.",
        );

    let mut buf = String::new();
    ic_diagnostic::emit_diagnostic(&mut buf, "unknown", "abc def ghi jkl", &diag).unwrap();
    println!("{buf}");
}

#[test]
fn single() {
    let diag = Diag::error("unexpected identifier `def`")
        .label(
            Label::new(4..7)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .warn("def is deprecated")
        .help("use `bar` instead")
        .note("foobar");

    let mut buf = String::new();
    ic_diagnostic::emit_diagnostic(&mut buf, "unknown", "abc def ghi", &diag).unwrap();
    println!("{buf}");
}

#[test]
fn plenty() {
    let diag = Diag::error("unexpected identifier `def`")
        .label(Label::new(0..1).message("first").color(Color::Red))
        .label(
            Label::new(1..2)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(2..3)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(3..4)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(4..5)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(5..6)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(6..7)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(7..8)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .label(
            Label::new(8..9)
                .message("unexpected identifier")
                .color(Color::Red),
        )
        .warn("def is deprecated")
        .help("use `bar` instead");

    let mut buf = String::new();
    ic_diagnostic::emit_diagnostic(&mut buf, "unknown", "abcdefghi", &diag).unwrap();
    println!("{buf}");
}

#[test]
fn compact() {
    let diag = Diag::error("unexpected identifier `def`")
        .label(Label::new(4..7).message("222").color(Color::Red))
        .label(Label::new(0..3).message("111").color(Color::Green))
        .label(Label::new(8..11).message("333").color(Color::Cyan))
        .label(Label::new(12..15).message("444").color(Color::Yellow))
        .warn("def is deprecated")
        .help("use `bar` instead")
        .note("foobar");

    let mut buf = String::new();
    ic_diagnostic::emit_compact(&mut buf, "test.idl", &diag).unwrap();
    println!("{buf}");
}
