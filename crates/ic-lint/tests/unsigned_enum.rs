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

mod common;

#[test]
fn test_unsigned_bit_bound() {
    let input = r"
        @bit_bound(8)
        enum Test {
            A,
            B = 200,
            C = 255
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unsigned_default() {
    let input = r"
        enum Test {
            A,
            @value(4294967295)
            B
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_unsigned_counter() {
    let input = r"
        @bit_bound(16)
        enum Test {
            A = 32767,
            B
        };
    ";

    let output = common::test_lint_hir(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_signed_no_warning() {
    let input = r"
        @bit_bound(8)
        enum Test {
            A = -128,
            B = 127
        };
    ";

    let output = common::test_lint_hir(input);
    assert!(!output.contains("unsigned-enum"));
}
