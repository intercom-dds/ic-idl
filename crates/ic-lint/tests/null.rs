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
fn test_null_union_variant() {
    let source = r"
        union OptionalValue switch(long) {
            case 0: null;
            case 1: long int_value;
            case 2: string str_value;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_multiple_null_variants() {
    let source = r"
        union MultiNull switch(short) {
            case 0: null;
            case 1: long value;
            case 2: null;
            default: string text;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_union_without_null() {
    let source = r"
        union StandardUnion switch(long) {
            case 1: long number;
            case 2: string text;
            case 3: boolean flag;
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for unions without null");
}

#[test]
fn test_null_with_default() {
    let source = r"
        union DefaultNull switch(octet) {
            case 1: string name;
            case 2: long count;
            default: null;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_nested_union_with_null() {
    let source = r"
        union Inner switch(boolean) {
            case TRUE: long value;
            case FALSE: null;
        };
        
        union Outer switch(long) {
            case 1: Inner inner;
            case 2: null;
        };
    ";

    assert_snapshot!(test_lint(source));
}
