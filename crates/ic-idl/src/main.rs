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
        Err(e) => {
            pretty::emit_errors(&e, &vfs);
            error!(
                "aborting due to {} previous error{}",
                e.len(),
                if e.len() > 1 { "s" } else { "" },
            );
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

fn try_main(options: &Options, vfs: &mut SourceMap) -> Result<Vec<File>, Vec<Error>> {
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
    let files = collect_files(&options.files)
        .map_err(|e| e.into_iter().map(Error::Io).collect::<Vec<_>>())?;

    for file in files {
        let ptree = try_parse(options, args.clone(), &file, vfs)?;
        trees.push(ptree);
    }

    try_ptree(options, &trees).map_err(|e| vec![e])
}

// To report as much information as possible at once, we keep going even if we
// meet an error and instead summarize everything at the end. This also applies
// to syntax errors in the input: the parser will attempt to recover so we can
// continue parsing and construct a partial AST.
fn try_parse(
    options: &Options,
    proc: ProcArgs,
    path: &Path,
    vfs: &mut SourceMap,
) -> Result<ParseResult, Vec<Error>> {
    let mut errors = vec![];
    let (ast, err) = ic_parse::from_path(path, proc, vfs).map_err(|e| {
        vec![Error::Custom(format!(
            "failed to open `{}`: {e}",
            path.display().yellow()
        ))]
    })?;
    errors.extend(err.into_iter().map(Into::into));

    if options.unstable.ast_dump {
        println!("{:#?}", ast.tree);
    }

    // Lint the AST
    let report = ic_lint::lint_syntax(&ast.tree, vfs);
    errors.extend(report.diagnostics.into_iter().map(Into::into));

    // Lower the AST to a HIR
    let hir = ic_hir::from_ast(ast.tree.clone());
    if options.unstable.hir_dump {
        ic_hir_tree::emit_tree(&hir);
    }

    // Lint the HIR
    let report = ic_lint::lint_hir(&hir, vfs);
    errors.extend(report.diagnostics.into_iter().map(Into::into));

    // Lower the HIR to a ptree, but only if construction of the HIR succeeded
    if errors.is_empty() && hir.errors.is_empty() {
        // FIXME: in the future we should construct the ptree from the HIR
        let ptree = ic_ptree_lower::from_ast(&ast, vfs);
        Ok(ptree)
    } else {
        errors.extend(hir.errors.into_iter().map(Into::into));
        Err(errors)
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
            &options.codegen.json_schema_out,
            ic_codegen_json_schema::codegen_json_schema,
        ),
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
