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

use std::path::{Path, PathBuf};

use ic_cli::color::Colorize;
use ic_emit::File;
use ic_preproc::ProcArgs;
use ic_ptree::ParseResult;
use ic_vfs::SourceMap;

mod pretty;

pub struct Config {
    pub defines: Vec<String>,
    pub includes: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum Error {
    Diagnostic(Box<ic_diagnostic::Diag>),
    Parse(Box<ic_parse::Error>),
    Preproc(ic_preproc::ProcError),
    Io(std::io::Error),
    Custom(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ic_diagnostic::Diag> for Error {
    fn from(value: ic_diagnostic::Diag) -> Self {
        Self::Diagnostic(Box::new(value))
    }
}

impl From<ic_parse::Error> for Error {
    fn from(value: ic_parse::Error) -> Self {
        Self::Parse(Box::new(value))
    }
}

pub fn generate_rust(config: Config) {
    let mut vfs = SourceMap::default();
    let generated = match try_main(config, &mut vfs) {
        Ok(v) => v,
        Err(e) => {
            pretty::emit_errors(&e, &vfs);
            panic!(
                "aborting due to {} previous error{}",
                e.len(),
                if e.len() > 1 { "s" } else { "" },
            );
        }
    };

    for f in generated {
        match f {
            File::Dep(_) => (),
            File::Generated { path, source } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::write(path, &source).unwrap();
            }
        }
    }
}

fn try_main(config: Config, vfs: &mut SourceMap) -> Result<Vec<File>, Vec<Error>> {
    let defines = config.defines.iter().map(|v| {
        v.split_once('=')
            .map_or_else(|| (v.as_str(), None), |(k, v)| (k, Some(v)))
    });

    let args = ProcArgs::default()
        .define("__IC_IDL__", None)
        .defines(defines)
        .includes(config.includes)
        .skip_comments(true);

    let mut trees = vec![];
    for file in config.files {
        let file = PathBuf::from(file);
        let ptree = try_parse(args.clone(), &file, vfs)?;
        trees.push(ptree);
    }

    let merged = ic_ptree::merge_trees(&trees);
    let dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let files = ic_codegen_rust::codegen_rust(&merged)
        .into_iter()
        .map(|v| match v {
            File::Generated { path, source } => File::Generated {
                path: dir.join(path),
                source,
            },
            File::Dep(_) => v,
        })
        .collect();

    Ok(files)
}

// To report as much information as possible at once, we keep going even if we
// meet an error and instead summarize everything at the end. This also applies
// to syntax errors in the input: the parser will attempt to recover so we can
// continue parsing and construct a partial AST.
fn try_parse(proc: ProcArgs, path: &Path, vfs: &mut SourceMap) -> Result<ParseResult, Vec<Error>> {
    let mut errors = vec![];
    let (ast, err) = ic_parse::from_path(path, proc, vfs).map_err(|e| {
        vec![Error::Custom(format!(
            "failed to open `{}`: {e}",
            path.display().yellow()
        ))]
    })?;
    errors.extend(err.into_iter().map(Into::into));

    // Lower the HIR to a ptree, but only if construction of the HIR succeeded
    if errors.is_empty() {
        // FIXME: in the future we should construct the ptree from the HIR
        let ptree = ic_ptree_lower::from_ast(&ast, vfs);
        Ok(ptree)
    } else {
        Err(errors)
    }
}
