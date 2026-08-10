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
fn map_in_bounds() {
    let source = r"
struct Good {
    @default({{1, 1}})
    map<uint8, uint8, 1> my_map;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn map_out_of_bounds() {
    let source = r"
struct Bad {
    @default({{1, 1}, {2, 2}})
    map<uint8, uint8, 1> my_map;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn sequence_in_bounds() {
    let source = r"

const sequence<int32, 3> const_sequence = {1, 2, 3};
const sequence<sequence<int32, 3>, 1> const_sequence2 = {{1, 2, 3}};

struct Good {
    @default({1, 2, 3})
    sequence<int32, 3> my_seq;
};
";
    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn sequence_out_of_bounds() {
    let source = r"

const sequence<int32, 3> const_sequence = {1, 2, 3, 4};
const sequence<sequence<int32, 3>, 1> const_sequence2 = {{1, 2, 3, 4}};

struct Bad {
    @default({1, 2, 3, 4})
    sequence<int32, 3> my_seq;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn string_in_bounds() {
    let source = r#"
const string<3> const_string = "123";

struct Good {
    @default("test")
    string<4> my_string;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn string_out_of_bounds() {
    let source = r#"
const string<3> const_string = "1234";

struct Bad {
    @default("hello")
    string<4> my_string;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn struct_in_bounds() {
    let source = r#"
    struct StringStruct {
        @default("1234")
        string<4> my_string;
    };
    const StringStruct const_struct = {"1234"};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn struct_out_of_bounds() {
    let source = r#"
    struct StringStruct {
        @default("12345")
        string<4> my_string;
    };
    const StringStruct const_struct = {"12345"};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn exception_in_bounds() {
    let source = r#"
    exception TestException {
        @default("1234")
        string<4> my_string;
    };
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn exception_out_of_bounds() {
    let source = r#"
    exception TestException {
        @default("12345")
        string<4> my_string;
    };
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn union_in_bounds() {
    let source = r#"
    union TestUnion switch(boolean) {
        case TRUE:
            @default("1234")
            string<4> my_string;
        case FALSE:
            @default({0})
            sequence<uint32, 1> my_sequence;
    };
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn union_out_of_bounds() {
    let source = r#"
    union TestUnion switch(boolean) {
        case TRUE:
            @default("12345")
            string<4> my_string;
        case FALSE:
            @default({0, 1})
            sequence<uint32, 1> my_sequence;
    };
"#;

    assert_snapshot!(test_lint_hir(source));
}
