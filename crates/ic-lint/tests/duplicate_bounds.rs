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
use common::{lint_hir, test_lint_hir};

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

#[test]
fn exclusive_minimum_conflicts_with_min_same_location() {
    let source = r"
struct Foo {
    @min(1)
    @ext::exclusive_minimum(2)
    int32 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn exclusive_maximum_conflicts_with_max_same_location() {
    let source = r"
struct Foo {
    @max(100)
    @ext::exclusive_maximum(99)
    int32 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_conflicts_with_exclusive_minimum_same_location() {
    let source = r"
struct Foo {
    @range(min = 1, max = 100)
    @ext::exclusive_minimum(2)
    int32 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_conflicts_with_exclusive_maximum_same_location() {
    let source = r"
struct Foo {
    @range(min = 1, max = 100)
    @ext::exclusive_maximum(99)
    int32 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn cross_edge_member_min_typedef_exclusive_maximum_silent() {
    let source = r"
@ext::exclusive_maximum(100) typedef int32 MyInt;

struct Foo {
    @min(1) MyInt value;
};
";
    let output = test_lint_hir(source);
    assert!(
        !output.contains("duplicate-bounds"),
        "Expected no duplicate-bounds error for cross-edge pair, got: {output}"
    );
}

#[test]
fn cross_edge_exclusive_minimum_and_exclusive_maximum_silent() {
    let source = r"
struct Foo {
    @ext::exclusive_minimum(1)
    @ext::exclusive_maximum(100)
    int32 value;
};
";
    let output = test_lint_hir(source);
    assert!(
        !output.contains("duplicate-bounds"),
        "Expected no duplicate-bounds error for half-open exclusive pair, got: {output}"
    );
}

#[test]
fn typedef_chain_min_then_range_pins_current_scope() {
    let source = r"
@min(1) typedef int32 TD1;
@range(min = 1, max = 10) typedef TD1 TD2;

struct Foo {
    TD2 value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn exclusive_minimum_string_value_gives_conflict_and_value_diagnostics() {
    let source = r#"
struct Foo {
    @min(1)
    @ext::exclusive_minimum("abc")
    int32 value;
};
"#;
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn exclusive_minimum_on_member_conflicts_with_min_on_typedef() {
    let source = r"
@min(1) typedef int32 MyInt;

struct Foo {
    @ext::exclusive_minimum(2) MyInt value;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn typedef_chain_walk_stops_at_non_alias_def() {
    let source = r"
struct Inner {
    long x;
};

typedef Inner MyInner;

struct Outer {
    @min(5) MyInner field;
};
";
    let output = test_lint_hir(source);
    assert!(
        !output.contains("duplicate-bounds"),
        "Expected the typedef chain walk to stop at the struct def, got: {output}"
    );
}

#[test]
fn user_defined_exclusive_minimum_is_not_a_builtin_bound() {
    let source = r"
module Custom {
    @annotation exclusive_minimum {
        long value;
    };

    struct Foo {
        @min(1)
        @exclusive_minimum(2)
        int32 value;
    };
};
";
    assert!(
        lint_hir(source).errors.is_empty(),
        "{}",
        test_lint_hir(source)
    );
}
