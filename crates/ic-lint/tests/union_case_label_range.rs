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
use common::test_lint_hir;

#[test]
fn test_union_case_label_exceeds_32_bits() {
    let input = r"
        union MyUnion switch(long) {
            case 0x100000000: long bigValue;  // 2^32, exceeds 32-bit range
            case 4294967295: long maxUint32;   // 2^32 - 1, still valid
            case -2147483648: long minInt32;   // -2^31, valid
            case 4294967296: long tooLarge;    // 2^32, exceeds range
        };
    ";
    let output = test_lint_hir(input);
    assert_snapshot!(output);
}

#[test]
fn test_union_case_label_valid_ranges() {
    let input = r"
        union ValidUnion switch(long) {
            case 0: long zero;
            case 2147483647: long maxInt32;     // 2^31 - 1
            case -2147483648: long minInt32;    // -2^31
            case 4294967295: long maxUint32;    // 2^32 - 1
            default: boolean other;
        };
    ";
    let output = test_lint_hir(input);
    assert_snapshot!(output);
}

#[test]
fn test_union_case_label_with_constants() {
    let input = r"
        const long SMALL_VALUE = 100;
        const long BIG_VALUE = 0x100000000;  // 2^32
        
        union UnionWithConstants switch(long) {
            case SMALL_VALUE: long small;
            case BIG_VALUE: long big;  // This should error
        };
    ";
    let output = test_lint_hir(input);
    assert_snapshot!(output);
}

#[test]
fn test_union_case_negative_exceeds_32_bits() {
    let input = r"
        const long long BIG_NEGATIVE = -2147483649;  // -(2^31 + 1)
        
        union NegativeUnion switch(long long) {
            case -1: short minusOne;
            case -2147483648: long minInt32;    // -2^31, valid
            case -2147483649: long tooNegative; // -(2^31 + 1), exceeds range
            case BIG_NEGATIVE: long constNeg;   // Should also error
        };
    ";
    let output = test_lint_hir(input);
    assert_snapshot!(output);
}
