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

fn case_insensitive(mut target: Target) -> Target {
    target.case_insensitive = true;
    target
}

fn module_children(hir: &ic_hir::ResolvedGraph, module: &str) -> Vec<(String, &'static str)> {
    let m = hir
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Module(m) if def.ident.name == module => Some(m),
            _ => None,
        })
        .unwrap();

    m.definitions
        .iter()
        .map(|&id| {
            let child = hir.context.type_of(id);
            let kind = match child.kind {
                DefKind::Struct(_) => "struct",
                DefKind::Module(_) => "module",
                _ => "other",
            };
            (child.ident.name.clone(), kind)
        })
        .collect()
}

fn struct_members(hir: &ic_hir::ResolvedGraph, name: &str) -> Vec<String> {
    hir.context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Struct(s) if def.ident.name == name => {
                Some(s.members.iter().map(|m| m.ident.name.clone()).collect())
            }
            _ => None,
        })
        .unwrap()
}

#[test]
fn case_insensitive_definitions_collide_across_case() {
    let idl = r"
        module test {
            struct property_t {};
            module Property {};
        };
    ";

    let sensitive = rename::transform(common::parse_and_resolve(idl), &test_rust_target());
    assert_eq!(
        module_children(&sensitive, "test"),
        vec![
            ("Property".to_string(), "struct"),
            ("property".to_string(), "module"),
        ]
    );

    let insensitive = rename::transform(
        common::parse_and_resolve(idl),
        &case_insensitive(test_rust_target()),
    );
    assert_eq!(
        module_children(&insensitive, "test"),
        vec![
            ("Property_".to_string(), "struct"),
            ("property".to_string(), "module"),
        ]
    );
}

#[test]
fn case_insensitive_case_only_rename_is_not_escaped() {
    let idl = r"
        module test {
            struct value {
                long Count;
            };
        };
    ";

    let mut target = case_insensitive(test_rust_target());
    target.convention.member = Some(Case::Snake);
    let renamed = rename::transform(common::parse_and_resolve(idl), &target);

    assert_eq!(
        module_children(&renamed, "test"),
        vec![("Value".to_string(), "struct")]
    );
    assert_eq!(struct_members(&renamed, "Value"), vec!["count"]);
}

#[test]
fn case_insensitive_keeps_existing_definition_name() {
    let idl = r"
        module test {
            struct Value {};
            module value_t {};
        };
    ";

    let mut target = case_insensitive(test_rust_target());
    target.convention.module = None;
    let renamed = rename::transform(common::parse_and_resolve(idl), &target);

    assert_eq!(
        module_children(&renamed, "test"),
        vec![
            ("Value".to_string(), "struct"),
            ("value_".to_string(), "module"),
        ]
    );
}

#[test]
fn case_insensitive_struct_members_collide_across_case() {
    let idl = r"
        module test {
            struct Holder {
                long value_t;
                long Value;
            };
        };
    ";

    let target = Target {
        convention: Convention {
            name_preprocessor: Some(strip_common_suffixes),
            ..Default::default()
        },
        ..Default::default()
    };

    let sensitive = rename::transform(common::parse_and_resolve(idl), &target);
    assert_eq!(struct_members(&sensitive, "Holder"), vec!["value", "Value"]);

    let insensitive = rename::transform(common::parse_and_resolve(idl), &case_insensitive(target));
    assert_eq!(
        struct_members(&insensitive, "Holder"),
        vec!["value_", "Value"]
    );
}

#[test]
fn case_insensitive_interface_operations_and_attributes_share_names() {
    let idl = r"
        module test {
            interface Api {
                void get_value();
                attribute long getvalue;
            };
        };
    ";

    let target = Target {
        convention: Convention {
            operation: Some(Case::Pascal),
            attribute: Some(Case::Snake),
            ..Default::default()
        },
        ..Default::default()
    };

    let names = |hir: &ic_hir::ResolvedGraph| {
        hir.context
            .definitions
            .iter()
            .find_map(|(_, def)| match &def.kind {
                DefKind::Interface(i) => Some((
                    i.prototypes[0].ident.name.clone(),
                    i.attributes[0].ident.name.clone(),
                )),
                _ => None,
            })
            .unwrap()
    };

    let sensitive = rename::transform(common::parse_and_resolve(idl), &target);
    assert_eq!(
        names(&sensitive),
        ("GetValue".to_string(), "getvalue".to_string())
    );

    let insensitive = rename::transform(common::parse_and_resolve(idl), &case_insensitive(target));
    assert_eq!(
        names(&insensitive),
        ("GetValue_".to_string(), "getvalue".to_string())
    );
}

#[test]
fn case_insensitive_unrenamed_definition_reserves_its_name() {
    let idl = r"
        module test {
            typedef long SOMEVALUE;
            struct some_value {};
        };
    ";

    let target = Target {
        case_insensitive: true,
        convention: Convention {
            struct_type: Some(Case::Pascal),
            ..Default::default()
        },
        ..Default::default()
    };
    let renamed = rename::transform(common::parse_and_resolve(idl), &target);

    assert_eq!(
        module_children(&renamed, "test"),
        vec![
            ("SOMEVALUE".to_string(), "other"),
            ("SomeValue_".to_string(), "struct"),
        ]
    );
}

#[test]
fn case_insensitive_case_only_rename_wins_regardless_of_order() {
    let target = Target {
        case_insensitive: true,
        convention: Convention {
            struct_type: Some(Case::Pascal),
            ..Default::default()
        },
        ..Default::default()
    };

    let forward = rename::transform(
        common::parse_and_resolve("module test { struct foo_bar {}; struct foobar {}; };"),
        &target,
    );
    assert_eq!(
        module_children(&forward, "test"),
        vec![
            ("FooBar_".to_string(), "struct"),
            ("Foobar".to_string(), "struct"),
        ]
    );

    let reversed = rename::transform(
        common::parse_and_resolve("module test { struct foobar {}; struct foo_bar {}; };"),
        &target,
    );
    assert_eq!(
        module_children(&reversed, "test"),
        vec![
            ("Foobar".to_string(), "struct"),
            ("FooBar_".to_string(), "struct"),
        ]
    );
}

#[test]
fn vacated_name_is_not_reassigned_after_being_claimed() {
    let idl = r"
        module test {
            typedef long foo_t_t;
            module foo_t {};
            struct foo_t_e {};
        };
    ";

    let target = Target {
        convention: Convention {
            name_preprocessor: Some(strip_common_suffixes),
            ..Default::default()
        },
        ..Default::default()
    };

    let sensitive = rename::transform(common::parse_and_resolve(idl), &target);
    assert_eq!(
        module_children(&sensitive, "test"),
        vec![
            ("foo_t".to_string(), "other"),
            ("foo".to_string(), "module"),
            ("foo_t_".to_string(), "struct"),
        ]
    );

    let mut target = case_insensitive(target);
    target.convention.struct_type = Some(Case::UpperSnake);
    let insensitive = rename::transform(common::parse_and_resolve(idl), &target);
    assert_eq!(
        module_children(&insensitive, "test"),
        vec![
            ("foo_t".to_string(), "other"),
            ("foo".to_string(), "module"),
            ("FOO_T_".to_string(), "struct"),
        ]
    );
}
