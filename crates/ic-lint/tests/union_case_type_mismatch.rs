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
fn case_label_from_wrong_enum() {
    let source = r"
enum Color { 
    RED = 1, 
    GREEN = 2, 
    BLUE = 3 
};

enum Size { 
    SMALL = 10, 
    MEDIUM = 11, 
    LARGE = 12 
};

union MyUnion switch (Color) {
    case RED: long r;
    case SMALL: string s;  // SMALL is from Size enum (value 10)
    case GREEN: long g;
    case LARGE: float l;   // LARGE is from Size enum (value 12)
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn correct_enum_cases() {
    let source = r"
enum Status { 
    PENDING, 
    ACTIVE, 
    DONE 
};

union StatusData switch (Status) {
    case PENDING: string message;
    case ACTIVE: long id;
    case DONE: boolean success;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no warnings, but got: {output}");
}

#[test]
fn union_with_non_enum_discriminator() {
    let source = r"
union IntUnion switch (long) {
    case 1: string one;
    case 2: string two;
    default: string other;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no warnings, but got: {output}");
}

#[test]
fn union_with_constants() {
    let source = r"
const long STATUS_OK = 200;
const long STATUS_ERROR = 500;

enum Code { OK, ERROR };

union Response switch (long) {
    case STATUS_OK: string message;    // This is fine - using constants with long discriminator
    case STATUS_ERROR: string error;
};

union CodeResponse switch (Code) {
    case OK: string message;           // This is fine - using correct enum
    case ERROR: string error;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no warnings, but got: {output}");
}

#[test]
fn multiple_wrong_enum_cases() {
    let source = r"
enum Primary { A, B, C };
enum Secondary { X, Y, Z };
enum Tertiary { P, Q, R };

union Mixed switch (Primary) {
    case A: string a;
    case X: string x;  // Wrong: from Secondary
    case Y: string y;  // Wrong: from Secondary  
    case B: string b;
    case P: string p;  // Wrong: from Tertiary
};
";

    assert_snapshot!(test_lint_hir(source));
}
