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

use common::{lint_hir, test_lint_hir};

#[test]
fn test_array_correct_size() {
    let report = lint_hir(r"const int32 MY_CONST[3] = {1, 2, 3};");
    assert!(report.errors.is_empty());
}

#[test]
fn test_array_too_few_elements() {
    insta::assert_snapshot!(test_lint_hir(r"const int32 MY_CONST[3] = {1, 2};"));
}

#[test]
fn test_array_too_many_elements() {
    insta::assert_snapshot!(test_lint_hir(r"const int32 MY_CONST[3] = {1, 2, 3, 4};"));
}

#[test]
fn test_struct_correct_field_count() {
    let report = lint_hir(
        r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        const Point MY_POINT = {1, 2, 3};
    ",
    );
    assert!(report.errors.is_empty());
}

#[test]
fn test_struct_too_few_fields() {
    insta::assert_snapshot!(test_lint_hir(
        r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        const Point MY_POINT = {1, 2};
    ",
    ));
}

#[test]
fn test_struct_too_many_fields() {
    insta::assert_snapshot!(test_lint_hir(
        r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        const Point MY_POINT = {1, 2, 3, 4};
    ",
    ));
}

#[test]
fn test_nested_array_validation() {
    insta::assert_snapshot!(test_lint_hir(
        r"
        struct MyStruct {
            int32 values[2];
        };
        const MyStruct s = {{1, 2, 3}};
    ",
    ));
}

#[test]
fn test_empty_struct_initializer() {
    insta::assert_snapshot!(test_lint_hir(
        r"
        struct Config {
            int32 port;
            string host;
        };
        const Config cfg = {};
    ",
    ));
}

#[test]
fn test_multidimensional_array() {
    insta::assert_snapshot!(test_lint_hir(r"const int32 matrix[2][3] = {{1, 2}, {3, 4, 5}};"));
}

#[test]
fn test_array_in_struct() {
    insta::assert_snapshot!(test_lint_hir(
        r"
        struct Container {
            int32 data[5];
        };
        const Container c = {{1, 2, 3}};
    ",
    ));
}