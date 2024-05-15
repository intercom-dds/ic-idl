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

use std::collections::HashSet;
use std::path::PathBuf;

use ic_cli::color::Colorize;
use ic_cli::Command;
use ic_preproc::preprocess;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::ThreadPoolBuilder;

mod info;

// TODO: expand Command to return a Vec<(option, description)>
/// Generic IDL code generator
#[derive(Command, Default)]
struct Options {
    /// Only preprocess the files
    #[option(short = 'E', long)]
    preprocessor_only: bool,

    /// Add directory to include search paths
    #[option(short = 'I', long, arg = "dir")]
    include: Vec<PathBuf>,

    /// Define <def> to <val> (or 1 if <val> is omitted)
    #[option(short = 'D', long, arg = "def>=<val")]
    define: Vec<String>,

    /// Unstable flags, see `ic-idl -Z help` for details
    #[option(short = 'Z', arg = "flag")]
    unstable: Vec<String>,

    /// Display version information
    #[option(short = 'V', long)]
    version: bool,

    #[option(positional)]
    files: HashSet<PathBuf>,
}

macro_rules! error {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("ic-idl: {} {}", "error:".red().bold(), $($arg)*);
    }}
}

#[derive(Command, Default)]
struct Unstable {
    /// Print the AST in a tree-like format
    #[option(long)]
    ast_dump: bool,

    /// Dump the AST as JSON
    #[option(long)]
    ast_json: bool,
}

fn unstable_help() {
    let command = Unstable::command();
    let flags = command.format_args(|_| true).join("\n");
    println!("{}", "unstable flags:".yellow());
    println!("{flags}");
    println!("\nRun with `{}`", "ic-idl -Z [FLAG] <files>...".green());
}

fn main() {
    let options = Options::parse();

    if options.version {
        println!("{}", info::version_info());
        return;
    }

    for flag in options.unstable {
        match flag.as_str() {
            "help" => {
                return unstable_help();
            }
            _ => {
                error!("unknown flag -Z{flag}");
                std::process::exit(1);
            }
        }
    }

    if options.files.is_empty() {
        error!("no input files");
        return;
    }

    let threads = std::thread::available_parallelism()
        .map_or(0, |v| v.get())
        .min(options.files.len());

    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    let generated: Result<Vec<_>, _> = options.files.par_iter().map(|f| preprocess(f)).collect();

    let generated = generated.unwrap();

    if options.preprocessor_only {
        println!("{}", generated.join("\n"));
    }
}
