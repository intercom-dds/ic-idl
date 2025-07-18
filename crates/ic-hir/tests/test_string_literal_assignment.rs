mod common;

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn test_string_literal_assigned_to_string() {
    let input = r#"
        const string value = "hello world";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    // Verify the constant has the correct string value
    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind {
        match &const_ty.value {
            Numeric::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string value, got {:?}", const_ty.value),
        }
    } else {
        panic!("Expected const definition");
    }
}

#[test]
fn test_wide_string_literal_assigned_to_wstring() {
    let input = r#"
        const wstring value = "wide string";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    // Verify the constant has the correct string value
    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind {
        match &const_ty.value {
            Numeric::String(s) => assert_eq!(s, "wide string"),
            _ => panic!("Expected string value, got {:?}", const_ty.value),
        }
    } else {
        panic!("Expected const definition");
    }
}

#[test]
fn test_string_literal_with_escapes() {
    let input = r#"
        const string value = "hello\nworld\ttab";
    "#;

    let result = common::parse_and_resolve_successfully(input);
    assert_eq!(result.order.len(), 1);

    // Verify the constant has the correct string value with escapes
    let def = result.context.definitions.get(result.order[0]);
    if let DefKind::Const(const_ty) = &def.kind {
        match &const_ty.value {
            Numeric::String(s) => assert_eq!(s, "hello\\nworld\\ttab"),
            _ => panic!("Expected string value, got {:?}", const_ty.value),
        }
    } else {
        panic!("Expected const definition");
    }
}
