use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

#[test]
fn test_expression_error_uses_directive_span_as_context() {
    let mut vfs = SourceMap::default();
    
    // Test that when expression parsing fails, we use the directive span as context
    let source = r#"
#if
int should_not_be_processed;
#endif
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for empty expression");
    
    // Debug print errors
    for error in errors {
        match error {
            ic_preproc::Error::Expr { message, span } => {
                println!("Expr error: {} (span: {:?})", message, span);
            }
            ic_preproc::Error::Syntax { message, span } => {
                println!("Syntax error: {} at {:?}", message, span);
            }
            _ => {}
        }
    }
    
    // When there's no expression after #if, we get a syntax error "expected value in expression"
    let syntax_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("expected value in expression"))
    }).expect("Expected to find syntax error for missing expression");
    
    // The error span should be after the #if
    if let ic_preproc::Error::Syntax { span, .. } = syntax_error {
        // The span points to the newline after #if
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert_eq!(error_text, "\n", 
            "Error span should point to the newline after 'if', got '{:?}'", error_text);
    }
}

#[test]
fn test_elif_expression_error_uses_elif_span() {
    let mut vfs = SourceMap::default();
    
    // Test that elif expression errors use the elif span
    let source = r#"
#if 0
int a;
#elif
int b;
#endif
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for empty elif expression");
    
    // Debug print errors
    for error in errors {
        match error {
            ic_preproc::Error::Expr { message, span } => {
                println!("Expr error: {} (span: {:?})", message, span);
            }
            ic_preproc::Error::Syntax { message, span } => {
                println!("Syntax error: {} at {:?}", message, span);
            }
            _ => {}
        }
    }
    
    // Similar to #if, we should get a syntax error for missing expression
    let syntax_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("expected value in expression"))
    }).expect("Expected to find syntax error for missing expression");
    
    // The error span should be after the #elif
    if let ic_preproc::Error::Syntax { span, .. } = syntax_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert_eq!(error_text, "\n", 
            "Error span should point to the newline after 'elif', got '{:?}'", error_text);
    }
}

#[test]
fn test_nested_expression_error_still_has_context() {
    let mut vfs = SourceMap::default();
    
    // Test that even nested expression errors have context
    let source = r#"
#if (1 + (2 * 
int should_not_be_processed;
#endif
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have errors
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected errors for incomplete expression");
    
    // All expression errors should have spans now
    let all_expr_errors_have_spans = errors.iter().all(|e| {
        match e {
            ic_preproc::Error::Expr { span, .. } => span.is_some(),
            _ => true,
        }
    });
    assert!(all_expr_errors_have_spans, "All expression errors should have spans");
}

#[test]
fn test_unexpected_end_in_defined_operator() {
    let mut vfs = SourceMap::default();
    
    // Test that when we have unexpected end in defined operator, we use proper span
    let source = r#"
#if defined
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for incomplete defined operator");
    
    // Find the expression error
    let expr_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Expr { message, .. } if message.contains("unexpected end after 'defined'"))
    });
    
    if let Some(ic_preproc::Error::Expr { span: Some(s), .. }) = expr_error {
        let error_text = &vfs.source_str(s.start.file_id)[s.range()];
        assert_eq!(error_text, "defined", 
            "Error span should point to 'defined', got '{}'", error_text);
    }
}