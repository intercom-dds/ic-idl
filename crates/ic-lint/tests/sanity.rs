// Copyright 2024 KONGSBERG
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

mod common;
use common::test_lint;

#[test]
fn valid_array_bounds() {
    let source = r"
struct Data {
    long values[10];
    string names[5];
    octet buffer[256];
};
";

    // Sanity lint only checks AST structure, not semantic issues
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn zero_array_bound() {
    let source = r"
struct Invalid {
    long empty_array[0];  // Zero-sized array
};
";

    // Note: Zero-sized arrays are a semantic issue, not an AST structure issue
    // The sanity lint only checks that arrays HAVE bounds, not that they're valid
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn negative_array_bound() {
    let source = r"
struct Invalid {
    long bad_array[-5];  // Negative array size
};
";

    // Note: Negative array bounds are a semantic issue, not an AST structure issue
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn zero_string_bound() {
    let source = r"
typedef string<0> EmptyString;  // Zero-length string
";

    // Note: Zero-length strings are a semantic issue, not an AST structure issue
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn negative_sequence_bound() {
    let source = r"
typedef sequence<long, -10> BadSequence;  // Negative sequence bound
";

    // Note: Negative sequence bounds are a semantic issue, not an AST structure issue
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn multiple_sanity_issues() {
    let source = r"
struct Problems {
    long zero_array[0];
    string<0> empty_string;
    sequence<boolean, -1> bad_seq;
    char negative_array[-100];
};
";

    // Note: These are all semantic issues, not AST structure issues
    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}
