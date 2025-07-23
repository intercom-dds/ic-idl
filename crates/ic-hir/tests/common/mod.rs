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

//! Common test utilities for HIR tests

use ic_hir::{AstInput, ResolvedGraph};
use ic_vfs::SourceMap;

/// Parse IDL input and return the HIR along with rendered diagnostics
pub fn parse_and_resolve(input: &str) -> (ResolvedGraph, SourceMap, String) {
    let mut source_map = SourceMap::default();
    let file = source_map.embed_with_name("test.idl", input);
    let parsed = ic_parse::from_file(file, ic_preproc::ProcArgs::default(), &mut source_map);

    // Check parse errors
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    // Parse built-in annotations (same as ic-idl does)
    let builtin_file_id = source_map.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(
        builtin_file_id,
        ic_preproc::ProcArgs::default(),
        &mut source_map,
    );

    let result = ic_hir::lower(AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: parsed.tree,
        include_in_output: false,
    });

    // Render all diagnostics (errors and warnings)
    let mut output = String::new();

    // Render errors
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }

    // Render warnings (if any)
    for warning in &result.warnings {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, warning).unwrap();
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }

    // Remove trailing newline if present
    if output.ends_with('\n') {
        output.pop();
    }

    (result, source_map, output)
}

/// Parse IDL input, expecting it to succeed without errors
#[allow(dead_code)]
pub fn parse_and_resolve_successfully(input: &str) -> ResolvedGraph {
    let (result, _, diagnostics) = parse_and_resolve(input);

    assert!(
        result.errors.is_empty(),
        "Expected no errors but got:\n{diagnostics}"
    );

    result
}

/// Parse IDL input, expecting it to fail with errors
#[allow(dead_code)]
pub fn parse_and_expect_errors(input: &str) -> String {
    let (result, _, diagnostics) = parse_and_resolve(input);

    assert!(!result.errors.is_empty(), "Expected errors but got none");

    diagnostics
}

/// Parse IDL input and return the result with warnings (for testing warning cases)
#[allow(dead_code)]
pub fn parse_and_get_warnings(input: &str) -> (ResolvedGraph, String) {
    let (result, _, diagnostics) = parse_and_resolve(input);

    assert!(
        result.errors.is_empty(),
        "Expected no errors but got:\n{diagnostics}"
    );

    assert!(
        !result.warnings.is_empty(),
        "Expected warnings but got none"
    );

    (result, diagnostics)
}

/// Parse IDL input and return only the diagnostics output (for snapshot testing)
#[allow(dead_code)]
pub fn compile_idl_with_warnings(input: &str) -> String {
    let (_, _, diagnostics) = parse_and_resolve(input);
    diagnostics
}


/// Parse IDL with custom builtins (for testing builtin behavior)
#[allow(dead_code)]
pub fn parse_with_custom_builtins(
    builtins: &str,
    user: &str,
    include_builtins_in_output: bool,
) -> (ResolvedGraph, SourceMap, String) {
    let mut source_map = SourceMap::default();
    
    let builtin_file = source_map.embed_with_name("<builtin>", builtins);
    let builtin_parsed = ic_parse::from_file(
        builtin_file,
        ic_preproc::ProcArgs::default(),
        &mut source_map,
    );
    
    let user_file = source_map.embed_with_name("test.idl", user);
    let user_parsed = ic_parse::from_file(
        user_file,
        ic_preproc::ProcArgs::default(),
        &mut source_map,
    );

    assert!(
        builtin_parsed.errors.is_empty(),
        "Builtin parse errors: {:?}",
        builtin_parsed.errors
    );
    assert!(
        user_parsed.errors.is_empty(),
        "User parse errors: {:?}",
        user_parsed.errors
    );

    let result = ic_hir::lower(AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: user_parsed.tree,
        include_in_output: include_builtins_in_output,
    });

    // Render all diagnostics
    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }
    for warning in &result.warnings {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, warning).unwrap();
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }
    if output.ends_with('\n') {
        output.pop();
    }

    (result, source_map, output)
}
