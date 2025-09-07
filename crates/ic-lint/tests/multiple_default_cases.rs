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
fn test_single_default_case() {
    let source = r"
union MyUnion switch(long) {
    case 1: long x;
    case 2: string s;
    default: float f;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_multiple_default_cases() {
    let source = r"
union MyUnion switch(long) {
    case 1: long x;
    default: string s;
    case 2: float f;
    default: double d;  // Error: multiple default cases
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_three_default_cases() {
    let source = r"
union MyUnion switch(long) {
    case 1: long x;
    default: string s;
    default: float f;
    default: double d;  // Error: 3 default cases
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_no_default_case() {
    let source = r"
union MyUnion switch(long) {
    case 1: long x;
    case 2: string s;
    case 3: float f;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_default_case_ordering() {
    let source = r"
union MyUnion switch(long) {
    default: string s;  // Default at beginning
    case 1: long x;
    case 2: float f;
    default: double d;  // Error: second default
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_enum_discriminator() {
    let source = r"
enum Color { RED, GREEN, BLUE };

union ColorData switch(Color) {
    case RED: long red_value;
    case GREEN: long green_value;
    default: long other_value;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_nested_unions() {
    let source = r"
union Outer switch(long) {
    case 1: long x;
    default: string s;
};

union Inner switch(long) {
    case 1: float f;
    default: double d1;
    default: double d2;  // Error in inner union
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_multiple_unions_with_defaults() {
    let source = r"
union Union1 switch(long) {
    case 1: long x;
    default: string s;
};

union Union2 switch(boolean) {
    case TRUE: long t;
    default: long f1;
    default: long f2;  // Error
};

union Union3 switch(char) {
    case 'a': string a;
    default: string other;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_complex_union() {
    let source = r"
struct Data {
    long value;
};

union ComplexUnion switch(unsigned long) {
    case 0: Data data;
    case 1: sequence<long> numbers;
    case 2: string text;
    default: boolean fallback1;
    case 3: float number;
    default: double fallback2;  // Error
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_union_in_interface() {
    let source = r"
interface Service2 {
    union Result switch(long) {
        case 0: string success;
        default: long error1;
        default: long error2;  // Error
    };
    
    Result doSomething();
};
";

    assert_snapshot!(test_lint_hir(source));
}
