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
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use ic_cli::color::Colorize;

const HEADER_TEMPLATE: &str = r#"{c} Copyright 2025 KONGSBERG
{c}
{c} Redistribution and use in source and binary forms, with or without
{c} modification, are permitted provided that the following conditions are met:
{c}
{c} 1. Redistributions of source code must retain the above copyright notice,
{c}    this list of conditions and the following disclaimer.
{c}
{c} 2. Redistributions in binary form must reproduce the above copyright notice,
{c}    this list of conditions and the following disclaimer in the documentation
{c}    and/or other materials provided with the distribution.
{c}
{c} 3. Neither the name of the copyright holder nor the names of its contributors
{c}    may be used to endorse or promote products derived from this software
{c}    without specific prior written permission.
{c}
{c} THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
{c} ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
{c} WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
{c} DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
{c} FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
{c} DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
{c} SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
{c} CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
{c} OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
{c} OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"#;

/// Check that all files have an IPR header
#[derive(Copy, Clone, ic_cli::Command, Default)]
pub struct Options {
    /// Add missing IPR headers
    #[option(short, long)]
    fix: bool,
}

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
    let p = &Path::new(name);
    let extension = p
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("snap"));

    !extension
        && !name.starts_with("external/fmt")
        && !name.contains("Cargo.lock")
        && name != "LICENSE"
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

fn comment_str(ext: Option<&OsStr>) -> Option<&str> {
    match ext?.to_str()? {
        "rs" | "cpp" | "h" | "ic" | "c" | "idl" | "toml" => Some("//"),
        _ => None,
    }
}

fn add_header(path: &Path) -> Option<()> {
    let comment = comment_str(path.extension())?;
    let contents = std::fs::read_to_string(path).ok()?;
    let header = format!("{}\n", HEADER_TEMPLATE.replace("{c}", comment));
    let new_contents = format!("{header}{contents}");
    std::fs::write(path, new_contents).ok()
}

pub fn check(options: Options) {
    let (mut missing, count) = find_missing();
    if options.fix {
        missing.retain(|path| add_header(path).is_none());
    }

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
