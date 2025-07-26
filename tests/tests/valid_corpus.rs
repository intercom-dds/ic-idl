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

//! Test valid IDL files that should successfully generate code.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn ic_idl_binary() -> PathBuf {
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
    
    assert!(path.exists(), "ic-idl binary not found at {:?}", path);
    path
}

fn project_root() -> PathBuf {
    let mut path = env::current_dir().expect("Failed to get current directory");
    
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

#[test]
fn test_valid_idl_files() {
    // Create test IDL files that are known to be valid
    let test_cases = vec![
        (
            "basic_struct.idl",
            r#"
struct Point {
    double x;
    double y;
};

struct Point3D : Point {
    double z;
};
"#,
        ),
        (
            "enums.idl",
            r#"
enum Color {
    RED,
    GREEN,
    BLUE
};

enum Status {
    OK = 0,
    ERROR = 1,
    PENDING = 2
};
"#,
        ),
        (
            "sequences_maps.idl",
            r#"
typedef sequence<string> StringList;
typedef map<string, long> StringMap;

struct Container {
    StringList items;
    StringMap data;
};
"#,
        ),
        (
            "constants.idl",
            r#"
const long MAX_SIZE = 100;
const double PI = 3.14159;
const string VERSION = "1.0.0";

struct Config {
    long size;
    double scale;
    string name;
};
"#,
        ),
        (
            "arrays.idl",
            r#"
struct Matrix {
    double values[3][3];
};

struct Buffer {
    octet data[1024];
};
"#,
        ),
        (
            "modules.idl",
            r#"
module geometry {
    struct Point {
        double x;
        double y;
    };
    
    struct Rectangle {
        Point topLeft;
        Point bottomRight;
    };
};
"#,
        ),
    ];
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut all_passed = true;
    
    for (filename, content) in test_cases {
        println!("\n=== Testing {} ===", filename);
        
        let idl_path = temp_dir.path().join(filename);
        fs::write(&idl_path, content).expect("Failed to write IDL file");
        
        // Test each backend
        for backend in &["rust", "cpp", "python"] {
            print!("  {} generation... ", backend);
            
            let output_dir = temp_dir.path().join(format!("{}-{}", filename, backend));
            fs::create_dir_all(&output_dir).expect("Failed to create output dir");
            
            let flag = match *backend {
                "rust" => "--rust-out",
                "cpp" => "--cpp-out",
                "python" => "--python-out",
                _ => unreachable!(),
            };
            
            let output = Command::new(ic_idl_binary())
                .args(&[
                    idl_path.to_str().unwrap(),
                    flag,
                    output_dir.to_str().unwrap(),
                ])
                .output()
                .expect("Failed to run ic-idl");
            
            if output.status.success() {
                println!("OK");
                
                // For Rust, try to compile if cargo is available
                if *backend == "rust" && Command::new("cargo").arg("--version").output().is_ok() {
                    print!("    Compiling Rust code... ");
                    
                    let cargo_toml = format!(
                        r#"[package]
name = "test-{}"
version = "0.1.0"
edition = "2021"

[dependencies]
intercom-cts = {{ path = "{}/library/rust/intercom-cts" }}
"#,
                        filename.replace('.', "_"),
                        project_root().display()
                    );
                    
                    fs::write(output_dir.join("Cargo.toml"), cargo_toml).unwrap();
                    
                    let src_dir = output_dir.join("src");
                    fs::create_dir_all(&src_dir).unwrap();
                    
                    fs::write(
                        src_dir.join("lib.rs"),
                        "#![allow(unused)]\ninclude!(\"../lib.rs\");\n"
                    ).unwrap();
                    
                    let check = Command::new("cargo")
                        .args(&["check", "--manifest-path", &output_dir.join("Cargo.toml").to_string_lossy()])
                        .output()
                        .expect("Failed to run cargo check");
                    
                    if check.status.success() {
                        println!("OK");
                    } else {
                        println!("FAILED");
                        eprintln!("Cargo check failed:\n{}", String::from_utf8_lossy(&check.stderr));
                        all_passed = false;
                    }
                }
            } else {
                println!("FAILED");
                eprintln!("ic-idl failed:\nstderr: {}", String::from_utf8_lossy(&output.stderr));
                all_passed = false;
            }
        }
    }
    
    assert!(all_passed, "Some tests failed");
}

#[test]
fn test_real_world_example() {
    // A more complex real-world-like IDL
    let idl_content = r#"
module messaging {
    // Message types
    enum MessageType {
        TEXT,
        IMAGE,
        VIDEO,
        DOCUMENT
    };
    
    // User information
    struct User {
        string id;
        string username;
        string display_name;
        boolean is_online;
    };
    
    // Message structure
    struct Message {
        string id;
        User sender;
        MessageType type;
        string content;
        int64 timestamp;
        sequence<string> attachments;
    };
    
    // Chat room
    struct ChatRoom {
        string id;
        string name;
        sequence<User> participants;
        sequence<Message> recent_messages;
    };
    
    // Service exceptions
    exception UserNotFound {
        string user_id;
    };
    
    exception RoomNotFound {
        string room_id;
    };
    
    exception PermissionDenied {
        string reason;
    };
    
    // Messaging service interface
    interface MessagingService {
        void send_message(in string room_id, in Message message)
            raises (RoomNotFound, PermissionDenied);
        
        sequence<Message> get_messages(in string room_id, in long count)
            raises (RoomNotFound, PermissionDenied);
        
        ChatRoom join_room(in string room_id, in User user)
            raises (RoomNotFound, UserNotFound);
        
        void leave_room(in string room_id, in string user_id)
            raises (RoomNotFound, UserNotFound);
    };
};
"#;
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let idl_path = temp_dir.path().join("messaging.idl");
    fs::write(&idl_path, idl_content).expect("Failed to write IDL file");
    
    println!("\n=== Testing real-world messaging example ===");
    
    // Test all backends
    for backend in &["rust", "cpp", "python"] {
        print!("  {} generation... ", backend);
        
        let flag = match *backend {
            "rust" => "--rust-out",
            "cpp" => "--cpp-out", 
            "python" => "--python-out",
            _ => unreachable!(),
        };
        
        let output = Command::new(ic_idl_binary())
            .args(&[
                idl_path.to_str().unwrap(),
                flag,
                temp_dir.path().to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run ic-idl");
        
        if output.status.success() {
            println!("OK");
        } else {
            println!("FAILED");
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Real-world example failed for {} backend", backend);
        }
    }
}