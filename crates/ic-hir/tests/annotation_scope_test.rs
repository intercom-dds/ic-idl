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

//! Tests that annotations and regular definitions don't collide.
//! Annotations are stored with an `@` prefix internally, so `@annotation foo`
//! and `struct foo` should coexist without conflict.

mod common;

use ic_hir::hir::DefKind;

/// Test that an annotation and a struct with the same name don't collide.
#[test]
fn test_annotation_and_struct_same_name_no_collision() {
    let input = r"
        @annotation foo {
            long value;
        };

        struct foo {
            long x;
        };

        @foo(42)
        struct bar {
            foo member;
        };
    ";

    let (hir, _, diagnostics) = common::parse_and_resolve(input);

    assert!(
        hir.errors.is_empty(),
        "Expected no errors but got:\n{diagnostics}"
    );

    // Verify we can find both definitions
    let foo_annotation = hir.context.lookup_symbol("@foo");
    let foo_struct = hir.context.lookup_symbol("foo");

    assert!(
        foo_annotation.is_some(),
        "Should find annotation @foo via lookup_symbol"
    );
    assert!(
        foo_struct.is_some(),
        "Should find struct foo via lookup_symbol"
    );

    // Verify they are different definitions
    assert_ne!(
        foo_annotation, foo_struct,
        "Annotation and struct should be different definitions"
    );

    // Verify their types
    let ann_def = hir.context.type_of(foo_annotation.unwrap());
    assert!(
        matches!(ann_def.kind, DefKind::Annotation(_)),
        "Expected annotation definition"
    );

    let struct_def = hir.context.type_of(foo_struct.unwrap());
    assert!(
        matches!(struct_def.kind, DefKind::Struct(_)),
        "Expected struct definition"
    );

    // Verify the annotation was correctly applied to bar
    let bar = hir.context.lookup_symbol("bar").expect("Should find bar");
    let bar_def = hir.context.type_of(bar);
    assert_eq!(bar_def.annotations.len(), 1);
    assert_eq!(bar_def.annotations[0].ident.name, "foo");

    // Verify bar's member has type foo (the struct, not the annotation)
    if let DefKind::Struct(s) = &bar_def.kind {
        assert_eq!(s.members.len(), 1);
        assert_eq!(s.members[0].ident.name, "member");
    } else {
        panic!("Expected bar to be a struct");
    }
}

/// Test that annotations in modules don't collide with types in modules.
#[test]
fn test_annotation_and_type_same_name_in_module() {
    let input = r"
        module M {
            @annotation test {};
            enum test { A, B, C };

            @test
            const test x = test::A;
        };
    ";

    let (hir, _, diagnostics) = common::parse_and_resolve(input);

    assert!(
        hir.errors.is_empty(),
        "Expected no errors but got:\n{diagnostics}"
    );

    // Verify the enum exists via lookup_symbol
    let test_enum = hir.context.lookup_symbol("M::test");
    assert!(test_enum.is_some(), "Should find enum M::test");

    // Verify the annotation exists by checking it was applied to x
    let x = hir.context.lookup_symbol("M::x").expect("Should find M::x");
    let x_def = hir.context.type_of(x);
    assert_eq!(x_def.annotations.len(), 1, "x should have one annotation");
    assert_eq!(x_def.annotations[0].ident.name, "test");
    // The annotation def_id should be Some and different from the enum
    assert!(
        x_def.annotations[0].def_id.is_some(),
        "Annotation should resolve"
    );
    assert_ne!(
        x_def.annotations[0].def_id,
        Some(test_enum.unwrap()),
        "Annotation def_id should differ from enum def_id"
    );
}

/// Test that using the wrong namespace fails appropriately.
#[test]
fn test_annotation_not_found_as_type() {
    let input = r"
        @annotation only_annotation {};

        // Try to use annotation as a type - should fail
        struct bad {
            only_annotation field;
        };
    ";

    let (hir, _, diagnostics) = common::parse_and_resolve(input);

    assert!(
        !hir.errors.is_empty(),
        "Expected errors when using annotation as type"
    );
    assert!(
        diagnostics.contains("no type named"),
        "Expected 'no type named' error, got:\n{diagnostics}"
    );
}

/// Test that using a type as an annotation results in unresolved annotation.
/// Unknown annotations are allowed (they just don't resolve to a `def_id`).
#[test]
fn test_type_not_found_as_annotation() {
    let input = r"
        struct only_struct {
            long x;
        };

        // Try to use struct as annotation - annotation won't resolve
        @only_struct
        struct bad {
            long y;
        };
    ";

    let (hir, _, diagnostics) = common::parse_and_resolve(input);

    // Should compile without errors (unknown annotations are allowed)
    assert!(
        hir.errors.is_empty(),
        "Should not error on unknown annotation:\n{diagnostics}"
    );

    // But the annotation should not resolve (def_id should be None)
    let bad = hir.context.lookup_symbol("bad").expect("Should find bad");
    let bad_def = hir.context.type_of(bad);
    assert_eq!(bad_def.annotations.len(), 1);
    assert_eq!(bad_def.annotations[0].ident.name, "only_struct");
    assert!(
        bad_def.annotations[0].def_id.is_none(),
        "Annotation should NOT resolve since only_struct is a type, not an annotation"
    );
}
