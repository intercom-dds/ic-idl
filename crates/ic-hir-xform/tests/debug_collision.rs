mod common;

use ic_emit::case::Case;
use ic_hir::hir::DefKind;
use ic_hir_xform::{Target, rename, strip_common_suffixes};

/// Helper to create a minimal Rust-like target for testing
fn test_rust_target() -> Target {
    Target {
        struct_type: Some(Case::Pascal),
        module: Some(Case::Snake),
        name_preprocessor: Some(strip_common_suffixes),
        ..Default::default()
    }
}

#[test]
#[ignore = "debug test"]
fn debug_property_collision() {
    let idl = r#"
        module mod_collision {
            struct property_t {};
            module Property {};
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    println!("Before transformation:");
    // Look for mod_collision module and its contents
    for def in hir.iter() {
        println!(
            "  {} ({:?})",
            def.ident.name,
            match &def.kind {
                DefKind::Module(m) => {
                    let mut result = format!("Module with {} definitions", m.definitions.len());
                    // Print the children
                    for &child_id in &m.definitions {
                        let child = hir.context.type_of(child_id);
                        result.push_str(&format!(
                            "\n    - {} ({:?})",
                            child.ident.name,
                            match &child.kind {
                                DefKind::Module(_) => "Module",
                                DefKind::Struct(_) => "Struct",
                                _ => "Other",
                            }
                        ));
                    }
                    result
                }
                DefKind::Struct(_) => "Struct".to_string(),
                _ => "Other".to_string(),
            }
        );
    }

    let renamed = rename::transform(hir, test_rust_target());

    println!("\nAfter transformation:");
    for def in renamed.iter() {
        println!(
            "  {} ({:?})",
            def.ident.name,
            match &def.kind {
                DefKind::Module(m) => {
                    let mut result = format!("Module with {} definitions", m.definitions.len());
                    // Print the children
                    for &child_id in &m.definitions {
                        let child = renamed.context.type_of(child_id);
                        result.push_str(&format!(
                            "\n    - {} ({:?})",
                            child.ident.name,
                            match &child.kind {
                                DefKind::Module(_) => "Module",
                                DefKind::Struct(_) => "Struct",
                                _ => "Other",
                            }
                        ));
                    }
                    result
                }
                DefKind::Struct(_) => "Struct".to_string(),
                _ => "Other".to_string(),
            }
        );
    }
}
