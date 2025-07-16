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
//! This crate provides a library interface to the ic-idl compiler, allowing
//! external users to parse IDL files and invoke code generation backends.
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
//!     Ok((ptree, diagnostics)) => {
//!         // Check for warnings
//!         if !diagnostics.warnings.is_empty() {
//!             // Use the pretty module to format warnings
//!             let formatted = ic_idl::pretty::format_warnings(&diagnostics.warnings, compiler.source_map());
//!             // User can print formatted warnings if desired
//!         }
//!         
//!         // Now you can use the ptree with any backend
//!         let files = ic_codegen_rust::codegen_rust(&ptree);
//!         // User can print or handle files as needed
//!     }
//!     Err(ic_idl::CompileError::Diagnostics(diagnostics)) => {
//!         // Format errors and warnings using the pretty module
//!         let formatted_errors = ic_idl::pretty::format_errors_with_expansion(
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
//! // Example 2: Parse to ptree and use a backend directly
//! let mut compiler = Compiler::new(CompilerOptions::default());
//! let ptree = compiler.parse_to_ptree(&PathBuf::from("example.idl")).unwrap();
//! let files = ic_codegen_rust::codegen_rust(&ptree);
//!
//! // Example 3: Access the compilation pipeline stages
//! let ast = compiler.parse_to_ast(&PathBuf::from("example.idl")).unwrap();
//! let hir = compiler.ast_to_hir(ast).unwrap();
//! // You can now use ic_idl::hir, ic_idl::parse, etc. to work with these types
//! ```

use std::path::{Path, PathBuf};

use ic_diagnostic::Diag;
use ic_preproc::{ExpansionInfo, ProcArgs};
use ic_vfs::SourceMap;

// Import modules
pub(crate) mod config;
pub mod pretty;
pub(crate) mod util;

// Re-export configuration types
pub use config::{
    CodegenOptions, CppOptions, IdlOptions, Options as CompilerOptions, PythonOptions, RustOptions,
    Unstable, Warnings,
};
// Re-export useful types
pub use ic_emit::File as GeneratedFile;
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

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(e) => write!(f, "I/O error: {e}"),
            CompileError::Diagnostics(diag) => {
                write!(f, "{} errors, {} warnings", diag.errors.len(), diag.warnings.len())
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

// Re-export core modules for the compilation pipeline
pub use ic_parse::ParseResult as AstResult;
pub use {ic_hir as hir, ic_ptree as ptree, ic_vfs as vfs};


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

    /// Parse a single IDL file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn parse_file(&mut self, path: &Path) -> Result<ptree::ParseResult, CompileError> {
        let proc_args = self.create_proc_args();
        let parsed = try_parse(&self.options, proc_args, path, &mut self.source_map);

        if !parsed.errors.is_empty() {
            return Err(CompileError::Diagnostics(CompileDiagnostics {
                errors: parsed.errors,
                warnings: parsed.warnings,
                expansion_info: std::collections::HashMap::new(),
            }));
        }

        parsed
            .result
            .ok_or_else(|| CompileError::Diagnostics(CompileDiagnostics {
                errors: vec![InternalError::Custom("Failed to parse file".to_string())],
                warnings: Vec::new(),
                expansion_info: std::collections::HashMap::new(),
            }))
    }

    /// Parse multiple IDL files.
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be read or parsed.
    pub fn parse_files(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<Vec<ptree::ParseResult>, CompileError> {
        let mut results = Vec::new();
        for path in paths {
            results.push(self.parse_file(path)?);
        }
        Ok(results)
    }

    /// Compile the configured IDL files to a merged ptree.
    ///
    /// Returns the compiled ptree and any diagnostics (warnings) that were generated.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails. The error will contain all diagnostics
    /// including both errors and warnings.
    pub fn compile(&mut self) -> Result<(ptree::ParseResult, CompileDiagnostics), CompileError> {
        if self.options.files.is_empty() {
            return Err(CompileError::Diagnostics(CompileDiagnostics {
                errors: vec![InternalError::Custom("no input files".to_string())],
                warnings: Vec::new(),
                expansion_info: std::collections::HashMap::new(),
            }));
        }

        // Use try_main to get the result with all diagnostics
        let (ptrees, diagnostics) = try_main_with_diagnostics(&self.options, &mut self.source_map)?;
        let merged_ptree = ptree::merge_trees(&ptrees);
        
        Ok((merged_ptree, diagnostics))
    }

    /// Compile the configured IDL files to individual ptrees.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails.
    pub fn compile_to_ptrees(&mut self) -> Result<Vec<ptree::ParseResult>, CompileError> {
        // Use try_main_with_diagnostics but discard the diagnostics
        let (ptrees, _diagnostics) = try_main_with_diagnostics(&self.options, &mut self.source_map)?;
        Ok(ptrees)
    }

    /// Parse files and get the AST.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be parsed.
    pub fn parse_to_ast(&mut self, path: &Path) -> Result<AstResult, CompileError> {
        let proc_args = self.create_proc_args();
        let ast = ic_parse::from_path(path, proc_args, &mut self.source_map).map_err(|e| {
            CompileError::Io(std::io::Error::other(format!("Failed to parse file: {e}")))
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

    /// Convert AST to HIR.
    ///
    /// # Errors
    ///
    /// Returns an error if the AST contains semantic errors.
    pub fn ast_to_hir(&self, ast: AstResult) -> Result<hir::ResolvedGraph, CompileError> {
        let hir = hir::from_ast(ast.tree);

        if !hir.errors.is_empty() {
            return Err(CompileError::Diagnostics(CompileDiagnostics {
                errors: hir.errors.into_iter().map(Into::into).collect(),
                warnings: Vec::new(),
                expansion_info: std::collections::HashMap::new(),
            }));
        }

        Ok(hir)
    }

    /// Convert HIR to ptree.
    pub fn hir_to_ptree(&self, hir: &hir::ResolvedGraph) -> ptree::ParseResult {
        ic_ptree_lower::from_hir(hir, &self.source_map)
    }

    /// Parse files and convert through the full pipeline to ptree.
    ///
    /// # Errors
    ///
    /// Returns an error if any stage of the pipeline fails.
    pub fn parse_to_ptree(&mut self, path: &Path) -> Result<ptree::ParseResult, CompileError> {
        let ast = self.parse_to_ast(path)?;
        let hir = self.ast_to_hir(ast)?;
        Ok(self.hir_to_ptree(&hir))
    }

    /// Create preprocessor arguments from options.
    fn create_proc_args(&self) -> ProcArgs {
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

/// Write generated files to disk if they have changed.
///
/// # Errors
///
/// Returns an error if any file cannot be written.
pub fn write_generated_files(files: &[GeneratedFile]) -> Result<(), CompileError> {
    for file in files {
        if let GeneratedFile::Generated { path, source } = file {
            util::write_if_changed(path, source)?;
        }
    }
    Ok(())
}

struct Parsed {
    result: Option<ic_ptree::ParseResult>,
    errors: Vec<InternalError>,
    warnings: Vec<Diag>,
    #[allow(dead_code)]
    expansion_info: std::collections::HashMap<ic_vfs::Span, ExpansionInfo>,
}

fn try_main_with_diagnostics(
    options: &CompilerOptions,
    vfs: &mut SourceMap,
) -> Result<(Vec<ptree::ParseResult>, CompileDiagnostics), CompileError> {
    let defines = options.define.iter().map(|v| {
        v.split_once('=')
            .map_or_else(|| (v.as_str(), None), |(k, v)| (k, Some(v)))
    });

    let args = ProcArgs::default()
        .define("__IC_IDL__", None)
        .defines(defines)
        .includes(options.include.clone())
        .skip_comments(options.ignore_comments);

    let mut trees = vec![];
    let mut all_errors = vec![];
    let mut all_warnings = vec![];

    let files = util::collect_files(&options.files).map_err(|e| {
        CompileError::Io(std::io::Error::other(format!(
            "Failed to collect files: {e:?}"
        )))
    })?;

    // Collect all expansion info across files
    let mut all_expansion_info = std::collections::HashMap::new();

    for file in files {
        let parsed = try_parse(options, args.clone(), &file, vfs);
        all_errors.extend(parsed.errors);
        all_warnings.extend(parsed.warnings);
        all_expansion_info.extend(parsed.expansion_info);
        if let Some(result) = parsed.result {
            trees.push(result);
        }
    }

    // If there were any errors, return them as diagnostics
    if !all_errors.is_empty() {
        return Err(CompileError::Diagnostics(CompileDiagnostics {
            errors: all_errors,
            warnings: all_warnings,
            expansion_info: all_expansion_info,
        }));
    }

    // Return success with any warnings as diagnostics
    Ok((trees, CompileDiagnostics {
        errors: Vec::new(),
        warnings: all_warnings,
        expansion_info: all_expansion_info,
    }))
}

fn try_parse(
    options: &CompilerOptions,
    proc: ProcArgs,
    path: &Path,
    vfs: &mut SourceMap,
) -> Parsed {
    let mut errors = vec![];
    let mut warnings = vec![];

    // Try to parse the file
    let ast = match ic_parse::from_path(path, proc, vfs) {
        Ok(ast) => ast,
        Err(e) => {
            // File couldn't be opened - return early with just this error
            errors.push(InternalError::Custom(format!(
                "failed to open `{}`: {e}",
                path.display(),
            )));
            return Parsed {
                result: None,
                errors,
                warnings,
                expansion_info: std::collections::HashMap::new(),
            };
        }
    };

    // Collect parse errors
    errors.extend(ast.errors.iter().cloned().map(Into::into));

    // Collect preprocessor warnings if enabled
    if options.warn.preprocessor_enabled() {
        warnings.extend(ast.warnings.iter().map(pretty::parse_error_to_warning));
    }

    let mut hir = None;

    // Only run linting if there are no parse errors
    if errors.is_empty() {
        // Create lint configuration from CLI flags
        let lint_config = options.warn.to_lint_config();

        // Lint the AST
        let report = ic_lint::lint_syntax_with_config(&ast.tree, vfs, &lint_config);
        errors.extend(report.errors.into_iter().map(Into::into));
        warnings.extend(report.warnings);

        // Lower the AST to a HIR
        let mut hir_result = ic_hir::from_ast(ast.tree.clone());

        // Only lint HIR if no errors so far
        if errors.is_empty() {
            // Lint the HIR
            let report = ic_lint::lint_hir_with_config(&hir_result, vfs, &lint_config);
            errors.extend(report.errors.into_iter().map(Into::into));
            warnings.extend(report.warnings);
        }

        let hir_errors = std::mem::take(&mut hir_result.errors);
        errors.extend(hir_errors.into_iter().map(Into::into));
        hir = Some(hir_result);
    }

    // Only lower to ptree if no errors and we have a HIR
    let result = if errors.is_empty() {
        hir.map(|h| ic_ptree_lower::from_hir(&h, vfs))
    } else {
        None
    };

    Parsed {
        result,
        errors,
        warnings,
        expansion_info: ast.expansion_info,
    }
}
