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

mod common;

use ic_hir::hir::{DefKind, TyKind};

#[test]
fn test_any_type_resolution() {
    let idl = r"
        // Test that 'any' is properly resolved as TyKind::Any
        struct TestAny {
            any simple_field;
            sequence<any> seq_field;
            map<string, any> map_field;
        };

        typedef any MyAny;

        interface TestInterface {
            any getValue();
            void setValue(in any value);
        };
    ";

    let (hir, _source_map, diagnostics) = common::parse_and_resolve(idl);
    assert!(
        diagnostics.is_empty(),
        "Unexpected diagnostics: {diagnostics}"
    );

    // Find the TestAny struct
    let test_any = hir
        .iter()
        .find(|def| def.ident.name == "TestAny")
        .expect("TestAny struct not found");

    if let DefKind::Struct(s) = &test_any.kind {
        // Check simple_field
        let simple_field = &s.members[0];
        assert_eq!(simple_field.ident.name, "simple_field");
        assert!(matches!(simple_field.ty.kind, TyKind::Any));

        // Check seq_field
        let seq_field = &s.members[1];
        assert_eq!(seq_field.ident.name, "seq_field");
        if let TyKind::Sequence { ty, .. } = &seq_field.ty.kind {
            assert!(matches!(ty.kind, TyKind::Any));
        } else {
            panic!("Expected sequence type for seq_field");
        }

        // Check map_field
        let map_field = &s.members[2];
        assert_eq!(map_field.ident.name, "map_field");
        if let TyKind::Map { elem, .. } = &map_field.ty.kind {
            assert!(matches!(elem.kind, TyKind::Any));
        } else {
            panic!("Expected map type for map_field");
        }
    } else {
        panic!("TestAny is not a struct");
    }

    // Check typedef
    let my_any = hir
        .iter()
        .find(|def| def.ident.name == "MyAny")
        .expect("MyAny typedef not found");

    if let DefKind::Alias(a) = &my_any.kind {
        assert!(matches!(a.ty.kind, TyKind::Any));
    } else {
        panic!("MyAny is not an alias");
    }

    // Check interface methods
    let test_interface = hir
        .iter()
        .find(|def| def.ident.name == "TestInterface")
        .expect("TestInterface not found");

    if let DefKind::Interface(i) = &test_interface.kind {
        // Check getValue return type
        let get_value = &i.prototypes[0];
        assert_eq!(get_value.ident.name, "getValue");
        assert!(matches!(get_value.ty.kind, TyKind::Any));

        // Check setValue parameter type
        let set_value = &i.prototypes[1];
        assert_eq!(set_value.ident.name, "setValue");
        assert_eq!(set_value.params.len(), 1);
        assert!(matches!(set_value.params[0].ty.kind, TyKind::Any));
    } else {
        panic!("TestInterface is not an interface");
    }
}
