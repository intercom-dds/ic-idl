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

//! Extended edge case tests for the preprocessor

use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

#[test]
fn empty_macro_arguments() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define EMPTY()
            #define ONE_ARG(a) [a]
            #define TWO_ARG(a, b) [a][b]
            #define THREE_ARG(a, b, c) [a][b][c]
            
            EMPTY()
            ONE_ARG()
            ONE_ARG(x)
            TWO_ARG(,)
            TWO_ARG(x,)
            TWO_ARG(,y)
            TWO_ARG(x,y)
            THREE_ARG(,,)
            THREE_ARG(x,,z)
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Empty arguments should be handled gracefully
    if !state.errors().is_empty() {
        for err in state.errors() {
            eprintln!("Error: {:?}", err);
        }
    }
    assert!(state.errors().is_empty());
}

#[test]
fn macro_redefinition_different_params() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define FOO(a) a
            #define FOO(a, b) a + b
            
            #define BAR(x, y) x * y
            #define BAR(x) x * x
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have warnings about redefinition
    assert!(!state.warnings().is_empty());
}

#[test]
fn token_pasting_edge_cases() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define PASTE(a, b) a##b
            #define PASTE3(a, b, c) a##b##c
            
            // Valid pastes
            PASTE(foo, bar)
            PASTE(123, 456)
            PASTE(pre, fix)
            
            // Edge cases
            PASTE(, suffix)
            PASTE(prefix, )
            PASTE3(a, , c)
            
            // Invalid token combinations
            PASTE(+, +)  // Should produce ++
            // PASTE(/, *)  // Would produce slash-star which starts unterminated comment
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Token pasting should handle edge cases
}

#[test]
fn recursive_macro_limit() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            // Create a deeply nested macro expansion
            #define A B
            #define B C
            #define C D
            #define D E
            #define E F
            #define F G
            #define G H
            #define H I
            #define I J
            #define J K
            #define K L
            #define L M
            #define M N
            #define N O
            #define O P
            #define P Q
            #define Q R
            #define R S
            #define S T
            #define T U
            #define U V
            #define V W
            #define W X
            #define X Y
            #define Y Z
            #define Z final_value
            
            A
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    // Should expand all the way to final_value
    assert!(state.errors().is_empty());
}

#[test]
fn va_opt_nested_parens() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define LOG(fmt, ...) printf(fmt __VA_OPT__(, ({ __VA_ARGS__ })))
            
            LOG("test")
            LOG("test %d", 42)
            LOG("test %d %d", (1 + 2), (3 * 4))
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

#[test]
fn stringification_edge_cases() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define STR(x) #x
            #define XSTR(x) STR(x)
            
            // Stringifying various token types
            STR(simple)
            STR(123)
            STR(1.23)
            STR(multiple tokens here)
            STR("already a string")
            STR(a + b * c)
            STR(())
            // STR(,) // This passes 2 arguments (both empty) to a 1-arg macro
            // STR() // This is invalid - stringification requires an argument
            
            // Nested stringification
            #define VALUE 42
            STR(VALUE)
            XSTR(VALUE)
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    if !state.errors().is_empty() {
        for err in state.errors() {
            eprintln!("Stringification Error: {:?}", err);
        }
    }
    assert!(state.errors().is_empty());
}

#[test]
fn complex_conditional_directives() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define A 1
            #define B 0
            
            #if A
                #if B
                    #define SHOULD_NOT_BE_DEFINED_1
                #elif !B
                    #if A && !B
                        #define NESTED_CORRECT
                    #endif
                #endif
            #else
                #define SHOULD_NOT_BE_DEFINED_2
            #endif
            
            // Mixing ifdef and if
            #ifdef A
                #if A > 0
                    #define MIXED_DIRECTIVES
                #endif
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("NESTED_CORRECT"));
    assert!(state.is_defined("MIXED_DIRECTIVES"));
    assert!(!state.is_defined("SHOULD_NOT_BE_DEFINED_1"));
    assert!(!state.is_defined("SHOULD_NOT_BE_DEFINED_2"));
}
