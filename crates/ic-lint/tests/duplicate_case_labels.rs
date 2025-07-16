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
fn valid_union_cases() {
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    case 3: boolean c;
    default: octet d;
};
";

    assert_snapshot!(test_lint_hir(source));
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
