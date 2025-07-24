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

use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_extreme_nesting() {
    let idl = r"
const long test1 = 1 | 2 & 3 ^ 4 + 5 * 6 - 7 / 8 % 9;
const long test2 = ((((1 | 2) & 3) ^ 4) + 5);
const long test3 = 1 | (2 & (3 ^ (4 + (5 * 6))));
const long test4 = ((((1) | 2) & ((3) ^ 4)) + (5));

const long test5 = (1 | 2) & 3 ^ (4 + 5) * 6;
const long test6 = 1 | (2 & 3 ^ 4) + 5 * 6;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_whitespace_and_formatting() {
    let idl = r"
const long test1 = 1|2&3;
const long test2 = 1 | 2 & 3;
const long test3 = 1  |  2  &  3;
const long test4 = 1
    | 2
    & 3;

const long test5 = 1 /* comment */ | 2 /* another */ & 3;
const long test6 = 1 | /* bitwise or */ 2 & /* bitwise and */ 3;

const long test7 = 1|(2&3);
const long test8 = 1 | ( 2 & 3 );
const long test9 = 1 | (  2 & 3  );
const long test10 = 1 | (
    2 & 3
);
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_numeric_literals() {
    let idl = r"
const long test1 = 0xFF & 0x0F + 1;
const long test2 = 0377 | 010 & 3;
const long test3 = 15 & 3 + 1;  // 0b1111 & 0b0011 + 1

const long test4 = 0xFFFFFFFF & 0x12345678 + 1;
const long test5 = 4294967295 | 2147483647 & 1000000;

const long test6 = -1 & 2 + 3;
const long test7 = 1 & -2 + 3;
const long test8 = 1 & 2 + -3;

const long test9 = (-1) & (2 + 3);
const long test10 = -(1 & 2) + 3;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_all_bitwise_combinations() {
    let idl = r"
const long test1 = a | b & c;
const long test2 = a | b ^ c;
const long test3 = a ^ b & c;
const long test4 = a & b | c;
const long test5 = a ^ b | c;
const long test6 = a & b ^ c;

const long test7 = a | b ^ c & d;
const long test8 = a & b | c ^ d;
const long test9 = a ^ b & c | d;

const long test10 = a | b | c;
const long test11 = a & b & c;
const long test12 = a ^ b ^ c;

const long test13 = a | (b & c);
const long test14 = (a | b) & c;
const long test15 = a ^ (b & c);
const long test16 = (a ^ b) & c;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_pathological_cases() {
    let idl = r"
const long test1 = (1) & (2) + (3);
const long test2 = ((1)) & ((2)) + ((3));
const long test3 = (((a))) | (((b))) & (((c)));

const long test5 = a1 & a2 + a3 | a4 ^ a5 & a6 + a7 | a8 ^ a9 & a10 + a11;

const long array_size = MAX_SIZE & ALIGN_MASK + 1;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_no_ambiguity_same_associativity() {
    let idl = r"
const long test1 = 1 + 2 + 3 + 4;
const long test2 = 1 - 2 - 3 - 4;
const long test3 = 1 + 2 - 3 + 4;
const long test4 = 1 * 2 * 3 * 4;
const long test5 = 1 / 2 / 3 / 4;
const long test6 = 1 * 2 / 3 * 4;
const long test7 = 1 % 2 % 3 % 4;
const long test8 = 1 * 2 % 3 / 4;

const long test9 = a | b | c | d;
const long test10 = a & b & c & d;
const long test11 = a ^ b ^ c ^ d;
";

    let output = test_lint(idl);
    assert!(
        output.is_empty(),
        "Expected no warnings for operators with same associativity, but got: {output}"
    );
}

#[test]
fn test_cross_type_expressions() {
    let idl = r"
const long test1 = ~a & b + c;
const long test2 = -a | b * c;
const long test3 = +a ^ b - c;

const long test4 = ~(a & b) + c;
const long test5 = -(a + b) & c;
const long test6 = ~a & ~b | ~c;

const long test7 = ~1 & 2;
const long test8 = -1 + 2;
const long test9 = ~a | ~b & ~c;
";

    assert_snapshot!(test_lint(idl));
}
