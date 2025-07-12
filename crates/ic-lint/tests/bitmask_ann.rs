use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_bitmask_in_annotation() {
    let source = r"
        @annotation MyAnnotation {
            bitmask<unsigned long> Flags {
                FLAG_A = 0x01,
                FLAG_B = 0x02,
                FLAG_C = 0x04
            };
            unsigned long default_flags;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_normal_bitmask_outside_annotation() {
    let source = r"
        bitmask<unsigned long> GlobalFlags {
            ENABLED = 0x01,
            VERBOSE = 0x02,
            DEBUG = 0x04
        };
        
        @annotation Settings {
            unsigned long flags;
        };
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for bitmasks outside annotations"
    );
}

#[test]
fn test_nested_bitmask_in_annotation() {
    let source = r"
        @annotation ComplexAnnotation {
            struct Config {
                string name;
            };
            
            bitmask<octet> Options {
                OPT_A = 1,
                OPT_B = 2
            };
            
            bitmask<unsigned short> MoreOptions {
                MORE_A = 0x10,
                MORE_B = 0x20
            };
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_empty_annotation() {
    let source = r"
        @annotation EmptyAnnotation {
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for empty annotations");
}
