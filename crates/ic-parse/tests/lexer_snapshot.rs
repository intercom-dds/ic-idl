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

// use ic_parse::lexer;

macro_rules! assert_snapshot {
    ($input:tt) => {{
        // TODO(idarcar);
        // let input = format!("tests/lexer/{}", $input);
        // let mut settings = insta::Settings::clone_current();
        // settings.set_prepend_module_to_snapshot(false);
        // settings.set_snapshot_path("lexer");
        // settings.set_input_file(&input);
        // settings.bind(|| {
        //     let input = std::fs::read_to_string(input).unwrap();
        //     let tokens = lexer::scan(&input);
        //     insta::assert_debug_snapshot!(tokens);
        // });
    }};
}

#[test]
fn chars() {
    assert_snapshot!("chars.idl");
}

#[test]
fn comments() {
    assert_snapshot!("comments.idl");
}

#[test]
fn control() {
    assert_snapshot!("control.idl");
}

#[test]
fn identifiers() {
    assert_snapshot!("identifiers.idl");
}

#[test]
fn string_lit() {
    assert_snapshot!("string_lit.idl");
}

#[test]
fn numbers() {
    assert_snapshot!("numbers.idl");
}

#[test]
fn keywords() {
    assert_snapshot!("keywords.idl");
}
