//! Tests for the complete `DefId` rewrite visitor.

mod common;

use std::collections::HashMap;

use ic_hir::hir::{DefId, DefKind};
use ic_hir::rewrite::replace_all_def_ids_in_def;

#[test]
fn replaces_ids_in_module_definition_lists() {
    let input = r"
module M {
    struct Foo {
        long x;
    };
};
";
    let graph = common::parse_and_resolve_successfully(input);
    let mut context = graph.context;

    let module_id = context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "M")
        .map(|(id, _)| id)
        .unwrap();
    let foo_id = context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Foo")
        .map(|(id, _)| id)
        .unwrap();

    let target = DefId::from(usize::from(foo_id) + 1000);
    let mapping = HashMap::from([(foo_id, target)]);

    replace_all_def_ids_in_def(&mut context, module_id, &mapping);

    let DefKind::Module(module) = &context.definitions.get(module_id).kind else {
        panic!("expected module");
    };
    assert!(module.definitions.contains(&target));
    assert!(!module.definitions.contains(&foo_id));
}

#[test]
fn replaces_ids_in_enum_fields() {
    let input = r"
enum Color {
    RED,
    GREEN
};
";
    let graph = common::parse_and_resolve_successfully(input);
    let mut context = graph.context;

    let enum_id = context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Color")
        .map(|(id, _)| id)
        .unwrap();
    let red_id = context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "RED")
        .map(|(id, _)| id)
        .unwrap();

    let target = DefId::from(usize::from(red_id) + 1000);
    let mapping = HashMap::from([(red_id, target)]);

    replace_all_def_ids_in_def(&mut context, enum_id, &mapping);

    let DefKind::Enum(e) = &context.definitions.get(enum_id).kind else {
        panic!("expected enum");
    };
    assert!(e.fields.contains(&target));
}

#[test]
fn replaces_ids_in_def_level_annotations() {
    let input = r"
@annotation vendor {
    long value default 0;
};

@vendor(value = 1)
struct Foo {
    long x;
};
";
    let graph = common::parse_and_resolve_successfully(input);
    let mut context = graph.context;

    let vendor_id = context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "vendor" && matches!(def.kind, DefKind::Annotation(_)))
        .map(|(id, _)| id)
        .unwrap();
    let foo_id = context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Foo")
        .map(|(id, _)| id)
        .unwrap();

    let target = DefId::from(usize::from(vendor_id) + 1000);
    let mapping = HashMap::from([(vendor_id, target)]);

    replace_all_def_ids_in_def(&mut context, foo_id, &mapping);

    let ann = context
        .definitions
        .get(foo_id)
        .annotations
        .iter()
        .find(|ann| ann.ident.name == "vendor")
        .unwrap();
    assert_eq!(ann.def_id, Some(target));
}
