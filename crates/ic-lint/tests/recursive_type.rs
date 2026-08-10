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

#[test]
fn direct_struct_recursion() {
    let output = common::test_lint_hir(
        r"
        struct Node {
            long value;
            Node next;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn struct_recursion_with_sequence() {
    let report = common::lint_hir(
        r"
        struct Node {
            long value;
            sequence<Node> children;
        };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn shared_member_bounds_size_but_not_value() {
    let output = common::test_lint_hir(
        r"
        struct Node {
            long value;
            @shared Node next;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn external_member_bounds_size_but_not_value() {
    let output = common::test_lint_hir(
        r"
        struct Node {
            long value;
            @external Node next;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn exception_recursion() {
    let output = common::test_lint_hir(
        r"
        exception RecursiveError {
            string message;
            RecursiveError cause;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn recursion_through_array() {
    let output = common::test_lint_hir(
        r"
        struct Node {
            Node items[10];
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn recursion_through_map_key() {
    let report = common::lint_hir(
        r"
        struct Node {
            map<Node, string> children;
        };
        ",
    );

    assert!(report.errors.is_empty());
}

#[test]
fn recursion_through_map_value() {
    let report = common::lint_hir(
        r"
        struct Node {
            map<string, Node> children;
        };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn recursion_in_sequence_element() {
    let report = common::lint_hir(
        r"
        struct Node {
            sequence<Node> nodes;
        };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn multiple_recursive_fields() {
    let output = common::test_lint_hir(
        r"
        struct Tree {
            long value;
            Tree left;
            Tree right;
            @shared Tree parent;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn non_recursive_struct() {
    let report = common::lint_hir(
        r"
        struct Point {
            long x;
            long y;
        };

        struct Rectangle {
            Point topLeft;
            Point bottomRight;
        };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn nested_struct_recursion() {
    let output = common::test_lint_hir(
        r"
        module Nested {
            struct Node {
                long value;
                Node next;
            };
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn mutual_recursion_two_types() {
    let output = common::test_lint_hir(
        r"
        struct B;
        struct A { B b; };
        struct B { A a; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn mutual_recursion_three_types() {
    let output = common::test_lint_hir(
        r"
        struct B;
        struct C;
        struct A { B b; };
        struct B { C c; };
        struct C { A a; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn mutual_recursion_broken_by_sequence() {
    let report = common::lint_hir(
        r"
        struct B;
        struct A { B b; };
        struct B { sequence<A> items; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn recursion_through_typedef() {
    let output = common::test_lint_hir(
        r"
        struct A;
        typedef A AliasOfA;
        struct A { AliasOfA self; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn recursion_through_inheritance() {
    let output = common::test_lint_hir(
        r"
        struct B;
        struct A { B b; };
        struct B : A { };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn recursion_through_valuetype() {
    let output = common::test_lint_hir(
        r"
        valuetype Node {
            public long value;
            public Node next;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn union_with_inline_recursive_branch() {
    let output = common::test_lint_hir(
        r"
        struct A;
        union U switch (long) {
            case 1: A a;
            case 2: long n;
        };
        struct A { U u; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn union_with_external_branch_and_finite_case() {
    let report = common::lint_hir(
        r"
        struct A;
        union U switch (long) {
            case 1: @external A a;
            case 2: long n;
        };
        struct A { U u; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn union_with_sequence_branch() {
    let report = common::lint_hir(
        r"
        struct A;
        union U switch (long) {
            case 1: sequence<A> items;
            case 2: long n;
        };
        struct A { U u; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn exhaustive_bool_union_with_external_arms() {
    let output = common::test_lint_hir(
        r"
        struct A;
        union W switch (boolean) {
            case TRUE:  @external A a;
            case FALSE: @external A b;
        };
        struct A { W w; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn exhaustive_enum_union_with_external_arms() {
    let output = common::test_lint_hir(
        r"
        enum Choice { FIRST, SECOND };
        struct A;
        union W switch (Choice) {
            case FIRST:  @external A a;
            case SECOND: @external A b;
        };
        struct A { W w; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn exhaustive_union_with_typedef_bool_discriminator() {
    let output = common::test_lint_hir(
        r"
        typedef boolean Flag;
        struct A;
        union W switch (Flag) {
            case TRUE:  @external A a;
            case FALSE: @external A b;
        };
        struct A { W w; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn exhaustive_union_with_typedef_enum_discriminator() {
    let output = common::test_lint_hir(
        r"
        enum Choice { FIRST, SECOND };
        typedef Choice Pick;
        struct A;
        union W switch (Pick) {
            case FIRST:  @external A a;
            case SECOND: @external A b;
        };
        struct A { W w; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn non_exhaustive_union_with_typedef_discriminator() {
    let report = common::lint_hir(
        r"
        typedef long Code;
        struct A;
        union V switch (Code) {
            case 1: @external A a;
            case 2: @external A b;
        };
        struct A { V v; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn non_exhaustive_union_with_recursive_arms() {
    let report = common::lint_hir(
        r"
        struct A;
        union V switch (long) {
            case 1: @external A a;
            case 2: @external A b;
        };
        struct A { V v; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn union_with_explicit_finite_default() {
    let report = common::lint_hir(
        r"
        struct A;
        union U switch (boolean) {
            case TRUE: @external A a;
            default:   long n;
        };
        struct A { U u; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn nested_sequence_recursion() {
    let report = common::lint_hir(
        r"
        struct A { sequence<sequence<A> > nested; };
        ",
    );

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn container_of_recursive_type_is_not_reported() {
    let output = common::test_lint_hir(
        r"
        struct B;
        struct A { B b; };
        struct B { A a; };
        struct Holder { A a; };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn direct_recursion_reports_size_error_only() {
    let output = common::test_lint_hir(
        r"
        struct A { A a; };
        ",
    );
    assert_snapshot!(output);
}
