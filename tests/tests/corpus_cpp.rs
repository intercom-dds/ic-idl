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

//! C++ code generation tests using the corpus of IDL files.

use std::fs;
use std::process::Command;

use tests::*;

/// Test that generated C++ code compiles.
fn verify_cpp_compilation(output_dir: &std::path::Path) -> Result<(), String> {
    // Find all generated header files
    let headers: Vec<_> = fs::read_dir(output_dir)
        .map_err(|e| format!("Failed to read output directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| ext == "h")
        })
        .collect();

    if headers.is_empty() {
        return Err("No C++ header files generated".to_string());
    }

    // Create a test program that includes all headers
    let test_cpp = output_dir.join("test_all.cpp");
    let mut test_content = String::from("// Test that all generated headers compile\n");
    
    for header in &headers {
        test_content.push_str(&format!(
            "#include \"{}\"\n",
            header.file_name().to_string_lossy()
        ));
    }
    
    test_content.push_str("\nint main() { return 0; }\n");
    
    fs::write(&test_cpp, test_content)
        .map_err(|e| format!("Failed to write test.cpp: {}", e))?;

    // Try to compile
    if std::env::var("OUT_DIR").is_ok() {
        // Use cc crate in build.rs context
        let mut build = cc::Build::new();
        build
            .cpp(true)
            .std("c++17")
            .include(output_dir)
            .include(project_root().join("library/cpp/defs"))
            .file(&test_cpp);
        
        // Also compile any generated .cpp files
        for entry in fs::read_dir(output_dir).unwrap().filter_map(Result::ok) {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("cpp") {
                build.file(entry.path());
            }
        }

        match build.try_compile("test_corpus") {
            Ok(_) => Ok(()),
            Err(e) => {
                // Linking errors are OK, we just care about compilation
                if e.to_string().contains("linker") || e.to_string().contains("undefined") {
                    Ok(())
                } else {
                    Err(format!("C++ compilation failed: {}", e))
                }
            }
        }
    } else if Command::new("c++").arg("--version").output().is_ok() {
        // Use direct c++ invocation
        let output = Command::new("c++")
            .args(&[
                "-std=c++17",
                "-c",
                "-I",
                output_dir.to_str().unwrap(),
                "-I",
                &project_root().join("library/cpp/defs").to_string_lossy(),
                "-o",
                &output_dir.join("test_all.o").to_string_lossy(),
                test_cpp.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to run c++ compiler: {}", e))?;

        if !output.status.success() {
            Err(format!(
                "C++ compilation failed:\nstderr: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        } else {
            Ok(())
        }
    } else {
        // No compiler available, skip compilation test
        println!("No C++ compiler available, skipping compilation test");
        Ok(())
    }
}

#[test]
fn test_corpus_cpp_generation_and_compilation() {
    let test_dir = create_test_dir("corpus-cpp-all");
    
    // Generate C++ code for the entire corpus directory
    generate_corpus("cpp", &test_dir)
        .expect("Failed to generate C++ code from corpus");
    
    // Verify compilation
    if let Err(e) = verify_cpp_compilation(&test_dir) {
        panic!("C++ compilation verification failed: {}", e);
    }
}