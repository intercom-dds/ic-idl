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

use ic_hir::hir::DefKind;
use ic_idl::hir;
use ic_lint::LintConfig;
use ic_parse::SourceMap;
use ic_preproc::ProcArgs;

/// Helper function to convert AST to HIR with built-in annotations injected
fn ast_to_hir_with_builtins(
    ast: Vec<ic_syntax::Item>,
    source_map: &mut SourceMap,
    lint_config: &LintConfig,
) -> Result<hir::ResolvedGraph, ic_idl::CompileError> {
    // Add built-in annotations to the source map
    let builtin_file_id = source_map.embed_with_name(
        "<builtin-annotations>",
        include_str!("../idl/annotations.idl"),
    );

    // Parse the built-in annotations
    let builtin_parsed = ic_parse::from_file(builtin_file_id, ProcArgs::default(), source_map);
    assert!(
        builtin_parsed.errors.is_empty(),
        "Failed to parse built-in annotations"
    );

    // Use from_ast_with_builtins to properly handle built-in injection
    let mut hir = hir::from_ast_with_builtins(builtin_parsed.tree, ast);

    let mut all_warnings = Vec::new();
    let mut all_errors = Vec::new();

    // Take warnings and errors from HIR
    all_warnings.extend(std::mem::take(&mut hir.warnings));
    all_errors.extend(std::mem::take(&mut hir.errors));

    // Lint the HIR if no errors so far
    if all_errors.is_empty() {
        let report = ic_lint::lint_hir_with_config(&hir, source_map, lint_config);
        all_errors.extend(report.errors);
        all_warnings.extend(report.warnings);
    }

    if !all_errors.is_empty() {
        return Err(ic_idl::CompileError::Diagnostics(
            ic_idl::CompileDiagnostics {
                errors: all_errors.into_iter().map(Into::into).collect(),
                warnings: all_warnings,
                expansion_info: std::collections::HashMap::new(),
            },
        ));
    }

    // Put warnings back for tests that want to check them
    hir.warnings = all_warnings;
    Ok(hir)
}

#[test]
fn test_multiple_builtin_annotations() {
    let input = r#"
        // Test various built-in annotations
        @final
        struct ImmutableData {
            @key
            long id;
            
            @optional
            string description;
        };
        
        @mutable
        struct MutableData {
            @range(min=0, max=100)
            long percentage;
            
            @default(42)
            long answer;
        };
        
        enum Status {
            @value(0)
            UNKNOWN,
            @value(1)
            ACTIVE,
            @value(2)
            INACTIVE
        };
        
        @topic(name="events", namespace="com.example")
        struct Event {
            long timestamp;
            string data;
        };
    "#;

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);

    assert!(parsed.errors.is_empty());

    let hir_result = ast_to_hir_with_builtins(parsed.tree, &mut source_map, &LintConfig::default());

    // Should succeed with built-in annotations available
    assert!(
        hir_result.is_ok(),
        "HIR conversion should succeed with built-in annotations"
    );

    let hir = hir_result.unwrap();

    // Verify all user-defined types are present
    let types: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter(|(_, def)| !def.ident.name.starts_with("intercom"))
        .map(|(_, def)| &def.ident.name)
        .collect();

    assert!(types.contains(&&"ImmutableData".to_string()));
    assert!(types.contains(&&"MutableData".to_string()));
    assert!(types.contains(&&"Status".to_string()));
    assert!(types.contains(&&"Event".to_string()));
}

#[test]
fn test_extensibility_annotation_aliases() {
    // Test that extensibility annotation works (aliases were removed)
    let input = r"
        @extensibility
        struct S1 {};
        
        @appendable
        struct S2 {};
        
        @final
        struct S3 {};
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);

    assert!(parsed.errors.is_empty());

    let hir_result = ast_to_hir_with_builtins(parsed.tree, &mut source_map, &LintConfig::default());

    // Should succeed - enum aliases in built-in annotations are allowed
    assert!(
        hir_result.is_ok(),
        "HIR conversion should succeed with enum aliases"
    );
}

#[test]
fn test_unknown_annotation_warning() {
    let input = r"
        @unknown_annotation
        struct S {
            long field;
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);

    assert!(parsed.errors.is_empty());

    let hir_result = ast_to_hir_with_builtins(parsed.tree, &mut source_map, &LintConfig::default());

    // Should succeed with warnings
    let hir = match hir_result {
        Ok(hir) => hir,
        Err(e) => panic!("HIR conversion should not fail for unknown annotations: {e:?}"),
    };

    // Should have a warning about the unknown annotation
    assert!(
        !hir.warnings.is_empty(),
        "Should have warnings about unknown annotation"
    );
    assert!(
        hir.warnings
            .iter()
            .any(|w| { w.to_string().contains("unknown annotation") }),
        "Should have warning about unknown annotation"
    );

    // The struct should not have the unknown annotation
    let struct_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .expect("Should find struct S");

    if let DefKind::Struct(s) = &struct_def.kind {
        // Unknown annotations should be filtered out
        assert_eq!(
            s.members[0].annotations.len(),
            0,
            "Unknown annotations should be filtered out"
        );
    }
}
