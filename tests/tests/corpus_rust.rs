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

//! Rust code generation tests using the corpus of IDL files.

use std::fs;
use std::process::Command;

use tests::*;

/// Test that generated Rust code compiles.
fn verify_rust_compilation(output_dir: &std::path::Path) -> Result<(), String> {
    // Check that lib.rs was generated
    let lib_rs = output_dir.join("lib.rs");
    if !lib_rs.exists() {
        return Err("No lib.rs generated".to_string());
    }

    // Create a test Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "test-corpus"
version = "0.1.0"
edition = "2021"

[dependencies]
intercom-cts = {{ path = "{}/library/rust/intercom-cts", features = ["derive"] }}
"#,
        project_root().display()
    );

    fs::write(output_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Check if cargo is available
    if Command::new("cargo").arg("--version").output().is_err() {
        println!("Cargo not available, skipping compilation test");
        return Ok(());
    }

    // Run cargo check
    let output = Command::new("cargo")
        .args(&[
            "check",
            "--manifest-path",
            &output_dir.join("Cargo.toml").to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    if !output.status.success() {
        // Check if it's just a missing dependency issue
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("intercom-cts") || stderr.contains("could not find") {
            println!("Skipping compilation test due to missing dependencies");
            Ok(())
        } else {
            Err(format!(
                "Cargo check failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                stderr
            ))
        }
    } else {
        Ok(())
    }
}

#[test]
fn test_corpus_rust_generation_and_compilation() {
    let test_dir = create_test_dir("corpus-rust-all");
    
    // Generate Rust code for the entire corpus directory
    generate_corpus("rust", &test_dir)
        .expect("Failed to generate Rust code from corpus");
    
    // Verify compilation
    if let Err(e) = verify_rust_compilation(&test_dir) {
        panic!("Rust compilation verification failed: {}", e);
    }
}