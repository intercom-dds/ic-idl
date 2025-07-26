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
use std::path::PathBuf;
use std::process::Command;

fn ic_idl_binary() -> PathBuf {
    // Get the path to the ic-idl binary built by cargo
    let mut path = std::env::current_exe()
        .expect("Failed to get current exe path")
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();
    
    path.push("ic-idl");
    
    // On Windows, add .exe extension
    if cfg!(windows) {
        path.set_extension("exe");
    }
    
    path
}

fn run_ic_idl(args: &[&str]) -> std::process::Output {
    let binary = ic_idl_binary();
    
    Command::new(&binary)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run ic-idl: {e}"))
}

fn setup_test_dir(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-idl-test-{name}"));

    if path.exists() {
        fs::remove_dir_all(&path).expect("Failed to clean test directory");
    }
    
    fs::create_dir_all(&path).expect("Failed to create test directory");
    path
}

#[test]
fn test_cpp_codegen_basic() {
    let test_dir = setup_test_dir("cpp-basic");
    let idl_file = test_dir.join("test.idl");
    
    // Write a simple IDL file
    fs::write(
        &idl_file,
        r"
struct Point {
    double x;
    double y;
};

enum Color {
    RED,
    GREEN,
    BLUE
};

interface Shape {
    void move(in Point delta);
    Point get_center();
};
",
    )
    .expect("Failed to write IDL file");

    // Run ic-idl to generate C++ code
    let output = run_ic_idl(&[
        &idl_file.to_string_lossy(),
        "--cpp-out", &test_dir.to_string_lossy(),
    ]);
    
    assert!(output.status.success(), 
        "ic-idl failed: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Check that expected files were generated
    assert!(test_dir.join("test.h").exists(), "Header file not generated");
    assert!(test_dir.join("test.cpp").exists(), "Source file not generated");
    
    // Try to compile the generated C++ code
    let cpp_output = Command::new("c++")
        .args([
            "-std=c++17",
            "-c",
            "-I", &test_dir.to_string_lossy(),
            "-I", "library/cpp/defs",
            "-o", &test_dir.join("test.o").to_string_lossy(),
            &test_dir.join("test.cpp").to_string_lossy(),
        ])
        .output();
    
    if let Ok(output) = cpp_output {
        assert!(output.status.success(),
            "C++ compilation failed: {}",
            String::from_utf8_lossy(&output.stderr));
    } else {
        eprintln!("Warning: C++ compiler not available, skipping compilation test");
    }
}

#[test]
fn test_rust_codegen_basic() {
    let test_dir = setup_test_dir("rust-basic");
    let idl_file = test_dir.join("test.idl");
    
    // Write a simple IDL file
    fs::write(
        &idl_file,
        r"
struct Message {
    string content;
    uint32 timestamp;
};

enum Status {
    OK,
    ERROR,
    PENDING
};
",
    )
    .expect("Failed to write IDL file");

    // Run ic-idl to generate Rust code
    let output = run_ic_idl(&[
        &idl_file.to_string_lossy(),
        "--rust-out", &test_dir.to_string_lossy(),
    ]);
    
    assert!(output.status.success(), 
        "ic-idl failed: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Check that expected file was generated
    assert!(test_dir.join("lib.rs").exists(), "Rust file not generated");
    
    // Create a simple Cargo.toml to test compilation
    let project_root = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let cargo_toml = format!(r#"
[package]
name = "test-rust-codegen"
version = "0.1.0"
edition = "2021"

[dependencies]
intercom-cts = {{ path = "{}/library/rust/intercom-cts" }}
"#, project_root.display());
    
    fs::write(test_dir.join("Cargo.toml"), cargo_toml)
        .expect("Failed to write Cargo.toml");
    
    // Create src directory and lib.rs that includes generated code
    let src_dir = test_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    
    fs::write(src_dir.join("lib.rs"), r#"
include!("../lib.rs");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message {
            content: "Hello".to_string(),
            timestamp: 12345,
        };
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.timestamp, 12345);
    }
    
    #[test]
    fn test_enum_values() {
        assert_eq!(Status::OK as i32, 0);
        assert_eq!(Status::ERROR as i32, 1);
        assert_eq!(Status::PENDING as i32, 2);
    }
}
"#).expect("Failed to write lib.rs");
    
    // Try to compile the generated Rust code
    let cargo_output = Command::new("cargo")
        .args([
            "check",
            "--manifest-path",
            &test_dir.join("Cargo.toml").to_string_lossy(),
        ])
        .output();
    
    if let Ok(output) = cargo_output {
        assert!(output.status.success(),
            "Rust compilation failed: {}",
            String::from_utf8_lossy(&output.stderr));
    } else {
        eprintln!("Warning: Cargo not available, skipping Rust compilation test");
    }
}

#[test]
fn test_complex_types() {
    let test_dir = setup_test_dir("complex-types");
    let idl_file = test_dir.join("complex.idl");
    
    // Write a more complex IDL file with various features
    fs::write(
        &idl_file,
        r"
module test {
    typedef sequence<string> StringList;
    typedef map<string, long> StringToIntMap;
    
    struct ComplexStruct {
        StringList names;
        StringToIntMap counts;
        double values[10];
    };
    
    union Result switch(short) {
        case 0: string error_message;
        case 1: ComplexStruct data;
        default: long error_code;
    };
    
    exception DataError {
        string reason;
        long error_code;
    };
    
    interface DataService {
        Result process_data(in ComplexStruct input) raises (DataError);
    };
};
",
    )
    .expect("Failed to write IDL file");

    // Test with multiple backends
    for (lang, flag) in &[("cpp", "--cpp-out"), ("rust", "--rust-out"), ("python", "--python-out")] {
        let output = run_ic_idl(&[
            &idl_file.to_string_lossy(),
            flag, &test_dir.to_string_lossy(),
        ]);
        
        if !output.status.success() {
            eprintln!("ic-idl failed for {lang}");
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        assert!(output.status.success(), 
            "ic-idl failed for {}: {}",
            lang,
            String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn test_error_handling() {
    let test_dir = setup_test_dir("error-handling");
    let idl_file = test_dir.join("invalid.idl");
    
    // Write an invalid IDL file
    fs::write(
        &idl_file,
        r"
struct Invalid {
    unknown_type field;  // This should cause an error
};
",
    )
    .expect("Failed to write IDL file");

    // Run ic-idl and expect it to fail
    let output = run_ic_idl(&[
        &idl_file.to_string_lossy(),
        "--cpp-out", &test_dir.to_string_lossy(),
    ]);
    
    assert!(!output.status.success(), "ic-idl should have failed on invalid input");
    assert!(!String::from_utf8_lossy(&output.stderr).is_empty(), 
        "Error output should not be empty");
}

#[test]
fn test_include_paths() {
    let test_dir = setup_test_dir("include-paths");
    let include_dir = test_dir.join("include");
    fs::create_dir_all(&include_dir).expect("Failed to create include directory");
    
    // Write a base IDL file in include directory
    fs::write(
        include_dir.join("base.idl"),
        r"
struct BaseStruct {
    long id;
};
",
    )
    .expect("Failed to write base IDL file");

    // Write main IDL file that includes the base
    let main_idl = test_dir.join("main.idl");
    fs::write(
        &main_idl,
        r"
#include <base.idl>

struct DerivedStruct : BaseStruct {
    string name;
};
",
    )
    .expect("Failed to write main IDL file");

    // Run ic-idl with include path
    let output = run_ic_idl(&[
        &main_idl.to_string_lossy(),
        "-I", &include_dir.to_string_lossy(),
        "--cpp-out", &test_dir.to_string_lossy(),
    ]);
    
    assert!(output.status.success(), 
        "ic-idl failed with includes: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_annotations() {
    let test_dir = setup_test_dir("annotations");
    let idl_file = test_dir.join("annotated.idl");
    
    // Write IDL with annotations
    fs::write(
        &idl_file,
        r"
@range(min=0, max=100)
typedef long Percentage;

struct Config {
    @optional
    string description;
    
    @range(min=1, max=65535)
    uint16 server_port;
    
    @optional
    boolean legacy_mode;
};
",
    )
    .expect("Failed to write IDL file");

    // Run ic-idl - annotations should be handled properly
    let output = run_ic_idl(&[
        &idl_file.to_string_lossy(),
        "--rust-out", &test_dir.to_string_lossy(),
    ]);
    
    assert!(output.status.success(), 
        "ic-idl failed with annotations: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
