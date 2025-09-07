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
    let input = r"const int32 MY_CONST[3] = {1, 2, 3};";
    let report = lint_hir(input);
    assert!(report.errors.is_empty());
}

#[test]
#[ignore]
fn test_array_too_few_elements() {
    let input = r"const int32 MY_CONST[3] = {1, 2};";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
#[ignore]
fn test_array_too_many_elements() {
    let input = r"const int32 MY_CONST[3] = {1, 2, 3, 4};";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_struct_correct_field_count() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        const Point MY_POINT = {1, 2, 3};
    ";
    let report = lint_hir(input);
    assert!(report.errors.is_empty());
}

#[test]
#[ignore]
fn test_struct_too_few_fields() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        const Point MY_POINT = {1, 2};
    ";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
#[ignore]
fn test_struct_too_many_fields() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        const Point MY_POINT = {1, 2, 3, 4};
    ";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
#[ignore]
fn test_nested_array_validation() {
    let input = r"
        struct MyStruct {
            int32 values[2];
        };
        const MyStruct s = {{1, 2, 3}};
    ";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
#[ignore]
fn test_empty_struct_initializer() {
    let input = r"
        struct Config {
            int32 port;
            string host;
        };
        const Config cfg = {};
    ";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
#[ignore]
fn test_multidimensional_array() {
    let input = r"const int32 matrix[2][3] = {{1, 2}, {3, 4, 5}};";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
#[ignore]
fn test_array_in_struct() {
    let input = r"
        struct Container {
            int32 data[5];
        };
        const Container c = {{1, 2, 3}};
    ";
    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_single_element_array() {
    let input = r"const float MY_VALUE[1] = {3.14};";
    let report = lint_hir(input);
    assert!(report.errors.is_empty());
}

#[test]
fn test_multiple_constants_correct_sizes() {
    let input = r"
        struct Vec3 {
            float x;
            float y;
            float z;
        };
        const Vec3 UNIT_X = {1.0, 0.0, 0.0};
        const Vec3 UNIT_Y = {0.0, 1.0, 0.0};
        const Vec3 UNIT_Z = {0.0, 0.0, 1.0};
        const int32 PRIMES[5] = {2, 3, 5, 7, 11};
    ";
    let report = lint_hir(input);
    assert!(report.errors.is_empty());
}

#[test]
fn test_deeply_nested_arrays() {
    let input = r"
        const int32 cube[2][2][2] = {
            {{1, 2}, {3, 4}},
            {{5, 6}, {7, 8}}
        };
    ";
    let report = lint_hir(input);
    assert!(report.errors.is_empty());
}

#[test]
fn test_struct_with_all_primitive_types() {
    let input = r#"
        struct AllTypes {
            boolean b;
            char c;
            octet o;
            int16 i16;
            uint16 u16;
            int32 i32;
            uint32 u32;
            int64 i64;
            uint64 u64;
            float f;
            double d;
            string s;
        };
        const AllTypes all = {
            true,
            'A',
            255,
            -32768,
            65535,
            -2147483648,
            4294967295,
            -9223372036854775807,
            18446744073709551615,
            3.14,
            2.71828,
            "hello"
        };
    "#;
    let report = lint_hir(input);
    assert!(report.errors.is_empty());
}
