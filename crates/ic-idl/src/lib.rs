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

//! # IDL Compilation Pipeline
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
//! // Example 1: Compile IDL files to HIR
//! let mut options = CompilerOptions::default();
//! options.files.push(PathBuf::from("example.idl"));
//!
//! let mut compiler = Compiler::new(options);
//! match compiler.compile() {
//!     Ok((hir, diagnostics)) => {
//!         // Check for warnings
//!         if !diagnostics.warnings.is_empty() {
//!             let formatted = ic_idl::pretty::fmt_warnings(
//!                 &diagnostics.warnings,
//!                 compiler.source_map(),
//!                 ic_idl::ErrorFormat::Human,
//!             );
//!             eprintln!("{formatted}");
//!         }
//!
//!         // Use the HIR for analysis or code generation
//!     }
//!     Err(ic_idl::CompileError::Diagnostics(diagnostics)) => {
//!         // Format errors and warnings using the pretty module
//!         let formatted_errors = ic_idl::pretty::fmt_errors(
//!             &diagnostics.errors,
//!             compiler.source_map(),
//!             &diagnostics.expansion_info,
//!             ic_idl::ErrorFormat::Human,
//!         );
//!         eprintln!("{formatted_errors}");
//!     }
//!     Err(e) => {
//!         // Handle I/O errors
//!     }
//! }
//!
//! ```

use std::path::{Path, PathBuf};

use ic_diagnostic::Diag;
use ic_preproc::{ExpansionInfo, ProcArgs};
use ic_vfs::SourceMap;
use tracing::{info, info_span};

// Import modules
mod builtin;
mod config;
mod parse;
pub mod pretty;
pub mod util;

pub use config::{ErrorFormat, Options as CompilerOptions, Unstable, Warnings};
use ic_cli::color::Colorize as _;
pub use ic_emit::File;
pub use util::Error as DiagnosticError;
pub use {ic_hir as hir, ic_hir_lower as hir_lower, ic_vfs as vfs};

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
    pub fn format(&self, source_map: &SourceMap, format: config::ErrorFormat) -> String {
        let mut result = String::new();

        if !self.warnings.is_empty() {
            result.push_str(&pretty::fmt_warnings(&self.warnings, source_map, format));
            if !self.errors.is_empty() {
                result.push('\n');
            }
        }

        if !self.errors.is_empty() {
            result.push_str(&pretty::fmt_errors(
                &self.errors,
                source_map,
                &self.expansion_info,
                format,
            ));
        }

        result
    }
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

    /// Create preprocessor arguments from options.
    pub fn proc_args(&self) -> ProcArgs {
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

    /// Compile the configured IDL files to HIR.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails.
    ///
    /// # Panics
    ///
    /// Panics if the built-in annotations file fails to parse. This should never
    /// happen in practice as the built-in annotations are embedded in the binary.
    pub fn compile(&mut self) -> Result<(hir::ResolvedGraph, CompileDiagnostics), CompileError> {
        // Parse built-in annotations once
        let builtin_file_id = self.source_map.embed_with_name(
            "<builtin-annotations>",
            include_str!("../idl/annotations.idl"),
        );
        let builtin_parsed =
            parse::from_file(builtin_file_id, ProcArgs::default(), &mut self.source_map);

        assert!(
            builtin_parsed.errors.is_empty(),
            "Failed to parse built-in annotations: {:?}",
            builtin_parsed.errors
        );

        // Compile each file to a separate HIR
        let mut hirs = Vec::new();
        let mut all_diagnostics = CompileDiagnostics {
            errors: Vec::new(),
            warnings: Vec::new(),
            expansion_info: std::collections::HashMap::new(),
        };

        {
            let file_count = self.options.files.len();
            let _parse_span = info_span!("parse", files = file_count).entered();

            for file in &self.options.files.clone() {
                match self.compile_file(file, true, &builtin_parsed.tree) {
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
        }

        if !all_diagnostics.errors.is_empty() {
            return Err(CompileError::Diagnostics(all_diagnostics));
        }

        // Merge all HIRs
        let hir = {
            let _merge_span = info_span!("merge", hirs = hirs.len()).entered();
            let merged = hir::merge::merge_hir_trees(&hirs);

            // Add merge errors to diagnostics
            all_diagnostics
                .errors
                .extend(merged.errors.into_iter().map(Into::into));

            if !all_diagnostics.errors.is_empty() {
                return Err(CompileError::Diagnostics(all_diagnostics));
            }

            hir::ResolvedGraph {
                context: merged.context,
                order: merged.order,
                builtin_order: merged.builtin_order,
                errors: Vec::new(),
                warnings: Vec::new(),
            }
        };

        // Apply HIR transformations
        let hir = {
            let _xform_span = info_span!("xform").entered();

            // Coerce @default annotation values to match their target types
            let hir = ic_hir_xform::default_annotation::transform(hir);

            // Convert `@position` annotations to bitmask values
            let hir = ic_hir_xform::position_annotation::transform(hir);

            // Mark types with `IS_TRIVIAL` and `TOTAL_ORDER` flags
            let hir = ic_hir_xform::type_flags::transform(hir);

            // Add implicit default cases to incomplete unions
            let hir = ic_hir_xform::implicit_default::transform(hir);

            // Coalesce multiple null variants in unions
            let hir = ic_hir_xform::coalesce_null_variants::transform(hir);

            // Verify that, after all transformations, the HIR is still consistent
            #[cfg(debug_assertions)]
            ic_hir_xform::normalize::normalize(&hir);

            hir
        };

        Ok((hir, all_diagnostics))
    }

    fn compile_file(
        &mut self,
        path: &Path,
        include_builtins: bool,
        builtin_ast: &[ic_syntax::Item],
    ) -> Result<(hir::ResolvedGraph, CompileDiagnostics), CompileError> {
        let proc_args = self.proc_args();
        let ast = parse::from_path(path, proc_args, &mut self.source_map).map_err(|e| {
            CompileError::Io(std::io::Error::new(e.kind(), format_io_error(&e, path)))
        })?;

        let item_count = ast.tree.len();
        info!(file = %path.display(), items = item_count, "parsed");

        let mut diagnostics = CompileDiagnostics {
            errors: ast.errors,
            warnings: vec![],
            expansion_info: ast.expansion_info,
        };

        if !diagnostics.errors.is_empty() {
            return Err(CompileError::Diagnostics(diagnostics));
        }

        // Create lint config
        let lint_config = self.options.warn.to_lint_config();

        // Run AST lints first
        if diagnostics.errors.is_empty() {
            let _lint_span = info_span!("lint_syntax").entered();
            let syntax_input = ic_lint::SyntaxInput {
                tree: &ast.tree,
                orphaned_annotations: &ast.orphaned_annotations,
                preproc_warnings: &ast.preproc_warnings,
                expansion_info: Some(&diagnostics.expansion_info),
            };
            let report =
                ic_lint::lint_syntax_with_config(&syntax_input, &self.source_map, &lint_config);
            diagnostics
                .errors
                .extend(report.errors.into_iter().map(Into::into));
            diagnostics.warnings.extend(report.warnings);
        }

        // Compile with or without built-in context
        let mut hir = {
            let _lower_span = info_span!("lower").entered();
            if include_builtins {
                hir_lower::from_ast(hir_lower::AstInput::WithBuiltins {
                    builtins: builtin_ast.to_vec(),
                    user: ast.tree,
                    include_in_output: false,
                })
            } else {
                hir_lower::from_ast(hir_lower::AstInput::User(ast.tree))
            }
        };

        let def_count = hir.order.len();
        info!(definitions = def_count, "lowered");

        // Run HIR lints
        if diagnostics.errors.is_empty() {
            let _lint_span = info_span!("lint_hir").entered();
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
