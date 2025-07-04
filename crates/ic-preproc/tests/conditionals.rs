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
fn if_expression_evaluation() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 1 + 1 == 2
            #define MATH_WORKS
            #endif
            
            #if 10 > 5
            #define COMPARISON_WORKS
            #endif
            
            #if 3 * 4 / 2 == 6
            #define COMPLEX_MATH
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("MATH_WORKS"));
    assert!(state.is_defined("COMPARISON_WORKS"));
    assert!(state.is_defined("COMPLEX_MATH"));
}

#[test]
fn nested_conditionals() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define OUTER 1
            #define INNER 1
            
            #if OUTER
                #define OUTER_TRUE
                #if INNER
                    #define BOTH_TRUE
                #else
                    #define INNER_FALSE
                #endif
            #else
                #define OUTER_FALSE
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("OUTER_TRUE"));
    assert!(state.is_defined("BOTH_TRUE"));
    assert!(!state.is_defined("INNER_FALSE"));
    assert!(!state.is_defined("OUTER_FALSE"));
}

#[test]
fn elif_chain() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define VERSION 3
            
            #if VERSION == 1
                #define V1
            #elif VERSION == 2
                #define V2
            #elif VERSION == 3
                #define V3
            #else
                #define VUNKNOWN
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(!state.is_defined("V1"));
    assert!(!state.is_defined("V2"));
    assert!(state.is_defined("V3"));
    assert!(!state.is_defined("VUNKNOWN"));
}

#[test]
fn ifdef_ifndef() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define DEFINED_MACRO
            
            #ifdef DEFINED_MACRO
                #define IFDEF_WORKS
            #endif
            
            #ifndef UNDEFINED_MACRO
                #define IFNDEF_WORKS
            #endif
            
            #ifdef UNDEFINED_MACRO
                #define SHOULD_NOT_BE_DEFINED
            #endif
            
            #ifndef DEFINED_MACRO
                #define ALSO_SHOULD_NOT_BE_DEFINED
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("IFDEF_WORKS"));
    assert!(state.is_defined("IFNDEF_WORKS"));
    assert!(!state.is_defined("SHOULD_NOT_BE_DEFINED"));
    assert!(!state.is_defined("ALSO_SHOULD_NOT_BE_DEFINED"));
}

#[test]
fn logical_operators() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define A 1
            #define B 0
            
            #if A && B
                #define AND_FALSE
            #endif
            
            #if A || B
                #define OR_TRUE
            #endif
            
            #if !B
                #define NOT_TRUE
            #endif
            
            #if A && !B
                #define COMPLEX_TRUE
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(!state.is_defined("AND_FALSE"));
    assert!(state.is_defined("OR_TRUE"));
    assert!(state.is_defined("NOT_TRUE"));
    assert!(state.is_defined("COMPLEX_TRUE"));
}

#[test]
fn defined_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define EXISTING
            
            #if defined(EXISTING)
                #define DEFINED_CHECK_WORKS
            #endif
            
            #if !defined(NONEXISTING)
                #define NOT_DEFINED_WORKS
            #endif
            
            #if defined(EXISTING) && !defined(NONEXISTING)
                #define COMPLEX_DEFINED
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("DEFINED_CHECK_WORKS"));
    assert!(state.is_defined("NOT_DEFINED_WORKS"));
    assert!(state.is_defined("COMPLEX_DEFINED"));
}

#[test]
fn missing_endif() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 1
            #define SOMETHING
            // Missing #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have an error about unterminated #if
    assert!(!state.errors().is_empty());
}

#[test]
fn extra_endif() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 1
            #define SOMETHING
            #endif
            #endif  // Extra endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have an error about unexpected #endif
    assert!(!state.errors().is_empty());
}

#[test]
fn elif_without_if() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #elif 1
            #define SOMETHING
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have an error about #elif without #if
    assert!(!state.errors().is_empty());
}
