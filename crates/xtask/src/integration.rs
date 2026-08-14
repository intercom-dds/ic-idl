// Copyright 2026 KONGSBERG
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

use std::collections::HashSet;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run integration tests
#[derive(ic_cli::Command, Default)]
pub struct Options {
    /// Languages to test: python, typescript, csharp, cpp, java, rust
    #[option(short, long, arg = "lang")]
    pub lang: HashSet<String>,
}

fn git_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();

    PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim())
}

fn run_command(mut cmd: Command, command_name: &str) {
    let status = match cmd.status() {
        Ok(status) => status,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("error: {command_name} not found");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: failed to run {command_name}: {e}");
            std::process::exit(1);
        }
    };

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_typescript_tests(integration_dir: &Path) {
    let mut cmd = Command::new("bun");
    cmd.current_dir(integration_dir.join("typescript"))
        .arg("test");
    run_command(cmd, "bun");
}

fn run_csharp_tests(integration_dir: &Path) {
    let mut cmd = Command::new("dotnet");
    cmd.current_dir(integration_dir.join("csharp"))
        .args(["test", "--verbosity", "minimal"]);
    run_command(cmd, "dotnet");
}

fn run_cpp_tests(integration_dir: &Path) {
    let jobs = std::thread::available_parallelism()
        .map_or(4, NonZero::get)
        .to_string();

    let cpp_dir = integration_dir.join("cpp");
    let mut cmd = Command::new("cmake");
    cmd.current_dir(&cpp_dir).args(["-S", ".", "-B", "build"]);
    run_command(cmd, "cmake");

    let mut cmd = Command::new("cmake");
    cmd.current_dir(&cpp_dir)
        .args(["--build", "build", "--target", "test", "-j", &jobs]);
    run_command(cmd, "cmake");
}

fn run_python_tests(integration_dir: &Path) {
    let mut cmd = Command::new("uv");
    cmd.current_dir(integration_dir.join("python"))
        .args(["run", "pytest", "-n", "auto"]);
    run_command(cmd, "uv");
}

fn run_java_tests(integration_dir: &Path) {
    let mut cmd = Command::new("mvn");
    cmd.current_dir(integration_dir.join("java")).args(["test"]);
    run_command(cmd, "mvn");
}

fn run_rust_tests(integration_dir: &Path) {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(integration_dir.join("rust")).args(["test"]);
    run_command(cmd, "cargo");
}

pub fn run(opts: &Options) {
    let integration_dir = git_root().join("integration-tests");
    let all_languages = ["python", "typescript", "csharp", "cpp", "java", "rust"];
    let languages: HashSet<_> = if opts.lang.is_empty() || opts.lang.contains("all") {
        all_languages.iter().map(ToString::to_string).collect()
    } else {
        opts.lang.clone()
    };

    for lang in &languages {
        match lang.as_str() {
            "python" | "py" => run_python_tests(&integration_dir),
            "typescript" | "ts" => run_typescript_tests(&integration_dir),
            "csharp" | "cs" => run_csharp_tests(&integration_dir),
            "cpp" | "c++" => run_cpp_tests(&integration_dir),
            "java" => run_java_tests(&integration_dir),
            "rust" | "rs" => run_rust_tests(&integration_dir),
            _ => {
                eprintln!("error: unknown or unsupported language '{lang}'");
                eprintln!("supported languages: python, typescript, csharp, cpp, java, rust, all");
                std::process::exit(1);
            }
        }
    }

    println!("Integration tests passed!");
}
