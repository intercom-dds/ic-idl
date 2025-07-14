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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::Path;

use config::Options;
use ic_cli::color::Colorize;
use ic_cli::{Command, ParseError};
use ic_diagnostic::Diag;
use ic_emit::File;
use ic_preproc::ProcArgs;
use ic_ptree::ParseResult;
use ic_vfs::SourceMap;
use util::{Error, collect_files, write_if_changed};

mod config;
mod info;
mod panic;
mod pretty;
mod unstable;
mod util;

fn main() {
    let result = Options::command()
        .split_flags(false)
        .align_sections(true)
        .try_parse();

    let result = match result {
        Ok(v) => v,
        Err(ParseError::Help(v)) => {
            println!("{v}");
            return;
        }
        Err(ParseError::Status(v)) => {
            error!("{v}");
            std::process::exit(1);
        }
    };

    let options = Options::from_result(&result);
    if options.version {
        println!("{}", info::version());
        return;
    }

    if options.files.is_empty() {
        error!("no input files");
        return;
    }

    // Install a panic handler to catch failed asserts.
    panic::install_hook();

    let mut vfs = SourceMap::default();
    let generated = match try_main(&options, &mut vfs) {
        Ok(v) => v,
        Err((errors, warning_count)) => {
            let error_plural = if errors.len() > 1 { "s" } else { "" };
            let warning_plural = if warning_count > 1 { "s" } else { "" };
            if warning_count > 0 {
                error!(
                    "aborting due to {} previous error{}, {} warning{}",
                    errors.len(),
                    error_plural,
                    warning_count,
                    warning_plural,
                );
            } else {
                error!(
                    "aborting due to {} previous error{}",
                    errors.len(),
                    error_plural,
                );
            }
            std::process::exit(1);
        }
    };

    for f in generated {
        if options.list {
            println!("{f}");
        } else if let File::Generated { path, source } = f {
            write_if_changed(path, &source).unwrap();
        }
    }
}

fn try_main(options: &Options, vfs: &mut SourceMap) -> Result<Vec<File>, (Vec<Error>, usize)> {
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

    let files = collect_files(&options.files)
        .map_err(|e| (e.into_iter().map(Error::Io).collect::<Vec<_>>(), 0))?;

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
        return Err((all_errors, all_warnings.len()));
    }

    let result = try_ptree(options, &trees).map_err(|e| (vec![e], all_warnings.len()))?;

    // Emit warning summary if there were warnings but no errors
    if !all_warnings.is_empty() {
        let warning_plural = if all_warnings.len() > 1 { "s" } else { "" };
        warn!("{} warning{} emitted", all_warnings.len(), warning_plural);
    }

    Ok(result)
}

struct Parsed {
    result: Option<ParseResult>,
    errors: Vec<Error>,
    warnings: Vec<Diag>,
    expansion_info: std::collections::HashMap<ic_vfs::Span, ic_preproc::ExpansionInfo>,
}

// To report as much information as possible at once, we keep going even if we
// meet an error and instead summarize everything at the end. This also applies
// to syntax errors in the input: the parser will attempt to recover so we can
// continue parsing and construct a partial AST.
fn try_parse(options: &Options, proc: ProcArgs, path: &Path, vfs: &mut SourceMap) -> Parsed {
    let mut errors = vec![];
    let mut warnings = vec![];

    // Try to parse the file
    let ast = match ic_parse::from_path(path, proc, vfs) {
        Ok(ast) => ast,
        Err(e) => {
            // File couldn't be opened - return early with just this error
            errors.push(Error::Custom(format!(
                "failed to open `{}`: {e}",
                path.display().purple(),
            )));
            return Parsed {
                result: None,
                errors,
                warnings,
                expansion_info: std::collections::HashMap::new(),
            };
        }
    };

    // Collect parse errors and warnings
    errors.extend(ast.errors.iter().cloned().map(Into::into));

    // Filter preprocessor warnings based on options
    if options.warn.preprocessor_enabled() {
        warnings.extend(ast.warnings.iter().map(pretty::parse_error_to_warning));
    } else {
        // Only include non-preprocessor warnings
        warnings.extend(
            ast.warnings
                .iter()
                .filter(|w| w.label != Some("preprocessor warning"))
                .map(pretty::parse_error_to_warning),
        );
    }

    if options.unstable.ast_dump {
        println!("{:#?}", ast.tree);
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
        if options.unstable.hir_dump {
            ic_hir_tree::emit_tree(&hir_result);
        }

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
        hir.and_then(|h| Some(ic_ptree_lower::from_hir(&h, vfs)))
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

fn try_ptree(options: &Options, parsed: &[ParseResult]) -> Result<Vec<File>, Error> {
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
        (&options.codegen.json_out, ic_codegen_json::codegen_json),
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
