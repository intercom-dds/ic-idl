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
use ic_idl::util::{self, IgnoreBrokenPipe};
use ic_idl::{CompileDiagnostics, CompileError, Compiler, CompilerOptions};
use tracing::{Level, info, info_span};
use tracing_subscriber::fmt;

mod info;
mod parse;
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

    // Enable tracing
    if let Some(level) = options.unstable.trace.as_deref() {
        init_tracing(level);
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
    info!(files = ?options.files, "starting compilation");

    // Create and run the compiler
    let mut compiler = Compiler::new(options);

    // Handle preprocessor-only mode
    if compiler.options().preprocessor_only {
        let proc_args = compiler.proc_args();
        for file in &compiler.options().files {
            match parse::preprocess_only(file, proc_args.clone()) {
                Ok(output) => {
                    println!("{output}");
                }
                Err(e) => {
                    error!("{e}");
                    std::process::exit(1);
                }
            }
        }
        return;
    }

    // Handle parse-only mode
    if compiler.options().unstable.parse_only {
        let proc_args = compiler.proc_args();
        for file in compiler.options().files.clone() {
            match parse::from_path(&file, proc_args.clone(), compiler.source_map_mut()) {
                Ok(ast) => {
                    if !ast.errors.is_empty() {
                        let formatted = ic_idl::pretty::fmt_errors(
                            &ast.errors,
                            compiler.source_map(),
                            &ast.expansion_info,
                        );
                        eprintln!("{formatted}");
                        info!(errors = ast.errors.len(), "failed");
                        std::process::exit(1);
                    }

                    if compiler.options().unstable.ast_dump {
                        println!("{:#?}", ast.tree);
                    }
                    info!(errors = 0, "completed");
                }
                Err(e) => {
                    error!("{e}");
                    std::process::exit(1);
                }
            }
        }
        return;
    }

    // Compile to HIR
    let (hir, diagnostics) = match compiler.compile() {
        Ok((hir, diag)) => (hir, diag),
        Err(CompileError::Io(e)) => {
            error!("{e}");
            std::process::exit(1);
        }
        Err(CompileError::Diagnostics(diagnostics)) => {
            emit_diagnostics(&compiler, &diagnostics);
            info!(
                errors = diagnostics.errors.len(),
                warnings = diagnostics.warnings.len(),
                "failed"
            );
            std::process::exit(1);
        }
    };

    // Extract warnings from compile diagnostics
    let warning_count = diagnostics.warnings.len();
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

    // Dump HIR if requested
    if compiler.options().unstable.hir_dump {
        let tree = ic_hir_tree::emit_tree(&hir, compiler.source_map());
        println!("{tree}");
    }

    // Generate code using backends (they will convert HIR to ptree as needed)
    let generated = match generate_code(compiler.options(), &hir, compiler.source_map()) {
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

    info!(
        errors = 0,
        warnings = warning_count,
        outputs = generated.len(),
        "completed"
    );
}

#[allow(clippy::too_many_lines)]
fn generate_code(
    options: &CompilerOptions,
    hir: &ic_hir::ResolvedGraph,
    vfs: &ic_vfs::SourceMap,
) -> Result<Vec<File>, util::Error> {
    let _codegen_span = info_span!("codegen").entered();
    let mut generated = vec![];

    if let Some(output_dir) = &options.codegen.cpp_out {
        let _span = info_span!("cpp", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_cpp::codegen_cpp(hir, vfs, options.cpp.clone()),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.csharp_out {
        let _span = info_span!("csharp", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_csharp::codegen_csharp(hir, vfs, options.csharp),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.rust_out {
        let _span = info_span!("rust", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_rust::codegen_rust(hir, options.rust),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.python_out {
        let _span = info_span!("python", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_python::codegen_python(hir, vfs, options.python.clone()),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.idl_out {
        let _span = info_span!("idl", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_idl::codegen_idl(hir, vfs, options.idl),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.java_out {
        let _span = info_span!("java", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_java::codegen_java(hir, options.java.clone()),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.json_out {
        let _span = info_span!("json", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_json::codegen_json(hir, vfs),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.json_schema_out {
        let _span = info_span!("json_schema", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_json_schema::codegen_schema(hir, vfs, options.json_schema.clone()),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.xml_out {
        let _span = info_span!("xml", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_xml::codegen_xml(hir, vfs),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.proto_out {
        let _span = info_span!("proto", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_protobuf::codegen_proto(hir),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    if let Some(output_dir) = &options.codegen.typescript_out {
        let _span = info_span!("typescript", output_dir = %output_dir.display()).entered();
        let files = invoke_backend(
            output_dir,
            || ic_codegen_typescript::codegen_typescript(hir, options.typescript.clone()),
            options.purge_dirs,
        )?;
        info!(files = files.len(), "generated");
        generated.extend(files);
    }

    Ok(generated)
}

fn invoke_backend(
    output_dir: &std::path::Path,
    backend_fn: impl FnOnce() -> Vec<File>,
    purge_dirs: bool,
) -> Result<Vec<File>, util::Error> {
    let dir = std::path::absolute(output_dir)?;
    if purge_dirs {
        util::safe_purge(&dir)?;
        std::fs::create_dir_all(&dir)?;
    }

    let files = backend_fn()
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
            tracing::debug!(file = %path.display(), bytes = source.len(), "generated");
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

fn init_tracing(level: &str) {
    let level = match level {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::TRACE,
    };

    fmt()
        .with_writer(|| IgnoreBrokenPipe(std::io::stderr()))
        .with_timer(fmt::time::uptime())
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_max_level(level)
        .with_ansi(ic_cli::color::has_colors())
        .init();
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
