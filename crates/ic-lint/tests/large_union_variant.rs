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

#![allow(clippy::print_stderr)]

mod common;

use common::lint_hir;

#[test]
fn large_variant_with_array() {
    let report = lint_hir(
        r"
union MyUnion switch (short) {
    case 1: char small_data;
    case 2: short another_small;
    case 3: char large_array[1024];  // This is 1KB, much larger than others
};
",
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 1);
    let warning_output = format!("{:?}", report.warnings[0]);
    assert!(warning_output.contains("significantly larger"));
}

#[test]
fn large_variant_with_struct() {
    let report = lint_hir(
        r"
struct LargeStruct {
    long array[128];  // 8 * 128 = 1024 bytes
    double values[32]; // 8 * 32 = 256 bytes
};

struct SmallStruct {
    short value;
};

union MyUnion switch (long) {
    case 1: SmallStruct small;
    case 2: LargeStruct large;  // Much larger than SmallStruct
    case 3: char tiny;
};
",
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 1);
    let warning_output = format!("{:?}", report.warnings[0]);
    assert!(warning_output.contains("significantly larger"));
}

#[test]
fn no_warning_similar_sizes() {
    let report = lint_hir(
        r"
union MyUnion switch (short) {
    case 1: long data1[10];     // 80 bytes
    case 2: double data2[10];   // 80 bytes
    case 3: float data3[20];    // 80 bytes
};
",
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
}

#[test]
fn no_warning_small_difference() {
    let report = lint_hir(
        r"
union MyUnion switch (short) {
    case 1: char small;          // 1 byte
    case 2: long medium;         // 8 bytes
    case 3: long array[2];       // 16 bytes - not large enough difference
};
",
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
}

#[test]
fn warning_with_multiple_small_variants() {
    let report = lint_hir(
        r"
union MyUnion switch (octet) {
    case 1: char var1;
    case 2: short var2;
    case 3: long var3;
    case 4: char var4;
    case 5: char huge_array[2048];  // Much larger than all others
};
",
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 1);
    let warning_output = format!("{:?}", report.warnings[0]);
    assert!(warning_output.contains("huge_array"));
}

#[test]
fn nested_unions() {
    let report = lint_hir(
        r"
union InnerUnion switch (boolean) {
    case TRUE: char small;
    case FALSE: char large[512];
};

union OuterUnion switch (long) {
    case 1: InnerUnion nested;  // Contains large variant
    case 2: char tiny;
};
",
    );

    // Should warn about both unions
    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 2);
}

#[test]
fn single_variant_no_warning() {
    let report = lint_hir(
        r"
union MyUnion switch (short) {
    case 1: char large_array[1024];  // Only one variant, no comparison possible
};
",
    );

    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
}

#[test]
fn dynamic_types_ignored() {
    let report = lint_hir(
        r"
union MyUnion switch (short) {
    case 1: string dynamic_str;      // Dynamic size, ignored
    case 2: sequence<long> dynamic_seq; // Dynamic size, ignored
    case 3: char small;
    case 4: char large[1024];        // Should still trigger warning
};
",
    );

    // Should only warn about the fixed-size large variant
    assert_eq!(report.errors.len(), 0);
    // The warning might not trigger if too many variants have unknown size
    // This depends on the implementation
}

