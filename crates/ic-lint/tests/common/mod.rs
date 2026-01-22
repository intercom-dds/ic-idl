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

use ic_cli::color::ColorMode;
use ic_diagnostic::Level;
use ic_lint::{Category, LintConfig, Report, SyntaxInput};
use ic_vfs::SourceMap;

fn format_report(source: &str, report: &Report) -> String {
    let mut output = String::new();
    let mut emitter = ic_diagnostic::DiagnosticEmitter::new();
    for (i, diag) in report.errors.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        emitter
            .emit_with_source(&mut output, "test.idl", source, diag)
            .expect("Failed to format diagnostic");
    }

    for (i, diag) in report.warnings.iter().enumerate() {
        if i > 0 || !report.errors.is_empty() {
            output.push('\n');
        }
        emitter
            .emit_with_source(&mut output, "test.idl", source, diag)
            .expect("Failed to format diagnostic");
    }

    output
}

#[allow(dead_code)]
pub fn test_lint(source: &str) -> String {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);
    let ast = ic_parse::from_file(file_id, &vfs);

    assert!(
        ast.errors.is_empty(),
        "Parse errors in test code: {:?}",
        ast.errors
    );

    let mut config = LintConfig::new();
    config.set_category_level(Category::Pedantic, Level::Warning);

    let input = SyntaxInput {
        tree: &ast.tree,
        orphaned_annotations: &ast.orphaned_annotations,
        preproc_warnings: &[],
        expansion_info: None,
    };
    let report = ic_lint::lint_syntax_with_config(&input, &vfs, &config);

    format_report(source, &report)
}

#[allow(dead_code)]
pub fn test_lint_preproc(source: &str) -> String {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);

    let mut state = ic_preproc::State::new();
    let tokens: Vec<_> = ic_preproc::with_state(
        file_id,
        ic_preproc::ProcArgs::default(),
        &mut state,
        &mut vfs,
    )
    .collect();
    let ast = ic_parse::from_iter(tokens, &vfs);

    let mut config = LintConfig::new();
    config.set_category_level(Category::Preprocessor, Level::Warning);

    let input = SyntaxInput {
        tree: &ast.tree,
        orphaned_annotations: &ast.orphaned_annotations,
        preproc_warnings: state.warnings(),
        expansion_info: Some(&state.expansion_info),
    };
    let report = ic_lint::lint_syntax_with_config(&input, &vfs, &config);

    format_report(source, &report)
}

#[allow(dead_code)]
pub fn lint_hir(source: &str) -> Report {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);
    let ast = ic_parse::from_file(file_id, &vfs);

    let builtin_file_id = vfs.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file_id, &vfs);

    let hir = ic_hir_lower::from_ast(ic_hir_lower::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: ast.tree,
        include_in_output: false,
    });

    let mut config = LintConfig::new();
    config.set_category_level(Category::Semantic, Level::Error);

    let mut report = ic_lint::lint_hir_with_config(&hir, &vfs, &config);
    report.errors.extend(hir.errors);

    report
}

#[allow(dead_code)]
pub fn test_lint_hir(source: &str) -> String {
    ic_cli::color::set_color_override(ColorMode::Never);
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);
    let ast = ic_parse::from_file(file_id, &vfs);

    let builtin_file_id = vfs.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file_id, &vfs);

    assert!(
        ast.errors.is_empty(),
        "Parse errors in test code: {:?}",
        ast.errors
    );
    assert!(
        builtin_parsed.errors.is_empty(),
        "Parse errors in builtin annotations: {:?}",
        builtin_parsed.errors
    );

    let hir = ic_hir_lower::from_ast(ic_hir_lower::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: ast.tree,
        include_in_output: false,
    });

    assert!(
        !hir.order.is_empty() || !hir.errors.is_empty(),
        "HIR has no definitions and no errors were reported"
    );

    let mut config = LintConfig::new();
    config.set_category_level(Category::Semantic, Level::Error);
    config.set_category_level(Category::Pedantic, Level::Warning);
    config.set_category_level(Category::Annotation, Level::Warning);
    config.set_category_level(Category::Unsupported, Level::Warning);

    let mut report = ic_lint::lint_hir_with_config(&hir, &vfs, &config);
    report.errors.extend(hir.errors);

    format_report(source, &report)
}
