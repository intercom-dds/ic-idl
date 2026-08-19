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
fn test_builtin_annotation_case() {
    let input = r"
        struct Test {
            @Optional string value;
        };
    ";

    insta::assert_snapshot!(common::test_lint_hir(input));
}

#[test]
fn test_user_annotation_case() {
    let input = r"
        @annotation mine {
            boolean value default TRUE;
        };

        struct Test {
            @Mine long field;
        };
    ";

    insta::assert_snapshot!(common::test_lint_hir(input));
}

#[test]
fn test_scoped_annotation_case() {
    let input = r"
        module outer {
            @annotation mine {
                boolean value default TRUE;
            };
        };

        struct Test {
            @outer::Mine long field;
        };
    ";

    insta::assert_snapshot!(common::test_lint_hir(input));
}

#[test]
fn test_matching_case_is_clean() {
    let input = r"
        @annotation mine {
            boolean value default TRUE;
        };

        struct Test {
            @optional string a;
            @mine long b;
        };
    ";

    let output = common::test_lint_hir(input);
    assert!(output.is_empty(), "Expected no warnings, but got: {output}");
}
