mod common;

#[test]
fn test_int_literal_assigned_to_string() {
    let input = r"
        const string value = 123;
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_string_literal_assigned_to_int() {
    let input = r#"
        const int32 value = "hello";
    "#;

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_float_literal_assigned_to_string() {
    let input = r"
        const string value = 3.14;
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_bool_literal_assigned_to_string() {
    let input = r"
        const string value = TRUE;
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}
