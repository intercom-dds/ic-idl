use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_lowercase_true_false() {
    let source = r"
        const boolean LOWERCASE_TRUE = true;
        const boolean LOWERCASE_FALSE = false;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_mixed_case_booleans() {
    let source = r"
        const boolean MIXED_1 = True;
        const boolean MIXED_2 = False;
        const boolean MIXED_3 = tRuE;
        const boolean MIXED_4 = fAlSe;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_uppercase_booleans_no_warning() {
    let source = r"
        const boolean UPPER_TRUE = TRUE;
        const boolean UPPER_FALSE = FALSE;
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for uppercase TRUE/FALSE"
    );
}

#[test]
fn test_boolean_in_struct_default() {
    let source = r"
        struct Config {
            boolean enabled = true;
            boolean verbose = false;
            boolean strict = TRUE;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_boolean_in_expressions() {
    let source = r"
        const boolean EXPR1 = true && FALSE;
        const boolean EXPR2 = TRUE || false;
        const boolean EXPR3 = !true;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_boolean_in_annotation() {
    let source = r"
        @annotation Feature(boolean enabled = true) {
            boolean experimental = false;
        };
    ";

    assert_snapshot!(test_lint(source));
}
