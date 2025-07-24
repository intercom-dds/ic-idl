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

use ic_diagnostic::Level;
use ic_lint::{Category, LintConfig, Report};
use ic_vfs::SourceMap;

#[allow(dead_code)]
pub fn test_lint(source: &str) -> String {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);

    // Parse the IDL code
    let args = ic_preproc::ProcArgs::default();
    let ast = ic_parse::from_file(file_id, args, &mut vfs);

    // Assert no parse errors in test code
    assert!(
        ast.errors.is_empty(),
        "Parse errors in test code: {:?}",
        ast.errors
    );

    // Configure lint to enable pedantic warnings
    let mut config = LintConfig::new();
    config.set_category_level(Category::Pedantic, Level::Warning);

    // Run lints
    let report = ic_lint::lint_syntax_with_config(&ast.tree, &vfs, &config);

    // Format all diagnostics
    let mut output = String::new();

    // Emit errors
    for (i, diag) in report.errors.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        ic_diagnostic::emit_with_source(&mut output, "test.idl", source, diag)
            .expect("Failed to format diagnostic");
    }

    // Emit warnings
    for (i, diag) in report.warnings.iter().enumerate() {
        if i > 0 || !report.errors.is_empty() {
            output.push('\n');
        }
        ic_diagnostic::emit_with_source(&mut output, "test.idl", source, diag)
            .expect("Failed to format diagnostic");
    }

    output
}

#[allow(dead_code)]
pub fn lint_hir(source: &str) -> Report {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);

    // Parse the IDL code
    let args = ic_preproc::ProcArgs::default();
    let ast = ic_parse::from_file(file_id, args, &mut vfs);

    // Parse built-in annotations (same as ic-idl does)
    let builtin_file_id = vfs.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed =
        ic_parse::from_file(builtin_file_id, ic_preproc::ProcArgs::default(), &mut vfs);

    // Lower to HIR with built-ins
    let hir = ic_hir::from_ast(ic_hir::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: ast.tree,
        include_in_output: false,
    });

    // Configure lint to enable semantic errors
    let mut config = LintConfig::new();
    config.set_category_level(Category::Semantic, Level::Error);

    // Run HIR lints
    let mut report = ic_lint::lint_hir_with_config(&hir, &vfs, &config);

    // Add any HIR errors to the report
    report.errors.extend(hir.errors);

    report
}

#[allow(dead_code)]
pub fn test_lint_hir(source: &str) -> String {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(source);

    // Parse the IDL code
    let args = ic_preproc::ProcArgs::default();
    let ast = ic_parse::from_file(file_id, args, &mut vfs);

    // Parse built-in annotations (same as ic-idl does)
    let builtin_file_id = vfs.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed =
        ic_parse::from_file(builtin_file_id, ic_preproc::ProcArgs::default(), &mut vfs);

    // Assert no parse errors in test code
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

    // Lower to HIR with built-ins
    let hir = ic_hir::from_ast(ic_hir::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: ast.tree,
        include_in_output: false,
    });

    // Assert that we have either definitions or errors
    assert!(
        !hir.order.is_empty() || !hir.errors.is_empty(),
        "HIR has no definitions and no errors were reported. This indicates a bug in parsing or \
         HIR construction."
    );

    // Configure lint to enable semantic errors, pedantic warnings, and annotation warnings
    let mut config = LintConfig::new();
    config.set_category_level(Category::Semantic, Level::Error);
    config.set_category_level(Category::Pedantic, Level::Warning);
    config.set_category_level(Category::Annotation, Level::Warning);

    // Run HIR lints
    let mut report = ic_lint::lint_hir_with_config(&hir, &vfs, &config);

    // Add any HIR errors to the report
    report.errors.extend(hir.errors);

    // Format all diagnostics
    let mut output = String::new();

    // Emit errors
    for (i, diag) in report.errors.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        ic_diagnostic::emit_with_source(&mut output, "test.idl", source, diag)
            .expect("Failed to format diagnostic");
    }

    // Emit warnings
    for (i, diag) in report.warnings.iter().enumerate() {
        if i > 0 || !report.errors.is_empty() {
            output.push('\n');
        }
        ic_diagnostic::emit_with_source(&mut output, "test.idl", source, diag)
            .expect("Failed to format diagnostic");
    }

    output
}
