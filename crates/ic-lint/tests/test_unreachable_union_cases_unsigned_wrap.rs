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
fn negative_values_wrap_for_unsigned_discriminator() {
    let idl = r"
        union MyUnion switch(unsigned short) {
        case -1:  // Should wrap to 65535
            string a;
        case 65535: // Same as -1 for unsigned short
            long b;
        };
    ";
    // This should produce a warning about duplicate case labels, not about out-of-range
    let report = lint_hir(idl);
    assert_eq!(report.warnings.len(), 1);
    let warning_str = report.warnings[0].to_string();
    assert!(warning_str.contains("duplicate"));
}

#[test]
fn negative_values_wrap_uint8() {
    let idl = r"
        union MyUnion switch(octet) {
        case -1:  // Should wrap to 255
            string a;
        case -128: // Should wrap to 128
            long b;
        case 255:
            float c;
        };
    ";
    // No out-of-range warnings expected
    let report = lint_hir(idl);
    assert!(report.warnings.is_empty());
}

#[test]
fn negative_values_wrap_uint32() {
    let idl = r"
        union MyUnion switch(unsigned long) {
        case -1:  // Should wrap to 4294967295
            string a;
        case -2147483648: // Should wrap to 2147483648
            long b;
        };
    ";
    // No out-of-range warnings expected
    let report = lint_hir(idl);
    assert!(report.warnings.is_empty());
}

#[test]
fn actual_out_of_range_for_signed() {
    let idl = r"
        union MyUnion switch(short) {
        case -32769: // Out of range for short
            string a;
        case 32768:  // Out of range for short
            long b;
        };
    ";
    // Should produce 2 warnings
    let report = lint_hir(idl);
    assert_eq!(report.warnings.len(), 2);
}

#[test]
fn wrapping_produces_duplicate() {
    let idl = r"
        union MyUnion switch(octet) {
        case -1:   // Wraps to 255
            string a;
        case 255:  // Duplicate
            long b;
        };
    ";
    // Should produce a duplicate case label warning
    let report = lint_hir(idl);
    assert_eq!(report.warnings.len(), 1);
    let warning_str = report.warnings[0].to_string();
    assert!(warning_str.contains("duplicate"));
}
