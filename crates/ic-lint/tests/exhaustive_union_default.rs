// Copyright 2025 KONGSBERG
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
fn test_exhaustive_enum_single_value() {
    let source = r"
enum MyEnum {
    ZERO
};

union MyUnion switch(MyEnum) {
case ZERO:
    string value;
default:
    int32 foo;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exhaustive_enum_multiple_values() {
    let source = r"
enum Color {
    RED,
    GREEN,
    BLUE
};

union ColorData switch(Color) {
case RED:
    long red_value;
case GREEN:
    long green_value;
case BLUE:
    long blue_value;
default:
    long other_value;  // Error: all enum values covered
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_non_exhaustive_enum() {
    let source = r"
enum Status {
    OK,
    WARNING,
    ERROR
};

union StatusData switch(Status) {
case OK:
    string message;
case ERROR:
    long error_code;
default:
    double data;  // OK: WARNING not covered
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_exhaustive_boolean() {
    let source = r"
union BoolUnion switch(boolean) {
case TRUE:
    string yes;
case FALSE:
    string no;
default:
    int32 shouldNotBeAllowed;  // Error: both boolean values covered
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_non_exhaustive_boolean() {
    let source = r"
union BoolUnion switch(boolean) {
case TRUE:
    string yes;
default:
    string other;  // OK: FALSE not covered
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_exhaustive_int8() {
    // This would need 256 cases to be exhaustive, so default is allowed
    let source = r"
union Int8Union switch(int8) {
case -128:
    string min;
case 0:
    string zero;
case 127:
    string max;
default:
    string other;  // OK: not all int8 values covered
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_integer_discriminator() {
    let source = r"
union IntUnion switch(long) {
case 0:
    string zero;
case 1:
    string one;
case 2:
    string two;
default:
    int32 other;  // OK: integers can't be exhaustively checked (except small ones)
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_char_discriminator() {
    let source = r"
union CharUnion switch(char) {
case 'a':
    string a;
case 'b':
    string b;
default:
    string other;  // OK: char type not checked for exhaustiveness
};
";

    // char discriminator may have warnings but should not error on exhaustiveness
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
}

#[test]
fn test_multiple_labels_per_case() {
    let source = r"
enum Result {
    SUCCESS,
    FAILURE,
    PENDING
};

union ResultData switch(Result) {
case SUCCESS:
case PENDING:
    string message;
case FAILURE:
    long error_code;
default:
    double data;  // Error: all enum values covered
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_no_default_case() {
    let source = r"
enum Type {
    A, B, C
};

union TypeData switch(Type) {
case A:
    long a;
case B:
    string b;
case C:
    float c;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_only_default_case() {
    let source = r"
enum MyEnum {
    NONE
};

union EmptyData switch(MyEnum) {
default:
    long value;  // OK: only default case
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_exhaustive_with_struct_member() {
    let source = r"
enum Outer { X, Y };

struct InnerData {
    long value;
};

union OuterUnion switch(Outer) {
case X:
    InnerData inner;
case Y:
    string y;
default:
    float f;  // Error: all Outer values covered
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exhaustive_boolean_default() {
    let source = r"
union BoolUnion switch(boolean) {
case TRUE:
case FALSE:
default:
    boolean my_value;
};
";

    assert_snapshot!(test_lint_hir(source));
}
