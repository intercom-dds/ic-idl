mod common;

use ic_emit::case::Case;
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
    let idl = r"
        module mod_collision {
            struct property_t {};
            module Property {};
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let renamed = rename::transform(hir, &test_rust_target());

    // Verify transformation succeeded
    assert!(renamed.iter().any(|def| def.ident.name == "mod_collision"));
}
