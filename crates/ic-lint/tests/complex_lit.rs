use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_struct_initializer_in_const() {
    let source = r"
        struct Point {
            double x;
            double y;
        };
        
        const Point ORIGIN = { x: 0.0, y: 0.0 };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_array_initializer_in_const() {
    let source = r"
        const long PRIMES[5] = { 2, 3, 5, 7, 11 };
        const double MATRIX[2][2] = { {1.0, 0.0}, {0.0, 1.0} };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_initializer_in_annotation() {
    let source = r#"
        struct Config {
            string name;
            long value;
        };
        
        @annotation Settings(Config default_config = { name: "default", value: 42 }) {
            boolean enabled;
        };
    "#;

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_simple_literals_allowed() {
    let source = r#"
        const long NUM = 42;
        const string TEXT = "hello";
        const boolean FLAG = TRUE;
        const double PI = 3.14159;
    "#;

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for simple literals");
}

#[test]
fn test_nested_initializers() {
    let source = r#"
        struct Inner {
            long a;
            long b;
        };
        
        struct Outer {
            Inner inner;
            string name;
        };
        
        const Outer NESTED = {
            inner: { a: 1, b: 2 },
            name: "test"
        };
    "#;

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_sequence_initializer() {
    let source = r"
        typedef sequence<long> LongSeq;
        const LongSeq NUMBERS = { 1, 2, 3, 4, 5 };
    ";

    assert_snapshot!(test_lint(source));
}
