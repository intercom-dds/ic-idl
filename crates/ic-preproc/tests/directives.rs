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

use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

#[test]
fn error_directive() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #error "This is an error message"
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have an error from #error directive
    assert!(!state.errors().is_empty());
}

#[test]
fn warning_directive() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #warning "This is a warning message"
            #define AFTER_WARNING
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have a warning but no errors
    assert!(!state.warnings().is_empty());
    assert!(state.errors().is_empty());
    assert!(state.is_defined("AFTER_WARNING"));
}

#[test]
fn conditional_error() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define DEBUG_MODE
            
            #ifdef DEBUG_MODE
                #warning "Debug mode is enabled"
            #else
                #error "Debug mode must be enabled"
            #endif
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should only have a warning, not an error
    assert!(!state.warnings().is_empty());
    assert!(state.errors().is_empty());
}

#[test]
fn line_directive() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #line 100
            #define AT_LINE_100
            #line 200 "custom_file.idl"
            #define AT_LINE_200
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("AT_LINE_100"));
    assert!(state.is_defined("AT_LINE_200"));
}

#[test]
fn include_angle_brackets() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #include <tests/pragma_once.idl>
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // The exact behavior depends on include paths, but should not crash
    // File might not be found, which is okay for this test
}

#[test]
fn include_quotes() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #include "tests/pragma_once.idl"
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should succeed if file exists
    assert!(state.errors().is_empty());
}

#[test]
fn malformed_include() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #include
            #include <>
            #include ""
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have errors for malformed includes
    assert!(!state.errors().is_empty());
}

#[test]
fn whitespace_handling() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #  define   SPACES   123
            #	ifdef	SPACES
            #		define	TABS_WORK
            #	endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("SPACES"));
    assert!(state.is_defined("TABS_WORK"));
}

#[test]
fn comment_in_directive() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define FOO /* comment */ 42
            #ifdef FOO // line comment
                #define BAR
            #endif /* block comment */
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("FOO"));
    assert!(state.is_defined("BAR"));
}

#[test]
fn multiline_macro() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define LONG_MACRO \
                do { \
                    something(); \
                    something_else(); \
                } while(0)
            LONG_MACRO
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("LONG_MACRO"));
}
