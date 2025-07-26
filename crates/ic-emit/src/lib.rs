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

//! Code generation framework for the IDL compiler.
//!
//! This crate provides common utilities and traits for code generators to
//! produce output in various target languages. It includes:
//!
//! - Case conversion utilities for different naming conventions
//! - Pretty-printing infrastructure for code formatting
//! - File abstraction for generated code and dependencies
//!
//! # Example
//!
//! ```ignore
//! use ic_emit::{File, printer::Printer};
//!
//! let mut printer = Printer::new();
//! printer.line("// Generated code");
//! printer.line("struct Example {");
//! printer.indent();
//! printer.line("field: i32,");
//! printer.dedent();
//! printer.line("}");
//!
//! let file = File::Generated {
//!     path: "example.rs".into(),
//!     source: printer.finish(),
//! };
//! ```

use std::path::PathBuf;

/// Case conversion utilities for different naming conventions.
pub mod case;
mod ffi;
/// Pretty-printing utilities for code generation.
pub mod printer;

/// Represents a file in the code generation output.
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
