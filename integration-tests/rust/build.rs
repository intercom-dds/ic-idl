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

use std::path::PathBuf;

use intercom_build::Codegen;

fn main() {
    let idl_files: Vec<_> = std::fs::read_dir("../corpus")
        .expect("corpus dir")
        .map(|res| res.map(|e| e.path()).expect("corpus IDL file path"))
        .collect();

    let idl_compiler = std::env::var_os("IDL_COMPILER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../../target/debug/ic-idl"));

    // Rebuild if corpus or ic-idl binary changes
    println!("cargo::rerun-if-changed=../corpus/");
    println!("cargo::rerun-if-changed={}", idl_compiler.display());
    println!("cargo::rerun-if-env-changed=IDL_COMPILER");

    Codegen::new("corpus")
        .executable(idl_compiler)
        .include("../corpus")
        .input(&idl_files)
        .generate()
        .expect("Generated corpus IDL");
}
