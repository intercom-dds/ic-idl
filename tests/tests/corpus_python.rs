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

//! Python code generation tests using the corpus of IDL files.

use std::fs;
use std::process::Command;

use tests::*;

/// Test that generated Python code has valid syntax.
fn verify_python_syntax(output_dir: &std::path::Path) -> Result<(), String> {
    // Find all generated Python files and modules
    let mut python_files = Vec::new();
    
    fn find_python_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    // Check if it's a Python package (has __init__.py)
                    if path.join("__init__.py").exists() {
                        find_python_files(&path, files);
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
                    files.push(path);
                }
            }
        }
    }
    
    find_python_files(output_dir, &mut python_files);
    
    if python_files.is_empty() {
        return Err("No Python files generated".to_string());
    }

    // Check if Python is available
    if Command::new("python3").arg("--version").output().is_err() {
        println!("Python3 not available, skipping syntax check");
        return Ok(());
    }

    // Check syntax of each file
    for py_file in &python_files {
        let output = Command::new("python3")
            .args(&["-m", "py_compile"])
            .arg(py_file)
            .output()
            .map_err(|e| format!("Failed to run python3: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Python syntax check failed for {:?}:\nstderr: {}",
                py_file,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(())
}

#[test]
fn test_corpus_python_generation_and_syntax() {
    let test_dir = create_test_dir("corpus-python-all");
    
    // Generate Python code for the entire corpus directory
    generate_corpus("python", &test_dir)
        .expect("Failed to generate Python code from corpus");
    
    // Verify syntax
    if let Err(e) = verify_python_syntax(&test_dir) {
        panic!("Python syntax verification failed: {}", e);
    }
}