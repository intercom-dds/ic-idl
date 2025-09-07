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

#![allow(clippy::print_stderr)]

mod common;

use common::lint_hir;

#[test]
fn zero_sized_array() {
    let report = lint_hir(
        r"
struct Foo {
    long field[0];
};
",
    );

    assert_eq!(report.errors.len(), 1);
    // Check that the error message contains the expected text
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("array size must be greater than zero"));
}

#[test]
fn zero_bound_sequence() {
    let report = lint_hir(
        r"
struct Foo {
    sequence<long, 0> field;
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("sequence bound must be greater than zero"));
}

#[test]
fn zero_bound_string() {
    let report = lint_hir(
        r"
struct Foo {
    string<0> field;
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("string bound must be greater than zero"));
}

#[test]
fn zero_bound_map() {
    let report = lint_hir(
        r"
struct Foo {
    map<string, long, 0> field;
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("map bound must be greater than zero"));
}

#[test]
fn valid_bounds() {
    let report = lint_hir(
        r"
struct Foo {
    long array_field[10];
    sequence<long, 100> seq_field;
    string<255> str_field;
    map<string, long, 50> map_field;
    sequence<long> unbounded_seq;
    string unbounded_str;
    map<string, long> unbounded_map;
};
",
    );

    if !report.errors.is_empty() {
        eprintln!("Unexpected errors:");
        for error in &report.errors {
            eprintln!("{error:?}");
        }
    }
    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
}

#[test]
fn multiple_zero_bounds() {
    let report = lint_hir(
        r"
struct Foo {
    long field1[0];
    sequence<string, 0> field2;
    string<0> field3;
    map<long, string, 0> field4;
};
",
    );

    assert_eq!(report.errors.len(), 4);
}

#[test]
fn nested_zero_bounds() {
    let report = lint_hir(
        r"
typedef long BadArray[0];
typedef sequence<long, 0> BadSequence;
typedef string<0> BadString;
typedef map<string, long, 0> BadMap;

struct Foo {
    BadArray field1;
    BadSequence field2;
    BadString field3;
    BadMap field4;
};
",
    );

    // Should catch the zero bounds in the typedef definitions
    assert_eq!(report.errors.len(), 4);
}
