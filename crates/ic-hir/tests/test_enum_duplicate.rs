mod common;

#[test]
fn test_enum_duplicate_detection() {
    let input = r#"
enum MyEnum {
    ZERO
};

const octet ZERO = 0;
"#;

    let (result, _, diagnostics) = common::parse_and_resolve(input);

    // Should have an error about duplicate definition
    assert!(
        !result.errors.is_empty(),
        "Expected duplicate definition error"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_enum_no_duplicate_different_case() {
    let input = r#"
enum MyEnum {
    ZERO
};

const octet zero = 0;
"#;

    let (result, _, diagnostics) = common::parse_and_resolve(input);

    // Should have an error because IDL is case-insensitive
    assert!(
        !result.errors.is_empty(),
        "Expected duplicate definition error for case-insensitive match"
    );

    // Snapshot test the error message
    insta::assert_snapshot!(diagnostics);
}
