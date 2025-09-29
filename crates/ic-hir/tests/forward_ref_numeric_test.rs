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

use ic_hir::hir::{DefKind, Numeric, TyKind};

#[test]
fn test_const_sequence_forward_ref_update() {
    let input = r"
        // Constant using forward-declared type
        const sequence<Foo> x = {};
        
        // Actual definition
        struct Foo {
            long value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Find the constant
    let const_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            matches!(&def.kind, DefKind::Const(_)) && def.ident.name == "x"
        })
        .expect("Constant x not found");

    let const_def = result.context.definitions.get(*const_id);
    let DefKind::Const(const_ty) = &const_def.kind else {
        panic!("Expected const");
    };

    // Check that the type points to the actual struct definition, not forward decl
    if let TyKind::Sequence { ty, .. } = &const_ty.ty.kind {
        if let TyKind::Adt(struct_id) = &ty.kind {
            let struct_def = result.context.definitions.get(*struct_id);
            // The definition should be a struct, not a forward declaration
            assert!(
                matches!(&struct_def.kind, DefKind::Struct(_)),
                "Sequence element type should point to struct definition, not forward declaration"
            );
        } else {
            panic!("Expected ADT type for sequence element");
        }
    } else {
        panic!("Expected sequence type");
    }
}

#[test]
fn test_const_array_forward_ref_update() {
    let input = r"
        // Constant array using forward-declared type
        const Foo x[2] = { {value: 10}, {value: 20} };
        
        // Actual definition
        struct Foo {
            long value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Find the constant
    let const_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            matches!(&def.kind, DefKind::Const(_)) && def.ident.name == "x"
        })
        .expect("Constant x not found");

    let const_def = result.context.definitions.get(*const_id);
    let DefKind::Const(const_ty) = &const_def.kind else {
        panic!("Expected const");
    };

    // Check that the array element type points to the actual struct
    if let TyKind::Array { ty, .. } = &const_ty.ty.kind {
        if let TyKind::Adt(struct_id) = &ty.kind {
            let struct_def = result.context.definitions.get(*struct_id);
            assert!(
                matches!(&struct_def.kind, DefKind::Struct(_)),
                "Array element type should point to struct definition, not forward declaration"
            );
        }
    }

    // Check that the struct initializers in the value also point to the actual struct
    if let Numeric::Array { values, .. } = &const_ty.value {
        for value in values {
            if let Numeric::Struct { ty, .. } = value {
                let struct_def = result.context.definitions.get(*ty);
                assert!(
                    matches!(&struct_def.kind, DefKind::Struct(_)),
                    "Struct initializer should point to struct definition, not forward declaration"
                );
            }
        }
    }
}

#[test]
fn test_const_map_forward_ref_update() {
    let input = r#"
        // Constant map using forward-declared types
        const map<string, Bar> m = { { "key", {x: 42} } };
        
        // Actual definition
        struct Bar {
            long x;
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Find the constant
    let const_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            matches!(&def.kind, DefKind::Const(_)) && def.ident.name == "m"
        })
        .expect("Constant m not found");

    let const_def = result.context.definitions.get(*const_id);
    let DefKind::Const(const_ty) = &const_def.kind else {
        panic!("Expected const");
    };

    // Check that the map value type points to the actual struct
    if let TyKind::Map { elem, .. } = &const_ty.ty.kind {
        if let TyKind::Adt(struct_id) = &elem.kind {
            let struct_def = result.context.definitions.get(*struct_id);
            assert!(
                matches!(&struct_def.kind, DefKind::Struct(_)),
                "Map value type should point to struct definition, not forward declaration"
            );
        }
    }

    // Check that struct initializers in the map entries also point to actual struct
    if let Numeric::Map { entries, .. } = &const_ty.value {
        for (_, value) in entries {
            if let Numeric::Struct { ty, .. } = value {
                let struct_def = result.context.definitions.get(*ty);
                assert!(
                    matches!(&struct_def.kind, DefKind::Struct(_)),
                    "Map entry struct should point to struct definition, not forward declaration"
                );
            }
        }
    }
}

#[test]
fn test_const_ref_chain_forward_update() {
    let input = r"
        // Constant referencing another constant that uses forward-declared type
        const Baz DEFAULT_BAZ = { value: DEFAULT_FOO };
        const Foo DEFAULT_FOO = { x: 0 };
        
        struct Foo {
            long x;
        };
        
        struct Baz {
            Foo value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Both constants should resolve to actual struct definitions
    for const_name in &["DEFAULT_BAZ", "DEFAULT_FOO"] {
        let const_id = result
            .order
            .iter()
            .find(|&&def_id| {
                let def = result.context.definitions.get(def_id);
                matches!(&def.kind, DefKind::Const(_)) && def.ident.name == *const_name
            })
            .unwrap_or_else(|| panic!("Constant {const_name} not found"));

        let const_def = result.context.definitions.get(*const_id);
        if let DefKind::Const(c) = &const_def.kind {
            // Check that any struct references in the value point to definitions
            if let Numeric::Struct { ty, .. } = &c.value {
                let struct_def = result.context.definitions.get(*ty);
                assert!(
                    matches!(&struct_def.kind, DefKind::Struct(_)),
                    "Constant {const_name} should reference struct definition, not forward \
                     declaration"
                );
            }
        }
    }
}
