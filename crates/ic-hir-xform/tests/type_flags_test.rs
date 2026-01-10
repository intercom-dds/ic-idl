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

mod common;

use ic_hir::hir::{DefFlags, DefKind};
use ic_hir_xform::type_flags;

#[test]
fn test_primitive_struct_is_trivial_and_ordered() {
    let idl = r"
        struct Example {
            long value;
            short count;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with only primitives should be trivial"
    );
    assert!(
        example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with only primitives should have total order"
    );
}

#[test]
fn test_float_struct_not_ordered() {
    let idl = r"
        struct Example {
            float value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with float should be trivial"
    );
    assert!(
        !example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with float should NOT have total order"
    );
}

#[test]
fn test_double_struct_not_ordered() {
    let idl = r"
        struct Example {
            double value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with double should be trivial"
    );
    assert!(
        !example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with double should NOT have total order"
    );
}

#[test]
fn test_string_struct_not_trivial() {
    let idl = r"
        struct Example {
            string value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        !example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with string should NOT be trivial"
    );
    assert!(
        example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with string should have total order"
    );
}

#[test]
fn test_sequence_struct_not_trivial() {
    let idl = r"
        struct Example {
            sequence<long> values;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        !example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with sequence should NOT be trivial"
    );
    assert!(
        example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with sequence of ints should have total order"
    );
}

#[test]
fn test_sequence_of_floats_not_ordered() {
    let idl = r"
        struct Example {
            sequence<float> values;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        !example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with sequence should NOT be trivial"
    );
    assert!(
        !example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with sequence of floats should NOT have total order"
    );
}

#[test]
fn test_array_struct_is_trivial() {
    let idl = r"
        struct Example {
            long values[10];
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with array of primitives should be trivial"
    );
    assert!(
        example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with array of ints should have total order"
    );
}

#[test]
fn test_array_of_floats_not_ordered() {
    let idl = r"
        struct Example {
            float values[10];
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with array of floats should be trivial"
    );
    assert!(
        !example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with array of floats should NOT have total order"
    );
}

#[test]
fn test_map_struct_not_trivial() {
    let idl = r"
        struct Example {
            map<long, string> data;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    assert!(
        !example.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with map should NOT be trivial"
    );
    assert!(
        example.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with map of orderable types should have total order"
    );
}

#[test]
fn test_nested_struct_propagates_flags() {
    let idl = r"
        struct Inner {
            float value;
        };

        struct Outer {
            Inner inner;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let outer = transformed
        .iter()
        .find(|def| def.ident.name == "Outer")
        .expect("Outer struct should exist");

    assert!(
        !outer.flags.contains(DefFlags::TOTAL_ORDER),
        "Outer struct containing Inner with float should NOT have total order"
    );
}

#[test]
fn test_union_with_float_not_ordered() {
    let idl = r"
        union Example switch (long) {
            case 0: long value;
            case 1: float other;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example union should exist");

    assert!(
        !example.flags.contains(DefFlags::TOTAL_ORDER),
        "Union with float variant should NOT have total order"
    );
}

#[test]
fn test_struct_with_string_not_trivial_but_ordered() {
    let idl = r"
        struct Node {
            long value;
            string name;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let node = transformed
        .iter()
        .find(|def| def.ident.name == "Node")
        .expect("Node struct should exist");

    assert!(
        !node.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with string should NOT be trivial"
    );
    assert!(
        node.flags.contains(DefFlags::TOTAL_ORDER),
        "Struct with string and int should have total order"
    );
}

#[test]
fn test_recursive_struct_with_only_ints_is_ordered() {
    let idl = r"
        struct Node {
            long value;
            @external Node next;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let node = transformed
        .iter()
        .find(|def| def.ident.name == "Node")
        .expect("Node struct should exist");

    assert!(
        node.flags.contains(DefFlags::TOTAL_ORDER),
        "Recursive struct with only ints should have total order"
    );
}

#[test]
fn test_recursive_struct_with_float_not_ordered() {
    let idl = r"
        struct Node {
            float value;
            @external Node next;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let node = transformed
        .iter()
        .find(|def| def.ident.name == "Node")
        .expect("Node struct should exist");

    assert!(
        !node.flags.contains(DefFlags::TOTAL_ORDER),
        "Recursive struct with float should NOT have total order"
    );
}

#[test]
fn test_mutually_recursive_with_float_not_ordered() {
    let idl = r"
        struct B;

        struct A {
            float value;
            @external B other;
        };

        struct B {
            long value;
            @external A other;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("A struct should exist");

    let b = transformed
        .iter()
        .find(|def| def.ident.name == "B" && matches!(def.kind, DefKind::Struct(_)))
        .expect("B struct should exist");

    assert!(
        !a.flags.contains(DefFlags::TOTAL_ORDER),
        "A with float should NOT have total order"
    );
    assert!(
        !b.flags.contains(DefFlags::TOTAL_ORDER),
        "B referencing A (with float) should NOT have total order"
    );
}

#[test]
fn test_nested_struct_with_float_not_ordered() {
    let idl = r"
        struct A {
            float value;
        };

        struct B {
            long value;
            A other;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("A struct should exist");

    let b = transformed
        .iter()
        .find(|def| def.ident.name == "B")
        .expect("B struct should exist");

    assert!(
        !a.flags.contains(DefFlags::TOTAL_ORDER),
        "A with float should NOT have total order"
    );
    assert!(
        !b.flags.contains(DefFlags::TOTAL_ORDER),
        "B referencing A (with float) should NOT have total order"
    );
}

#[test]
fn test_indirect_float_reference_not_ordered() {
    // This tests the case from ast.idl: Group -> Expr -> Literal -> LiteralValue -> Float
    let idl = r"
        union Value switch (long) {
            case 0: long int_val;
            case 1: double float_val;
        };

        struct Container {
            Value val;
        };

        struct Wrapper {
            Container inner;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let wrapper = transformed
        .iter()
        .find(|def| def.ident.name == "Wrapper")
        .expect("Wrapper struct should exist");

    assert!(
        !wrapper.flags.contains(DefFlags::TOTAL_ORDER),
        "Wrapper indirectly containing double should NOT have total order"
    );
}

#[test]
fn test_enum_is_trivial_and_ordered() {
    let idl = r"
        enum Color {
            RED,
            GREEN,
            BLUE
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let color = transformed
        .iter()
        .find(|def| def.ident.name == "Color")
        .expect("Color enum should exist");

    assert!(
        color.flags.contains(DefFlags::IS_TRIVIAL),
        "Enum should be trivial"
    );
    assert!(
        color.flags.contains(DefFlags::TOTAL_ORDER),
        "Enum should have total order"
    );
}

#[test]
fn test_bitmask_is_trivial_and_ordered() {
    let idl = r"
        bitmask Flags {
            FLAG_A,
            FLAG_B
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let flags = transformed
        .iter()
        .find(|def| def.ident.name == "Flags")
        .expect("Flags bitmask should exist");

    assert!(
        flags.flags.contains(DefFlags::IS_TRIVIAL),
        "Bitmask should be trivial"
    );
    assert!(
        flags.flags.contains(DefFlags::TOTAL_ORDER),
        "Bitmask should have total order"
    );
}

#[test]
fn test_external_member_not_trivial() {
    let idl = r"
        struct Node {
            long value;
            @external Node next;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let node = transformed
        .iter()
        .find(|def| def.ident.name == "Node")
        .expect("Node struct should exist");

    assert!(
        !node.flags.contains(DefFlags::IS_TRIVIAL),
        "Struct with @external member should NOT be trivial (it's heap-allocated)"
    );
}

#[test]
fn test_struct_with_inherited_float_not_ordered() {
    let idl = r"
        valuetype Base {
            public float value;
        };

        valuetype Derived : Base {
            public long other;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = type_flags::transform(hir);

    let derived = transformed
        .iter()
        .find(|def| def.ident.name == "Derived")
        .expect("Derived valuetype should exist");

    assert!(
        !derived.flags.contains(DefFlags::TOTAL_ORDER),
        "Derived inheriting from Base with float should NOT have total order"
    );
}
