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
fn test_consistent_annotation_redefinition() {
    let input = r#"
        @annotation my_doc {
            string value;
        };

        @annotation my_doc {
            string value;
        };

        @my_doc("test")
        struct TestStruct {
            long field;
        };
    "#;

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_consistent_annotation_redefinition_with_defaults() {
    let input = r"
        @annotation config {
            long timeout default 30;
            boolean retry default true;
        };

        @annotation config {
            long timeout default 30;
            boolean retry default true;
        };

        @config(60)
        struct TestStruct {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_inconsistent_annotation_different_param_count() {
    let input = r"
        @annotation my_doc {
            string value;
        };

        @annotation my_doc {
            string value;
            long priority;
        };

        struct TestStruct {
            long field;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_inconsistent_annotation_different_param_name() {
    let input = r"
        @annotation my_doc {
            string value;
        };

        @annotation my_doc {
            string description;
        };

        struct TestStruct {
            long field;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_inconsistent_annotation_different_param_type() {
    let input = r"
        @annotation my_doc {
            string value;
        };

        @annotation my_doc {
            long value;
        };

        struct TestStruct {
            long field;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_inconsistent_annotation_different_default() {
    let input = r"
        @annotation config {
            long timeout default 30;
        };

        @annotation config {
            long timeout default 60;
        };

        struct TestStruct {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_inconsistent_annotation_added_default() {
    let input = r"
        @annotation config {
            long timeout;
        };

        @annotation config {
            long timeout default 30;
        };

        struct TestStruct {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_inconsistent_annotation_removed_default() {
    let input = r"
        @annotation config {
            long timeout default 30;
        };

        @annotation config {
            long timeout;
        };

        struct TestStruct {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_consistent_annotation_with_complex_defaults() {
    let input = r#"
        @annotation bounds {
            long min default 0;
            long max default 100;
            string label default "default";
        };

        @annotation bounds {
            long min default 0;
            long max default 100;
            string label default "default";
        };

        @bounds(min=10, max=50)
        struct TestStruct {};
    "#;

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_consistent_annotation_with_multiple_params() {
    let input = r"
        @annotation options {
            boolean flag;
            long count;
        };

        @annotation options {
            boolean flag;
            long count;
        };

        struct TestStruct {
            long field;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_consistent_annotation_with_sequence_type() {
    let input = r"
        @annotation seq_doc {
            sequence<string, 10> values;
        };

        @annotation seq_doc {
            sequence<string, 10> values;
        };

        struct TestStruct {
            long field;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_inconsistent_annotation_different_sequence_bound() {
    let input = r"
        @annotation seq_doc {
            sequence<string, 10> values;
        };

        @annotation seq_doc {
            sequence<string, 20> values;
        };

        struct TestStruct {
            long field;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}
