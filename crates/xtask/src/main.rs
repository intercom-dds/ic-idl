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

#![allow(clippy::print_stdout, clippy::print_stderr, clippy::large_enum_variant)]

use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use ic_cli::Command;

mod codegen;
mod deny;
mod integration;
mod ipr;
mod release;

fn idl_compiler(workspace_root: &Path, explicit: Option<String>) -> PathBuf {
    let path = if let Some(path) = explicit {
        PathBuf::from(path)
    } else {
        let status = ProcessCommand::new("cargo")
            .current_dir(workspace_root)
            .args(["build", "-p", "ic-idl"])
            .status()
            .unwrap_or_else(|e| {
                eprintln!("error: failed to run cargo: {e}");
                std::process::exit(1);
            });

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        workspace_root
            .join("target")
            .join("debug")
            .join("ic-idl")
            .with_extension(std::env::consts::EXE_EXTENSION)
    };

    if !path.exists() {
        eprintln!("error: ic-idl not found at {}", path.display());
        std::process::exit(1);
    }

    path.canonicalize().unwrap()
}

/// Polyfill for building and releasing ic-idl
#[derive(Command)]
enum Commands {
    Ipr(ipr::Options),
    Deny(deny::Options),
    Release(release::Options),
    Codegen(codegen::Options),
    Integration(integration::Options),
}

fn main() {
    let result = Commands::command().hide_flags(true, true).parse();
    let cmd = Commands::from_result(&result);

    match cmd {
        Commands::Ipr(v) => ipr::check(v),
        Commands::Release(v) => release::build(v),
        Commands::Deny(_) => deny::check(),
        Commands::Codegen(v) => codegen::run(v),
        Commands::Integration(v) => integration::run(&v),
    }
}
