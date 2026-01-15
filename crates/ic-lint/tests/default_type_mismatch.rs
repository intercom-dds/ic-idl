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
use common::test_lint_hir;

#[test]
fn string_to_int() {
    let source = r#"
struct Bad {
    @default("hello")
    long my_int;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn int_to_string() {
    let source = r"
struct Bad {
    @default(123)
    string my_string;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn bool_to_int() {
    let source = r"
struct Bad {
    @default(true)
    long my_int;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_int_default() {
    let source = r"
struct Good {
    @default(42)
    long my_int;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_string_default() {
    let source = r#"
struct Good {
    @default("hello")
    string my_string;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_float_default() {
    let source = r"
struct Good {
    @default(3.14)
    float my_float;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_bool_default() {
    let source = r"
struct Good {
    @default(TRUE)
    boolean my_bool;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_sequence_default() {
    let source = r"
struct Good {
    @default({1, 2, 3})
    sequence<long> my_seq;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_array_default() {
    let source = r"
struct Good {
    @default({1, 2, 3})
    long my_array[3];
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_enum_default() {
    let source = r"
enum Color { RED, GREEN, BLUE };

struct Good {
    @default(GREEN)
    Color my_color;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn wrong_enum_value() {
    let source = r"
enum Color { RED, GREEN, BLUE };
enum Size { SMALL, MEDIUM, LARGE };

struct Bad {
    @default(SMALL)
    Color my_color;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn sequence_element_type_mismatch() {
    let source = r#"
struct Bad {
    @default({"a", "b"})
    sequence<long> my_seq;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}
