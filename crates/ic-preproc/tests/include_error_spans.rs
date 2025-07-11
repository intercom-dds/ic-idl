use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

#[test]
fn test_include_error_span_highlights_path() {
    let mut vfs = SourceMap::default();
    
    // Test case 1: Simple missing file with quotes
    let source = r#"
#include "missing_file.idl"
int main();
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for missing include file");
    
    // Find the error for the missing file
    let include_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("file"))
    }).expect("Expected to find file-related error");
    
    // Check that the error span highlights the string literal, not the #include keyword
    if let ic_preproc::Error::Syntax { span, .. } = include_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(error_text.contains("missing_file.idl"), 
            "Error span should highlight the file path '{}', not the directive", error_text);
        assert!(!error_text.contains("#include"), 
            "Error span should not include the #include directive");
    }
}

#[test]
fn test_include_error_span_system_includes() {
    let mut vfs = SourceMap::default();
    
    // Test case 2: System include with angle brackets
    let source = r#"
#include <system/missing.h>
int main();
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for missing system include");
    
    // Find the error for the missing file
    let include_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("file"))
    }).expect("Expected to find file-related error");
    
    // Check that the error span highlights the path inside angle brackets
    if let ic_preproc::Error::Syntax { span, .. } = include_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(error_text.contains("system/missing.h"), 
            "Error span should highlight the file path '{}', not the directive", error_text);
        assert!(!error_text.contains("#include"), 
            "Error span should not include the #include directive");
    }
}

#[test]
fn test_nested_include_error_span() {
    let mut vfs = SourceMap::default();
    
    // Test case 3: Deeply nested includes error
    let source = r#"
#include "deeply/nested/file.idl"
"#;
    
    let file_id = vfs.embed(source);
    let args = ProcArgs::default().recursion_depth(0); // Set depth to 0 to trigger the error immediately
    let mut state = ic_preproc::State::new();
    let _iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);
    
    // Consume all tokens to process directives
    for _ in _iter {}
    
    // Check that we have an error
    let errors = state.errors();
    let nested_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Syntax { message, .. } if message.contains("nested"))
    });
    
    if let Some(ic_preproc::Error::Syntax { span, .. }) = nested_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(error_text.contains("deeply/nested/file.idl"), 
            "Error span should highlight the file path '{}', not the directive", error_text);
    }
}