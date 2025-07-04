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
use ic_vfs::{Include, SourceMap};

#[test]
fn include_guard_pattern() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #include "tests/include_guard.idl"
            #include "tests/include_guard.idl"
            #include "tests/include_guard.idl"
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("INCLUDE_GUARD_IDL"));
    assert!(state.is_defined("GUARD_CONTENT"));
}

#[test]
fn circular_inclusion() {
    let mut vfs = SourceMap::default();
    let (id, _) = vfs.open("tests/circular_a.idl", Include::Local).unwrap();

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should handle circular includes gracefully with include guards
    assert!(state.errors().is_empty());
    assert!(state.is_defined("CIRCULAR_A_IDL"));
    assert!(state.is_defined("CIRCULAR_B_IDL"));
    assert!(state.is_defined("FROM_A"));
    assert!(state.is_defined("FROM_B"));
}

#[test]
fn empty_file() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed("");

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

#[test]
fn only_whitespace() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed("   \n\t\n   \r\n   ");

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

#[test]
fn hash_without_directive() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #
            # 
            #123
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Behavior depends on implementation, but should not crash
}

#[test]
#[ignore] // TODO: Implement stringification operator (#)
fn stringification() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define STRINGIFY(x) #x
            #define XSTRINGIFY(x) STRINGIFY(x)
            #define VERSION 123
            
            STRINGIFY(hello world)
            XSTRINGIFY(VERSION)
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    assert!(state.errors().is_empty());
    // Just verify we processed without errors
    assert!(state.is_defined("STRINGIFY"));
    assert!(state.is_defined("XSTRINGIFY"));
}

#[test]
#[ignore] // TODO: Implement token pasting operator (##)
fn token_pasting() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define PASTE(a, b) a##b
            #define MAKE_VAR(name) int var_##name
            
            PASTE(foo, bar)
            MAKE_VAR(test)
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

#[test]
fn predefined_macros() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #ifdef __FILE__
                #define HAS_FILE
            #endif
            
            #ifdef __LINE__
                #define HAS_LINE
            #endif
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Implementation might or might not define these
    assert!(state.errors().is_empty());
}

#[test]
fn macro_recursion() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define RECURSIVE RECURSIVE
            RECURSIVE
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should handle recursive macros without infinite loop
    assert!(state.errors().is_empty());
}

#[test]
fn command_line_defines() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #ifdef CMD_DEFINE
                #define FOUND_CMD_DEFINE
            #endif
        "#,
    );

    let args = ProcArgs::default()
        .define("CMD_DEFINE", None);
    
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("FOUND_CMD_DEFINE"));
}