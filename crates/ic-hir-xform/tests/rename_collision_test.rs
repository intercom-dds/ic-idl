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

use ic_emit::case::Case;
use ic_hir::hir::DefKind;
use ic_hir_xform::{Target, rename, strip_common_suffixes};

/// Helper to create a minimal Rust-like target for testing collisions
fn test_rust_target() -> Target {
    Target {
        struct_type: Some(Case::Pascal),
        module: Some(Case::Snake),
        name_preprocessor: Some(strip_common_suffixes),
        ..Default::default()
    }
}

#[test]
fn test_namespace_aware_collision_handling() {
    let idl = r#"
        module test {
            // Should become "Property" - types are PascalCase
            struct property_t {};
            
            // Should become "property" - modules are snake_case
            // This should NOT collide with the struct above
            module Property {};
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let renamed = rename::transform(hir, &test_rust_target());

    // Find the definitions
    let mut struct_name = None;
    let mut module_name = None;

    // Now look in the test module specifically
    for def in renamed.iter() {
        if let DefKind::Module(m) = &def.kind {
            if def.ident.name == "test" {
                // Look at children of the test module
                for &child_id in &m.definitions {
                    let child = renamed.context.type_of(child_id);
                    match &child.kind {
                        DefKind::Struct(_) => {
                            struct_name = Some(child.ident.name.clone());
                        }
                        DefKind::Module(_) => {
                            module_name = Some(child.ident.name.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    assert_eq!(
        struct_name,
        Some("Property".to_string()),
        "Struct should be Property, not Property_"
    );
    assert_eq!(
        module_name,
        Some("property".to_string()),
        "Module should be property"
    );
}

#[test]
fn test_same_namespace_collision() {
    let idl = r#"
        module test {
            // Should keep "MyStruct" since it's already PascalCase
            struct MyStruct {};
            
            // Should become "MyStruct_" due to collision
            struct my_struct {};
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let renamed = rename::transform(hir, &test_rust_target());

    let mut struct_names = Vec::new();

    // Look in the test module for structs
    for def in renamed.iter() {
        if let DefKind::Module(m) = &def.kind {
            if def.ident.name == "test" {
                // Look at children of the test module
                for &child_id in &m.definitions {
                    let child = renamed.context.type_of(child_id);
                    if let DefKind::Struct(_) = &child.kind {
                        struct_names.push(child.ident.name.clone());
                    }
                }
            }
        }
    }

    struct_names.sort();

    assert_eq!(struct_names, vec!["MyStruct", "MyStruct_"]);
}
