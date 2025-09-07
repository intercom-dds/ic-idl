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
fn test_optional_discriminator() {
    let source = r"
union OptionalDisc switch(@optional long) {
case 1:
    string value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_key_on_union_member() {
    let source = r"
union KeyMember switch(long) {
case 1:
    @key string value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_key_on_discriminator_valid() {
    let source = r"
union KeyDisc switch(@key long) {
case 1:
    string value;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_multiple_violations() {
    let source = r"
union MultiViolations switch(@optional long) {
case 1:
    @key string first;
case 2:
    @key long second;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_optional_with_enum_discriminator() {
    let source = r"
enum Status {
    OK,
    ERROR
};

union StatusData switch(@optional Status) {
case OK:
    string message;
case ERROR:
    long code;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_key_on_multiple_variants() {
    let source = r"
enum Color {
    RED,
    GREEN,
    BLUE
};

union ColorData switch(Color) {
case RED:
    @key long red_value;
case GREEN:
    @key long green_value;  
case BLUE:
    long blue_value;  // OK - no @key
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_nested_unions() {
    let source = r"
union Inner switch(@optional boolean) {
case TRUE:
    string value;
case FALSE:
    long number;
};

union Outer switch(long) {
case 1:
    @key Inner inner;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_valid_union_no_violations() {
    let source = r"
union ValidUnion switch(octet) {
case 0:
    string text;
case 1:
    long number;
case 2:
    float decimal;
default:
    boolean flag;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_union_with_char_discriminator() {
    // This tests both union_key and char_discriminator lints
    let source = r"
union CharUnion switch(@optional char) {
case 'a':
    @key string a_value;
case 'b':
    string b_value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_empty_union_with_optional() {
    let source = r"
union EmptyUnion switch(@optional long) {
default:
    long value;
};
";

    assert_snapshot!(test_lint_hir(source));
}
