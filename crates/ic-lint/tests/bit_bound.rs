// Copyright 2026 KONGSBERG
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
use common::{lint_hir, test_lint_hir};

#[test]
fn zero_enum_bit_bound() {
    let source = r"
@bit_bound(0)
enum EmptyBound {
    EMPTY_VALUE
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_bit_bound_above_maximum() {
    let source = r"
@bit_bound(33)
enum WideBound {
    WIDE_VALUE
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn zero_bitmask_bit_bound() {
    let source = r"
@bit_bound(0)
bitmask EmptyBound {
    EMPTY_VALUE
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn bitmask_bit_bound_above_maximum() {
    let source = r"
@bit_bound(65)
bitmask WideBound {
    WIDE_VALUE
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn maximum_enum_bit_bound_is_valid() {
    let source = r"
@bit_bound(32)
enum WideEnum {
    ENUM_VALUE
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
}

#[test]
fn maximum_bitmask_bit_bound_is_valid() {
    let source = r"
@bit_bound(64)
bitmask WideMask {
    MASK_VALUE
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
}
