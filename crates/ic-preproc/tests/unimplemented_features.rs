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

//! Tests documenting features that need to be implemented in ic-preproc.
//! All tests in this file should be marked with #[ignore] until implemented.

use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

// ==================== Expression Evaluation ====================

#[test]
fn parentheses_in_expressions() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if (2 + 3) * 4 == 20
                #define PARENTHESES_WORK
            #endif
            
            #if ((1 + 2) * 3) / 2 == 4
                #define NESTED_PARENS
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("PARENTHESES_WORK"));
    assert!(state.is_defined("NESTED_PARENS"));
}

#[test]
fn modulo_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 10 % 3 == 1
                #define MODULO_WORKS
            #endif
            
            #if 20 % 4 == 0
                #define MODULO_ZERO
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("MODULO_WORKS"));
    assert!(state.is_defined("MODULO_ZERO"));
}

#[test]
fn unary_minus_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if -5 + 10 == 5
                #define UNARY_MINUS_WORKS
            #endif
            
            #if -(-10) == 10
                #define DOUBLE_NEGATIVE
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("UNARY_MINUS_WORKS"));
    assert!(state.is_defined("DOUBLE_NEGATIVE"));
}

#[test]
fn bitwise_not_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if ~0 == -1
                #define BITWISE_NOT_WORKS
            #endif
            
            #if (~0xFF & 0xFFFF) == 0xFF00
                #define BITWISE_NOT_MASK
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("BITWISE_NOT_WORKS"));
    assert!(state.is_defined("BITWISE_NOT_MASK"));
}

#[test]
fn logical_not_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if !0
                #define NOT_ZERO_IS_TRUE
            #endif
            
            #if !1 == 0
                #define NOT_ONE_IS_FALSE
            #endif
            
            #if !(5 > 10)
                #define NOT_FALSE_IS_TRUE
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("NOT_ZERO_IS_TRUE"));
    assert!(state.is_defined("NOT_ONE_IS_FALSE"));
    assert!(state.is_defined("NOT_FALSE_IS_TRUE"));
}

#[test]
fn ternary_conditional_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if (1 ? 42 : 0) == 42
                #define TERNARY_TRUE_BRANCH
            #endif
            
            #if (0 ? 42 : 99) == 99
                #define TERNARY_FALSE_BRANCH
            #endif
            
            #if (5 > 3 ? 1 : 0)
                #define TERNARY_WITH_COMPARISON
            #endif
            
            #if (1 ? (2 ? 3 : 4) : 5) == 3
                #define NESTED_TERNARY
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("TERNARY_TRUE_BRANCH"));
    assert!(state.is_defined("TERNARY_FALSE_BRANCH"));
    assert!(state.is_defined("TERNARY_WITH_COMPARISON"));
    assert!(state.is_defined("NESTED_TERNARY"));
}

#[test]
fn defined_operator_basic() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define EXISTING_MACRO
            
            #if defined(EXISTING_MACRO)
                #define DEFINED_WORKS
            #endif
            
            #if defined EXISTING_MACRO
                #define DEFINED_WITHOUT_PARENS
            #endif
            
            #if !defined(NONEXISTENT_MACRO)
                #define NOT_DEFINED_WORKS
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("DEFINED_WORKS"));
    assert!(state.is_defined("DEFINED_WITHOUT_PARENS"));
    assert!(state.is_defined("NOT_DEFINED_WORKS"));
}

#[test]
fn defined_operator_complex() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define A 1
            #define B 2
            
            #if defined(A) && defined(B)
                #define BOTH_DEFINED
            #endif
            
            #if defined(A) || defined(C)
                #define AT_LEAST_ONE_DEFINED
            #endif
            
            #if defined(A) && !defined(C)
                #define A_NOT_C
            #endif
            
            #if defined(A) && A > 0
                #define DEFINED_AND_VALUE_CHECK
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("BOTH_DEFINED"));
    assert!(state.is_defined("AT_LEAST_ONE_DEFINED"));
    assert!(state.is_defined("A_NOT_C"));
    assert!(state.is_defined("DEFINED_AND_VALUE_CHECK"));
}

// ==================== Macro Features ====================

#[test]
fn function_like_macros() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define MAX(a, b) ((a) > (b) ? (a) : (b))
            #define MIN(a, b) ((a) < (b) ? (a) : (b))
            #define SQUARE(x) ((x) * (x))
            
            MAX(10, 20)
            MIN(5, 3)
            SQUARE(4)
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    assert!(state.errors().is_empty());
    assert!(state.is_defined("MAX"));
    assert!(state.is_defined("MIN"));
    assert!(state.is_defined("SQUARE"));
    // Should expand to proper expressions
}

#[test]
fn variadic_macros() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define PRINTF(fmt, ...) printf(fmt, __VA_ARGS__)
            #define DEBUG(...) fprintf(stderr, __VA_ARGS__)
            #define LOG(level, ...) log_impl(level, __VA_ARGS__)
            
            PRINTF("Hello %s", "world")
            DEBUG("Error: %d", 42)
            LOG(INFO, "Starting")
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("PRINTF"));
    assert!(state.is_defined("DEBUG"));
    assert!(state.is_defined("LOG"));
}

#[test]
fn stringification_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define STRINGIFY(x) #x
            #define XSTRINGIFY(x) STRINGIFY(x)
            #define MAKE_STRING(a, b) #a " " #b
            
            STRINGIFY(hello world)
            STRINGIFY(123)
            XSTRINGIFY(VERSION)
            MAKE_STRING(foo, bar)
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    assert!(state.errors().is_empty());
    // Should produce "hello world", "123", "VERSION", "foo" " " "bar"
}

#[test]
fn token_pasting_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define PASTE(a, b) a##b
            #define MAKE_VAR(prefix, name) prefix##_##name
            #define CONCAT3(a, b, c) a##b##c
            
            PASTE(foo, bar)
            MAKE_VAR(get, value)
            CONCAT3(x, y, z)
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let _output: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();

    assert!(state.errors().is_empty());
    // Should produce: foobar, get_value, xyz
}

#[test]
fn va_opt_macro() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define LOG(fmt, ...) printf(fmt __VA_OPT__(,) __VA_ARGS__)
            
            LOG("Hello")
            LOG("Hello %s", "world")
            LOG("Values: %d %d", 1, 2)
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

// ==================== Directive Features ====================

#[test]
fn line_directive_basic() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #line 100
            #error "Should be at line 100"
            
            #line 200 "custom_file.c"
            #error "Should be at line 200 in custom_file.c"
            
            #line 1 "reset.h"
            #error "Back to line 1"
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Errors should report the modified line numbers and filenames
}

#[test]
fn predefined_line_file_macros() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define LOCATION __FILE__ ":" __LINE__
            
            #if __LINE__ > 0
                #define LINE_MACRO_WORKS
            #endif
            
            LOCATION
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("LINE_MACRO_WORKS"));
}

#[test]
fn predefined_date_time_macros() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            const char* build_date = __DATE__;
            const char* build_time = __TIME__;
            
            #ifdef __DATE__
                #define HAS_DATE_MACRO
            #endif
            
            #ifdef __TIME__
                #define HAS_TIME_MACRO
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("HAS_DATE_MACRO"));
    assert!(state.is_defined("HAS_TIME_MACRO"));
}

#[test]
fn pragma_operator() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #define PRAGMA(x) _Pragma(#x)
            
            PRAGMA(once)
            _Pragma("pack(push, 1)")
            _Pragma("pack(pop)")
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
}

// ==================== Advanced Expression Features ====================

#[test]
fn operator_precedence() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 2 + 3 * 4 == 14
                #define MULTIPLY_BEFORE_ADD
            #endif
            
            #if 10 - 2 - 3 == 5
                #define LEFT_TO_RIGHT_MINUS
            #endif
            
            #if 1 << 2 + 1 == 8
                #define SHIFT_AFTER_ADD
            #endif
            
            #if 5 > 3 == 1
                #define COMPARISON_RETURNS_ONE
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("MULTIPLY_BEFORE_ADD"));
    assert!(state.is_defined("LEFT_TO_RIGHT_MINUS"));
    assert!(state.is_defined("SHIFT_AFTER_ADD"));
    assert!(state.is_defined("COMPARISON_RETURNS_ONE"));
}

// ==================== Error Handling ====================

#[test]
fn expression_error_messages() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #if 1 + + 2
                #define DOUBLE_PLUS
            #endif
            
            #if (1 + 2
                #define UNCLOSED_PAREN
            #endif
            
            #if 1 2 3
                #define MISSING_OPERATORS
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    // Should have clear error messages for each case
    assert!(!state.errors().is_empty());
}

#[test]
#[ignore = "TODO: Handle recursive macro expansion in expressions"]
fn macro_expansion_in_expressions() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define A 1
            #define B A + 1
            #define C B * 2
            
            #if A == 1
                #define A_IS_ONE
            #endif
            
            #if B == 2
                #define B_IS_TWO
            #endif
            
            #if C == 4
                #define C_IS_FOUR
            #endif
            
            #define COMPLEX ((A + B) * C)
            #if COMPLEX == 12
                #define COMPLEX_EXPANSION_WORKS
            #endif
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("A_IS_ONE"));
    assert!(state.is_defined("B_IS_TWO"));
    assert!(state.is_defined("C_IS_FOUR"));
    assert!(state.is_defined("COMPLEX_EXPANSION_WORKS"));
}
