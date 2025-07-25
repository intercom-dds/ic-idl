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

//! # ic-idl library
//!
//! This crate provides a library interface to the ic-idl compiler frontend,
//! allowing external users to parse and analyze IDL files.
//!
//! ## Example
//!
//! ```no_run
//! use ic_idl::{CompilerOptions, Compiler};
//! use std::path::PathBuf;
//!
//! // Example 1: Use the built-in compilation pipeline
//! let mut options = CompilerOptions::default();
//! options.files.push(PathBuf::from("example.idl"));
//!
//! let mut compiler = Compiler::new(options);
//! match compiler.compile() {
//!     Ok(compiled_ast) => {
//!         // Check for warnings
//!         if !compiled_ast.diagnostics.warnings.is_empty() {
//!             // Use the pretty module to format warnings
//!             let formatted = ic_idl::pretty::fmt_warnings(&compiled_ast.diagnostics.warnings, compiler.source_map());
//!             // User can print formatted warnings if desired
//!         }
//!
//!         // Now you can use the AST items for further processing
//!         // compiled_ast.items contains all parsed AST items
//!     }
//!     Err(ic_idl::CompileError::Diagnostics(diagnostics)) => {
//!         // Format errors and warnings using the pretty module
//!         let formatted_errors = ic_idl::pretty::fmt_errors(
//!             &diagnostics.errors,
//!             compiler.source_map(),
//!             &diagnostics.expansion_info
//!         );
//!         // User can print formatted errors if desired
//!     }
//!     Err(e) => {
//!         // Handle other errors as needed
//!     }
//! }
//!
//! // Example 2: Compile to HIR
//! let mut compiler = Compiler::new(CompilerOptions::default());
//! match compiler.compile_hir() {
//!     Ok((hir, diagnostics)) => {
//!         // Use the HIR for analysis or code generation
//!     }
//!     Err(e) => {
//!         // Handle errors
//!     }
//! }
//!
//! // Example 3: Parse a single file
//! let file_path = PathBuf::from("example.idl");
//! let mut source_map = ic_vfs::SourceMap::default();
//! let file_id = source_map.open(&file_path, ic_vfs::Include::Static).unwrap().0;
//! let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
//!
//! ```

use std::path::{Path, PathBuf};

use ic_diagnostic::Diag;
use ic_preproc::{ExpansionInfo, ProcArgs};
use ic_vfs::SourceMap;

// Import modules
mod builtin;
pub(crate) mod config;
pub mod pretty;
pub mod util;

pub use config::{
    CodegenOptions, CppOptions, IdlOptions, Options as CompilerOptions, PythonOptions, RustOptions,
    Unstable, Warnings,
};
use ic_cli::color::Colorize;
pub use ic_lint::{Category as LintCategory, Level as LintLevel, LintConfig};
pub use util::Error as DiagnosticError;
use util::Error as InternalError;

/// Error type for compilation failures.
#[derive(Debug)]
pub enum CompileError {
    /// I/O error (e.g., file not found).
    Io(std::io::Error),

    /// Parse or semantic analysis error with diagnostics.
    Diagnostics(CompileDiagnostics),
}

/// Diagnostics collected during compilation.
#[derive(Debug)]
pub struct CompileDiagnostics {
    /// Raw errors
    pub errors: Vec<DiagnosticError>,

    /// Warning diagnostics
    pub warnings: Vec<Diag>,

    /// Expansion info for macro expansion contexts
    pub expansion_info: std::collections::HashMap<ic_vfs::Span, ExpansionInfo>,
}

/// Result of AST compilation with all parsed items.
#[derive(Debug)]
pub struct CompiledAst {
    /// All parsed AST items from all files
    pub items: Vec<AstItem>,

    /// Diagnostics collected during parsing
    pub diagnostics: CompileDiagnostics,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(e) => write!(f, "{e}"),
            CompileError::Diagnostics(diag) => {
                write!(
                    f,
                    "{} errors, {} warnings",
                    diag.errors.len(),
                    diag.warnings.len()
                )
            }
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e)
    }
}

impl CompileDiagnostics {
    /// Check if there are any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the total count of diagnostics.
    #[must_use]
    pub fn count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Format all diagnostics for display.
    #[must_use]
    pub fn format(&self, source_map: &SourceMap) -> String {
        let mut result = String::new();

        if !self.warnings.is_empty() {
            result.push_str(&pretty::fmt_warnings(&self.warnings, source_map));
            if !self.errors.is_empty() {
                result.push('\n');
            }
        }

        if !self.errors.is_empty() {
            result.push_str(&pretty::fmt_errors(
                &self.errors,
                source_map,
                &self.expansion_info,
            ));
        }

        result
    }
}

// Re-export core modules for the compilation pipeline
pub use ic_parse::ParseResult as AstResult;
pub use ic_syntax::Item as AstItem;
pub use {ic_hir as hir, ic_ptree as ptree, ic_vfs as vfs};

/// Convert AST to HIR.
///
/// # Errors
///
/// Returns an error if the AST contains semantic errors.
pub fn ast_to_hir<I>(
    ast: I,
    source_map: &SourceMap,
    lint_config: &LintConfig,
) -> Result<hir::ResolvedGraph, CompileError>
where
    I: IntoIterator<Item = AstItem>,
{
    let ast_vec: Vec<_> = ast.into_iter().collect();
    let mut all_errors = Vec::new();
    let mut all_warnings = Vec::new();

    // Lint the user's AST
    let report = ic_lint::lint_syntax_with_config(&ast_vec, source_map, lint_config);
    all_errors.extend(report.errors.into_iter().map(Into::into));
    all_warnings.extend(report.warnings);

    // Lower to HIR
    let mut hir = hir::from_ast(hir::AstInput::User(ast_vec));

    // Lint the HIR if no errors so far
    if all_errors.is_empty() {
        let report = ic_lint::lint_hir_with_config(&hir, source_map, lint_config);
        all_errors.extend(report.errors.into_iter().map(Into::into));
        all_warnings.extend(report.warnings);
    }

    let hir_errors = std::mem::take(&mut hir.errors);
    all_errors.extend(hir_errors.into_iter().map(Into::into));

    let hir_warnings = std::mem::take(&mut hir.warnings);
    all_warnings.extend(hir_warnings);

    if !all_errors.is_empty() {
        return Err(CompileError::Diagnostics(CompileDiagnostics {
            errors: all_errors,
            warnings: all_warnings,
            expansion_info: std::collections::HashMap::new(),
        }));
    }

    Ok(hir)
}

/// Convert HIR to ptree.
pub fn hir_to_ptree(hir: &hir::ResolvedGraph, source_map: &SourceMap) -> ptree::ParseResult {
    ic_ptree_lower::from_hir(hir, source_map)
}

/// Convert multiple HIRs to ptrees.
#[must_use]
pub fn hirs_to_ptrees(
    hirs: &[hir::ResolvedGraph],
    source_map: &SourceMap,
) -> Vec<ptree::ParseResult> {
    hirs.iter().map(|h| hir_to_ptree(h, source_map)).collect()
}

/// Main compiler interface.
#[must_use]
pub struct Compiler {
    options: CompilerOptions,
    source_map: SourceMap,
}

impl Compiler {
    /// Create a new compiler with the given options.
    pub fn new(options: CompilerOptions) -> Self {
        Self {
            options,
            source_map: SourceMap::default(),
        }
    }

    /// Get a reference to the options.
    pub fn options(&self) -> &CompilerOptions {
        &self.options
    }

    /// Get a mutable reference to the options.
    pub fn options_mut(&mut self) -> &mut CompilerOptions {
        &mut self.options
    }

    /// Get a reference to the source map.
    #[must_use]
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Get a mutable reference to the source map.
    #[must_use]
    pub fn source_map_mut(&mut self) -> &mut SourceMap {
        &mut self.source_map
    }

    /// Compile the configured IDL files to AST.
    ///
    /// Returns all parsed AST items and any diagnostics (warnings) that were generated.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails. The error will contain all diagnostics
    /// including both errors and warnings.
    pub fn compile(&mut self) -> Result<CompiledAst, CompileError> {
        if self.options.files.is_empty() {
            return Err(CompileError::Diagnostics(CompileDiagnostics {
                errors: vec![InternalError::Custom("no input files".to_string())],
                warnings: Vec::new(),
                expansion_info: std::collections::HashMap::new(),
            }));
        }

        // Parse all files to AST
        let (items, diagnostics) = try_compile_to_ast(&self.options, &mut self.source_map)?;

        Ok(CompiledAst { items, diagnostics })
    }

    /// Compile the configured IDL files directly to HIR.
    ///
    /// This is a shorthand for `compile()` followed by `ast_to_hir()`.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails.
    pub fn compile_hir(
        &mut self,
    ) -> Result<(hir::ResolvedGraph, CompileDiagnostics), CompileError> {
        if self.options.files.is_empty() {
            return Err(CompileError::Diagnostics(CompileDiagnostics {
                errors: vec![InternalError::Custom("no input files".to_string())],
                warnings: Vec::new(),
                expansion_info: std::collections::HashMap::new(),
            }));
        }

        // Compile each file to a separate HIR (with built-ins in context)
        let mut hirs = Vec::new();
        let mut all_diagnostics = CompileDiagnostics {
            errors: Vec::new(),
            warnings: Vec::new(),
            expansion_info: std::collections::HashMap::new(),
        };

        for file in &self.options.files.clone() {
            match self.compile_file_to_hir_without_builtins(file) {
                Ok((hir, diag)) => {
                    hirs.push(hir);
                    all_diagnostics.warnings.extend(diag.warnings);
                    all_diagnostics.expansion_info.extend(diag.expansion_info);
                }
                Err(CompileError::Diagnostics(diag)) => {
                    all_diagnostics.errors.extend(diag.errors);
                    all_diagnostics.warnings.extend(diag.warnings);
                    all_diagnostics.expansion_info.extend(diag.expansion_info);
                }
                Err(e) => return Err(e),
            }
        }

        if !all_diagnostics.errors.is_empty() {
            return Err(CompileError::Diagnostics(all_diagnostics));
        }

        // Merge all HIRs
        let merged = hir::merge::merge_hir_trees(&hirs);

        // Add merge errors to diagnostics
        all_diagnostics
            .errors
            .extend(merged.errors.into_iter().map(Into::into));

        if !all_diagnostics.errors.is_empty() {
            return Err(CompileError::Diagnostics(all_diagnostics));
        }

        let merged_hir = hir::ResolvedGraph {
            context: merged.context,
            order: merged.order,
            builtin_order: merged.builtin_order,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        Ok((merged_hir, all_diagnostics))
    }

    /// Compile a single file to HIR without built-in annotations.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails.
    fn compile_file_to_hir_without_builtins(
        &mut self,
        path: &Path,
    ) -> Result<(hir::ResolvedGraph, CompileDiagnostics), CompileError> {
        let proc_args = self.proc_args();
        let ast = ic_parse::from_path(path, proc_args, &mut self.source_map).map_err(|e| {
            CompileError::Io(std::io::Error::new(e.kind(), format_io_error(&e, path)))
        })?;

        let mut diagnostics = CompileDiagnostics {
            errors: ast.errors.into_iter().map(Into::into).collect(),
            warnings: Vec::new(),
            expansion_info: ast.expansion_info,
        };

        if !diagnostics.errors.is_empty() {
            return Err(CompileError::Diagnostics(diagnostics));
        }

        // Collect preprocessor warnings if enabled
        if self.options.warn.preprocessor_enabled() {
            diagnostics
                .warnings
                .extend(ast.warnings.iter().map(pretty::to_warning));
        }

        // Convert to HIR without built-in annotations
        let lint_config = self.options.warn.to_lint_config();

        // Run AST linting first
        if diagnostics.errors.is_empty() {
            let report =
                ic_lint::lint_syntax_with_config(&ast.tree, &self.source_map, &lint_config);
            diagnostics
                .errors
                .extend(report.errors.into_iter().map(Into::into));
            diagnostics.warnings.extend(report.warnings);
        }

        // Parse built-in annotations
        let builtin_file_id = self.source_map.embed_with_name(
            "<builtin-annotations>",
            include_str!("../idl/annotations.idl"),
        );
        let builtin_parsed =
            ic_parse::from_file(builtin_file_id, ProcArgs::default(), &mut self.source_map);

        assert!(
            builtin_parsed.errors.is_empty(),
            "Failed to parse built-in annotations: {:?}",
            builtin_parsed.errors
        );

        // Compile with built-in context
        let mut hir = hir::from_ast(hir::AstInput::WithBuiltins {
            builtins: builtin_parsed.tree,
            user: ast.tree,
            include_in_output: false,
        });

        // Run HIR linting
        if diagnostics.errors.is_empty() {
            let report = ic_lint::lint_hir_with_config(&hir, &self.source_map, &lint_config);
            diagnostics
                .errors
                .extend(report.errors.into_iter().map(Into::into));
            diagnostics.warnings.extend(report.warnings);
        }

        // Extract HIR errors and warnings
        let hir_errors = std::mem::take(&mut hir.errors);
        diagnostics
            .errors
            .extend(hir_errors.into_iter().map(Into::into));

        let hir_warnings = std::mem::take(&mut hir.warnings);
        diagnostics.warnings.extend(hir_warnings);

        if !diagnostics.errors.is_empty() {
            return Err(CompileError::Diagnostics(diagnostics));
        }

        Ok((hir, diagnostics))
    }

    /// Compile a single file to HIR with built-in annotations.
    /// This is kept for backward compatibility.
    #[allow(dead_code)]
    fn compile_file_to_hir(
        &mut self,
        path: &Path,
    ) -> Result<(hir::ResolvedGraph, CompileDiagnostics), CompileError> {
        self.compile_file_to_hir_without_builtins(path)
    }

    /// Parse files and get the AST.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be parsed.
    pub fn parse_to_ast(&mut self, path: &Path) -> Result<AstResult, CompileError> {
        let proc_args = self.proc_args();
        let ast = ic_parse::from_path(path, proc_args, &mut self.source_map).map_err(|e| {
            CompileError::Io(std::io::Error::new(e.kind(), format_io_error(&e, path)))
        })?;

        if !ast.errors.is_empty() {
            return Err(CompileError::Diagnostics(CompileDiagnostics {
                errors: ast.errors.into_iter().map(Into::into).collect(),
                warnings: Vec::new(),
                expansion_info: std::collections::HashMap::new(),
            }));
        }

        Ok(ast)
    }

    /// Add a file to be compiled.
    pub fn add_file(&mut self, path: PathBuf) -> &mut Self {
        self.options.files.push(path);
        self
    }

    /// Add multiple files to be compiled.
    pub fn add_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> &mut Self {
        self.options.files.extend(paths);
        self
    }

    /// Clear all files.
    pub fn clear_files(&mut self) -> &mut Self {
        self.options.files.clear();
        self
    }

    /// Create preprocessor arguments from options.
    fn proc_args(&self) -> ProcArgs {
        let defines = self.options.define.iter().map(|v| {
            v.split_once('=')
                .map_or_else(|| (v.as_str(), None), |(k, v)| (k, Some(v)))
        });

        ProcArgs::default()
            .define("__IC_IDL__", None)
            .defines(defines)
            .includes(self.options.include.clone())
            .skip_comments(self.options.ignore_comments)
    }
}

fn try_compile_to_ast(
    options: &CompilerOptions,
    vfs: &mut SourceMap,
) -> Result<(Vec<AstItem>, CompileDiagnostics), CompileError> {
    let defines = options.define.iter().map(|v| {
        v.split_once('=')
            .map_or_else(|| (v.as_str(), None), |(k, v)| (k, Some(v)))
    });

    let args = ProcArgs::default()
        .define("__IC_IDL__", None)
        .defines(defines)
        .includes(options.include.clone())
        .skip_comments(options.ignore_comments);

    let mut all_asts = vec![];
    let mut all_errors = vec![];
    let mut all_warnings = vec![];
    let mut all_expansion_info = std::collections::HashMap::new();

    let files = util::collect_files(&options.files).map_err(|e| {
        CompileError::Io(std::io::Error::other(format!(
            "Failed to collect files: {e:?}"
        )))
    })?;

    // Parse all files to AST
    for file in files {
        let ast = match ic_parse::from_path(&file, args.clone(), vfs) {
            Ok(ast) => ast,
            Err(e) => {
                all_errors.push(InternalError::Custom(format!(
                    "failed to open `{}`: {e}",
                    file.display(),
                )));
                continue;
            }
        };

        // Collect parse errors
        all_errors.extend(ast.errors.iter().cloned().map(Into::into));

        // Collect preprocessor warnings if enabled
        if options.warn.preprocessor_enabled() {
            all_warnings.extend(ast.warnings.iter().map(pretty::to_warning));
        }

        all_expansion_info.extend(ast.expansion_info);
        all_asts.extend(ast.tree);
    }

    // If there were parse errors, return early
    if !all_errors.is_empty() {
        return Err(CompileError::Diagnostics(CompileDiagnostics {
            errors: all_errors,
            warnings: all_warnings,
            expansion_info: all_expansion_info,
        }));
    }

    // Return the AST with any warnings as diagnostics
    Ok((
        all_asts,
        CompileDiagnostics {
            errors: Vec::new(),
            warnings: all_warnings,
            expansion_info: all_expansion_info,
        },
    ))
}

/// Format an I/O error with a filename for user-friendly output.
fn format_io_error(error: &std::io::Error, path: &Path) -> String {
    let message = match error.kind() {
        std::io::ErrorKind::NotFound => "no such file or directory",
        std::io::ErrorKind::PermissionDenied => "permission denied",
        std::io::ErrorKind::InvalidData => "invalid file contents",
        std::io::ErrorKind::UnexpectedEof => "unexpected end of file",
        _ => return format!("{}: '{}'", error, path.display().to_string().yellow()),
    };
    format!("{}: '{}'", message, path.display().to_string().yellow())
}
