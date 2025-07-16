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

use common::test_lint_hir;
use insta::assert_snapshot;

#[test]
#[ignore = "Annotation lowering not implemented"]
fn valid_bit_positions() {
    let source = r"
bitmask MyFlags {
    @bit(0) FLAG_A,
    @bit(7) FLAG_B
};

bitmask<unsigned short> LargeFlags {
    @bit(0) FLAG_X,
    @bit(15) FLAG_Y
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn bit_exceeds_octet_width() {
    let source = r"
bitmask MyFlags {
    @bit(8) FLAG_TOO_HIGH
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn bit_exceeds_custom_type_width() {
    let source = r"
bitmask<unsigned short> MyFlags {
    @bit(16) FLAG_OUT_OF_BOUNDS
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn explicit_value_exceeds_bounds() {
    let source = r"
bitmask MyFlags {
    FLAG_A = 256  // Too large for octet (8-bit)
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn mixed_valid_invalid_bits() {
    let source = r"
bitmask<unsigned long> Flags {
    @bit(0) FLAG_VALID1,
    @bit(31) FLAG_VALID2,
    @bit(32) FLAG_INVALID,  // Out of bounds for 32-bit
    @bit(63) FLAG_INVALID2  // Way out of bounds
};
";
    assert_snapshot!(test_lint_hir(source));
}
