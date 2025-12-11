// Copyright 2025 KONGSBERG
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
fn duplicate_max_on_member_and_typedef() {
    let source = r"
@max(5) typedef int32 MyInt;

struct Foo {
    @max(10) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_min_on_member_and_typedef() {
    let source = r"
@min(0) typedef int32 MyInt;

struct Foo {
    @min(5) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_range_on_member_and_typedef() {
    let source = r"
@range(min = 0, max = 100) typedef int32 MyInt;

struct Foo {
    @range(min = 10, max = 50) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_with_min_same_location() {
    let source = r"
struct Foo {
    @range(min = 0, max = 100)
    @min(10)
    int32 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_with_max_same_location() {
    let source = r"
struct Foo {
    @range(min = 0, max = 100)
    @max(50)
    int32 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn min_on_member_range_on_typedef() {
    let source = r"
@range(min = 0, max = 100) typedef int32 MyInt;

struct Foo {
    @min(10) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn max_on_member_range_on_typedef() {
    let source = r"
@range(min = 0, max = 100) typedef int32 MyInt;

struct Foo {
    @max(50) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_on_member_min_on_typedef() {
    let source = r"
@min(0) typedef int32 MyInt;

struct Foo {
    @range(min = 10, max = 100) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_max_chained_typedefs() {
    let source = r"
@max(100) typedef int32 BaseInt;
@max(50) typedef BaseInt MyInt;

struct Foo {
    MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_max_deep_chain() {
    let source = r"
@max(100) typedef int32 Level1;
typedef Level1 Level2;
typedef Level2 Level3;

struct Foo {
    @max(50) Level3 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_single_max_on_typedef() {
    let source = r"
@max(100) typedef int32 MyInt;

struct Foo {
    MyInt value;
};
";
    let output = test_lint_hir(source);
    assert!(
        !output.contains("duplicate-bounds"),
        "Expected no duplicate-bounds error, got: {output}"
    );
}

#[test]
fn valid_single_max_on_member() {
    let source = r"
typedef int32 MyInt;

struct Foo {
    @max(100) MyInt value;
};
";
    let output = test_lint_hir(source);
    assert!(
        !output.contains("duplicate-bounds"),
        "Expected no duplicate-bounds error, got: {output}"
    );
}

#[test]
fn valid_min_and_max_same_location() {
    let source = r"
struct Foo {
    @min(0)
    @max(100)
    int32 value;
};
";
    let output = test_lint_hir(source);
    assert!(
        !output.contains("duplicate-bounds"),
        "Expected no duplicate-bounds error, got: {output}"
    );
}

#[test]
fn duplicate_on_union_variant() {
    let source = r"
@max(100) typedef int32 MyInt;

union Foo switch (long) {
    case 0:
        @max(50) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}
