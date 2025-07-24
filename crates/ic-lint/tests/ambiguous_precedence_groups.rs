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
fn test_parentheses_suppress_warnings() {
    let idl = r"
const long test1 = 1 & (2 + 3);
const long test2 = (1 & 2) + 3;
const long test3 = 1 | (2 & 3);
const long test4 = (1 | 2) & 3;
const long test5 = 1 ^ (2 & 3);
const long test6 = (1 ^ 2) & 3;

const long test7 = ((1 & 2) + 3);
const long test8 = (1 & (2 + 3));
const long test9 = ((1 | 2) & (3 ^ 4));

const long test10 = (1 + 2) & (3 * 4);
const long test11 = 1 | ((2 & 3) + 4);
const long test12 = ((1 | 2) & 3) + 4;
";

    let output = test_lint(idl);
    assert!(
        output.is_empty(),
        "Expected no warnings when parentheses are used to clarify precedence, but got: {output}"
    );
}

#[test]
fn test_partial_parentheses_still_warn() {
    let idl = r"
const long test1 = (1 | 2) & 3 + 4;
const long test2 = 1 | 2 & (3 + 4);
const long test3 = 1 + 2 & (3 | 4);
const long test4 = (1 + 2) & 3 | 4;

const long test5 = (1 + 2) & 3;
const long test6 = 1 & (2 + 3);

const long test7 = 1 | 2 & 3 ^ 4;
const long test8 = (1 | 2) & 3 ^ 4;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_deeply_nested_expressions() {
    let idl = r"
const long test1 = 1 | (2 & (3 ^ (4 + 5)));
const long test2 = 1 | 2 & 3 ^ 4 + 5;
const long test3 = ((((1 | 2) & 3) ^ 4) + 5);
const long test4 = 1 | ((2 & 3) ^ (4 + 5));

const long test5 = (1 | 2) & ((3 ^ 4) + 5);
const long test6 = 1 | (2 & 3) ^ 4 + 5;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_no_false_positives() {
    let idl = r"
const long test1 = (1);
const long test2 = ((1));
const long test3 = (1 + 2);
const long test4 = ((1 + 2));
const long test5 = (1 + 2 + 3);
const long test6 = (1 * 2 * 3);
const long test7 = (1 | 2 | 3);
const long test8 = (1 & 2 & 3);

const long test9 = ~(1 & 2);
const long test10 = -(1 + 2);
const long test11 = (~1) & 2;
const long test12 = (-1) + 2;
";

    let output = test_lint(idl);
    assert!(
        output.is_empty(),
        "Expected no false positives for well-parenthesized expressions, but got: {output}"
    );
}

#[test]
fn test_real_world_parentheses_patterns() {
    let idl = r"
const long MASK_LOWER = 0xFF;
const long MASK_UPPER = 0xFF00;

const long combined1 = (MASK_LOWER & value) | (MASK_UPPER & (value << 8));
const long combined2 = (flags & MASK_LOWER);  // Simplified - comparison not supported
const long combined3 = (base + offset) & ~(alignment - 1);

const long bad1 = MASK_LOWER & value | MASK_UPPER & value << 8;
const long bad2 = flags & MASK_LOWER + 1;
const long bad3 = base + offset & ~alignment - 1;
";

    assert_snapshot!(test_lint(idl));
}
