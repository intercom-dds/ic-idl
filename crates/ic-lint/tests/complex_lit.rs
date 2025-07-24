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
fn test_struct_initializer_in_const() {
    let source = r"
        struct Point {
            double x;
            double y;
        };
        
        const Point ORIGIN = { 0.0, 0.0 };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_array_initializer_in_const() {
    let source = r"
        const long PRIMES[5] = { 2, 3, 5, 7, 11 };
        const double MATRIX[2][2] = { {1.0, 0.0}, {0.0, 1.0} };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_initializer_in_annotation() {
    let source = r#"
        struct Config {
            string name;
            long value;
        };

        struct HasConfig {
            @default({"default", 42})
            Config my_config;
        };
    "#;

    let output = test_lint(source);
    assert!(output.is_empty());
}

#[test]
fn test_simple_literals_allowed() {
    let source = r#"
        const long NUM = 42;
        const string TEXT = "hello";
        const boolean FLAG = TRUE;
        const double PI = 3.14159;
    "#;

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for simple literals");
}

#[test]
fn test_nested_initializers() {
    let source = r#"
        struct Inner {
            long a;
            long b;
        };
        
        struct Outer {
            Inner inner;
            string name;
        };
        
        const Outer NESTED = {
            { 1, 2 },
            "test"
        };
    "#;

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_sequence_initializer() {
    let source = r"
        typedef sequence<long> LongSeq;
        const LongSeq NUMBERS = { 1, 2, 3, 4, 5 };
    ";

    assert_snapshot!(test_lint(source));
}
