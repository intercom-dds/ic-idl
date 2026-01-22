// Copyright 2025 KONGSBERG
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

//! Tests for the unused-include lint.

use std::path::PathBuf;

use ic_cli::color::ColorMode;
use ic_diagnostic::Level;
use ic_lint::{Category, LintConfig};
use ic_vfs::SourceMap;
use insta::assert_snapshot;

/// Parse source using the preprocessor (for tests that need include handling).
fn parse_with_preproc(
    file_id: ic_vfs::FileId,
    args: ic_preproc::ProcArgs,
    vfs: &mut SourceMap,
) -> ic_parse::ParseResult {
    let mut state = ic_preproc::State::new();
    let iter = ic_preproc::with_state(file_id, args, &mut state, vfs);
    let tokens: Vec<_> = iter
        .filter(|t| !matches!(t.kind, ic_lexer::token::Kind::Newline))
        .collect();
    ic_parse::from_iter(tokens, vfs)
}

/// Helper function to test include-related lints with file-based includes.
fn test_lint_with_includes(main_source: &str) -> String {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut vfs = SourceMap::default();

    // Get the path to the test fixtures directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("tests/unused_include_fixtures");

    // Set up preprocessor args with the fixtures directory as an include path
    let args = ic_preproc::ProcArgs::default().includes([fixtures_dir]);

    let file_id = vfs.embed_with_name("test.idl", main_source);

    // Parse the IDL code (needs preprocessor for #include handling)
    let ast = parse_with_preproc(file_id, args, &mut vfs);

    // Assert no parse errors in test code
    assert!(
        ast.errors.is_empty(),
        "Parse errors in test code: {:?}",
        ast.errors
    );

    // Parse built-in annotations (no includes, simple parse)
    let builtin_file_id = vfs.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file_id, &vfs);

    assert!(
        builtin_parsed.errors.is_empty(),
        "Parse errors in builtin annotations: {:?}",
        builtin_parsed.errors
    );

    // Lower to HIR with built-ins
    let hir = ic_hir_lower::from_ast(ic_hir_lower::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: ast.tree,
        include_in_output: false,
    });

    // Configure lint to enable pedantic warnings
    let mut config = LintConfig::new();
    config.set_category_level(Category::Pedantic, Level::Warning);
    config.set_category_level(Category::Semantic, Level::Error);

    // Run HIR lints
    let report = ic_lint::lint_hir_with_config(&hir, &vfs, &config);

    // Format all diagnostics
    let mut output = String::new();
    let mut emitter = ic_diagnostic::DiagnosticEmitter::new();

    // Emit errors
    for (i, diag) in report.errors.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        emitter
            .emit_with_source(&mut output, "test.idl", main_source, diag)
            .expect("Failed to format diagnostic");
    }

    // Emit warnings
    for (i, diag) in report.warnings.iter().enumerate() {
        if i > 0 || !report.errors.is_empty() {
            output.push('\n');
        }
        emitter
            .emit_with_source(&mut output, "test.idl", main_source, diag)
            .expect("Failed to format diagnostic");
    }

    output
}

#[test]
fn test_no_includes_no_warnings() {
    // No includes at all - should not produce any warnings
    let source = r"
        struct LocalStruct {
            long value;
        };
    ";

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when there are no includes"
    );
}

#[test]
fn test_used_include_no_warning() {
    // Include a file and use a type from it - should not warn
    let source = r#"
        #include "used_types.idl"
        
        struct LocalStruct {
            UsedStruct base;
        };
    "#;

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when included types are used: {output}"
    );
}

#[test]
fn test_unused_include_warning() {
    // Include a file but don't use any types from it - should warn
    let source = r#"
        #include "unused_types.idl"
        
        struct LocalStruct {
            long value;
        };
    "#;

    assert_snapshot!(test_lint_with_includes(source));
}

#[test]
fn test_mixed_used_and_unused() {
    // Include both used and unused files
    let source = r#"
        #include "used_types.idl"
        #include "unused_types.idl"
        
        struct LocalStruct {
            UsedStruct base;
        };
    "#;

    assert_snapshot!(test_lint_with_includes(source));
}

#[test]
fn test_transitive_include_used() {
    // Include a file that transitively includes another file, use the transitive type
    let source = r#"
        #include "transitive_types.idl"
        
        struct LocalStruct {
            TransitiveStruct base;
        };
    "#;

    let output = test_lint_with_includes(source);
    // transitive_types.idl includes used_types.idl and uses UsedStruct,
    // so no warning should be issued
    assert!(
        output.is_empty(),
        "Should not warn when transitive includes are used: {output}"
    );
}

#[test]
fn test_empty_include_warning() {
    // Include an empty file - should warn
    let source = r#"
        #include "empty_file.idl"
        
        struct LocalStruct {
            long value;
        };
    "#;

    assert_snapshot!(test_lint_with_includes(source));
}

#[test]
fn test_typedef_from_include_used() {
    // Use a typedef from an included file
    let source = r#"
        #include "used_types.idl"
        
        struct LocalStruct {
            UsedAlias value;
        };
    "#;

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when typedef from include is used: {output}"
    );
}

#[test]
fn test_annotation_from_include_used() {
    // Include a file that defines an annotation and apply it
    let source = r#"
        #include "custom_annotation.idl"
        
        @CustomAnnotation(value = "test")
        struct LocalStruct {
            long value;
        };
    "#;

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when annotation from include is used: {output}"
    );
}

#[test]
fn test_unused_annotation_include_warning() {
    // Include a file that defines an annotation but don't use it
    let source = r#"
        #include "unused_annotation.idl"
        
        struct LocalStruct {
            long value;
        };
    "#;

    assert_snapshot!(test_lint_with_includes(source));
}

#[test]
fn test_struct_inheritance_from_include() {
    // Include a file providing a parent struct and inherit from it
    let source = r#"
        #include "parent_struct.idl"
        
        struct ChildStruct : ParentStruct {
            string name;
        };
    "#;

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when include is used via struct inheritance: {output}"
    );
}

#[test]
fn test_interface_inheritance_from_include() {
    // Include a file providing a parent interface and extend it
    let source = r#"
        #include "parent_interface.idl"
        
        interface ChildInterface : ParentInterface {
            void doSomethingElse();
        };
    "#;

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when include is used via interface inheritance: {output}"
    );
}

#[test]
fn test_valuetype_inheritance_from_include() {
    // Include a file providing a parent valuetype and inherit from it
    let source = r#"
        #include "parent_valuetype.idl"
        
        valuetype ChildValuetype : ParentValuetype {
            public string name;
        };
    "#;

    let output = test_lint_with_includes(source);
    assert!(
        output.is_empty(),
        "Should not warn when include is used via valuetype inheritance: {output}"
    );
}
