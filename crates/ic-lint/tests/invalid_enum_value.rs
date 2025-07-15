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
#[ignore] // HIR enum lowering not fully working
fn valid_enum_default_type() {
    let report = lint_hir(
        r"
enum Color {
    RED = 0,
    GREEN = 1,
    BLUE = 2
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn valid_enum_implicit_values() {
    let report = lint_hir(
        r"
enum Status {
    PENDING,    // 0
    ACTIVE,     // 1
    COMPLETED   // 2
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn duplicate_explicit_values() {
    let report = lint_hir(
        r"
enum Priority {
    LOW = 1,
    MEDIUM = 2,
    HIGH = 1    // Duplicate value
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("already used"));
    assert!(error_output.contains("'LOW'"));
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn duplicate_implicit_value() {
    let report = lint_hir(
        r"
enum Mixed {
    A = 0,
    B = 2,
    C,      // Implicit value 3
    D = 3   // Duplicate of C's implicit value
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("already used"));
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn value_out_of_range_octet() {
    let report = lint_hir(
        r"
enum SmallEnum : octet {
    A = 100,
    B = 200,
    C = 300  // Out of range for octet (0-255)
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("outside the range"));
    assert!(error_output.contains("[0, 255]"));
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn implicit_value_overflow() {
    let report = lint_hir(
        r"
enum OverflowEnum : octet {
    A = 254,
    B,      // 255 - still valid
    C       // 256 - overflows octet range
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("implicit"));
    assert!(error_output.contains("outside the range"));
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn negative_value_unsigned_type() {
    let report = lint_hir(
        r"
enum UnsignedEnum : unsigned short {
    A = -1,  // Negative value for unsigned type
    B = 0,
    C = 1
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("outside the range"));
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn multiple_duplicates() {
    let report = lint_hir(
        r"
enum MultiDup {
    A = 1,
    B = 2,
    C = 1,  // Duplicate of A
    D = 2,  // Duplicate of B
    E = 3
};
",
    );

    assert_eq!(report.errors.len(), 2);
}

#[test]
#[ignore] // HIR enum lowering not fully working
fn large_values_int64() {
    let report = lint_hir(
        r"
enum LargeEnum : long long {
    MIN = -9223372036854775808,
    MAX = 9223372036854775807,
    ZERO = 0
};
",
    );

    assert_eq!(report.errors.len(), 0);
}
