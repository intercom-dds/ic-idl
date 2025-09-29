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

use ic_hir::hir::{DefKind, TyKind};

#[test]
fn test_const_sequence_forward_ref_update() {
    let input = r"
        // Forward declaration
        struct Foo;
        
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
        // Forward declaration
        struct Foo;
        
        // Constant array using forward-declared type
        const Foo x[2] = { { value = 10 }, { value = 20 } };
        
        // Actual definition
        struct Foo {
            long value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    // This should fail because struct Foo is not fully defined when we try to evaluate the constant
    assert!(
        !result.errors.is_empty(),
        "Expected error due to forward reference"
    );
}

#[test]
fn test_const_map_forward_ref() {
    let input = r#"
        const map<string, long> m = {};
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_const_ref_chain_forward_update() {
    let input = r"
        // Forward declarations
        struct Foo;
        struct Baz;
        
        // Constants using forward-declared types
        const Foo DEFAULT_FOO = { x = 0 };
        const Baz DEFAULT_BAZ = { value = DEFAULT_FOO };
        
        struct Foo {
            long x;
        };
        
        struct Baz {
            Foo value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(
        !result.errors.is_empty(),
        "Expected error due to forward reference in struct init"
    );
}
