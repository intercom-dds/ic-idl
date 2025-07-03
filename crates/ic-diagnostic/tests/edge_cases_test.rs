use ic_diagnostic::{Color, Diag, Label};
use ic_vfs::{FileId, Location, Span};

fn make_span(start: u32, end: u32) -> Span {
    let file_id = FileId::_do_not_use(); // For testing only
    Span {
        start: Location::new(start, file_id),
        end: Location::new(end, file_id),
    }
}

#[test]
fn test_overlapping_spans() {
    let source = "struct Data {\n    int field1;\n    int field2;\n    int field3;\n}";

    // Test overlapping spans
    let diag = Diag::error("overlapping spans")
        .label(
            Label::new(make_span(14, 40)) // From "int field1" to "field2"
                .message("first overlap")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(28, 54)) // From "int field2" to "field3"
                .message("second overlap")
                .color(Color::Yellow),
        );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Overlapping spans ===");
    println!("{}", buf);
}

#[test]
fn test_single_char_span() {
    let source = "int x = 5;";

    // Test single character span
    let diag = Diag::error("single character span").label(
        Label::new(make_span(4, 5)) // Just the 'x'
            .message("variable name")
            .color(Color::Blue),
    );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Single character span ===");
    println!("{}", buf);
}

#[test]
fn test_adjacent_spans() {
    let source = "int add(int a, int b) { return a + b; }";

    // Test adjacent spans with no gap
    let diag = Diag::error("adjacent spans")
        .label(
            Label::new(make_span(8, 13)) // "int a"
                .message("first parameter")
                .color(Color::Yellow),
        )
        .label(
            Label::new(make_span(15, 20)) // "int b"
                .message("second parameter")
                .color(Color::Green),
        );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Adjacent spans ===");
    println!("{}", buf);
}

#[test]
fn test_nested_spans() {
    let source = "void process(struct Data { int x; int y; } data);";

    // Test nested spans - one span inside another
    let diag = Diag::error("nested structures")
        .label(
            Label::new(make_span(13, 43)) // The entire struct definition
                .message("inline struct definition")
                .color(Color::Blue),
        )
        .label(
            Label::new(make_span(27, 33)) // "int x;"
                .message("first field")
                .color(Color::Yellow),
        )
        .label(
            Label::new(make_span(34, 40)) // "int y;"
                .message("second field")
                .color(Color::Green),
        );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Nested spans ===");
    println!("{}", buf);
}

#[test]
fn test_empty_lines() {
    let source = "interface Test {\n\n    void method();\n\n}";

    // Test span across empty lines
    let diag = Diag::error("span across empty lines").label(
        Label::new(make_span(16, 38)) // From start of empty line to end of method
            .message("includes empty lines")
            .color(Color::Red),
    );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Span across empty lines ===");
    println!("{}", buf);
}

#[test]
fn test_many_labels_single_line() {
    let source = "calculate(a, b, c, d, e, f, g);";

    // Test many labels on single line
    let diag = Diag::error("too many parameters")
        .label(
            Label::new(make_span(10, 11)) // 'a'
                .message("first")
                .color(Color::Red),
        )
        .label(
            Label::new(make_span(13, 14)) // 'b'
                .message("second")
                .color(Color::Yellow),
        )
        .label(
            Label::new(make_span(16, 17)) // 'c'
                .message("third")
                .color(Color::Green),
        )
        .label(
            Label::new(make_span(19, 20)) // 'd'
                .message("fourth")
                .color(Color::Blue),
        );

    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Many labels on single line ===");
    println!("{}", buf);
}
