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

//! Shared helpers for backends that emit source code or serialisation metadata.
//!
//! This crate provides common utilities and traits for code generators. It
//! includes:
//!
//! - Case conversion utilities for different naming conventions
//! - Pretty-printing infrastructure for code formatting
//! - File abstraction for generated code and dependencies
//!
//! # Example
//!
//! ```ignore
//! use ic_emit::printer::PrettyPrinter;
//!
//! let mut pp = PrettyPrinter::new();
//! pp.text("// Generated").endl();
//! pp.text("struct Example {").indent().endl();
//! pp.text("value: i32,").endl();
//! pp.dedent().text("}").endl();
//! ```
//!
//! When writing backends, the [`Twine`](printer::Twine) helper together with
//! the [`w!`](crate::w) macro offer a convenient way to stream fragments into
//! an output buffer. `Twine` mirrors block delimiters and indentation
//! automatically:
//!
//! ```ignore
//! use ic_emit::{printer::Twine, w};
//!
//! let mut out = Twine::new();
//! w!(out, "pub struct Example {\n");
//! w!(out, "pub value: i32,\n");
//! w!(out, "}\n");
//!
//! let source = out.finish();
//! ```

use std::path::PathBuf;

/// Case conversion utilities for different naming conventions.
pub mod case;

/// Pretty-printing utilities for code generation.
pub mod printer;

mod ffi;

/// Represents a file in the code generation output.
#[derive(Debug)]
pub enum File {
    /// A dependency file that should be tracked but not generated.
    Dep(String),

    /// A generated source file with its path and contents.
    Generated { path: PathBuf, source: String },
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            File::Dep(v) => write!(f, "dep:{v}"),
            File::Generated { path, .. } => write!(f, "gen:{}", path.display()),
        }
    }
}
