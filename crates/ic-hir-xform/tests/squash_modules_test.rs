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

use ic_hir::hir::DefKind;
use ic_hir_xform::squash_modules;

#[test]
fn test_simple_module_squashing() {
    let idl = r#"
        module A {
            struct Foo {
                long x;
            };
        };
        
        module A {
            struct Bar {
                string y;
            };
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let transformed = squash_modules::transform(hir);

    // There should be only one module A in the order list
    let module_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A")
        .count();
    assert_eq!(
        module_count, 1,
        "Should have exactly one module A after squashing"
    );

    // Find module A and its ID
    let (module_a_id, module_a) = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A")
        .expect("Module A should exist");

    // Check that module A contains both structs
    if let DefKind::Module(module_ty) = &module_a.kind {
        assert_eq!(
            module_ty.definitions.len(),
            2,
            "Module A should contain 2 definitions"
        );

        let mut struct_names: Vec<_> = module_ty
            .definitions
            .iter()
            .map(|&id| transformed.context.type_of(id).ident.name.clone())
            .collect();
        struct_names.sort();

        assert_eq!(struct_names, vec!["Bar", "Foo"]);

        // Verify that both structs have module A as their parent
        for &def_id in &module_ty.definitions {
            let def = transformed.context.type_of(def_id);
            assert_eq!(def.parent, Some(module_a_id));
        }
    } else {
        panic!("Module A should be a module");
    }
}

#[test]
fn test_nested_module_squashing() {
    let idl = r#"
        module A {
            module B {
                struct Foo {};
            };
        };
        
        module A {
            module B {
                struct Bar {};
            };
            
            struct Baz {};
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let transformed = squash_modules::transform(hir);

    // There should be only one module A in the order
    let module_a_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A")
        .count();
    assert_eq!(module_a_count, 1, "Should have exactly one module A");

    // Module B is nested so won't appear in top-level order
    // Check through module A's contents instead

    // Find module A from the order list
    let module_a = transformed
        .iter()
        .find(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A")
        .expect("Module A should exist");

    // Get the module A def ID by finding it in the arena
    let _module_a_id = transformed
        .order
        .iter()
        .find(|&&id| {
            let def = transformed.context.type_of(id);
            matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A"
        })
        .copied()
        .expect("Module A should be in order");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // Debug: print what's in module A
        eprintln!("Module A contains:");
        for &id in &module_a_ty.definitions {
            let def = transformed.context.type_of(id);
            eprintln!(
                "  - {} ({})",
                def.ident.name,
                match &def.kind {
                    DefKind::Module(_) => "module",
                    DefKind::Struct(_) => "struct",
                    _ => "other",
                }
            );
        }

        // Module A should contain module B and struct Baz
        assert_eq!(
            module_a_ty.definitions.len(),
            2,
            "Module A should contain 2 definitions"
        );

        // Find module B within A
        let module_b_id = module_a_ty
            .definitions
            .iter()
            .find(|&&id| {
                let def = transformed.context.type_of(id);
                matches!(def.kind, DefKind::Module(_)) && def.ident.name == "B"
            })
            .expect("Module B should be in module A");

        let module_b = transformed.context.type_of(*module_b_id);
        if let DefKind::Module(module_b_ty) = &module_b.kind {
            // Module B should contain both Foo and Bar
            assert_eq!(
                module_b_ty.definitions.len(),
                2,
                "Module B should contain 2 definitions"
            );

            let mut struct_names: Vec<_> = module_b_ty
                .definitions
                .iter()
                .map(|&id| transformed.context.type_of(id).ident.name.clone())
                .collect();
            struct_names.sort();

            assert_eq!(struct_names, vec!["Bar", "Foo"]);
        }
    }
}

#[test]
fn test_multiple_reopened_modules() {
    let idl = r#"
        module A {
            struct One {};
        };
        
        module A {
            struct Two {};
        };
        
        module A {
            struct Three {};
        };
        
        module B {
            struct Four {};
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let transformed = squash_modules::transform(hir);

    // There should be only one module A and one module B in the order list
    let module_a_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A")
        .count();
    assert_eq!(module_a_count, 1, "Should have exactly one module A");

    let module_b_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "B")
        .count();
    assert_eq!(module_b_count, 1, "Should have exactly one module B");

    // Check module A contents
    let (_, module_a) = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_ty) = &module_a.kind {
        assert_eq!(
            module_ty.definitions.len(),
            3,
            "Module A should contain 3 definitions"
        );

        let mut struct_names: Vec<_> = module_ty
            .definitions
            .iter()
            .map(|&id| transformed.context.type_of(id).ident.name.clone())
            .collect();
        struct_names.sort();

        assert_eq!(struct_names, vec!["One", "Three", "Two"]);
    }
}

#[test]
fn test_preserve_single_modules() {
    let idl = r#"
        module A {
            struct Foo {};
            struct Bar {};
        };
        
        module B {
            struct Baz {};
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let original_module_count = hir
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Module(_)))
        .count();

    let transformed = squash_modules::transform(hir);

    let transformed_module_count = transformed
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Module(_)))
        .count();

    assert_eq!(
        original_module_count, transformed_module_count,
        "Module count should remain the same when no modules are re-opened"
    );
}
