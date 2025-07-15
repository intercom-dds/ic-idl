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

use common::lint_hir;

#[test]
#[ignore] // Ignore until annotation lowering is working
fn valid_bit_positions() {
    let report = lint_hir(
        r"
bitmask MyFlags {
    @bit(0) FLAG_A,
    @bit(7) FLAG_B
};

bitmask<unsigned short> LargeFlags {
    @bit(0) FLAG_X,
    @bit(15) FLAG_Y
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn bit_exceeds_octet_width() {
    let report = lint_hir(
        r"
bitmask MyFlags {
    @bit(8) FLAG_TOO_HIGH
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("exceeds type bit width"));
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn bit_exceeds_custom_type_width() {
    let report = lint_hir(
        r"
bitmask<unsigned short> MyFlags {
    @bit(16) FLAG_OUT_OF_BOUNDS
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("@bit(16) exceeds type bit width of 16"));
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn explicit_value_exceeds_bounds() {
    let report = lint_hir(
        r"
bitmask MyFlags {
    FLAG_A = 256  // Too large for octet (8-bit)
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("exceeds type bit width"));
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn mixed_valid_invalid_bits() {
    let report = lint_hir(
        r"
bitmask<unsigned long> Flags {
    @bit(0) FLAG_VALID1,
    @bit(31) FLAG_VALID2,
    @bit(32) FLAG_INVALID,  // Out of bounds for 32-bit
    @bit(63) FLAG_INVALID2  // Way out of bounds
};
",
    );

    assert_eq!(report.errors.len(), 2);
}
