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

#![allow(unused, clippy::print_stdout, clippy::print_stderr)]

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use config::{
    CodegenOptions, CppOptions, IdlOptions, Options, PythonOptions, RustOptions, Unstable,
};
use ic_cli::color::Colorize;
use ic_cli::{Command, ParseError};
use ic_preproc::ProcArgs;
use ic_ptree::ParseResult;
use ic_vfs::SourceMap;
// use ic_preproc::preprocess;

mod config;
mod info;
mod panic;
mod pretty;
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
        eprintln!("{} {}", "warning:".yellow().bold(), format!($($arg)*));
    }}
}

fn main() {
    let result = Options::command()
        .split_flags(false)
        .align_sections(true)
        .section("c++ options", CppOptions::command())
        .section("rust options", RustOptions::command())
        .section("python options", PythonOptions::command())
        .section("idl options", IdlOptions::command())
        .section("backends", CodegenOptions::command())
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

    let options = Options {
        codegen: CodegenOptions::from_result(&result),
        cpp: CppOptions::from_result(&result),
        rust: RustOptions::from_result(&result),
        python: PythonOptions::from_result(&result),
        idl: IdlOptions::from_result(&result),
        ..Options::from_result(&result)
    };

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

    let generated = match try_main(&options) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };

    if options.list {
        for f in generated {
            println!("{f}");
        }
    }
}

enum File {
    Dep(String),
    Generated(String),
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            File::Dep(v) => write!(f, "dep:{v}"),
            File::Generated(v) => write!(f, "gen:{v}"),
        }
    }
}

fn parse_file(options: &Options, path: &Path) -> anyhow::Result<String> {
    let input = std::fs::read_to_string(path)?;
    // preprocess(path, &options.define, &options.include)?
    Ok(input)
}

fn collect_files<'a, I>(paths: I) -> anyhow::Result<HashSet<PathBuf>>
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    fn collect(p: &Path, files: &mut HashSet<PathBuf>) -> anyhow::Result<()> {
        let meta = std::fs::metadata(p)?;
        if meta.is_dir() {
            let iter = match std::fs::read_dir(p) {
                Ok(v) => v,
                Err(e) => bail!("couldn't open {}: {e}", p.display()),
            };

            for file in std::fs::read_dir(p).unwrap().flatten() {
                collect(&file.path(), files);
            }
        } else if let Some(ext) = p.extension() {
            if ext.eq_ignore_ascii_case("idl") {
                files.insert(p.to_owned());
            }
        }
        Ok(())
    }

    let mut files = HashSet::new();
    for path in paths {
        if std::fs::metadata(path).map_or(false, |v| v.is_dir()) {
            collect(path, &mut files)?;
        } else {
            files.insert(path.clone());
        }
    }
    Ok(files)
}

fn try_main(options: &Options) -> anyhow::Result<Vec<File>> {
    let mut vfs = SourceMap::default();

    let files = collect_files(&options.files)?;
    for file in files {
        // let input = match std::fs::read_to_string(&file) {
        //     Ok(v) => v,
        //     Err(e) => bail!("couldn't read '{}': {e}", file.display().yellow()),
        // };
        let ast = ic_parse::from_path(&file, &mut vfs);
        if options.unstable.token_dump {
            // println!("{:#?}", ic_parse::lexer::scan(&input));
        }

        match ast {
            Ok(v) => {
                // Lint the AST
                let report = ic_lint::lint_syntax(&v.tree);

                if options.unstable.ast_dump {
                    println!("{:#?}", v.tree);
                }

                // Lower the AST to a HIR
                // let hir = ic_hir::lower_ast(v.tree);

                for diag in &report.diagnostics {
                    let mut buf = String::new();
                    // ic_diagnostic::emit_diagnostic(&mut buf, &input, diag);
                    eprintln!("{buf}");
                }

                // if options.unstable.hir_dump {
                //     println!("{hir:#?}");
                // }

                let ptree = ic_ptree::lower_ast(&v.tree);
                try_ptree(options, &ptree)?;
            }
            Err(e) => {
                // pretty::emit_errors(&input, &e);
                error!(
                    "aborting due to {} previous error{}",
                    e.len(),
                    if e.len() > 1 { "s" } else { "" },
                );
            }
        }
    }
    Ok(vec![])
}

fn try_ptree(options: &Options, merged: &ParseResult) -> anyhow::Result<Vec<String>> {
    // let preprocessed = options
    //     .files
    //     .iter()
    //     .map(|f| parse_file(options, f))
    //     .collect::<Result<Vec<_>, _>>()?;

    // if options.preprocessor_only {
    //     println!("{}", preprocessed.join("\n"));
    //     return Ok(vec![]);
    // }

    // let parsed = preprocessed
    //     .iter()
    //     .map(|v| ic_ptree::parse_idl(v))
    //     .collect::<Result<Vec<_>, _>>()?;

    // let merged = ic_ptree::merge_trees(&parsed);

    if options.unstable.ptree_dump {
        ic_ptree_pretty::ptree_dump(merged);
    }

    let backends: &[(_, fn(_, _) -> _)] = &[
        (&options.codegen.cpp_out, ic_codegen_cxx::codegen_cpp),
        (&options.codegen.idl_out, ic_codegen_idl::codegen_idl),
        (&options.codegen.json_out, ic_codegen_json::codegen_json),
        (
            &options.codegen.proto_out,
            ic_codegen_protobuf::codegen_proto,
        ),
        (
            &options.codegen.python_out,
            ic_codegen_python::codegen_python,
        ),
        (&options.codegen.rust_out, ic_codegen_rust::codegen_rust),
    ];

    let mut generated = vec![];
    for (dir, backend) in backends
        .into_iter()
        .filter_map(|(v, t)| v.as_ref().map(|v| (v, t)))
    {
        if options.purge_dirs {
            if let Ok(v) = std::fs::metadata(dir) {
                if v.is_dir() {
                    std::fs::remove_dir_all(dir)?;
                }
            }
        }
        let files = backend(merged, dir);
        generated.extend(files);
    }
    Ok(generated)
}
