// Copyright 2026 KONGSBERG
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
fn test_shift_in_sequence_bound() {
    let source = r"
        typedef sequence<long, 1 << 8> BoundedSeq;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_shift_in_string_bound() {
    let source = r"
        typedef string<1 << 10> BoundedString;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_shift_in_map_bound() {
    let source = r"
        typedef map<string, long, 1 << 4> BoundedMap;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_rshift_in_bound() {
    let source = r"
        typedef sequence<long, 256 >> 2> BoundedSeq;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_nested_sequence_with_shift() {
    let source = r"
        typedef sequence<sequence<long, 1 << 4>, 1 << 8> NestedSeq;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_no_warning_for_constant_bound() {
    let source = r"
        const long SIZE = 256;
        typedef sequence<long, SIZE> BoundedSeq;
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for constant bounds");
}

#[test]
fn test_no_warning_for_literal_bound() {
    let source = r"
        typedef sequence<long, 256> BoundedSeq;
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for literal bounds");
}

#[test]
fn test_no_warning_for_arithmetic_bound() {
    let source = r"
        typedef sequence<long, 16 * 16> BoundedSeq;
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for non-shift arithmetic"
    );
}

#[test]
fn test_shift_in_complex_expression() {
    let source = r"
        typedef sequence<long, (1 << 8) + 1> BoundedSeq;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_shift_in_fixed_bound() {
    let source = r"
        typedef fixed<1 << 4, 2 >> 1> BoundedFixed;
    ";

    assert_snapshot!(test_lint(source));
}
