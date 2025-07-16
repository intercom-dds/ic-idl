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

use ic_cli::{Command, ParseError};
use ic_emit::File;
// Import the error macro from the library
use ic_idl::error;
use ic_idl::{CompileError, Compiler, CompilerOptions, GeneratedFile, write_generated_files};

mod info;
mod panic;
mod unstable;

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
    }

    if options.warn.help {
        unstable::warning_help();
    }

    if options.files.is_empty() {
        error!("no input files");
        return;
    }

    // Install a panic handler to catch failed asserts.
    panic::install_hook();

    // Create and run the compiler
    let mut compiler = Compiler::new(options);
    let ptree = match compiler.compile() {
        Ok(v) => v,
        Err(CompileError::Io(e)) => {
            error!("I/O error: {}", e);
            std::process::exit(1);
        }
        Err(CompileError::Analysis(errors, warning_count)) => {
            // Errors have already been printed by the library
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

    // Dump ptree if requested
    if compiler.options().unstable.ptree_dump {
        ic_ptree_dump::ptree_dump(&ptree);
    }

    // Generate code using backends
    let generated = match generate_code(compiler.options(), &ptree) {
        Ok(files) => files,
        Err(e) => {
            error!("code generation error: {}", e);
            std::process::exit(1);
        }
    };

    // Handle output
    if compiler.options().list {
        for f in &generated {
            println!("{f}");
        }
    } else if let Err(e) = write_generated_files(&generated) {
        error!("failed to write files: {}", e);
        std::process::exit(1);
    }
}

fn generate_code(
    options: &CompilerOptions,
    ptree: &ic_idl::ptree::ParseResult,
) -> Result<Vec<GeneratedFile>, String> {
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
        let dir = std::path::absolute(dir).map_err(|e| format!("Failed to resolve path: {e}"))?;

        if options.purge_dirs {
            std::fs::remove_dir_all(&dir).ok(); // Ignore if doesn't exist
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
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
