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

#![allow(unused)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use config::{Options, Unstable};
use ic_cli::color::Colorize;
use ic_cli::Command;
use ic_preproc::preprocess;

mod config;
mod info;

macro_rules! error {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("ic-idl: {} {}", "error:".red().bold(), format!($($arg)*));
    }}
}

macro_rules! warn {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("ic-idl: {} {}", "warning:".yellow().bold(), format!($($arg)*));
    }}
}

fn unstable_help() {
    let command = Unstable::command();
    let flags = command.format_args(|_| true).join("\n");

    println!("{}", "\nunstable flags:".yellow());
    println!("{flags}");
    println!("\nRun with `{}`\n", "ic-idl -Z [FLAG] <files>...".green());
    println!(
        "{} unstable flags may change at any time in backward-incompatible ways",
        "warning:".yellow(),
    );
}

fn parse_file(options: &Options, path: &Path) -> anyhow::Result<String> {
    let input = if options.preprocessor_skip {
        std::fs::read_to_string(path)?
    } else {
        preprocess(path)?
    };

    Ok(input)
}

fn main() {
    let options = Options::parse();

    if options.version {
        println!("{}", info::version());
        return;
    }

    if !options.unstable.is_empty() {
        unstable_help();
        return;
    }

    if options.files.is_empty() {
        error!("no input files");
        return;
    }

    if let Err(e) = try_main(&options) {
        error!("{e}");
        std::process::exit(1);
    }
}

#[cfg(feature = "hir")]
fn try_main(options: &Options) -> anyhow::Result<()> {
    // For the time being, lexing and parsing happens in two separate stages as
    // debugging them separately is easier. This can be changed later so we
    // instead lazily scan the input as we parse. That should in theory be
    // faster as we (1) avoid collecting the tokens in an intermediate
    // container, and (2) we can error out earlier.
    let input = options.files.iter().next().unwrap();
    let input = std::fs::read_to_string(input)?;
    // let tokens = ic_parse::lexer::scan(&input);
    // println!("tokens: {tokens:#?}");

    let ast = ic_parse::from_str(&input);
    match ast {
        Ok(v) => {
            dbg!(&v.tree);
            dbg!(ic_lint::lint_syntax(&v.tree));
        }
        Err(e) => {
            // TODO: emit summary of errors + warnings
            error!("aborting due to 1 previous error");
        }
    }

    Ok(())
}

#[cfg(not(feature = "hir"))]
fn try_main(options: &Options) -> anyhow::Result<()> {
    let preprocessed = options
        .files
        .iter()
        .map(|f| parse_file(options, f))
        .collect::<Result<Vec<_>, _>>()?;

    if options.preprocessor_only {
        println!("{}", preprocessed.join("\n"));
        return Ok(());
    }

    let parsed = preprocessed
        .iter()
        .map(|v| ic_ptree::parse_idl(v))
        .collect::<Result<Vec<_>, _>>()?;

    let merged = ic_ptree::merge_trees(&parsed);

    if options.ast_dump {
        ic_ptree::ast_dump(&merged);
    }

    if let Some(dir) = &options.csharp_out {
        ic_ptree::codegen_csharp(&merged, dir);
    }

    if let Some(dir) = &options.cpp_out {
        ic_ptree::codegen_cpp(&merged, dir);
    }

    if let Some(dir) = &options.java_out {
        ic_ptree::codegen_java(&merged, dir);
    }

    if let Some(dir) = &options.proto_out {
        ic_ptree::codegen_proto(&merged, dir);
    }

    if let Some(dir) = &options.python_out {
        ic_ptree::codegen_python(&merged, dir);
    }

    if let Some(msg) = merged.diagnostics() {
        warn!("{msg}");
    }

    Ok(())
}
