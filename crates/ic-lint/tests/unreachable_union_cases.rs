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

use common::{lint_hir, test_lint_hir};
use insta::assert_snapshot;

#[test]
fn valid_union_with_default_last() {
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    default: boolean c;
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn case_after_default() {
    // Default cases can appear anywhere - this is now valid
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    default: string b;
    case 2: boolean c;  // This is now valid
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn multiple_cases_after_default() {
    // Default cases can appear anywhere - these are now valid
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    default: string b;
    case 2: boolean c;  // This is now valid
    case 3: octet d;    // This is also valid
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn case_label_negative_for_unsigned() {
    let source = r"
union MyUnion switch (unsigned short) {
    case 100: long a;
    case -1: string b;  // Negative value for unsigned type
};
";
    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for negative values on unsigned types (they wrap around), but got: \
         {output}"
    );
}

#[test]
fn default_in_middle() {
    // Test that default can appear in the middle
    let source = r"
union MyUnion switch (long) {
    case 1: long a;
    case 2: string b;
    default: boolean c;
    case 3: octet d;
    case 4: float e;
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn default_at_beginning() {
    // Test that default can appear at the beginning
    let source = r"
union MyUnion switch (long) {
    default: boolean a;
    case 1: long b;
    case 2: string c;
    case 3: octet d;
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}
