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
use common::test_lint;

#[test]
fn test_array_as_parameter() {
    let source = r"
        interface Test {
            void process_array(in long data[10]);
            void process_matrix(in double matrix[3][3]);
            void process_string_array(in string names[]);
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_no_warning_for_sequences() {
    let source = r"
        interface Test {
            void process_sequence(in sequence<long> data);
            void process_bounded_seq(in sequence<double, 10> values);
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for sequence parameters");
}

#[test]
fn test_array_in_struct_member() {
    let source = r"
        struct Data {
            long values[100];
        };
        
        interface Test {
            void process_data(in Data d);
        };
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for arrays in struct members"
    );
}

#[test]
fn test_multiple_array_params() {
    let source = r"
        interface Calculator {
            double dot_product(in double vec1[3], in double vec2[3]);
            void matrix_multiply(
                in double a[4][4],
                in double b[4][4],
                out double result[4][4]
            );
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_array_with_different_directions() {
    let source = r"
        interface Buffer {
            void fill_buffer(out octet buffer[1024]);
            void process_buffer(inout octet buffer[512]);
        };
    ";

    assert_snapshot!(test_lint(source));
}
