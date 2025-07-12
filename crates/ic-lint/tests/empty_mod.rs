use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_empty_module() {
    let source = r"
        module EmptyModule {
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_multiple_empty_modules() {
    let source = r"
        module Empty1 {
        };
        
        module Empty2 {
        };
        
        module Empty3 {
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_non_empty_module() {
    let source = r"
        module DataTypes {
            struct Point {
                double x;
                double y;
            };
            
            typedef sequence<Point> PointList;
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for non-empty modules");
}

#[test]
fn test_nested_empty_modules() {
    let source = r"
        module Outer {
            module InnerEmpty {
            };
            
            struct Data {
                long value;
            };
        };
        
        module AnotherEmpty {
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_module_with_only_comments() {
    let source = r"
        module DocumentedButEmpty {
            // This module is reserved for future use
            // TODO: Add types here
        };
    ";

    assert_snapshot!(test_lint(source));
}
