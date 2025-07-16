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
use ic_idl::{CompileError, Compiler, CompilerOptions, write_generated_files};

mod info;
mod panic;
mod unstable;

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("ic-idl: {} {}", "error:".red().bold(), format!($($arg)*));
    }}
}

#[macro_export]
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
    let generated = match compiler.compile() {
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
        Err(CompileError::Codegen(e)) => {
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
