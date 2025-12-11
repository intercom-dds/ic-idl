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
fn test_unknown_annotation_on_struct() {
    let input = r"
        @UnknownAnnotation
        struct TestStruct {
            @AnotherUnknown
            long value;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unknown_annotation_on_interface() {
    let input = r"
        @NonExistent
        interface TestInterface {
            void doSomething();
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unknown_annotation_mixed_with_known() {
    let input = r"
        @annotation KnownAnnotation {
            long value;
        };
        
        @KnownAnnotation(42)
        @UnknownAnnotation
        struct TestStruct {
            @id(1)
            @UnknownField
            long field1;
            
            @KnownAnnotation(10)
            long field2;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unknown_annotation_in_module() {
    let input = r"
        @UnknownModule
        module TestModule {
            @UnknownInner
            struct InnerStruct {
                long value;
            };
            
            @UnknownEnum
            enum Status {
                @UnknownEnumerator
                ACTIVE,
                INACTIVE
            };
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unknown_annotation_on_bitmask_and_bitset() {
    let input = r"
        @UnknownBitmask
        bitmask Flags {
            @UnknownFlag
            FLAG_A,
            FLAG_B
        };
        
        @UnknownBitset
        bitset Configuration {
            @UnknownField
            bitfield<3, unsigned short> mode;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_no_warning_for_known_annotations() {
    let input = r"
        @annotation CustomAnnotation {};
        
        @CustomAnnotation
        struct ValidStruct {
            @id(1)
            long field1;
            
            @optional
            string field2;
        };
        
        @id(test_interface)
        interface ValidInterface {
            void method();
        };
    ";

    let output = common::test_lint_hir(input);
    // Should not have warnings about unknown annotations
    assert!(!output.contains("unknown annotation"));
}

#[test]
fn test_unknown_annotation_on_union() {
    let input = r"
        @UnknownUnion
        union TestUnion switch (long) {
            case 0:
                @UnknownVariant
                string text;
            case 1:
                @AnotherUnknown
                long number;
            default:
                boolean flag;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unknown_annotation_on_exception() {
    let input = r"
        @UnknownException
        exception TestError {
            @UnknownMember
            string message;
            long code;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_typo_suggestion_key() {
    let input = r"
        struct TestStruct {
            @ky
            long field;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_typo_suggestion_optional() {
    let input = r"
        struct TestStruct {
            @optoinal
            long field;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_no_suggestion_for_unrelated() {
    let input = r"
        struct TestStruct {
            @completely_unrelated_name
            long field;
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}
