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
use ic_idl::{
    CompileDiagnostics, CompileError, Compiler, CompilerOptions,
    hir, util,
};

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
            error!("I/O error: {}", e);
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

    // Dump HIR if requested (after transformations)
    if compiler.options().unstable.hir_dump {
        ic_hir_tree::emit_tree(&hir);
    }

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
            std::fs::create_dir_all(&dir)?;
        }

        // Invoke the backend and update the file paths
        let files = backend(ptree).into_iter().map(|v| match v {
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
    } else if diagnostics.warnings.is_empty() {
        error!(
            "aborting due to {} previous error{}",
            diagnostics.errors.len(),
            error_plural,
        );
    } else if diagnostics.warnings.is_empty() {
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

