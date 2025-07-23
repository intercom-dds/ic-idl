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

mod common;

use insta::assert_snapshot;

fn test_overflow(source: &str) -> String {
    let (_, _, output) = common::parse_and_resolve(source);
    output
}

#[test]
fn test_signed_overflow_addition() {
    let source = r"
const int32 MAX = 2147483647;
const int32 OVERFLOW = MAX + 1;
";

    assert_snapshot!(test_overflow(source));
}

#[test]
fn test_signed_overflow_subtraction() {
    let source = r"
const int32 MIN = -2147483648;
const int32 OVERFLOW = MIN - 1;
";

    assert_snapshot!(test_overflow(source));
}

#[test]
fn test_signed_overflow_multiplication() {
    let source = r"
const int32 BIG = 1000000;
const int32 OVERFLOW = BIG * BIG;
";

    assert_snapshot!(test_overflow(source));
}

#[test]
fn test_unsigned_underflow_allowed() {
    let source = r"
const uint32 ZERO = 0;
const uint32 UNDERFLOW = ZERO - 1;
const uint32 MAX_VIA_UNDERFLOW = -1;
";

    assert_snapshot!(test_overflow(source));
}

#[test]
fn test_no_overflow_normal_operations() {
    let source = r"
const int32 A = 100;
const int32 B = A + 50;
const int32 C = B * 2;
const int32 D = C - 100;
";

    assert_snapshot!(test_overflow(source));
}

#[test]
fn test_mixed_type_overflow() {
    let source = r"
const int64 BIG = 9223372036854775807;
const int64 OVERFLOW = BIG + 1;
";

    assert_snapshot!(test_overflow(source));
}
