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
use ic_hir_xform::flatten;
use ic_hir_xform::rename::{self, Convention, Target, strip_common_suffixes};

fn test_rust_target() -> Target {
    Target {
        convention: Convention {
            struct_type: Some(Case::Pascal),
            module: Some(Case::Snake),
            name_preprocessor: Some(strip_common_suffixes),
            ..Default::default()
        },
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
    for def in &renamed {
        if let DefKind::Module(m) = &def.kind
            && def.ident.name == "test"
        {
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
    for def in &renamed {
        if let DefKind::Module(m) = &def.kind
            && def.ident.name == "test"
        {
            // Look at children of the test module
            for &child_id in &m.definitions {
                let child = renamed.context.type_of(child_id);
                if let DefKind::Struct(_) = &child.kind {
                    struct_names.push(child.ident.name.clone());
                }
            }
        }
    }

    struct_names.sort();

    assert_eq!(struct_names, vec!["MyStruct", "MyStruct_"]);
}

#[test]
fn moved_nested_module_does_not_take_interface_name() {
    let idl = r"
        module qux {
            interface quux {
                struct zork {};
            };
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let (hir, moved_defs) = ic_hir_xform::move_nested::transform(hir);
    let hir = ic_hir_xform::squash_modules::transform(hir);
    let mut target = test_rust_target();
    target.convention.interface = Some(Case::Pascal);
    target.moved_defs = moved_defs;
    let renamed = rename::transform(hir, &target);

    let interface = renamed
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| matches!(def.kind, DefKind::Interface(_)).then_some(def))
        .unwrap();

    assert_eq!(interface.ident.name, "Quux");
}

#[test]
fn forward_declaration_and_definition_share_moved_name() {
    let idl = r"
        module outer {
            struct Value;
            struct Value {
                long data;
            };
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let flattened = flatten::transform(hir, "_");
    let target = Target {
        moved_defs: flattened.moved_defs,
        ..Default::default()
    };
    let renamed = rename::transform(flattened.hir, &target);

    let names: Vec<_> = renamed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Decl(_) | DefKind::Struct(_)))
        .map(|def| def.ident.name.as_str())
        .collect();

    assert_eq!(names, vec!["outer_Value", "outer_Value"]);
}

#[test]
fn collision_after_suffix_removal() {
    let idl = r"
        module mod_collision {
            struct property_t {};
            module Property {};
        };
    ";

    let target = Target {
        convention: Convention {
            struct_type: Some(Case::Pascal),
            module: Some(Case::Snake),
            name_preprocessor: Some(strip_common_suffixes),
            ..Default::default()
        },
        ..Default::default()
    };

    let hir = common::parse_and_resolve(idl);
    let renamed = rename::transform(hir, &target);

    // Verify transformation succeeded
    assert!(renamed.iter().any(|def| def.ident.name == "mod_collision"));
}
