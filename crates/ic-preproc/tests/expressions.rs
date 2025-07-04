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

// Note: The ic-preproc expression evaluator has limited support.
// These tests document what works and what doesn't.

#[test]
fn simple_arithmetic() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 1 + 1 == 2
                #define SIMPLE_ADD
            #endif
            
            #if 5 - 3 == 2
                #define SIMPLE_SUB
            #endif
            
            #if 2 * 3 == 6
                #define SIMPLE_MUL
            #endif
            
            #if 6 / 2 == 3
                #define SIMPLE_DIV
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("SIMPLE_ADD"));
    assert!(state.is_defined("SIMPLE_SUB"));
    assert!(state.is_defined("SIMPLE_MUL"));
    assert!(state.is_defined("SIMPLE_DIV"));
}

#[test]
fn bitwise_operations() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if (0x0F & 0x06) == 0x06
                #define AND_WORKS
            #endif
            
            #if (0x0F | 0xF0) == 0xFF
                #define OR_WORKS
            #endif
            
            #if (0x0F ^ 0x06) == 0x09
                #define XOR_WORKS
            #endif
            
            #if (~0x00) != 0
                #define NOT_WORKS
            #endif
            
            #if (1 << 4) == 16
                #define SHIFT_LEFT
            #endif
            
            #if (32 >> 2) == 8
                #define SHIFT_RIGHT
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("AND_WORKS"));
    assert!(state.is_defined("OR_WORKS"));
    assert!(state.is_defined("XOR_WORKS"));
    assert!(state.is_defined("NOT_WORKS"));
    assert!(state.is_defined("SHIFT_LEFT"));
    assert!(state.is_defined("SHIFT_RIGHT"));
}

#[test]
fn comparison_operators() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 5 < 10
                #define LESS_THAN
            #endif
            
            #if 10 > 5
                #define GREATER_THAN
            #endif
            
            #if 5 <= 5
                #define LESS_EQUAL
            #endif
            
            #if 10 >= 10
                #define GREATER_EQUAL
            #endif
            
            #if 5 == 5
                #define EQUAL
            #endif
            
            #if 5 != 10
                #define NOT_EQUAL
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("LESS_THAN"));
    assert!(state.is_defined("GREATER_THAN"));
    assert!(state.is_defined("LESS_EQUAL"));
    assert!(state.is_defined("GREATER_EQUAL"));
    assert!(state.is_defined("EQUAL"));
    assert!(state.is_defined("NOT_EQUAL"));
}

#[test]
fn ternary_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if (1 ? 42 : 0) == 42
                #define TERNARY_TRUE
            #endif
            
            #if (0 ? 42 : 99) == 99
                #define TERNARY_FALSE
            #endif
            
            #if (5 > 3 ? 10 : 20) == 10
                #define TERNARY_CONDITION
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("TERNARY_TRUE"));
    assert!(state.is_defined("TERNARY_FALSE"));
    assert!(state.is_defined("TERNARY_CONDITION"));
}

#[test]
fn simple_macro_in_expression() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define VALUE 10
            #define MULTIPLIER 3
            
            #if VALUE * MULTIPLIER == 30
                #define MACRO_MATH
            #endif
            
            #if VALUE == 10
                #define MACRO_COMPARISON
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("MACRO_MATH"));
    assert!(state.is_defined("MACRO_COMPARISON"));
}

#[test]
fn complex_expressions() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define A 5
            #define B 3
            
            #if (A > B) && (A - B == 2) || (B * 2 < A)
                #define COMPLEX_LOGIC
            #endif
            
            #if ((A << 1) & 0xFF) > (B | 0x04)
                #define COMPLEX_BITWISE
            #endif
            
            #if A > 0 ? (B < 10 ? 1 : 0) : 0
                #define NESTED_TERNARY
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("COMPLEX_LOGIC"));
    assert!(state.is_defined("COMPLEX_BITWISE"));
    assert!(state.is_defined("NESTED_TERNARY"));
}

#[test]
fn undefined_in_expression() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if UNDEFINED_MACRO
                #define SHOULD_BE_FALSE
            #endif
            
            #if !UNDEFINED_MACRO
                #define SHOULD_BE_TRUE
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Undefined macros in expressions are treated as 0
    assert!(state.errors().is_empty());
    assert!(!state.is_defined("SHOULD_BE_FALSE"));
    assert!(state.is_defined("SHOULD_BE_TRUE"));
}

#[test]
fn division_by_zero() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 10 / 0
                #define DIV_BY_ZERO
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have an error for division by zero
    assert!(!state.errors().is_empty());
}
