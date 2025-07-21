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

use ic_preproc::{Error, ProcArgs, State, with_state};
use ic_vfs::SourceMap;

fn preprocess(input: &str) -> (Vec<String>, Vec<String>) {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(input);

    let args = ProcArgs::default();
    let mut state = State::new();
    let tokens: Vec<_> = with_state(file_id, args, &mut state, &mut vfs).collect();

    let token_strings: Vec<String> = tokens
        .iter()
        .map(|tok| vfs.source_str(tok.span.start.file_id)[tok.span.range()].to_string())
        .collect();

    let warnings: Vec<String> = state
        .warnings()
        .iter()
        .map(|err| match err {
            Error::Syntax { message, span } => {
                format!(
                    "{}: {}",
                    &vfs.source_str(span.start.file_id)[span.range()],
                    message
                )
            }
            _ => format!("{err:?}"),
        })
        .collect();

    (token_strings, warnings)
}

#[test]
fn test_unknown_pragma_warning() {
    let input = r"
#pragma unknown_directive
int x = 5;
";

    let (_, warnings) = preprocess(input);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("unknown pragma directive"));
    assert!(warnings[0].contains("unknown_directive"));
}

#[test]
fn test_known_pragma_no_warning() {
    let input = r"
#pragma once
#pragma warning(push)
#pragma region MyRegion
#pragma endregion
int x = 5;
";

    let (_, warnings) = preprocess(input);
    assert_eq!(warnings.len(), 0);
}

#[test]
fn test_multiple_unknown_pragmas() {
    let input = r"
#pragma foo
#pragma bar
#pragma baz
int x = 5;
";

    let (_, warnings) = preprocess(input);
    assert_eq!(warnings.len(), 3);
    assert!(warnings[0].contains("foo"));
    assert!(warnings[1].contains("bar"));
    assert!(warnings[2].contains("baz"));
}

#[test]
fn test_unknown_pragma_in_inactive_code() {
    let input = r"
#if 0
#pragma unknown_directive
#endif
int x = 5;
";

    let (_, warnings) = preprocess(input);
    // Should not warn about pragmas in inactive code
    assert_eq!(warnings.len(), 0);
}

#[test]
fn test_empty_pragma() {
    let input = r"
#pragma
int x = 5;
";

    let (_, warnings) = preprocess(input);
    // Empty pragmas are allowed and shouldn't generate warnings
    assert_eq!(warnings.len(), 0);
}

#[test]
fn test_pragma_with_complex_tokens() {
    let input = r"
#pragma unknown_with_args(1, 2, 3)
int x = 5;
";

    let (_, warnings) = preprocess(input);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("unknown_with_args"));
}

#[test]
fn test_pragma_line_continuation() {
    // Test that line continuation works correctly with pragmas.
    // When we have "#pragma \" followed by content on the next line,
    // it should be treated as a single pragma directive.

    // Test 1: Known pragma with line continuation (no warning)
    let input1 = r"#pragma \
once
int x = 5;
";
    let (_, warnings1) = preprocess(input1);
    assert_eq!(warnings1.len(), 0);

    // Test 2: Unknown pragma with line continuation (should generate warning)
    let input2 = r"#pragma \
unknown_continued
int x = 5;
";
    let (_, warnings2) = preprocess(input2);
    assert_eq!(warnings2.len(), 1);
    assert!(warnings2[0].contains("unknown_continued"));

    // Test 3: Mixed known and unknown pragmas
    let input3 = r"#pragma \
once
#pragma unknown_test
#pragma \
unknown_continued
int x = 5;
";
    let (_, warnings3) = preprocess(input3);
    assert_eq!(warnings3.len(), 2);
    assert!(warnings3.iter().any(|w| w.contains("unknown_test")));
    assert!(warnings3.iter().any(|w| w.contains("unknown_continued")));
}
