// Copyright 2024 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

#[test]
fn test_division_by_zero_span() {
    let mut vfs = SourceMap::default();

    // Test division by zero
    let source = r"
#if 5 / 0
int should_not_be_processed;
#endif
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for division by zero");

    // Find the division by zero error
    let div_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Expr { message, .. } if message.contains("division by zero"))
    }).expect("Expected to find division by zero error");

    // Check that the error has a span pointing to an operand
    if let ic_preproc::Error::Expr { span, .. } = div_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        // The span should point to one of the operands
        assert!(
            error_text == "5" || error_text == "0",
            "Error span should point to an operand, got '{error_text}'"
        );
    }
}

#[test]
fn test_modulo_by_zero_span() {
    let mut vfs = SourceMap::default();

    // Test modulo by zero
    let source = r"
#if 10 % 0
int should_not_be_processed;
#endif
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for modulo by zero");

    // Find the modulo by zero error
    let mod_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Expr { message, .. } if message.contains("modulo by zero"))
    }).expect("Expected to find modulo by zero error");

    // The error should have a span
    if let ic_preproc::Error::Expr { span, .. } = mod_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        // The span should point to one of the operands
        assert!(
            error_text == "10" || error_text == "0",
            "Error span should point to an operand, got '{error_text}'"
        );
    }
}

#[test]
fn test_elif_after_else_span() {
    let mut vfs = SourceMap::default();

    // Test #elif after #else
    let source = r"
#if 0
int a;
#else
int b;
#elif 1
int c;
#endif
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for #elif after #else");

    // Find the error
    let elif_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Expr { message, .. } if message.contains("#elif after #else"))
    }).expect("Expected to find #elif after #else error");

    // Check that the error has a span
    if let ic_preproc::Error::Expr { span, .. } = elif_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(
            error_text == "elif",
            "Error span should point to 'elif', got '{error_text}'"
        );
    }
}

#[test]
fn test_else_after_else_span() {
    let mut vfs = SourceMap::default();

    // Test #else after #else
    let source = r"
#if 0
int a;
#else
int b;
#else
int c;
#endif
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(!errors.is_empty(), "Expected error for #else after #else");

    // Find the error
    let else_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Expr { message, .. } if message.contains("#else after #else"))
    }).expect("Expected to find #else after #else error");

    // Check that the error has a span
    if let ic_preproc::Error::Expr { span, .. } = else_error {
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        assert!(
            error_text == "else",
            "Error span should point to 'else', got '{error_text}'"
        );
    }
}

#[test]
fn test_unexpected_end_of_expression_span() {
    let mut vfs = SourceMap::default();

    // Test unexpected end in expression
    let source = r"
#if (1 + 2
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(
        !errors.is_empty(),
        "Expected error for unclosed parenthesis"
    );

    // Errors have been captured

    // All errors should have spans - either Syntax or Expr
    let all_have_spans = errors.iter().all(|e| {
        matches!(
            e,
            ic_preproc::Error::Expr { .. } | ic_preproc::Error::Syntax { .. }
        )
    });
    assert!(all_have_spans, "All errors should have proper spans");
}

#[test]
fn test_complex_expression_error_span() {
    let mut vfs = SourceMap::default();

    // Test complex expression with division by zero
    let source = r"
#if (1 + 2) * (3 / (2 - 2))
int should_not_be_processed;
#endif
";

    let file_id = vfs.embed(source);
    let args = ProcArgs::default();
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, &mut vfs);

    // Consume all tokens to process directives
    for _ in iter {}

    // Check that we have an error
    let errors = state.errors();
    assert!(
        !errors.is_empty(),
        "Expected error for division by zero in complex expression"
    );

    // Find the division by zero error
    let div_error = errors.iter().find(|e| {
        matches!(e, ic_preproc::Error::Expr { message, .. } if message.contains("division by zero"))
    }).expect("Expected to find division by zero error");

    // Check that the error has a span
    if let ic_preproc::Error::Expr { span, .. } = div_error {
        // All expression errors now have spans, no need to check
        let error_text = &vfs.source_str(span.start.file_id)[span.range()];
        // The span should point to one of the operands in the division
        assert!(
            error_text == "3" || error_text == "2" || error_text == "0",
            "Error span should point to an operand in the division, got '{error_text}'"
        );
    }
}
