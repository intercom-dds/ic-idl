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
//! options.codegen.cpp_out = Some(PathBuf::from("generated/cpp"));
//!
//! let mut compiler = Compiler::new(options);
//! match compiler.compile() {
//!     Ok(files) => {
//!         for file in files {
//!             println!("Generated: {}", file.path().display());
//!         }
//!     }
//!     Err(e) => eprintln!("Compilation failed: {}", e),
//! }
//!
//! // Example 2: Parse to ptree and use a backend directly
//! let mut compiler = Compiler::new(CompilerOptions::default());
//! let ptree = compiler.parse_to_ptree(&PathBuf::from("example.idl")).unwrap();
//! let files = ic_codegen_rust::codegen_rust(&ptree);
//! ```

use std::path::{Path, PathBuf};

use ic_cli::color::Colorize;
use ic_diagnostic::Diag;
use ic_emit::File;
use ic_preproc::{ExpansionInfo, ProcArgs};
use ic_vfs::SourceMap;

// Import modules
pub(crate) mod config;
pub(crate) mod pretty;
pub(crate) mod util;

// Re-export configuration types
pub use config::{
    CodegenOptions, CppOptions, IdlOptions, Options as CompilerOptions, PythonOptions, RustOptions,
    Unstable, Warnings,
};
// Re-export useful types
pub use ic_emit::File as GeneratedFile;
pub use ic_lint::{Category as LintCategory, Level as LintLevel, LintConfig};
use util::Error as InternalError;

/// Error type for compilation failures.
#[derive(Debug)]
pub enum CompileError {
    /// I/O error (e.g., file not found).
    Io(std::io::Error),
    /// Parse or semantic analysis error.
    Analysis(Vec<String>, usize), // errors, warning count
    /// Backend code generation error.
    Codegen(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Io(e) => write!(f, "I/O error: {e}"),
            CompileError::Analysis(errors, _) => {
                let errors_str = errors.join("\n");
                write!(f, "Analysis errors:\n{errors_str}")
            }
            CompileError::Codegen(e) => write!(f, "Code generation error: {e}"),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e)
    }
}

// Re-export types for the compilation pipeline
pub use ic_hir::ResolvedGraph as HirResult;
pub use ic_parse::ParseResult as AstResult;
pub use ic_ptree::ParseResult as PTreeResult;

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
    pub fn parse_file(&mut self, path: &Path) -> Result<ic_ptree::ParseResult, CompileError> {
        let proc_args = self.create_proc_args();
        let parsed = try_parse(&self.options, proc_args, path, &mut self.source_map);

        if !parsed.errors.is_empty() {
            let error_strings = parsed
                .errors
                .into_iter()
                .map(|e| format!("{e:?}"))
                .collect();
            return Err(CompileError::Analysis(error_strings, parsed.warnings.len()));
        }

        parsed
            .result
            .ok_or_else(|| CompileError::Analysis(vec!["Failed to parse file".to_string()], 0))
    }

    /// Parse multiple IDL files.
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be read or parsed.
    pub fn parse_files(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<Vec<ic_ptree::ParseResult>, CompileError> {
        let mut results = Vec::new();
        for path in paths {
            results.push(self.parse_file(path)?);
        }
        Ok(results)
    }

    /// Compile the configured IDL files and generate code.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails.
    pub fn compile(&mut self) -> Result<Vec<GeneratedFile>, CompileError> {
        if self.options.files.is_empty() {
            return Err(CompileError::Analysis(
                vec!["no input files".to_string()],
                0,
            ));
        }

        // Use the existing try_main logic
        let result = try_main(&self.options, &mut self.source_map)?;
        Ok(result)
    }

    /// Parse files and get the AST.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be parsed.
    pub fn parse_to_ast(&mut self, path: &Path) -> Result<ic_parse::ParseResult, CompileError> {
        let proc_args = self.create_proc_args();
        let ast = ic_parse::from_path(path, proc_args, &mut self.source_map).map_err(|e| {
            CompileError::Io(std::io::Error::other(format!("Failed to parse file: {e}")))
        })?;

        if !ast.errors.is_empty() {
            let error_strings = ast.errors.iter().map(|e| format!("{e:?}")).collect();
            return Err(CompileError::Analysis(error_strings, ast.warnings.len()));
        }

        Ok(ast)
    }

    /// Convert AST to HIR.
    ///
    /// # Errors
    ///
    /// Returns an error if the AST contains semantic errors.
    pub fn ast_to_hir(
        &self,
        ast: ic_parse::ParseResult,
    ) -> Result<ic_hir::ResolvedGraph, CompileError> {
        let hir = ic_hir::from_ast(ast.tree);

        if !hir.errors.is_empty() {
            let error_strings = hir.errors.iter().map(|e| format!("{e:?}")).collect();
            return Err(CompileError::Analysis(error_strings, 0));
        }

        Ok(hir)
    }

    /// Convert HIR to ptree.
    pub fn hir_to_ptree(&self, hir: &ic_hir::ResolvedGraph) -> ic_ptree::ParseResult {
        ic_ptree_lower::from_hir(hir, &self.source_map)
    }

    /// Parse files and convert through the full pipeline to ptree.
    ///
    /// # Errors
    ///
    /// Returns an error if any stage of the pipeline fails.
    pub fn parse_to_ptree(&mut self, path: &Path) -> Result<ic_ptree::ParseResult, CompileError> {
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
    expansion_info: std::collections::HashMap<ic_vfs::Span, ExpansionInfo>,
}

fn try_main(options: &CompilerOptions, vfs: &mut SourceMap) -> Result<Vec<File>, CompileError> {
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

    // Emit all warnings regardless of errors
    if !all_warnings.is_empty() {
        pretty::emit_warnings(&all_warnings, vfs);
    }

    // If there were any errors, emit them and return
    if !all_errors.is_empty() {
        pretty::emit_errors_with_expansion(&all_errors, vfs, &all_expansion_info);
        let error_strings = all_errors.into_iter().map(|e| format!("{e:?}")).collect();
        return Err(CompileError::Analysis(error_strings, all_warnings.len()));
    }

    // Emit warning summary if there were warnings but no errors
    #[allow(clippy::print_stderr)]
    {
        if !all_warnings.is_empty() {
            let warning_plural = if all_warnings.len() > 1 { "s" } else { "" };
            eprintln!(
                "{} {} warning{} emitted",
                "warning:".purple().bold(),
                all_warnings.len(),
                warning_plural
            );
        }
    }

    let result = try_ptree(options, &trees)
        .map_err(|e| CompileError::Analysis(vec![format!("{:?}", e)], all_warnings.len()))?;
    Ok(result)
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

fn try_ptree(
    options: &CompilerOptions,
    parsed: &[ic_ptree::ParseResult],
) -> Result<Vec<File>, InternalError> {
    // Merge multiple ptrees into one
    let merged = ic_ptree::merge_trees(parsed);
    if options.unstable.ptree_dump {
        ic_ptree_dump::ptree_dump(&merged);
    }

    let backends: &[(_, fn(_) -> _)] = &[
        (&options.codegen.cpp_out, ic_codegen_cxx::codegen_cpp),
        (&options.codegen.idl_out, ic_codegen_idl::codegen_idl),
        (&options.codegen.json_out, ic_codegen_json::codegen_json),
        (&options.codegen.xml_out, ic_codegen_xml::codegen_xml),
        (&options.codegen.rust_out, ic_codegen_rust::codegen_rust),
        (
            &options.codegen.proto_out,
            ic_codegen_protobuf::codegen_proto,
        ),
        (
            &options.codegen.python_out,
            ic_codegen_python::codegen_python,
        ),
    ];

    let mut generated = vec![];
    for (dir, backend) in backends
        .iter()
        .filter_map(|(v, t)| v.as_ref().map(|v| (v, t)))
    {
        let dir = std::path::absolute(dir)?;
        if options.purge_dirs {
            util::safe_purge(&dir)?;
        }

        // Invoke the backend and update the file paths
        let files = backend(&merged).into_iter().map(|v| match v {
            File::Generated { path, source } => File::Generated {
                path: dir.join(path),
                source,
            },
            File::Dep(_) => v,
        });
        generated.extend(files);
    }
    Ok(generated)
}
