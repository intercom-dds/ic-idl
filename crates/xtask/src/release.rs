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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ic_cli::color::Colorize;

const INSTALL_NAME: &str = "ic-idl";
const INSTALL_DIR: &str = "install";

/// Build a release archive of ic-idl
#[derive(ic_cli::Command, Default)]
pub struct Options {
    #[option(short, long, arg = "dir")]
    destination: Option<PathBuf>,
}

fn git_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();

    PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim())
}

fn git_branch() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();

    std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .to_string()
}

fn commit_date() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap()
}

fn commit_hash() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap()
}

fn build_info() -> String {
    let date = commit_date();
    let hash = commit_hash();
    let branch = git_branch();
    let version = env!("CARGO_PKG_VERSION");

    format!(
        "\
# Copyright 2024 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

build date: {date}
version: {version}
branch: {branch}
commit: {hash}
"
    )
}

fn library_files() -> Vec<PathBuf> {
    let output = Command::new("git")
        .args(["ls-files", "--full-name", ":/library"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(PathBuf::from)
        .collect()
}

fn install_binary(cwd: &Path, dst: &Path) {
    let status = Command::new("cargo")
        .current_dir(cwd)
        .args([
            "install",
            "--bin=ic-idl",
            "--path=crates/ic-idl",
            "--no-track",
            "--force",
            "--root",
        ])
        .arg(dst)
        .status()
        .unwrap();

    assert!(status.success());
}

pub fn build(options: Options) {
    let root = git_root();
    let install_dir = options
        .destination
        .unwrap_or_else(|| PathBuf::from(&root).join(INSTALL_DIR));

    let files_dir = install_dir.join(INSTALL_NAME);

    if install_dir.exists() {
        // std::fs::remove_dir_all(install_dir).unwrap();
    }

    std::fs::create_dir_all(&files_dir).unwrap();

    // Generate a buildinfo.txt
    let info = build_info();
    fs::write(files_dir.join("buildinfo.txt"), info).unwrap();

    // Copy the license file
    let license = root.join("LICENSE");
    fs::copy(license, files_dir.join("LICENSE")).unwrap();

    // Installl the serialization libraries
    let libraries = library_files();
    for file in libraries {
        let src = root.join(&file);
        let dst = files_dir.join(&file);

        if let Some(v) = dst.parent() {
            std::fs::create_dir_all(v).unwrap();
        }
        fs::copy(src, dst).unwrap();
    }

    // Install the ic-idl binary
    install_binary(&root, &files_dir);

    // Create an archive
    let name = format!("ic-idl_{}.tar.gz", env!("CARGO_PKG_VERSION"));
    let archive = install_dir.join(&name);
    let status = Command::new("tar")
        .current_dir(install_dir)
        .arg("-czvf")
        .arg(&name)
        .arg(INSTALL_NAME)
        .status()
        .unwrap();
    assert!(status.success());

    let abs = std::path::absolute(archive).unwrap();
    println!("{} {}", "installed:".green(), abs.display());
}
