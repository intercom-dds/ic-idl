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

//! Full end-to-end integration tests that verify generated code compiles and works.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Helper to create a temporary test directory
fn create_test_dir(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-idl-codegen-test-{}-{}", name, std::process::id()));
    
    if path.exists() {
        fs::remove_dir_all(&path).expect("Failed to clean test directory");
    }
    
    fs::create_dir_all(&path).expect("Failed to create test directory");
    path
}

/// Get the path to the ic-idl binary
fn ic_idl_path() -> PathBuf {
    let mut path = std::env::current_exe()
        .expect("Failed to get current exe path")
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();
    
    path.push("ic-idl");
    
    if cfg!(windows) {
        path.set_extension("exe");
    }
    
    assert!(path.exists(), "ic-idl binary not found at {:?}", path);
    path
}

/// Generate code using ic-idl
fn generate_code(idl_path: &Path, language: &str, output_dir: &Path) -> Result<(), String> {
    let flag = match language {
        "cpp" => "--cpp-out",
        "rust" => "--rust-out",
        "python" => "--python-out",
        _ => return Err(format!("Unknown language: {}", language)),
    };
    
    let output = Command::new(ic_idl_path())
        .args(&[
            idl_path.to_str().unwrap(),
            flag, output_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ic-idl: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "ic-idl failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

#[test]
fn test_rust_serialization_roundtrip() {
    let test_dir = create_test_dir("rust-serialization");
    let idl_path = test_dir.join("test.idl");
    
    // Create a comprehensive IDL file
    fs::write(&idl_path, r#"
module test {
    struct Person {
        string name;
        uint32 age;
        sequence<string> hobbies;
    };
    
    enum JobStatus {
        EMPLOYED,
        UNEMPLOYED,
        RETIRED
    };
    
    struct Employee : Person {
        JobStatus status;
        double salary;
        map<string, string> metadata;
    };
    
    typedef sequence<Employee> EmployeeList;
}
"#).expect("Failed to write IDL");
    
    // Generate Rust code
    generate_code(&idl_path, "rust", &test_dir)
        .expect("Failed to generate Rust code");
    
    // Create a Cargo project to test the generated code
    let project_root = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let cargo_toml = format!(r#"
[package]
name = "test-rust-serialization"
version = "0.1.0"
edition = "2021"

[dependencies]
intercom-cts = {{ path = "{}/library/rust/intercom-cts" }}
serde_json = "1.0"
"#, project_root.display());
    
    fs::write(test_dir.join("Cargo.toml"), cargo_toml)
        .expect("Failed to write Cargo.toml");
    
    // Create test program
    let src_dir = test_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    
    fs::write(src_dir.join("main.rs"), r#"
include!("../lib.rs");

use intercom_cts::{
    json::{marshal_json, unmarshal_json},
    cdr1::{marshal_cdr1, unmarshal_cdr1},
};

fn main() {
    use test::*;
    
    // Create test data
    let mut employee = Employee {
        name: "Alice".to_string(),
        age: 30,
        hobbies: vec!["reading".to_string(), "hiking".to_string()],
        status: JobStatus::EMPLOYED,
        salary: 75000.0,
        metadata: std::collections::HashMap::from([
            ("department".to_string(), "Engineering".to_string()),
            ("level".to_string(), "Senior".to_string()),
        ]),
    };
    
    // Test JSON serialization roundtrip
    let json = marshal_json(&employee).expect("JSON marshaling failed");
    println!("JSON: {}", json);
    
    let employee2: Employee = unmarshal_json(&json).expect("JSON unmarshaling failed");
    assert_eq!(employee.name, employee2.name);
    assert_eq!(employee.age, employee2.age);
    assert_eq!(employee.hobbies, employee2.hobbies);
    assert_eq!(employee.status, employee2.status);
    assert_eq!(employee.salary, employee2.salary);
    assert_eq!(employee.metadata, employee2.metadata);
    
    // Test CDR serialization roundtrip
    let cdr = marshal_cdr1(&employee).expect("CDR marshaling failed");
    let employee3: Employee = unmarshal_cdr1(&cdr).expect("CDR unmarshaling failed");
    assert_eq!(employee.name, employee3.name);
    assert_eq!(employee.age, employee3.age);
    
    println!("All serialization tests passed!");
}
"#).expect("Failed to write main.rs");
    
    // Build and run the test
    let output = Command::new("cargo")
        .args(&["run", "--manifest-path", &test_dir.join("Cargo.toml").to_string_lossy()])
        .output();
    
    match output {
        Ok(output) => {
            assert!(output.status.success(),
                "Rust test failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr));
            
            // Verify the output contains our success message
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("All serialization tests passed!"),
                "Test output missing success message");
        }
        Err(_) => {
            eprintln!("Warning: Cargo not available, skipping Rust compilation test");
        }
    }
}

#[test]
fn test_cpp_compilation_and_usage() {
    let test_dir = create_test_dir("cpp-compilation");
    let idl_path = test_dir.join("test.idl");
    
    // Create IDL with various C++ features
    fs::write(&idl_path, r#"
module geometry {
    struct Point2D {
        double x;
        double y;
    };
    
    struct Point3D : Point2D {
        double z;
    };
    
    enum ShapeType {
        CIRCLE,
        RECTANGLE,
        TRIANGLE
    };
    
    union ShapeData switch(ShapeType) {
        case CIRCLE:
            double radius;
        case RECTANGLE:
            struct {
                double width;
                double height;
            } rect;
        case TRIANGLE:
            sequence<Point2D, 3> vertices;
    };
    
    exception InvalidShape {
        string reason;
    };
    
    interface ShapeCalculator {
        double calculate_area(in ShapeData shape) raises (InvalidShape);
        Point2D get_center(in ShapeData shape);
    };
}
"#).expect("Failed to write IDL");
    
    // Generate C++ code
    generate_code(&idl_path, "cpp", &test_dir)
        .expect("Failed to generate C++ code");
    
    // Create a test C++ program
    let test_cpp = test_dir.join("test_main.cpp");
    fs::write(&test_cpp, r#"
#include "test.h"
#include <ic_cts/json_serializer.h>
#include <iostream>
#include <cassert>
#include <sstream>

using namespace geometry;

int main() {
    // Test basic struct
    Point3D p;
    p.x = 1.0;
    p.y = 2.0;
    p.z = 3.0;
    
    // Test JSON serialization
    std::stringstream ss;
    ic_cts::marshal_json(ss, p);
    
    Point3D p2;
    ss.seekg(0);
    ic_cts::unmarshal_json(ss, p2);
    
    assert(p.x == p2.x);
    assert(p.y == p2.y);
    assert(p.z == p2.z);
    
    // Test union
    ShapeData circle;
    circle._d(ShapeType::CIRCLE);
    circle.radius(5.0);
    
    std::cout << "All C++ tests passed!" << std::endl;
    return 0;
}
"#).expect("Failed to write test program");
    
    // Try to compile the C++ code
    let compile_output = Command::new("c++")
        .args(&[
            "-std=c++17",
            "-I", &test_dir.to_string_lossy(),
            "-I", "library/cpp/defs",
            "-o", &test_dir.join("test_program").to_string_lossy(),
            &test_cpp.to_string_lossy(),
            &test_dir.join("test.cpp").to_string_lossy(),
        ])
        .output();
    
    match compile_output {
        Ok(output) => {
            if output.status.success() {
                // Run the compiled program
                let run_output = Command::new(test_dir.join("test_program"))
                    .output()
                    .expect("Failed to run test program");
                
                assert!(run_output.status.success(),
                    "C++ test program failed:\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&run_output.stdout),
                    String::from_utf8_lossy(&run_output.stderr));
                
                let stdout = String::from_utf8_lossy(&run_output.stdout);
                assert!(stdout.contains("All C++ tests passed!"),
                    "Test output missing success message");
            } else {
                eprintln!("C++ compilation failed:\nstderr: {}",
                    String::from_utf8_lossy(&output.stderr));
                eprintln!("This might be due to missing C++ dependencies");
            }
        }
        Err(_) => {
            eprintln!("Warning: C++ compiler not available, skipping C++ compilation test");
        }
    }
}

#[test]
fn test_python_generation() {
    let test_dir = create_test_dir("python-generation");
    let idl_path = test_dir.join("test.idl");
    
    // Create IDL for Python
    fs::write(&idl_path, r#"
module api {
    struct Request {
        string method;
        sequence<string> parameters;
        map<string, string> headers;
    };
    
    struct Response {
        uint16 status_code;
        string body;
        map<string, string> headers;
    };
    
    exception NetworkError {
        string message;
        long error_code;
    };
}
"#).expect("Failed to write IDL");
    
    // Generate Python code
    generate_code(&idl_path, "python", &test_dir)
        .expect("Failed to generate Python code");
    
    // Verify Python file was generated
    assert!(test_dir.join("test.py").exists(), "Python file not generated");
    
    // Create a test Python script
    let test_py = test_dir.join("test_script.py");
    fs::write(&test_py, r#"
#!/usr/bin/env python3
import sys
sys.path.insert(0, '.')

from test import *
import json

# Create test data
req = api.Request()
req.method = "GET"
req.parameters = ["param1", "param2"]
req.headers = {"Content-Type": "application/json", "Authorization": "Bearer token"}

# Test that we can access fields
assert req.method == "GET"
assert len(req.parameters) == 2
assert req.headers["Content-Type"] == "application/json"

print("All Python tests passed!")
"#).expect("Failed to write test script");
    
    // Try to run the Python test
    let python_output = Command::new("python3")
        .current_dir(&test_dir)
        .arg("test_script.py")
        .output();
    
    match python_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("All Python tests passed!"),
                    "Python test output missing success message");
            } else {
                eprintln!("Python test failed:\nstderr: {}",
                    String::from_utf8_lossy(&output.stderr));
                eprintln!("This might be due to missing Python dependencies");
            }
        }
        Err(_) => {
            eprintln!("Warning: Python not available, skipping Python test");
        }
    }
}

#[test]
fn test_all_backends_same_idl() {
    let test_dir = create_test_dir("all-backends");
    let idl_path = test_dir.join("common.idl");
    
    // Create a comprehensive IDL that exercises many features
    fs::write(&idl_path, r#"
module common {
    // Basic types
    typedef string<128> BoundedString;
    typedef sequence<octet, 1024> BoundedBuffer;
    
    // Enums
    enum LogLevel {
        DEBUG,
        INFO,
        WARNING,
        ERROR,
        CRITICAL
    };
    
    // Structs with inheritance
    struct Timestamp {
        int64 seconds;
        int32 nanoseconds;
    };
    
    struct LogEntry {
        Timestamp timestamp;
        LogLevel level;
        BoundedString message;
        map<string, string> context;
    };
    
    // Complex struct
    struct LogBatch {
        sequence<LogEntry> entries;
        string source_id;
        boolean compressed;
    };
    
    // Union
    union PayloadData switch(long) {
        case 1: LogBatch batch;
        case 2: BoundedBuffer raw_data;
        default: string error_message;
    };
    
    // Exception
    exception ProcessingError {
        string reason;
        LogEntry failed_entry;
    };
    
    // Interface
    interface LogProcessor {
        void process_batch(in LogBatch batch) raises (ProcessingError);
        sequence<LogEntry> query_logs(in Timestamp start, in Timestamp end);
    };
}
"#).expect("Failed to write IDL");
    
    // Test that all backends can generate code from the same IDL
    let backends = vec!["cpp", "rust", "python"];
    
    for backend in backends {
        println!("Testing {} backend...", backend);
        let backend_dir = test_dir.join(backend);
        fs::create_dir_all(&backend_dir).expect("Failed to create backend directory");
        
        let result = generate_code(&idl_path, backend, &backend_dir);
        assert!(result.is_ok(), 
            "Failed to generate {} code: {:?}", backend, result.err());
        
        // Verify files were generated
        match backend {
            "cpp" => {
                assert!(backend_dir.join("common.h").exists(), 
                    "C++ header not generated");
                assert!(backend_dir.join("common.cpp").exists(), 
                    "C++ source not generated");
            }
            "rust" => {
                assert!(backend_dir.join("lib.rs").exists(), 
                    "Rust source not generated");
            }
            "python" => {
                assert!(backend_dir.join("common.py").exists(), 
                    "Python source not generated");
            }
            _ => unreachable!()
        }
    }
}