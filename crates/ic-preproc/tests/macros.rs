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
fn simple_object_macro() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define VERSION 100
            #define NAME "test"
            VERSION
            NAME
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    assert!(state.errors().is_empty());
    assert!(state.is_defined("VERSION"));
    assert!(state.is_defined("NAME"));

    // Check that we got some output tokens
    assert!(!_output.is_empty());
}

#[test]
#[ignore] // TODO: Implement function-like macros
fn function_macro() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define MAX(a, b) ((a) > (b) ? (a) : (b))
            #define MIN(a, b) ((a) < (b) ? (a) : (b))
            MAX(10, 20)
            MIN(x, y)
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    assert!(state.errors().is_empty());
    assert!(state.is_defined("MAX"));
    assert!(state.is_defined("MIN"));
}

#[test]
fn macro_redefinition() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define FOO 1
            #define FOO 2
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have a warning about redefinition
    assert!(!state.warnings().is_empty());
}

#[test]
fn undef_macro() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define TEMP 42
            #ifdef TEMP
            #define TEMP_DEFINED
            #endif
            #undef TEMP
            #ifdef TEMP
            #define TEMP_STILL_DEFINED
            #endif
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("TEMP_DEFINED"));
    assert!(!state.is_defined("TEMP_STILL_DEFINED"));
    assert!(!state.is_defined("TEMP"));
}

#[test]
fn nested_macro_expansion() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define INNER 5
            #define OUTER INNER + INNER
            #define RESULT OUTER * 2
            RESULT
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

#[test]
#[ignore] // TODO: Implement function-like macros in conditionals
fn macro_in_conditional() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define DEBUG 1
            #if DEBUG
            #define LOG(msg) print(msg)
            #else
            #define LOG(msg)
            #endif
            LOG("test message")
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("DEBUG"));
    assert!(state.is_defined("LOG"));
}

#[test]
fn empty_macro() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define EMPTY
            #ifdef EMPTY
            #define EMPTY_IS_DEFINED
            #endif
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("EMPTY"));
    assert!(state.is_defined("EMPTY_IS_DEFINED"));
}
