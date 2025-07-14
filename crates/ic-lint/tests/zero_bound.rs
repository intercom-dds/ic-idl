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

use common::lint_hir;

#[test]
fn zero_sized_array() {
    let report = lint_hir(
        r#"
struct Foo {
    field: long[0];
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    // Check that the error message contains the expected text
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("array size must be greater than zero"));
}

#[test]
fn zero_bound_sequence() {
    let report = lint_hir(
        r#"
struct Foo {
    field: sequence<long, 0>;
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("sequence bound must be greater than zero"));
}

#[test]
fn zero_bound_string() {
    let report = lint_hir(
        r#"
struct Foo {
    field: string<0>;
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("string bound must be greater than zero"));
}

#[test]
fn zero_bound_map() {
    let report = lint_hir(
        r#"
struct Foo {
    field: map<string, long, 0>;
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("map bound must be greater than zero"));
}

#[test]
fn valid_bounds() {
    let report = lint_hir(
        r#"
struct Foo {
    array_field: long[10];
    seq_field: sequence<long, 100>;
    str_field: string<255>;
    map_field: map<string, long, 50>;
    unbounded_seq: sequence<long>;
    unbounded_str: string;
    unbounded_map: map<string, long>;
};
"#,
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
}

#[test]
fn multiple_zero_bounds() {
    let report = lint_hir(
        r#"
struct Foo {
    field1: long[0];
    field2: sequence<string, 0>;
    field3: string<0>;
    field4: map<long, string, 0>;
};
"#,
    );

    assert_eq!(report.errors.len(), 4);
}

#[test]
fn nested_zero_bounds() {
    let report = lint_hir(
        r#"
typedef sequence<long[0]> BadSequence;
typedef map<string, sequence<long, 0>> BadMap;

struct Foo {
    field1: BadSequence;
    field2: BadMap;
};
"#,
    );

    // Should catch the zero bounds in the typedef definitions
    assert_eq!(report.errors.len(), 2);
}