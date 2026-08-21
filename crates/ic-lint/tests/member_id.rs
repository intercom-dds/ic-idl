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

mod common;

use common::{lint_hir, test_lint_hir};
use insta::assert_snapshot;

#[test]
fn valid_member_ids() {
    let source = r"
struct Foo {
    @id(268435455) long explicit_id;
    @hashid long hashed_id;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
}

#[test]
fn member_id_out_of_range() {
    let source = r"
struct Foo {
    @id(268435456) long value;
};

struct Generated {
    @id(268435455) long last_valid;
    long out_of_range;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_member_ids() {
    let source = r"
struct Base {
    @id(1) long inherited;
};

struct Derived : Base {
    @id(1) long duplicate;
};

struct Generated {
    long first;
    @id(0) long duplicate;
};

union Choice switch (long) {
    case 0: @id(0) long discriminator_collision;
    case 1: @id(2) long first;
    case 2: @id(2) long second;
};
";

    assert_snapshot!(test_lint_hir(source));
}
