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

use std::path::Path;

const GLOBAL_INCLUDES: &[&str] = &[
    "../ic-ptree/cpp/defs",
    "../../external/fmt/defs",
    "../../external/utils/defs",
    "../../library/cpp/defs",
];

const GLOBAL_DEFINES: &[(&str, &str)] = &[("FMT_HEADER_ONLY", "1"), ("FMT_CONSTEVAL", "")];

pub fn build<P>(files: P, includes: P)
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
        .files(&files)
        .includes(includes);

    for (k, v) in GLOBAL_DEFINES {
        compiler.define(k, *v);
    }

    // Enable exceptions for clang-cl
    if compiler.get_compiler().is_like_msvc() {
        compiler.flag("/EHsc");
    }

    // Upgrade warnings to errors in CI pipelines
    if is_ci() {
        compiler.warnings_into_errors(true);
    }

    compiler.compile(env!("CARGO_PKG_NAME"));

    // Rerun if toolchain has changed
    println!("cargo:rerun-if-env-changed=CI");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
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
