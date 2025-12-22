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
use ic_hir_xform::move_constants::{self, CONSTANTS_MODULE_NAME};

/// Simple escape function that appends an underscore
fn escape_with_underscore(name: &str) -> String {
    format!("{name}_")
}

#[test]
fn test_top_level_constants() {
    let idl = r"
        const long X = 1;
        const long Y = 2;
        struct Foo { long x; };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Should have a Constants module at top level
    let constants_module = transformed
        .iter()
        .find(|def| def.ident.name == CONSTANTS_MODULE_NAME)
        .expect("Constants module should exist");

    if let DefKind::Module(module_ty) = &constants_module.kind {
        assert_eq!(module_ty.definitions.len(), 2, "Should have 2 constants");

        let const_names: Vec<_> = module_ty
            .definitions
            .iter()
            .map(|&id| transformed.context.type_of(id).ident.name.clone())
            .collect();
        assert!(const_names.contains(&"X".to_string()));
        assert!(const_names.contains(&"Y".to_string()));
    } else {
        panic!("Constants should be a module");
    }

    // Foo should still be at top level
    assert!(
        transformed.iter().any(|def| def.ident.name == "Foo"),
        "Foo should still be at top level"
    );
}

#[test]
fn test_constants_in_module() {
    let idl = r"
        module A {
            const long X = 1;
            const long Y = 2;
            struct Foo { long x; };
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // Should have Constants module and Foo
        assert_eq!(
            module_a_ty.definitions.len(),
            2,
            "Module A should have 2 definitions"
        );

        // Find Constants module inside A
        let constants_id = module_a_ty
            .definitions
            .iter()
            .find(|&&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME)
            .expect("Constants module should be in A");

        let constants_module = transformed.context.type_of(*constants_id);
        if let DefKind::Module(constants_ty) = &constants_module.kind {
            assert_eq!(constants_ty.definitions.len(), 2);
        } else {
            panic!("Constants should be a module");
        }
    } else {
        panic!("A should be a module");
    }
}

#[test]
fn test_nested_modules_each_get_constants() {
    let idl = r"
        module A {
            const long X = 1;
            
            module B {
                const long Y = 2;
                struct Bar { long y; };
            };
            
            struct Foo { long x; };
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // A should have: Constants, B, Foo
        assert_eq!(module_a_ty.definitions.len(), 3);

        // Find Constants in A
        let a_constants_id = module_a_ty
            .definitions
            .iter()
            .find(|&&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME)
            .expect("Constants module should be in A");

        let a_constants = transformed.context.type_of(*a_constants_id);
        if let DefKind::Module(a_constants_ty) = &a_constants.kind {
            assert_eq!(
                a_constants_ty.definitions.len(),
                1,
                "A::Constants should have X"
            );
            let const_name = transformed
                .context
                .type_of(a_constants_ty.definitions[0])
                .ident
                .name
                .clone();
            assert_eq!(const_name, "X");
        }

        // Find module B in A
        let module_b_id = module_a_ty
            .definitions
            .iter()
            .find(|&&id| transformed.context.type_of(id).ident.name == "B")
            .expect("Module B should be in A");

        let module_b = transformed.context.type_of(*module_b_id);
        if let DefKind::Module(module_b_ty) = &module_b.kind {
            // B should have: Constants, Bar
            assert_eq!(module_b_ty.definitions.len(), 2);

            // Find Constants in B
            let b_constants_id = module_b_ty
                .definitions
                .iter()
                .find(|&&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME)
                .expect("Constants module should be in B");

            let b_constants = transformed.context.type_of(*b_constants_id);
            if let DefKind::Module(b_constants_ty) = &b_constants.kind {
                assert_eq!(
                    b_constants_ty.definitions.len(),
                    1,
                    "B::Constants should have Y"
                );
                let const_name = transformed
                    .context
                    .type_of(b_constants_ty.definitions[0])
                    .ident
                    .name
                    .clone();
                assert_eq!(const_name, "Y");
            }
        }
    }
}

#[test]
fn test_no_constants_no_module() {
    let idl = r"
        module A {
            struct Foo { long x; };
            struct Bar { long y; };
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // Should NOT have a Constants module since there are no constants
        let has_constants = module_a_ty
            .definitions
            .iter()
            .any(|&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME);
        assert!(!has_constants, "Should not create empty Constants module");

        // Should still have Foo and Bar
        assert_eq!(module_a_ty.definitions.len(), 2);
    }
}

#[test]
fn test_enum_constants_not_moved() {
    let idl = r"
        module A {
            enum Color { RED, GREEN, BLUE };
            const long X = 1;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // Should have: Constants (with X), Color enum
        assert_eq!(module_a_ty.definitions.len(), 2);

        // Find Constants module
        let constants_id = module_a_ty
            .definitions
            .iter()
            .find(|&&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME)
            .expect("Constants module should exist");

        let constants_module = transformed.context.type_of(*constants_id);
        if let DefKind::Module(constants_ty) = &constants_module.kind {
            // Only X should be in Constants, not enum fields
            assert_eq!(constants_ty.definitions.len(), 1);
            let const_name = transformed
                .context
                .type_of(constants_ty.definitions[0])
                .ident
                .name
                .clone();
            assert_eq!(const_name, "X");
        }

        // Color enum should still be directly in A
        let has_color = module_a_ty
            .definitions
            .iter()
            .any(|&id| transformed.context.type_of(id).ident.name == "Color");
        assert!(has_color, "Color enum should still be in A");
    }
}

#[test]
fn test_constants_parent_updated() {
    let idl = r"
        module A {
            const long X = 1;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find the constant X
    let const_x = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "X")
        .map(|(_, def)| def)
        .expect("X should exist");

    // X's parent should be the Constants module, not A directly
    let parent_id = const_x.parent.expect("X should have a parent");
    let parent = transformed.context.type_of(parent_id);
    assert_eq!(
        parent.ident.name, CONSTANTS_MODULE_NAME,
        "X's parent should be Constants module"
    );
}

#[test]
fn test_bitmask_constants_not_moved() {
    let idl = r"
        module A {
            bitmask Flags { FLAG_A, FLAG_B };
            const long X = 1;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // Find Constants module
        let constants_id = module_a_ty
            .definitions
            .iter()
            .find(|&&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME)
            .expect("Constants module should exist");

        let constants_module = transformed.context.type_of(*constants_id);
        if let DefKind::Module(constants_ty) = &constants_module.kind {
            // Only X should be in Constants, not bitmask fields
            assert_eq!(constants_ty.definitions.len(), 1);
            let const_name = transformed
                .context
                .type_of(constants_ty.definitions[0])
                .ident
                .name
                .clone();
            assert_eq!(const_name, "X");
        }
    }
}

#[test]
fn test_multiple_top_level_modules() {
    let idl = r"
        const long GLOBAL = 0;
        
        module A {
            const long X = 1;
        };
        
        module B {
            const long Y = 2;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Top level should have: Constants (with GLOBAL), A, B
    let top_level_names: Vec<_> = transformed
        .iter()
        .map(|def| def.ident.name.clone())
        .collect();

    assert!(top_level_names.contains(&CONSTANTS_MODULE_NAME.to_string()));
    assert!(top_level_names.contains(&"A".to_string()));
    assert!(top_level_names.contains(&"B".to_string()));

    // Each module should have its own Constants
    for module_name in ["A", "B"] {
        let module = transformed
            .iter()
            .find(|def| def.ident.name == module_name)
            .unwrap();

        if let DefKind::Module(module_ty) = &module.kind {
            let has_constants = module_ty
                .definitions
                .iter()
                .any(|&id| transformed.context.type_of(id).ident.name == CONSTANTS_MODULE_NAME);
            assert!(has_constants, "{module_name} should have Constants module");
        }
    }
}

#[test]
fn test_existing_constants_module_merged() {
    let idl = r"
        module A {
            module Constants {
                const long EXISTING = 0;
            };
            const long NEW = 1;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        // Should only have Constants module (NEW merged into it)
        assert_eq!(module_a_ty.definitions.len(), 1);

        let constants_id = module_a_ty.definitions[0];
        let constants_module = transformed.context.type_of(constants_id);

        assert_eq!(constants_module.ident.name, CONSTANTS_MODULE_NAME);

        if let DefKind::Module(constants_ty) = &constants_module.kind {
            // Should have both EXISTING and NEW
            assert_eq!(constants_ty.definitions.len(), 2);

            let const_names: Vec<_> = constants_ty
                .definitions
                .iter()
                .map(|&id| transformed.context.type_of(id).ident.name.clone())
                .collect();
            assert!(const_names.contains(&"EXISTING".to_string()));
            assert!(const_names.contains(&"NEW".to_string()));
        } else {
            panic!("Constants should be a module");
        }
    }
}

#[test]
fn test_collision_renamed() {
    let idl = r"
        module A {
            module Constants {
                const long X = 0;
            };
            const long X = 1;  // Same name - should be escaped
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        let constants_id = module_a_ty.definitions[0];
        let constants_module = transformed.context.type_of(constants_id);

        if let DefKind::Module(constants_ty) = &constants_module.kind {
            // Should have both X and X_
            assert_eq!(constants_ty.definitions.len(), 2);

            let const_names: Vec<_> = constants_ty
                .definitions
                .iter()
                .map(|&id| transformed.context.type_of(id).ident.name.clone())
                .collect();
            assert!(const_names.contains(&"X".to_string()));
            assert!(const_names.contains(&"X_".to_string()));
        } else {
            panic!("Constants should be a module");
        }
    }
}

#[test]
fn test_multiple_collisions_renamed() {
    let idl = r"
        module A {
            module Constants {
                const long X = 0;
                const long X_ = 1;  // Already escaped once
            };
            const long X = 2;  // Should become X__
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = move_constants::transform(hir, escape_with_underscore);

    // Find module A
    let module_a = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_a_ty) = &module_a.kind {
        let constants_id = module_a_ty.definitions[0];
        let constants_module = transformed.context.type_of(constants_id);

        if let DefKind::Module(constants_ty) = &constants_module.kind {
            // Should have X, X_, and X__
            assert_eq!(constants_ty.definitions.len(), 3);

            let const_names: Vec<_> = constants_ty
                .definitions
                .iter()
                .map(|&id| transformed.context.type_of(id).ident.name.clone())
                .collect();
            assert!(const_names.contains(&"X".to_string()));
            assert!(const_names.contains(&"X_".to_string()));
            assert!(const_names.contains(&"X__".to_string()));
        } else {
            panic!("Constants should be a module");
        }
    }
}
