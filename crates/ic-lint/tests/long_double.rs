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
use common::{lint_hir, test_lint_hir};

#[test]
fn long_double_struct_member() {
    assert_snapshot!(test_lint_hir(
        r"
struct Measurement {
    long double value;
    string unit;
};
"
    ));
}

#[test]
fn long_double_typedef() {
    assert_snapshot!(test_lint_hir(
        r"
typedef long double ExtendedPrecision;
"
    ));
}

#[test]
fn long_double_sequence() {
    assert_snapshot!(test_lint_hir(
        r"
typedef sequence<long double> LongDoubleList;
"
    ));
}

#[test]
fn long_double_array() {
    assert_snapshot!(test_lint_hir(
        r"
typedef long double LongDoubleArray[10];
"
    ));
}

#[test]
fn long_double_union_variant() {
    assert_snapshot!(test_lint_hir(
        r"
union Data switch (long) {
case 0:
    double normal;
case 1:
    long double extended;
};
"
    ));
}

#[test]
fn long_double_interface_operation() {
    assert_snapshot!(test_lint_hir(
        r"
interface Calculator {
    long double compute(in long double input);
};
"
    ));
}

#[test]
fn long_double_multiple() {
    assert_snapshot!(test_lint_hir(
        r"
struct Precision {
    long double a;
    long double b;
};
typedef long double LD;
"
    ));
}

#[test]
fn no_long_double() {
    let output = lint_hir(
        r"
struct Point {
    double x;
    double y;
};
",
    );
    assert!(output.warnings.is_empty());
    assert!(output.errors.is_empty());
}
