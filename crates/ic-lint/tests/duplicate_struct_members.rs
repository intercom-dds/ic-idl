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
use common::test_lint_hir;

#[test]
fn valid_struct_members() {
    let source = r"
struct Point {
    long x;
    long y;
    long z;
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for valid struct members, but got: {output}"
    );
}

#[test]
fn duplicate_struct_member() {
    let source = r"
struct Rectangle {
    long width;
    long height;
    long width;  // Duplicate member
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn escaped_identifier_collision() {
    let source = r"
struct Data {
    long value;
    long _value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_duplicate_members() {
    let source = r"
struct Data {
    string name;
    long value;
    string name;   // First duplicate
    boolean flag;
    long value;    // Second duplicate
    string name;   // Third occurrence
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_exception_members() {
    let source = r"
exception MyError {
    string message;
    long code;
    string message;  // Duplicate member in exception
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_array_member() {
    let source = r"
struct Arrays {
    long values[10];
    string names[5];
    long values[20];  // Duplicate member name, different array size
};
";

    assert_snapshot!(test_lint_hir(source));
}
