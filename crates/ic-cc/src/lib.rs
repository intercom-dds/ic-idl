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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![allow(clippy::print_stdout)]

use std::path::Path;

const GLOBAL_INCLUDES: &[&str] = &[
    "../ic-ptree/cpp/defs",
    "../../external/utils/defs",
    "../../library/cpp/defs",
];

const SYSTEM_INCLUDES: &[&str] = &["../../external/fmt/defs"];

const GLOBAL_DEFINES: &[(&str, &str)] = &[
    ("FMT_HEADER_ONLY", "1"),
    ("FMT_CONSTEVAL", ""),
    ("_CRT_SECURE_NO_WARNINGS", "1"),
];

pub fn build<P>(name: &str, files: P)
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    let files: Vec<_> = files.into_iter().collect();
    let mut compiler = cc::Build::new();
    compiler
        .cpp(true)
        .includes(GLOBAL_INCLUDES)
        .extra_warnings(true)
        .flag_if_supported("-Wpedantic")
        .flag_if_supported("-Wno-unused-function")
        .files(&files);

    for (k, v) in GLOBAL_DEFINES {
        compiler.define(k, *v);
    }

    // Add system includes
    for sys in SYSTEM_INCLUDES {
        compiler.flag_if_supported(format!("-isystem{sys}"));
        compiler.flag_if_supported(format!("/external:I{sys}"));
    }

    // Enable exceptions for clang-cl and enable C++17 support
    if compiler.get_compiler().is_like_msvc() {
        compiler.flag("/EHsc");
        compiler.flag("/std:c++17");
    } else {
        compiler.flag("-std=c++17");
    }

    // Upgrade warnings to errors in CI pipelines
    if is_ci() {
        compiler.warnings_into_errors(true);
    }

    compiler.compile(name);

    // Rerun if toolchain has changed
    println!("cargo:rerun-if-env-changed=CI");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");

    for f in files {
        println!("cargo:rerun-if-changed={}", f.as_ref().display());
    }
}

fn is_ci() -> bool {
    if let Ok(var) = std::env::var("CI") {
        if let Ok(v) = var.parse::<bool>() {
            v
        } else {
            var.parse::<usize>().unwrap_or(0) != 0
        }
    } else {
        false
    }
}
