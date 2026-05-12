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

//! Tests for updating forward declaration references to point to definitions.

mod common;

use ic_hir::hir::{DefFlags, DefKind, TyKind};

/// Test that type references are updated from forward declarations to definitions.
#[test]
fn test_forward_declaration_reference_update() {
    let source = r"
        // Forward declare A
        struct A;

        // Use A in B (should initially point to forward declaration)
        struct B {
            A field;
        };

        // Define A
        struct A {
            long x;
        };

        // Use A in C (should point to definition directly)
        struct C {
            A field;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(source);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find the definitions
    let mut forward_decl_a = None;
    let mut definition_a = None;
    let mut struct_b = None;
    let mut struct_c = None;

    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "A" {
            if def.flags.contains(DefFlags::IS_INCOMPLETE) {
                forward_decl_a = Some(def_id);
            } else if matches!(&def.kind, DefKind::Struct(_)) {
                definition_a = Some(def_id);
            }
        } else if def.ident.name == "B" {
            struct_b = Some(def_id);
        } else if def.ident.name == "C" {
            struct_c = Some(def_id);
        }
    }

    let forward_decl_a = forward_decl_a.expect("Should have forward declaration of A");
    let definition_a = definition_a.expect("Should have definition of A");
    let struct_b = struct_b.expect("Should have struct B");
    let struct_c = struct_c.expect("Should have struct C");

    // Check that B's field type points to the definition, not the forward declaration
    let b_def = result.context.definitions.get(struct_b);
    if let DefKind::Struct(struct_ty) = &b_def.kind {
        assert_eq!(struct_ty.members.len(), 1);
        let field_ty = &struct_ty.members[0].ty;
        if let TyKind::Adt(type_id) = &field_ty.kind {
            assert_eq!(
                *type_id, definition_a,
                "B's field should point to A's definition, not forward declaration"
            );
            assert_ne!(
                *type_id, forward_decl_a,
                "B's field should not point to A's forward declaration"
            );
        } else {
            panic!("Expected ADT type for B's field");
        }
    } else {
        panic!("Expected struct B");
    }

    // Check that C's field type also points to the definition
    let c_def = result.context.definitions.get(struct_c);
    if let DefKind::Struct(struct_ty) = &c_def.kind {
        assert_eq!(struct_ty.members.len(), 1);
        let field_ty = &struct_ty.members[0].ty;
        if let TyKind::Adt(type_id) = &field_ty.kind {
            assert_eq!(
                *type_id, definition_a,
                "C's field should point to A's definition"
            );
        } else {
            panic!("Expected ADT type for C's field");
        }
    } else {
        panic!("Expected struct C");
    }
}

/// Test with multiple forward declarations of the same type.
#[test]
fn test_multiple_forward_declarations() {
    let source = r"
        // Multiple forward declarations
        struct X;
        struct X;

        // Use X
        struct Y {
            X field;
        };

        // Another forward declaration
        struct X;

        // Define X
        struct X {
            string value;
        };

        // Use X again
        struct Z {
            X field;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(source);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find the definition of X
    let mut definition_x = None;
    let mut forward_decls_x = Vec::new();

    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "X" {
            if def.flags.contains(DefFlags::IS_INCOMPLETE) {
                forward_decls_x.push(def_id);
            } else if matches!(&def.kind, DefKind::Struct(_)) {
                definition_x = Some(def_id);
            }
        }
    }

    let definition_x = definition_x.expect("Should have definition of X");
    assert_eq!(
        forward_decls_x.len(),
        3,
        "Should have 3 forward declarations of X"
    );

    // Find Y and Z
    let mut struct_y = None;
    let mut struct_z = None;

    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "Y" {
            struct_y = Some(def_id);
        } else if def.ident.name == "Z" {
            struct_z = Some(def_id);
        }
    }

    let struct_y = struct_y.expect("Should have struct Y");
    let struct_z = struct_z.expect("Should have struct Z");

    // Both Y and Z should point to X's definition
    for (struct_id, name) in [(struct_y, "Y"), (struct_z, "Z")] {
        let def = result.context.definitions.get(struct_id);
        if let DefKind::Struct(struct_ty) = &def.kind {
            let field_ty = &struct_ty.members[0].ty;
            if let TyKind::Adt(type_id) = &field_ty.kind {
                assert_eq!(
                    *type_id, definition_x,
                    "{name}'s field should point to X's definition"
                );
            } else {
                panic!("Expected ADT type for {name}'s field");
            }
        } else {
            panic!("Expected struct {name}");
        }
    }
}

/// Test with nested types (arrays, sequences, maps).
#[test]
fn test_nested_type_reference_update() {
    let source = r"
        // Forward declare
        struct Element;

        // Use in nested types
        struct Container {
            Element array_field[10];
            sequence<Element> seq_field;
            map<string, Element> map_field;
        };

        // Define Element
        struct Element {
            long data;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(source);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find the definition of Element
    let mut definition_element = None;
    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "Element" && !def.flags.contains(DefFlags::IS_INCOMPLETE) {
            definition_element = Some(def_id);
        }
    }

    let definition_element = definition_element.expect("Should have definition of Element");

    // Find Container
    let mut struct_container = None;
    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "Container" {
            struct_container = Some(def_id);
        }
    }

    let struct_container = struct_container.expect("Should have struct Container");

    // Check all fields in Container point to Element's definition
    let container_def = result.context.definitions.get(struct_container);
    if let DefKind::Struct(struct_ty) = &container_def.kind {
        assert_eq!(struct_ty.members.len(), 3);

        // Check array field
        let array_field = &struct_ty.members[0];
        if let TyKind::Array { ty, .. } = &array_field.ty.kind {
            if let TyKind::Adt(type_id) = &ty.kind {
                assert_eq!(
                    *type_id, definition_element,
                    "Array element type should point to Element's definition"
                );
            } else {
                panic!("Expected ADT type for array element");
            }
        } else {
            panic!("Expected array type for array_field");
        }

        // Check sequence field
        let seq_field = &struct_ty.members[1];
        if let TyKind::Sequence { ty, .. } = &seq_field.ty.kind {
            if let TyKind::Adt(type_id) = &ty.kind {
                assert_eq!(
                    *type_id, definition_element,
                    "Sequence element type should point to Element's definition"
                );
            } else {
                panic!("Expected ADT type for sequence element");
            }
        } else {
            panic!("Expected sequence type for seq_field");
        }

        // Check map field
        let map_field = &struct_ty.members[2];
        if let TyKind::Map { elem, .. } = &map_field.ty.kind {
            if let TyKind::Adt(type_id) = &elem.kind {
                assert_eq!(
                    *type_id, definition_element,
                    "Map value type should point to Element's definition"
                );
            } else {
                panic!("Expected ADT type for map value");
            }
        } else {
            panic!("Expected map type for map_field");
        }
    } else {
        panic!("Expected struct Container");
    }
}

/// Test that inheritance from forward declarations is properly rejected.
/// Note: This behavior is intentional - you cannot inherit from a type that hasn't been defined yet.
#[test]
fn test_inheritance_from_forward_declaration_error() {
    let source = r"
        // Forward declare base
        interface Base;

        // Inherit from forward declaration - this should fail
        interface Derived : Base {
            void method();
        };

        // Define Base
        interface Base {
            void base_method();
        };
    ";

    let (result, _, diagnostics) = common::parse_and_resolve(source);

    // This should produce an error about inheriting from incomplete type
    assert!(!result.errors.is_empty());

    insta::assert_snapshot!(diagnostics);
}

/// Test inheritance when base is defined first.
#[test]
fn test_inheritance_with_defined_base() {
    let source = r"
        // Define Base first
        interface Base {
            void base_method();
        };

        // Now inherit from it
        interface Derived : Base {
            void method();
        };
    ";

    let (result, _, _) = common::parse_and_resolve(source);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find Base and Derived
    let mut base = None;
    let mut derived = None;

    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "Base" {
            base = Some(def_id);
        } else if def.ident.name == "Derived" {
            derived = Some(def_id);
        }
    }

    let base = base.expect("Should have Base");
    let derived = derived.expect("Should have Derived");

    // Check that Derived correctly inherits from Base
    let derived_def = result.context.definitions.get(derived);
    if let DefKind::Interface(interface_ty) = &derived_def.kind {
        assert_eq!(interface_ty.parents.len(), 1);
        assert_eq!(
            interface_ty.parents[0].def_id, base,
            "Derived should inherit from Base"
        );
    } else {
        panic!("Expected interface Derived");
    }
}

/// Test that only forward declarations get updated (not actual invalid references).
#[test]
fn test_no_update_for_non_forward_declarations() {
    let source = r"
        // Define A
        struct A {
            long x;
        };

        // Use A (points to definition)
        struct B {
            A field;
        };

        // Forward declare A again (should not affect B)
        struct A;
    ";

    let (result, _, _) = common::parse_and_resolve(source);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find the definition of A
    let mut definition_a = None;
    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "A" && !def.flags.contains(DefFlags::IS_INCOMPLETE) {
            definition_a = Some(def_id);
        }
    }

    let definition_a = definition_a.expect("Should have definition of A");

    // Find B
    let mut struct_b = None;
    for (def_id, def) in &result.context.definitions {
        if def.ident.name == "B" {
            struct_b = Some(def_id);
        }
    }

    let struct_b = struct_b.expect("Should have struct B");

    // B should still point to A's definition
    let b_def = result.context.definitions.get(struct_b);
    if let DefKind::Struct(struct_ty) = &b_def.kind {
        let field_ty = &struct_ty.members[0].ty;
        if let TyKind::Adt(type_id) = &field_ty.kind {
            assert_eq!(
                *type_id, definition_a,
                "B's field should still point to A's definition"
            );
        } else {
            panic!("Expected ADT type for B's field");
        }
    } else {
        panic!("Expected struct B");
    }
}
