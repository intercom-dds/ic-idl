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

//! Integration tests for ic-idl code generation.
//!
//! This crate contains tests that verify the full pipeline from IDL files
//! to compilable generated code. Tests use the corpus of IDL files and
//! verify that the generated code actually compiles.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Helper to create a temporary test directory.
pub fn create_test_dir(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-idl-test-{}-{}", name, std::process::id()));

    if path.exists() {
        fs::remove_dir_all(&path).expect("Failed to clean test directory");
    }

    fs::create_dir_all(&path).expect("Failed to create test directory");
    path
}

/// Get the path to the ic-idl binary.
pub fn get_ic_idl_binary() -> PathBuf {
    let mut path = std::env::current_exe().expect("Failed to get current exe path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("ic-idl");
    assert!(path.exists(), "ic-idl binary not found at {path:?}");
    path
}

/// Run ic-idl with the given arguments.
pub fn run_ic_idl(args: &[&str]) -> Output {
    Command::new(get_ic_idl_binary())
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run ic-idl: {e}"))
}

/// Generate code for a specific language.
/// The input can be a single IDL file or a directory containing IDL files.
pub fn generate_code(idl_path: &Path, language: &str, output_dir: &Path) -> Result<(), String> {
    let flag = match language {
        "cpp" => "--cpp-out",
        "rust" => "--rust-out",
        "python" => "--python-out",
        "json" => "--json-out",
        "xml" => "--xml-out",
        "proto" => "--proto-out",
        _ => return Err(format!("Unknown language: {language}")),
    };

    let output = Command::new(get_ic_idl_binary())
        .args([
            idl_path.to_str().unwrap(),
            flag,
            output_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ic-idl: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ic-idl failed:\nstderr: {}\nstdout: {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }

    Ok(())
}

/// Write IDL content to a file.
pub fn write_idl(path: &Path, content: &str) {
    fs::write(path, content).expect("Failed to write IDL file");
}

/// Get the project root directory.
pub fn project_root() -> PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");

    // Walk up until we find Cargo.toml with workspace
    loop {
        if path.join("Cargo.toml").exists() {
            let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
            if content.contains("[workspace]") {
                return path;
            }
        }

        if !path.pop() {
            panic!("Could not find workspace root");
        }
    }
}

/// Get the path to the corpus directory.
pub fn corpus_dir() -> PathBuf {
    project_root().join("tests").join("idl").join("corpus")
}

/// Generate code from the entire corpus directory for a specific backend.
pub fn generate_corpus(backend: &str, output_dir: &Path) -> Result<(), String> {
    generate_code(&corpus_dir(), backend, output_dir)
}
