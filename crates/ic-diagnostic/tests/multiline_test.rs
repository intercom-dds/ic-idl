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
fn test_multiline_span() {
    // Test with a multi-line span
    let source = "interface MyInterface {\n    void myMethod(\n        int param1,\n        string param2\n    );\n}";
    
    // Create a diagnostic that spans multiple lines (from "myMethod" to the closing paren)
    let diag = Diag::error("method spans multiple lines")
        .label(
            Label::new(make_span(32, 85))  // This should span from "myMethod" to ")"
                .message("this method definition spans multiple lines")
                .color(Color::Red)
        );
    
    let mut buf = String::new();
    ic_diagnostic::emit_with_source(&mut buf, "test.idl", source, &diag).unwrap();
    println!("=== Test 1: Multi-line span ===");
    println!("{}", buf);
    
    // Test with multiple labels on different lines
    let diag2 = Diag::error("multiple parameters on different lines")
        .label(
            Label::new(make_span(47, 57))  // "int param1"
                .message("first parameter")
                .color(Color::Yellow)
        )
        .label(
            Label::new(make_span(67, 80))  // "string param2"
                .message("second parameter")
                .color(Color::Blue)
        );
    
    let mut buf2 = String::new();
    ic_diagnostic::emit_with_source(&mut buf2, "test.idl", source, &diag2).unwrap();
    println!("\n=== Test 2: Multiple labels on different lines ===");
    println!("{}", buf2);
    
    // Test showing current behavior - only first line is highlighted
    let diag3 = Diag::error("large multi-line block")
        .label(
            Label::new(make_span(23, 92))  // From opening brace to closing brace
                .message("entire method block")
                .color(Color::Green)
        );
    
    let mut buf3 = String::new();
    ic_diagnostic::emit_with_source(&mut buf3, "test.idl", source, &diag3).unwrap();
    println!("\n=== Test 3: Large multi-line block ===");
    println!("{}", buf3);
}