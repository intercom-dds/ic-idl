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
use common::{lint_hir, test_lint_hir};

#[test]
fn valid_union_cases() {
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    case 3: boolean c;
    default: octet d;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn duplicate_case_values() {
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    case 1: boolean c;  // Duplicate of first case
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_enum_cases() {
    let source = r"
enum Color { RED, GREEN, BLUE };

union ColorData switch (Color) {
    case RED: long red_value;
    case GREEN: long green_value;
    case RED: long another_red;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_duplicate_values() {
    let source = r"
union MyUnion switch (short) {
    case 1: long a;
    case 2: string b;
    case 1: boolean c;  // First duplicate
    case 3: float d;
    case 2: double e;   // Second duplicate
    case 1: char f;     // Third duplicate of case 1
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_with_default() {
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    default: boolean c;
    case 1: octet d;  // Duplicate, even with default present
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn boolean_switch_duplicates() {
    let source = r"
union BoolUnion switch (boolean) {
    case TRUE: long true_val;
    case FALSE: string false_val;
    case TRUE: float another_true;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn char_switch_duplicates() {
    let source = r"
union CharUnion switch (char) {
    case 'a': long a_val;
    case 'b': string b_val;
    case 'a': boolean dup_a;  // Duplicate
    case 'c': float c_val;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn negative_case_values() {
    let source = r"
union NegativeUnion switch (long) {
    case -1: long neg_one;
    case 0: string zero;
    case 1: boolean pos_one;
    case -1: float dup_neg;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn hex_octal_decimal_same_value() {
    let source = r"
union NumberUnion switch (short) {
    case 10: long decimal;
    case 0xA: string hex;      // Same as 10, duplicate
    case 012: boolean octal;   // Same as 10, duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn large_case_values() {
    let source = r"
union LargeUnion switch (long long) {
    case 9223372036854775807: long max_val;
    case -9223372036854775808: string min_val;
    case 9223372036854775807: boolean dup_max;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn nested_union_duplicates() {
    let source = r"
module Outer {
    union OuterUnion switch (long) {
        case 1: long a;
        case 1: string b;  // Duplicate in outer
    };
    
    module Inner {
        union InnerUnion switch (long) {
            case 1: long c;    // Same value as outer, but different union
            case 2: string d;
            case 1: boolean e; // Duplicate in inner
        };
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_ordinal_duplicates() {
    let source = r"
enum Status { 
    PENDING = 1, 
    ACTIVE = 2, 
    COMPLETED = 3 
};

union StatusData switch (Status) {
    case PENDING: long pending_data;
    case ACTIVE: string active_data;
    case PENDING: boolean dup_pending;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn octet_switch_duplicates() {
    let source = r"
union OctetUnion switch (octet) {
    case 0: long zero;
    case 255: string max;
    case 128: boolean mid;
    case 0: float dup_zero;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn unsigned_switch_duplicates() {
    let source = r"
union UnsignedUnion switch (unsigned long) {
    case 0: long zero;
    case 4294967295: string max;
    case 1000: boolean thousand;
    case 4294967295: float dup_max;  // Duplicate
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_unions_same_discriminator() {
    let source = r"
union FirstUnion switch (long) {
    case 1: long a;
    case 2: string b;
    case 1: boolean c;  // Duplicate in first
};

union SecondUnion switch (long) {
    case 1: float d;     // Same value as first union, but different union
    case 3: double e;
    case 1: char f;      // Duplicate in second
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn const_case_duplicates() {
    let source = r"
const long A = 1;
const long B = 2;
const long C = 1;  // Same value as A

union ConstUnion switch (long) {
    case A: long a_val;
    case B: string b_val;
    case C: boolean c_val;  // Should be duplicate of A
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn expression_case_duplicates() {
    let source = r"
const long X = 5;

union ExprUnion switch (long) {
    case X: long x_val;
    case X + 1: string x_plus_one;
    case 5: boolean five;  // Duplicate of X
    case 6: float six;     // Duplicate of X + 1
};
";

    assert_snapshot!(test_lint_hir(source));
}
