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
use std::process::Command;

use ic_cli::color::Colorize;

/// Check that all files have an IPR header
#[derive(ic_cli::Command, Default)]
pub struct Options;

fn git_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();

    PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim())
}

fn tracked_files() -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-files", "--full-name", ":/"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

fn check_file(contents: &str) -> bool {
    contents
        .lines()
        .take(20)
        .any(|v| v.contains("Copyright") && v.contains("KONGSBERG"))
}

fn whitelist(name: &str) -> bool {
    !name.ends_with(".json")
        && !name.ends_with(".snap")
        && !name.starts_with("external/fmt")
        && !name.contains("Cargo.lock")
}

fn find_missing() -> (HashSet<PathBuf>, usize) {
    let root = git_root();
    let files = tracked_files();
    let mut missing = HashSet::new();

    for f in files.iter().filter(|v| whitelist(v)) {
        let path = root.join(f);
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if !check_file(&contents) {
                missing.insert(path);
            }
        }
    }
    (missing, files.len())
}

pub fn check() {
    let (missing, count) = find_missing();
    for f in &missing {
        eprintln!(
            "{}:0: missing license text at beginning of file",
            f.display(),
        );
    }

    if !missing.is_empty() {
        eprintln!(
            "{} files {}, {} {}",
            count - missing.len(),
            "ok".green(),
            missing.len(),
            "failed".red(),
        );
        std::process::exit(1);
    }
    println!("{count} files {}", "ok".green());
}
