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

use insta::assert_snapshot;

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

    assert_snapshot!(test_lint(source));
}

#[test]
fn zero_array_bound() {
    let source = r"
struct Invalid {
    long empty_array[0];  // Zero-sized array
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn negative_array_bound() {
    let source = r"
struct Invalid {
    long bad_array[-5];  // Negative array size
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn zero_string_bound() {
    let source = r"
typedef string<0> EmptyString;  // Zero-length string
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn negative_sequence_bound() {
    let source = r"
typedef sequence<long, -10> BadSequence;  // Negative sequence bound
";

    assert_snapshot!(test_lint(source));
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

    assert_snapshot!(test_lint(source));
}
