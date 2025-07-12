use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_scoped_enum_literal() {
    let source = r"
        enum Color {
            RED,
            GREEN,
            BLUE
        };
        
        const Color DEFAULT_COLOR = Color::RED;
        const Color SECONDARY = Color::GREEN;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_scoped_bitmask_literal() {
    let source = r"
        bitmask<unsigned long> Permissions {
            READ = 0x01,
            WRITE = 0x02,
            EXECUTE = 0x04
        };
        
        const Permissions DEFAULT_PERMS = Permissions::READ | Permissions::WRITE;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_unscoped_literals() {
    let source = r"
        enum Status {
            OK,
            ERROR,
            PENDING
        };
        
        const Status GOOD = OK;
        const Status BAD = ERROR;
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for unscoped literals");
}

#[test]
fn test_mixed_scoped_unscoped() {
    let source = r"
        enum Mode {
            NORMAL,
            FAST,
            SLOW
        };
        
        const Mode MODE1 = NORMAL;
        const Mode MODE2 = Mode::FAST;
        const Mode MODE3 = SLOW;
        const Mode MODE4 = Mode::SLOW;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_nested_scoped_access() {
    let source = r"
        module Types {
            enum Result {
                SUCCESS,
                FAILURE
            };
        };
        
        const Types::Result GOOD = Types::Result::SUCCESS;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_scoped_in_expressions() {
    let source = r"
        enum Level {
            LOW = 1,
            MEDIUM = 5,
            HIGH = 10
        };
        
        const boolean IS_HIGH = (Level::HIGH > Level::MEDIUM);
        const Level NEXT = (Level::LOW + 1);
    ";

    assert_snapshot!(test_lint(source));
}
