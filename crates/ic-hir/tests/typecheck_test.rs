// Test type checking phase

#[test]
fn test_string_assigned_to_int() {
    let input = r#"
        const string MY_STR = "foo";
        const int32 FOO = MY_STR;
    "#;

    let parsed = ic_parse::from_str(input);

    // Should parse successfully
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have a type error
    assert!(
        !result.errors.is_empty(),
        "Expected type error for string assigned to int, but got no errors"
    );
}

#[test]
fn test_int_overflow() {
    let input = r"
        const int8 SMALL = 256;  // Too large for int8
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have an overflow error
    assert!(
        !result.errors.is_empty(),
        "Expected overflow error for int8 = 256, but got no errors"
    );
}

#[test]
fn test_valid_constants() {
    let input = r"
        const int32 FOO = 42;
        const boolean FLAG = TRUE;
        const double PI = 3.14;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
#[ignore = "annotations not yet handled, can't set bit_bound"]
fn test_enum_value_overflow() {
    let input = r"
        enum SmallEnum {
            A = 100,
            B = 200  // Too large for int8
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have an overflow error
    assert!(
        !result.errors.is_empty(),
        "Expected overflow error for enum value 200 in int8, but got no errors"
    );
}

#[test]
fn test_union_case_type_mismatch() {
    let input = r#"
        union MyUnion switch (int32) {
            case "string":  // String literal for int32 discriminator
                string s;
            case 1:
                int32 i;
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have a type error
    assert!(
        !result.errors.is_empty(),
        "Expected type error for string case label with int32 discriminator, but got no errors"
    );
}
