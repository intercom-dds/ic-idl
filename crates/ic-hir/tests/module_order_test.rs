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

#[test]
fn test_only_top_level_items_in_order() {
    let idl = r"
        // Top-level struct
        struct TopLevelStruct {
            long x;
        };

        // Module containing nested definitions
        module TestModule {
            struct InsideModuleStruct {
                long y;
            };
            
            interface InsideModuleInterface {
                void method();
            };
            
            const long MODULE_CONST = 42;
            
            enum ModuleEnum {
                FIRST,
                SECOND
            };
        };

        // More top-level definitions
        interface TopLevelInterface {
            void anotherMethod();
        };

        const long TOP_LEVEL_CONST = 100;
        
        enum TopLevelEnum {
            A, B, C
        };
    ";

    let hir = common::parse_and_resolve_successfully(idl);

    // Check that order contains only top-level items
    let order_names: Vec<String> = hir
        .order
        .iter()
        .map(|&id| {
            let def = hir.context.definitions.get(id);
            def.ident.name.clone()
        })
        .collect();

    // Should contain only top-level items
    assert!(order_names.contains(&"TopLevelStruct".to_string()));
    assert!(order_names.contains(&"TestModule".to_string()));
    assert!(order_names.contains(&"TopLevelInterface".to_string()));
    assert!(order_names.contains(&"TOP_LEVEL_CONST".to_string()));
    assert!(order_names.contains(&"TopLevelEnum".to_string()));

    // Should NOT contain items defined inside the module
    assert!(!order_names.contains(&"InsideModuleStruct".to_string()));
    assert!(!order_names.contains(&"InsideModuleInterface".to_string()));
    assert!(!order_names.contains(&"MODULE_CONST".to_string()));
    assert!(!order_names.contains(&"ModuleEnum".to_string()));

    // Verify the exact count
    assert_eq!(
        order_names.len(),
        5,
        "Should have exactly 5 top-level items"
    );
}

#[test]
fn test_module_case_insensitive_reopening_with_order() {
    let idl = r"
        struct TopLevel1 {};
        
        module Foo {
            struct Bar {
                long x;
            };
        };
        
        struct TopLevel2 {};
        
        module FOO {  // Different case, same module
            struct Baz {
                long y;
            };
        };
        
        struct TopLevel3 {};
    ";

    let diagnostics = common::parse_and_expect_errors(idl);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_nested_modules_order() {
    let idl = r"
        module Outer {
            module Inner {
                struct DeepStruct {
                    long value;
                };
            };
            
            struct OuterStruct {
                long x;
            };
        };
        
        struct TopLevel {};
    ";

    let hir = common::parse_and_resolve_successfully(idl);

    // Check order - only top-level items should be in order
    let order_names: Vec<String> = hir
        .order
        .iter()
        .map(|&id| {
            let def = hir.context.definitions.get(id);
            def.ident.name.clone()
        })
        .collect();

    // Should only contain items defined at the root scope
    assert_eq!(order_names.len(), 2); // Only Outer and TopLevel
    assert!(order_names.contains(&"Outer".to_string()));
    assert!(order_names.contains(&"TopLevel".to_string()));

    // Should NOT contain anything defined inside modules
    assert!(!order_names.contains(&"Inner".to_string())); // Inner is inside Outer, not top-level
    assert!(!order_names.contains(&"DeepStruct".to_string()));
    assert!(!order_names.contains(&"OuterStruct".to_string()));
}

#[test]
fn test_type_alias_multiple_declarators_order() {
    let idl = r"
        typedef long TypeA, TypeB, TypeC;
        
        module M {
            typedef string ModuleTypeA, ModuleTypeB;
        };
    ";

    let hir = common::parse_and_resolve_successfully(idl);

    // Get order
    let order_items: Vec<_> = hir
        .order
        .iter()
        .map(|&id| {
            let def = hir.context.definitions.get(id);
            (def.ident.name.clone(), &def.kind)
        })
        .collect();

    // Count aliases at top level
    let top_level_aliases = order_items
        .iter()
        .filter(|(_, kind)| matches!(kind, DefKind::Alias(_)))
        .count();

    assert_eq!(top_level_aliases, 3); // TypeA, TypeB, TypeC

    // Module should be in order
    assert!(
        order_items
            .iter()
            .any(|(name, kind)| name == "M" && matches!(kind, DefKind::Module(_)))
    );

    // Module type aliases should NOT be in order
    assert!(!order_items.iter().any(|(name, _)| name == "ModuleTypeA"));
    assert!(!order_items.iter().any(|(name, _)| name == "ModuleTypeB"));
}
