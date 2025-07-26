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

//! Integration tests that run ic-idl on all test corpus files and verify
//! the generated code compiles successfully.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use walkdir::WalkDir;

/// Get the path to the ic-idl binary
fn ic_idl_binary() -> PathBuf {
    // The binary should be in the same target directory as our test binary
    let mut path = env::current_exe()
        .expect("Failed to get current exe")
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();
    
    path.push("ic-idl");
    
    if cfg!(windows) {
        path.set_extension("exe");
    }
    
    if !path.exists() {
        panic!("ic-idl binary not found at {:?}. Run 'cargo build --bin ic-idl' first.", path);
    }
    
    path
}

/// Get the project root directory
fn project_root() -> PathBuf {
    let mut path = env::current_dir().expect("Failed to get current directory");
    
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

/// Find all IDL files in the test corpus
fn find_corpus_files() -> Vec<PathBuf> {
    let root = project_root();
    let corpus_dir = root.join("tests").join("corpus");
    
    let mut files = Vec::new();
    
    for entry in WalkDir::new(&corpus_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("idl") {
            files.push(path.to_path_buf());
        }
    }
    
    files.sort();
    files
}

/// Run ic-idl on a file with the given backend
fn run_ic_idl(idl_file: &Path, backend: &str, output_dir: &Path) -> Result<(), String> {
    let flag = match backend {
        "cpp" => "--cpp-out",
        "rust" => "--rust-out",
        "python" => "--python-out",
        _ => return Err(format!("Unknown backend: {}", backend)),
    };
    
    let output = Command::new(ic_idl_binary())
        .args(&[
            idl_file.to_str().unwrap(),
            flag,
            output_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute ic-idl: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "ic-idl failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

/// Test that generated Rust code compiles
fn test_rust_compilation(generated_dir: &Path, idl_name: &str) -> Result<(), String> {
    // Create a test Cargo project
    let cargo_toml = format!(
        r#"[package]
name = "test-{}"
version = "0.1.0"
edition = "2021"

[dependencies]
intercom-cts = {{ path = "{}/library/rust/intercom-cts" }}
"#,
        idl_name,
        project_root().display()
    );
    
    fs::write(generated_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
    
    // Create src directory
    let src_dir = generated_dir.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;
    
    // Create a lib.rs that includes the generated code
    let lib_content = format!(
        r#"#![allow(unused)]
include!("../lib.rs");

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_types_exist() {{
        // Just ensure the code compiles
    }}
}}
"#
    );
    
    fs::write(src_dir.join("lib.rs"), lib_content)
        .map_err(|e| format!("Failed to write lib.rs: {}", e))?;
    
    // Run cargo check
    let output = Command::new("cargo")
        .args(&["check", "--manifest-path", &generated_dir.join("Cargo.toml").to_string_lossy()])
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "Cargo check failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

/// Test that generated C++ code compiles
fn test_cpp_compilation(generated_dir: &Path, idl_name: &str) -> Result<(), String> {
    // Find the generated header file
    let header = generated_dir.join(format!("{}.h", idl_name));
    if !header.exists() {
        return Err("Generated header file not found".to_string());
    }
    
    // Create a simple test program
    let test_cpp = generated_dir.join("test.cpp");
    let test_content = format!(
        r#"#include "{}.h"

// Just test that we can include the header
int main() {{
    return 0;
}}
"#,
        idl_name
    );
    
    fs::write(&test_cpp, test_content)
        .map_err(|e| format!("Failed to write test.cpp: {}", e))?;
    
    // Try to compile (just syntax check, don't link)
    let output = Command::new("c++")
        .args(&[
            "-std=c++17",
            "-c",
            "-I", generated_dir.to_str().unwrap(),
            "-I", &project_root().join("library/cpp/defs").to_string_lossy(),
            "-o", &generated_dir.join("test.o").to_string_lossy(),
            test_cpp.to_str().unwrap(),
        ])
        .output();
    
    match output {
        Ok(output) if !output.status.success() => {
            Err(format!(
                "C++ compilation failed:\nstderr: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to run c++ compiler: {}", e)),
    }
}

#[test]
fn test_corpus_rust_generation() {
    let files = find_corpus_files();
    assert!(!files.is_empty(), "No corpus files found");
    
    let mut failures = Vec::new();
    
    for idl_file in &files {
        let file_name = idl_file.file_stem().unwrap().to_str().unwrap();
        println!("Testing Rust generation for: {}", idl_file.display());
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Generate Rust code
        match run_ic_idl(idl_file, "rust", temp_dir.path()) {
            Ok(_) => {
                // Test compilation if cargo is available
                if Command::new("cargo").arg("--version").output().is_ok() {
                    if let Err(e) = test_rust_compilation(temp_dir.path(), file_name) {
                        failures.push(format!("{}: Rust compilation failed: {}", file_name, e));
                    }
                }
            }
            Err(e) => {
                failures.push(format!("{}: {}", file_name, e));
            }
        }
    }
    
    if !failures.is_empty() {
        panic!("Rust generation/compilation failures:\n{}", failures.join("\n"));
    }
}

#[test]
fn test_corpus_cpp_generation() {
    let files = find_corpus_files();
    assert!(!files.is_empty(), "No corpus files found");
    
    let mut failures = Vec::new();
    
    for idl_file in &files {
        let file_name = idl_file.file_stem().unwrap().to_str().unwrap();
        println!("Testing C++ generation for: {}", idl_file.display());
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Generate C++ code
        match run_ic_idl(idl_file, "cpp", temp_dir.path()) {
            Ok(_) => {
                // Test compilation if c++ compiler is available
                if Command::new("c++").arg("--version").output().is_ok() {
                    if let Err(e) = test_cpp_compilation(temp_dir.path(), file_name) {
                        failures.push(format!("{}: C++ compilation failed: {}", file_name, e));
                    }
                }
            }
            Err(e) => {
                failures.push(format!("{}: {}", file_name, e));
            }
        }
    }
    
    if !failures.is_empty() {
        panic!("C++ generation/compilation failures:\n{}", failures.join("\n"));
    }
}

#[test]
fn test_corpus_python_generation() {
    let files = find_corpus_files();
    assert!(!files.is_empty(), "No corpus files found");
    
    let mut failures = Vec::new();
    
    for idl_file in &files {
        let file_name = idl_file.file_stem().unwrap().to_str().unwrap();
        println!("Testing Python generation for: {}", idl_file.display());
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Generate Python code
        if let Err(e) = run_ic_idl(idl_file, "python", temp_dir.path()) {
            failures.push(format!("{}: {}", file_name, e));
        } else {
            // Check that Python file was generated
            let py_file = temp_dir.path().join(format!("{}.py", file_name));
            if !py_file.exists() {
                failures.push(format!("{}: Python file not generated", file_name));
            }
        }
    }
    
    if !failures.is_empty() {
        panic!("Python generation failures:\n{}", failures.join("\n"));
    }
}

#[test]
fn test_specific_features() {
    // Test specific IDL features that might be problematic
    let test_cases = vec![
        ("annotations.idl", "Tests annotation handling"),
        ("collections.idl", "Tests sequences and maps"),
        ("constants.idl", "Tests constant expressions"),
        ("expressions.idl", "Tests complex expressions"),
        ("typedef.idl", "Tests type aliases"),
        ("valuetype.idl", "Tests valuetype feature"),
    ];
    
    for (filename, description) in test_cases {
        println!("Testing {}: {}", filename, description);
        
        let idl_path = project_root().join("tests").join("corpus").join(filename);
        if !idl_path.exists() {
            eprintln!("Warning: {} not found, skipping", filename);
            continue;
        }
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test all backends
        for backend in &["rust", "cpp", "python"] {
            if let Err(e) = run_ic_idl(&idl_path, backend, temp_dir.path()) {
                panic!("{} failed for {} backend: {}", filename, backend, e);
            }
        }
    }
}