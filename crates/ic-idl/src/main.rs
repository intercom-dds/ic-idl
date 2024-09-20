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

use anyhow::anyhow;
use config::Options;
use ic_cli::{Command, ParseError};
use ic_emit::File;
use ic_preproc::ProcArgs;
use ic_ptree::ParseResult;
use ic_vfs::SourceMap;
use tracing_subscriber::filter::LevelFilter;
use util::{collect_files, write_if_changed};

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

    // Configure logging
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(LevelFilter::TRACE)
        .init();

    let generated = match try_main(&options) {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
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

fn try_main(options: &Options) -> anyhow::Result<Vec<File>> {
    let mut vfs = SourceMap::default();

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
    let files = collect_files(&options.files)?;
    for file in files {
        if options.preprocessor_only {
            let (output, _) = ic_preproc::to_string(&file, args.clone())?;
            println!("{output}");
        } else {
            let ast = try_parse(options, args.clone(), &file, &mut vfs)?;
            trees.push(ast);
        }
    }
    try_ptree(options, &trees)
}

fn try_parse(
    options: &Options,
    proc: ProcArgs,
    path: &Path,
    vfs: &mut SourceMap,
) -> anyhow::Result<ParseResult> {
    let ast = ic_parse::from_path(path, proc, vfs);

    match ast {
        Ok(v) => {
            if options.unstable.ast_dump {
                println!("{:#?}", v.tree);
            }

            // Lint the AST
            let report = ic_lint::lint_syntax(&v.tree);

            for diag in &report.diagnostics {
                let mut buf = String::new();

                // TODO: propagate file id here so we don't have to reopen it.
                // this isn't necessarily the correct file either, we need
                // to retrieve the FileId from the error
                let input = std::fs::read_to_string(path).unwrap();
                ic_diagnostic::emit_diagnostic(
                    &mut buf,
                    path.to_string_lossy().as_ref(),
                    &input,
                    diag,
                )?;
                eprintln!("{buf}");
            }

            // Lower the AST to a HIR
            // let hir = ic_hir::lower_ast(v.tree.clone());
            // if options.unstable.hir_dump {
            //     println!("{hir:#?}");
            // }
            Ok(ic_ptree::lower_ast(&v))
        }
        Err(e) => {
            // TODO: collect + join for all files
            pretty::emit_errors(&e, vfs);
            error!(
                "aborting due to {} previous error{}",
                e.len(),
                if e.len() > 1 { "s" } else { "" },
            );
            Err(anyhow!("parsing failed"))
        }
    }
}

fn try_ptree(options: &Options, parsed: &[ParseResult]) -> anyhow::Result<Vec<File>> {
    let merged = ic_ptree::merge_trees(parsed);
    if options.unstable.ptree_dump {
        ic_ptree_pretty::ptree_dump(&merged);
    }

    let backends: &[(_, fn(_) -> _)] = &[
        (&options.codegen.cpp_out, ic_codegen_cxx::codegen_cpp),
        (&options.codegen.idl_out, ic_codegen_idl::codegen_idl),
        (&options.codegen.json_out, ic_codegen_json::codegen_json),
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
