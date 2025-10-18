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

use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

#[test]
fn test_include_error_span_highlights_path() {
    let mut vfs = SourceMap::default();

    // Test case 1: Simple missing file with quotes
    let source = r#"
#include "missing_file.idl"
int main();
"#;

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(
        !errors.is_empty(),
        "Expected error for missing include file"
    );

    // Find the error for the missing file
    let include_error = errors
        .iter()
        .find(
            |e| matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("file")),
        )
        .expect("Expected to find file-related error");

    // Check that the error span highlights the string literal, not the #include keyword
    if let ic_preproc::Error::Syntax { span, .. } = include_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(
            error_text.contains("missing_file.idl"),
            "Error span should highlight the file path '{error_text}', not the directive"
        );
        assert!(
            !error_text.contains("#include"),
            "Error span should not include the #include directive"
        );
    }
}

#[test]
fn test_include_error_span_system_includes() {
    let mut vfs = SourceMap::default();

    // Test case 2: System include with angle brackets
    let source = r"
#include <system/missing.h>
int main();
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(
        !errors.is_empty(),
        "Expected error for missing system include"
    );

    // Find the error for the missing file
    let include_error = errors
        .iter()
        .find(
            |e| matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("file")),
        )
        .expect("Expected to find file-related error");

    // Check that the error span highlights the path inside angle brackets
    if let ic_preproc::Error::Syntax { span, .. } = include_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(
            error_text.contains("system/missing.h"),
            "Error span should highlight the file path '{error_text}', not the directive"
        );
        assert!(
            !error_text.contains("#include"),
            "Error span should not include the #include directive"
        );
    }
}

#[test]
fn test_nested_include_error_span() {
    let mut vfs = SourceMap::default();

    // Test case 3: Deeply nested includes error
    let source = r#"
#include "deeply/nested/file.idl"
"#;

    let file_id = vfs.embed(source);
    let args = ProcArgs::default().recursion_depth(0); // Set depth to 0 to trigger the error immediately
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    let nested_error = errors.iter().find(
        |e| matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("nested")),
    );

    if let Some(ic_preproc::Error::Syntax { span, .. }) = nested_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(
            error_text.contains("deeply/nested/file.idl"),
            "Error span should highlight the file path '{error_text}', not the directive"
        );
    }
}
