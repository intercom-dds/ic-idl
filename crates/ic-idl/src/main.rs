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

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{backtrace, panic};

use ic_cli::{Command, ParseError};
use ic_emit::File;
use ic_emit::case::Case;
use ic_idl::{CompileDiagnostics, CompileError, Compiler, CompilerOptions, util};

mod info;
mod unstable;

macro_rules! error {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("ic-idl: {} {}", "error:".red().bold(), format!($($arg)*));
    }}
}

macro_rules! warn {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("{} {}", "warning:".purple().bold(), format!($($arg)*));
    }}
}

fn main() {
    let result = CompilerOptions::command()
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

    let options = CompilerOptions::from_result(&result);

    // Handle special flags
    if options.version {
        println!("{}", info::version());
        return;
    }

    if options.unstable.help {
        unstable::unstable_help();
        return;
    }

    if options.warn.help {
        unstable::warning_help();
        return;
    }

    // Print unknown warnings
    for unknown in &options.warn.unknown_warnings {
        warn!("unknown warning '{}'", unknown.yellow());
    }

    // Expand files
    let files = match util::collect_files(&options.files) {
        Ok(files) => files,
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };

    // Replace the options files with the expanded list
    let mut options = options;
    options.files = files;

    if options.files.is_empty() {
        error!("no input files");
        return;
    }

    // Install a panic handler to catch failed asserts.
    panic::set_hook(Box::new(dump_backtrace));

    // Run the compilation pipeline
    try_compile(options);
}

fn try_compile(options: CompilerOptions) {
    // Create and run the compiler
    let mut compiler = Compiler::new(options);

    // Use the new compile_hir method which handles merging
    let (hir, diagnostics) = match compiler.compile_hir() {
        Ok((hir, diag)) => (hir, diag),
        Err(CompileError::Io(e)) => {
            error!("{e}");
            std::process::exit(1);
        }
        Err(CompileError::Diagnostics(diagnostics)) => {
            emit_diagnostics(&compiler, &diagnostics);
            std::process::exit(1);
        }
    };

    // Extract warnings from compile diagnostics
    let warnings = diagnostics.warnings;

    // Emit any warnings
    if !warnings.is_empty() {
        let diag = CompileDiagnostics {
            errors: Vec::new(),
            warnings,
            expansion_info: std::collections::HashMap::new(),
        };
        emit_diagnostics(&compiler, &diag);
    }

    // Apply HIR transformations
    let hir = ic_hir_xform::value_annotation::transform(hir);
    let hir = ic_hir_xform::position_annotation::transform(hir);

    // Move nested types into modules. Keep track of the moved nodes to
    // properly escape their names later on to ensure the correct node gets
    // precedence.
    let (hir, moved_defs) = ic_hir_xform::move_nested::transform(hir);

    // Squash reopened modules into single definitions
    let hir = ic_hir_xform::squash_modules::transform(hir);

    // Strip prefixes from enumerators
    let hir = ic_hir_xform::enum_prefix::transform(hir);

    // Mark types with IS_TRIVIAL and TOTAL_ORDER flags
    let hir = ic_hir_xform::type_flags::transform(hir);

    // Rename DDS::XTypes to DDS::xtypes
    let hir = ic_hir_xform::rename_xtypes::transform(hir);

    // Rename all nodes to conform to Rust's naming convention
    let hir = ic_hir_xform::rename::transform(
        hir,
        ic_hir_xform::rename::Target {
            struct_type: Some(Case::Pascal),
            union_type: Some(Case::Pascal),
            enum_type: Some(Case::Pascal),
            interface: Some(Case::Pascal),
            valuetype: Some(Case::Pascal),
            alias: Some(Case::Pascal),
            bitmask: Some(Case::Pascal),
            bitset: Some(Case::Pascal),
            exception: Some(Case::Pascal),
            annotation: Some(Case::Pascal),
            member: Some(Case::Snake),
            variant: Some(Case::Pascal),
            enumerator: Some(Case::Pascal),
            bit_flag: Some(Case::Snake),
            bitset_field: Some(Case::Snake),
            constant: Some(Case::Snake),
            module: Some(Case::Snake),
            operation: Some(Case::Snake),
            attribute: Some(Case::Snake),
            parameter: Some(Case::Snake),
            annotation_param: Some(Case::Snake),
            name_preprocessor: Some(ic_hir_xform::rename::strip_common_suffixes),
            moved_defs,
        },
    );

    // Add implicit default cases to incomplete unions
    let hir = ic_hir_xform::implicit_default::transform(hir);

    // Coalesce multiple null variants in unions
    let hir = ic_hir_xform::coalesce_null_variants::transform(hir);

    // Final normalization after all transformations
    let hir = ic_hir_xform::normalize::normalize(hir);

    // Dump HIR if requested (after transformations)
    if compiler.options().unstable.hir_dump {
        let tree = ic_hir_tree::emit_tree(&hir, compiler.source_map());
        println!("{tree}");
    }

    // For now, skip ptree conversion and code generation
    // Convert HIR to ptree for code generation
    let ptree = ic_idl::hir_to_ptree(&hir, compiler.source_map());

    // Dump ptree if requested
    if compiler.options().unstable.ptree_dump {
        ic_ptree_dump::ptree_dump(&ptree);
    }

    // Generate code using backends
    let generated = match generate_code(compiler.options(), &ptree) {
        Ok(files) => files,
        Err(e) => {
            error!("code generation error: {e}");
            std::process::exit(1);
        }
    };

    // Handle output
    if compiler.options().list {
        for f in &generated {
            println!("{f}");
        }
    } else if let Err(e) = write_files(&generated) {
        error!("failed to write files: {}", e);
        std::process::exit(1);
    }
}

fn generate_code(
    options: &CompilerOptions,
    ptree: &ic_idl::ptree::ParseResult,
) -> Result<Vec<File>, util::Error> {
    let mut generated = vec![];

    if let Some(output_dir) = &options.codegen.cpp_out {
        let files = invoke_backend(
            output_dir,
            ic_codegen_cxx::codegen_cpp,
            ptree,
            options.cpp.clone(),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.rust_out {
        let files = invoke_backend(
            output_dir,
            ic_codegen_rust::codegen_rust,
            ptree,
            options.rust.clone(),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.python_out {
        let files = invoke_backend(
            output_dir,
            ic_codegen_python::codegen_python,
            ptree,
            options.python.clone(),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.idl_out {
        let files = invoke_backend(
            output_dir,
            ic_codegen_idl::codegen_idl,
            ptree,
            options.idl.clone(),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.json_out {
        let files = invoke_backend(
            output_dir,
            |ptree, _: ()| ic_codegen_json::codegen_json(ptree),
            ptree,
            (),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.xml_out {
        let files = invoke_backend(
            output_dir,
            |ptree, _: ()| ic_codegen_xml::codegen_xml(ptree),
            ptree,
            (),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.proto_out {
        let files = invoke_backend(
            output_dir,
            |ptree, _: ()| ic_codegen_protobuf::codegen_proto(ptree),
            ptree,
            (),
            options.purge_dirs,
        )?;
        generated.extend(files);
    }

    Ok(generated)
}

fn invoke_backend<F, O>(
    output_dir: &std::path::Path,
    backend_fn: F,
    ptree: &ic_idl::ptree::ParseResult,
    options: O,
    purge_dirs: bool,
) -> Result<Vec<File>, util::Error>
where
    F: FnOnce(&ic_idl::ptree::ParseResult, O) -> Vec<File>,
{
    let dir = std::path::absolute(output_dir)?;

    if purge_dirs {
        util::safe_purge(&dir)?;
        std::fs::create_dir_all(&dir)?;
    }

    let files = backend_fn(ptree, options)
        .into_iter()
        .map(move |v| match v {
            File::Generated { path, source } => File::Generated {
                path: output_dir.join(path),
                source,
            },
            File::Dep(_) => v,
        })
        .collect();

    Ok(files)
}

fn write_files(files: &[File]) -> std::io::Result<()> {
    for file in files {
        if let File::Generated { path, source } = file {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            util::write_if_changed(path, source)?;
        }
    }
    Ok(())
}

#[allow(clippy::same_functions_in_if_condition)]
fn emit_diagnostics(compiler: &Compiler, diagnostics: &CompileDiagnostics) {
    if !diagnostics.warnings.is_empty() {
        let warnings = ic_idl::pretty::fmt_warnings(&diagnostics.warnings, compiler.source_map());
        eprintln!("{warnings}");
    }

    if !diagnostics.errors.is_empty() {
        let formatted = ic_idl::pretty::fmt_errors(
            &diagnostics.errors,
            compiler.source_map(),
            &diagnostics.expansion_info,
        );
        eprintln!("{formatted}");
    }

    let error_plural = if diagnostics.errors.len() > 1 {
        "s"
    } else {
        ""
    };
    let warning_plural = if diagnostics.warnings.len() > 1 {
        "s"
    } else {
        ""
    };

    if !diagnostics.warnings.is_empty() && !diagnostics.errors.is_empty() {
        error!(
            "aborting due to {} previous error{}, {} warning{}",
            diagnostics.errors.len(),
            error_plural,
            diagnostics.warnings.len(),
            warning_plural,
        );
    } else if !diagnostics.errors.is_empty() {
        error!(
            "aborting due to {} previous error{}",
            diagnostics.errors.len(),
            error_plural,
        );
    } else if !diagnostics.warnings.is_empty() {
        warn!(
            "{} warning{} emitted",
            diagnostics.warnings.len(),
            warning_plural,
        );
    }
}

fn dump_backtrace(info: &std::panic::PanicHookInfo) {
    let thread = std::thread::current();
    let thread = thread.name().unwrap_or("unknown");
    let trace = backtrace::Backtrace::force_capture();

    let msg = match info.payload().downcast_ref::<&str>() {
        Some(s) => *s,
        None => info
            .payload()
            .downcast_ref::<String>()
            .map_or("<null>", |s| &**s),
    };

    match info.location() {
        Some(loc) => {
            error!(
                "thread '{thread}' panicked at '{msg}', {}:{}",
                loc.file(),
                loc.line(),
            );
        }
        None => {
            error!("thread '{thread}' panicked at '{msg}'");
        }
    }

    if trace.status() == backtrace::BacktraceStatus::Captured {
        eprintln!("{trace:#?}");
    }
    eprintln!(
        "This is a compiler bug. Please report it to KONGSBERG <DDS-InterCOM@kda.kongsberg.com>.",
    );
}
