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
fn test_char_discriminator() {
    let input = r"
        union CharUnion switch (char) {
            case 'A':
                long number;
            case 'B':
            case 'C':
                string text;
            default:
                boolean flag;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_integer_discriminator_no_warning() {
    let input = r"
        union IntUnion switch (long) {
            case 1:
                long number;
            case 2:
            case 3:
                string text;
            default:
                boolean flag;
        };
    ";

    let output = common::test_lint_hir(input);
    // Should not contain char discriminator warnings
    assert!(!output.contains("char"));
}

#[test]
fn test_enum_discriminator_no_warning() {
    let input = r"
        enum MyType {
            TYPE_A,
            TYPE_B,
            TYPE_C
        };
        
        union EnumUnion switch (MyType) {
            case TYPE_A:
                long number;
            case TYPE_B:
            case TYPE_C:
                string text;
        };
    ";

    let output = common::test_lint_hir(input);
    // Should not contain char discriminator warnings
    assert!(!output.contains("char"));
}

#[test]
fn test_nested_char_union() {
    let input = r"
        module TestModule {
            struct Data {
                string value;
            };
            
            union NestedCharUnion switch (char) {
                case 'X':
                    Data data;
                case 'Y':
                case 'Z':
                    long count;
            };
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_multiple_char_unions() {
    let input = r"
        union FirstCharUnion switch (char) {
            case 'a':
                long value;
        };
        
        union SecondCharUnion switch (char) {
            case 'b':
                string text;
        };
        
        union IntUnion switch (short) {
            case 1:
                boolean flag;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}
