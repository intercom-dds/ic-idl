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

mod common;

#[test]
fn test_scalar_to_array_typedef() {
    let input = r"
        typedef octet MyArray[12];
        const MyArray DATA[12] = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 };
    ";

    let (result, source_map, _) = common::parse_and_resolve(input);

    assert!(
        !result.errors.is_empty(),
        "Expected error for scalar to array typedef assignment"
    );

    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::DiagnosticEmitter::new()
            .emit(&mut output, &source_map, error)
            .unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_scalar_to_sequence_typedef() {
    let input = r"
        typedef sequence<octet> MySeq;
        const MySeq DATA = 123;
    ";

    let (result, source_map, _) = common::parse_and_resolve(input);

    assert!(
        !result.errors.is_empty(),
        "Expected error for scalar to sequence typedef assignment"
    );

    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::DiagnosticEmitter::new()
            .emit(&mut output, &source_map, error)
            .unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_scalar_to_map_typedef() {
    let input = r"
        typedef map<long, string> MyMap;
        const MyMap DATA = 456;
    ";

    let (result, source_map, _) = common::parse_and_resolve(input);

    assert!(
        !result.errors.is_empty(),
        "Expected error for scalar to map typedef assignment"
    );

    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::DiagnosticEmitter::new()
            .emit(&mut output, &source_map, error)
            .unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_valid_array_typedef_init() {
    let input = r"
        typedef octet MyArray[3];
        const MyArray DATA = { 1, 2, 3 };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_valid_array_of_array_typedef_init() {
    let input = r"
        typedef octet MyArray[12];
        const MyArray DATA[1] = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 }};
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_nested_array_wrong_depth() {
    let input = r"
        typedef octet MyArray[3];
        const MyArray DATA[2] = { 1, 2, 3, 4, 5, 6 };
    ";

    let (result, source_map, _) = common::parse_and_resolve(input);

    assert!(
        !result.errors.is_empty(),
        "Expected error for wrong nesting depth"
    );

    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::DiagnosticEmitter::new()
            .emit(&mut output, &source_map, error)
            .unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}
