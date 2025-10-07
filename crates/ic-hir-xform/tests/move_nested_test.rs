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
use ic_hir_xform::move_nested;

#[test]
fn test_move_types_from_interface() {
    let idl = r#"
        interface MyInterface {
            struct NestedStruct {
                long value;
            };
            
            enum NestedEnum {
                FIRST,
                SECOND
            };
            
            string getName();
            void setName(string name);
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let (transformed, moved_defs) = move_nested::transform(hir);

    // Find the interface
    let interface = transformed
        .iter()
        .find(|def| matches!(def.kind, DefKind::Interface(_)) && def.ident.name == "MyInterface")
        .expect("Interface should exist");

    // Check that interface no longer has nested definitions
    if let DefKind::Interface(interface_ty) = &interface.kind {
        assert!(
            interface_ty.definitions.is_empty(),
            "Interface should not have nested definitions after transformation"
        );
    }

    // Find the new module
    let module = transformed
        .iter()
        .find(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "MyInterface")
        .expect("Module with interface name should exist");

    // Check module contains the extracted types
    if let DefKind::Module(module_ty) = &module.kind {
        assert_eq!(
            module_ty.definitions.len(),
            2,
            "Module should contain 2 extracted types"
        );

        // Check that extracted types have correct parent
        for &def_id in &module_ty.definitions {
            let def = transformed.context.type_of(def_id);
            assert_eq!(
                def.parent,
                Some(module.id),
                "Extracted type should have module as parent"
            );
            assert!(
                moved_defs.contains(&def_id),
                "Extracted type should be marked as moved"
            );
        }
    }
}

#[test]
fn test_move_types_from_valuetype() {
    let idl = r#"
        valuetype MyValueType {
            struct Inner {
                string name;
            };
            
            public long x;
            private float y;
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let (transformed, moved_defs) = move_nested::transform(hir);

    // Find the valuetype
    let valuetype = transformed
        .iter()
        .find(|def| matches!(def.kind, DefKind::Valuetype(_)) && def.ident.name == "MyValueType")
        .expect("Valuetype should exist");

    // Check that valuetype no longer has nested definitions
    if let DefKind::Valuetype(valuetype_ty) = &valuetype.kind {
        assert!(
            valuetype_ty.definitions.is_empty(),
            "Valuetype should not have nested definitions after transformation"
        );
        // Members should still be there
        assert_eq!(
            valuetype_ty.members.len(),
            2,
            "Valuetype should still have its members"
        );
    }

    // Find the new module
    let module = transformed
        .iter()
        .find(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "MyValueType")
        .expect("Module with valuetype name should exist");

    // Check module contains the extracted type
    if let DefKind::Module(module_ty) = &module.kind {
        assert_eq!(
            module_ty.definitions.len(),
            1,
            "Module should contain 1 extracted type"
        );

        let inner_struct_id = module_ty.definitions[0];
        let inner_struct = transformed.context.type_of(inner_struct_id);
        assert_eq!(inner_struct.ident.name, "Inner");
        assert_eq!(
            inner_struct.parent,
            Some(module.id),
            "Inner struct should have module as parent"
        );
        assert!(
            moved_defs.contains(&inner_struct_id),
            "Inner struct should be marked as moved"
        );
    }
}

#[test]
fn test_nested_interface_in_module() {
    let idl = r#"
        module MyModule {
            interface MyInterface {
                struct Data {
                    long id;
                };
            };
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let (transformed, moved_defs) = move_nested::transform(hir);

    // Find MyModule
    let my_module = transformed
        .iter()
        .find(|def| matches!(def.kind, DefKind::Module(_)) && def.ident.name == "MyModule")
        .expect("MyModule should exist");

    if let DefKind::Module(module_ty) = &my_module.kind {
        // Should contain interface and the new module for extracted types
        assert_eq!(
            module_ty.definitions.len(),
            2,
            "MyModule should contain interface and extracted types module"
        );
    }

    // Find the extracted types module inside MyModule
    let all_modules: Vec<_> = transformed
        .context
        .definitions
        .iter()
        .filter(|(_, def)| {
            matches!(def.kind, DefKind::Module(_))
                && def.ident.name == "MyInterface"
                && def.parent == Some(my_module.id)
        })
        .collect();

    assert_eq!(
        all_modules.len(),
        1,
        "Should have exactly one MyInterface module inside MyModule"
    );
}

#[test]
fn test_no_nested_types() {
    let idl = r#"
        interface SimpleInterface {
            void doSomething();
        };
        
        valuetype SimpleValue {
            public long value;
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let original_count = hir.context.definitions.len();
    let (transformed, moved_defs) = move_nested::transform(hir);

    assert!(
        moved_defs.is_empty(),
        "No types should be moved when there are no nested types"
    );

    // No new modules should be created
    assert_eq!(
        transformed.context.definitions.len(),
        original_count,
        "No new definitions should be created"
    );
}
