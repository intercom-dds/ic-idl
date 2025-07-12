use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_null_union_variant() {
    let source = r"
        union OptionalValue switch(long) {
            case 0: null;
            case 1: long int_value;
            case 2: string str_value;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_multiple_null_variants() {
    let source = r"
        union MultiNull switch(short) {
            case 0: null;
            case 1: long value;
            case 2: null;
            default: string text;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_union_without_null() {
    let source = r"
        union StandardUnion switch(long) {
            case 1: long number;
            case 2: string text;
            case 3: boolean flag;
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for unions without null");
}

#[test]
fn test_null_with_default() {
    let source = r"
        union DefaultNull switch(octet) {
            case 1: string name;
            case 2: long count;
            default: null;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_nested_union_with_null() {
    let source = r"
        union Inner switch(boolean) {
            case TRUE: long value;
            case FALSE: null;
        };
        
        union Outer switch(long) {
            case 1: Inner inner;
            case 2: null;
        };
    ";

    assert_snapshot!(test_lint(source));
}
