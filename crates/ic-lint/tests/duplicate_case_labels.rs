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
fn valid_union_cases() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    case 3: boolean c;
    default: octet d;
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
fn duplicate_case_values() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    case 1: boolean c;  // Duplicate of first case
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("duplicate case label"));
    assert!(error_output.contains("'1'"));
}

#[test]
fn multiple_defaults() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1: long a;
    default: string b;
    case 2: boolean c;
    default: octet d;  // Second default
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("multiple default cases"));
}

#[test]
fn duplicate_enum_case() {
    let report = lint_hir(
        r"
enum Color { RED, GREEN, BLUE };

union ColorUnion switch (Color) {
    case RED: long r;
    case GREEN: long g;
    case RED: long r2;  // Duplicate
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("duplicate case label"));
}

#[test]
fn multiple_labels_with_duplicate() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1:
    case 2: long a;
    case 3:
    case 2: string b;  // 2 is duplicate
    default: boolean c;
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("duplicate case label"));
    assert!(error_output.contains("'2'"));
}
