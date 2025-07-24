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
fn test_shift_operator_precedence() {
    let idl = r"
const long test1 = 1 << 2 + 3;
const long test2 = 1 + 2 << 3;
const long test3 = 1 >> 2 - 3;
const long test4 = 1 - 2 >> 3;

const long test5 = 1 << (2 + 3);
const long test6 = (1 + 2) << 3;
const long test7 = 1 >> (2 - 3);
const long test8 = (1 - 2) >> 3;

const long test9 = 1 << 2 * 3;
const long test10 = 1 * 2 << 3;
const long test11 = 1 >> 2 / 3;
const long test12 = 1 / 2 >> 3;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_shift_with_bitwise() {
    let idl = r"
const long test1 = 1 & 2 << 3;
const long test2 = 1 << 2 & 3;
const long test3 = 1 | 2 >> 3;
const long test4 = 1 >> 2 | 3;

const long test5 = 1 ^ 2 << 3;
const long test6 = 1 << 2 ^ 3;

const long test7 = 1 | 2 & 3 << 4;
const long test8 = 1 << 2 + 3 & 4;

const long test9 = (1 & 2) << 3;
const long test10 = 1 & (2 << 3);
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_complex_shift_expressions() {
    let idl = r"
const long BITS_PER_WORD = 32;
const long WORD_MASK = (1 << BITS_PER_WORD) - 1;

const long test1 = value >> offset & mask;
const long test2 = (value >> offset) & mask;
const long test3 = value & mask << offset;
const long test4 = value & (mask << offset);

const long test5 = flags | 1 << bit;
const long test6 = flags | (1 << bit);
const long test7 = flags & ~(1 << bit);
const long test8 = flags & ~1 << bit;

const long test9 = data >> start & (1 << length) - 1;
const long test10 = (data >> start) & ((1 << length) - 1);
";

    assert_snapshot!(test_lint(idl));
}
