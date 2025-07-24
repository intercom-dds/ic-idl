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
use common::test_lint_hir;

#[test]
fn valid_union_variants() {
    let source = r"
union Result switch (long) {
    case 0: string error_message;
    case 1: long success_code;
    case 2: boolean cancelled;
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for valid union variants, but got: {output}"
    );
}

#[test]
fn duplicate_variant_names() {
    let source = r"
union Value switch (short) {
    case 0: string text;
    case 1: long number;
    case 2: string text;  // Duplicate variant name
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_duplicate_variants() {
    let source = r"
union Data switch (octet) {
    case 0: string name;
    case 1: long value;
    case 2: string name;    // First duplicate
    case 3: boolean flag;
    case 4: long value;     // Second duplicate
    case 5: string name;    // Third occurrence
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_default_variant() {
    let source = r"
union Response switch (long) {
    case 1: string success;
    default: long error_code;
    case 2: boolean retry;
    default: string error_message;  // Duplicate default
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_variant_multiple_cases() {
    let source = r"
union Status switch (long) {
    case 1:
    case 2:
    case 3: string active;
    case 4: string inactive;
    case 5:
    case 6: string active;  // Duplicate variant name
};
";

    assert_snapshot!(test_lint_hir(source));
}
