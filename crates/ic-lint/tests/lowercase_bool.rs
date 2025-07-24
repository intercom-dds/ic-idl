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

use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_lowercase_true_false() {
    let source = r"
        const boolean LOWERCASE_TRUE = true;
        const boolean LOWERCASE_FALSE = false;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_uppercase_booleans_no_warning() {
    let source = r"
        const boolean UPPER_TRUE = TRUE;
        const boolean UPPER_FALSE = FALSE;
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for uppercase TRUE/FALSE"
    );
}

#[test]
fn test_boolean_in_struct_default() {
    let source = r"
        struct Config {
            boolean enabled;
            boolean verbose;
            boolean strict;
        };
        const Config DEFAULT_CONFIG = {true, false, TRUE};
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_boolean_in_expressions() {
    let source = r"
        const boolean EXPR1 = true;
        const boolean EXPR2 = false;
        const boolean EXPR3 = True;
        const boolean EXPR4 = False;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_boolean_in_annotation() {
    let source = r"
        @annotation Feature {
            boolean enabled default true;
            boolean experimental default false;
        };
    ";

    // Default values in annotations are not standard IDL
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings (parser doesn't support defaults in annotations), but got: {}",
        output
    );
}
