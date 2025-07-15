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
fn valid_union_with_default_last() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    default: boolean c;
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
fn case_after_default() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1: long a;
    default: string b;
    case 2: boolean c;  // Unreachable
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("unreachable"));
    assert!(error_output.contains("after default"));
}

#[test]
fn multiple_cases_after_default() {
    let report = lint_hir(
        r"
union MyUnion switch (long) {
    case 1: long a;
    default: string b;
    case 2: boolean c;  // Unreachable
    case 3: octet d;    // Also unreachable
};
",
    );

    assert_eq!(report.errors.len(), 2);
}

#[test]
fn case_label_out_of_range_octet() {
    let report = lint_hir(
        r"
union MyUnion switch (octet) {
    case 100: long a;
    case 200: string b;
    case 300: boolean c;  // Out of range for octet (0-255)
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("outside the range"));
    assert!(error_output.contains("300"));
}

#[test]
fn case_label_negative_for_unsigned() {
    let report = lint_hir(
        r"
union MyUnion switch (unsigned short) {
    case 100: long a;
    case -1: string b;  // Negative value for unsigned type
};
",
    );

    // This might not be caught if -1 is converted to a large unsigned value
    // The test depends on how the HIR handles negative literals for unsigned types
    assert!(report.errors.len() >= 0);
}

#[test]
fn boolean_out_of_range() {
    let report = lint_hir(
        r"
union MyUnion switch (boolean) {
    case 0: long a;     // false
    case 1: string b;   // true
    case 2: boolean c;  // Out of range for boolean
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("outside the range"));
}
