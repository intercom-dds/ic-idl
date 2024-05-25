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

use std::path::{Path, PathBuf};
use std::process::Command;

const PRE_COMMIT_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/etc/pre-commit.sh");

const PRE_COMMIT_DST: &str = "hooks/pre-commit";

/// Install pre-commit Git hooks
#[derive(ic_cli::Command, Default)]
#[command = "setup"]
pub struct Options {}

fn git_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .unwrap();

    PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim())
}

pub fn install() {
    let root = git_root();
    let dst = Path::new(&root).join(PRE_COMMIT_DST);

    if dst.exists() {
        eprintln!("error: pre-commit hook already exists. refusing to overwrite.");
        std::process::exit(1);
    }

    match std::fs::copy(PRE_COMMIT_SRC, dst) {
        Ok(_) => println!("pre-commit hook succesfully installed"),
        Err(e) => eprintln!("error: failed to install pre-commit hook: {e}"),
    }
}
