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

#![allow(unused, clippy::doc_markdown)]

use std::path::{Path, PathBuf};

use ic_cli::color::Colorize;
use ic_cli::convert::{self, ConvertError};
use ic_cli::Command;

use crate::warn;

intercom_cts::bitmask! {
    #[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub Warnings: u32 {
        DEPRECATED = 1 << 0,
        ANNOTATION = 1 << 1,
        UNKNOWN_ANNOTATION = 1 << 2,
        PEDANTIC = 1 << 3,
        ERROR = 1 << 4,
    }
}

/// Generic IDL code generator
#[derive(Command, Default)]
pub struct Options {
    /// Only run the preprocessor
    #[option(short = 'E', long)]
    pub preprocessor_only: bool,

    /// Do not generate code for included files
    #[option(short = 'H', long)]
    pub no_header_follow: bool,

    /// Add directory to include search paths
    #[option(short = 'I', long, arg = "dir")]
    pub include: Vec<String>,

    /// Define preprocessor directive <def> to <val>
    #[option(short = 'D', long, arg = "def>=<val")]
    pub define: Vec<String>,

    /// Output list of files to be generated
    #[option(short, long)]
    pub list: bool,

    /// Erase output directories before emitting code
    #[option(long)]
    pub purge_dirs: bool,

    /// Do not parse Doxy-like comments
    #[option(long)]
    pub ignore_comments: bool,

    /// Enable a warning, see `-W help` for details
    #[option(short = 'W', arg = "lint")]
    pub warn: Warnings,

    /// Unstable flags, see `-Z help` for details
    #[option(short = 'Z', arg = "flag")]
    pub unstable: Unstable,

    /// Display version information
    #[option(short = 'V', long)]
    pub version: bool,

    #[option(positional)]
    pub files: Vec<PathBuf>,

    #[option(section = "c++ options")]
    pub cpp: CppOptions,

    #[option(section = "rust options")]
    pub rust: RustOptions,

    #[option(section = "python options")]
    pub python: PythonOptions,

    #[option(section = "idl options")]
    pub idl: IdlOptions,

    #[option(section = "backends")]
    pub codegen: CodegenOptions,
}

#[derive(Command, Default, Debug)]
pub struct CppOptions {
    /// Generate scoped enums
    #[option(long)]
    pub scoped_enums: bool,

    /// Use access functions instead of direct member access
    #[option(long)]
    pub access_functions: bool,

    /// Do not generate ostream operators for serialization
    #[option(long)]
    pub no_stream_op: bool,

    /// Generate formatting specializations for fmtlib
    #[option(long)]
    pub use_fmt: bool,

    /// Use dllexport symbol
    #[option(long, arg = "sym")]
    pub dll_export: Option<String>,

    /// Use <ext> as file extension for C++ headers
    #[option(long, arg = "ext")]
    pub header_ext: Option<String>,

    /// Store header files inside a subfolder
    #[option(long, arg = "dir")]
    pub header_subfolder: Option<String>,

    /// Add a filename prefix to generated files
    #[option(long, arg = "sym")]
    pub file_prefix: Option<String>,
}

#[derive(Command, Default)]
pub struct RustOptions {
    /// Do not rename generated types
    #[option(long)]
    pub no_rename: bool,

    /// Annotate all types with `#[must_use]`
    #[option(long)]
    pub must_use: bool,
}

#[derive(Command, Default)]
pub struct PythonOptions {
    /// Rename all types to conform to PEP-8
    #[option(long)]
    pub use_pep8: bool,

    /// Postfix to use for global modules
    #[option(long)]
    pub global_postfix: Option<String>,
}

#[derive(Command, Default)]
pub struct IdlOptions {
    /// Output Doxygen-compatible IDL files
    #[option(long)]
    pub idl_doxygen: bool,

    /// Expand @DDSService interfaces
    #[option(long)]
    pub idl_expand: bool,
}

#[derive(Command, Default)]
#[allow(clippy::struct_field_names)]
pub struct CodegenOptions {
    /// Generate C++ files in <dir>
    #[option(long, arg = "dir")]
    pub cpp_out: Option<PathBuf>,

    /// Generate Rust files in <dir>
    #[option(long, arg = "dir")]
    pub rust_out: Option<PathBuf>,

    /// Generate Python files in <dir>
    #[option(long, arg = "dir")]
    pub python_out: Option<PathBuf>,

    /// Generate IDL files in <dir>
    #[option(long, arg = "dir")]
    pub idl_out: Option<PathBuf>,

    /// Generate Protobuf files in <dir>
    #[option(long, arg = "dir")]
    pub proto_out: Option<PathBuf>,

    /// Generate JSON files in <dir>
    #[option(long, arg = "dir")]
    pub json_out: Option<PathBuf>,

    /// Generate JSON Schema files in <dir>
    #[option(long, arg = "dir")]
    pub json_schema_out: Option<PathBuf>,
}

#[derive(Command, Default)]
pub struct Unstable {
    /// Dump out the AST exactly as it was parsed
    #[option(long)]
    pub ast_dump: bool,

    /// Dump out the type-resolved IR
    #[option(long)]
    pub hir_dump: bool,

    /// Print the ptree in a tree-like format
    #[option(long)]
    pub ptree_dump: bool,

    /// Dump out the preprocessed IDL tokens
    #[option(long)]
    pub token_dump: bool,
}

impl convert::Convert for Unstable {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut this = Self::default();
        for arg in input {
            match arg.as_str() {
                "ast-dump" => this.ast_dump = true,
                "hir-dump" => this.hir_dump = true,
                "ptree-dump" => this.ptree_dump = true,
                "token-dump" => this.token_dump = true,
                "help" => crate::unstable::unstable_help(),
                _ => {
                    return Err(ConvertError::InvalidValue(format!(
                        "unknown unstable option '{}'",
                        format!("-Z{arg}").yellow(),
                    )));
                }
            };
        }
        Ok(this)
    }
}

impl convert::Convert for Warnings {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut bits = Warnings::nil();

        for arg in input {
            let (arg, is_negated) = if let Some(arg) = arg.strip_prefix("no-") {
                (arg, true)
            } else {
                (arg.as_str(), false)
            };

            let bit = match arg {
                "all" => Warnings::all(),
                "deprecated" => Warnings::DEPRECATED,
                "annotation" => Warnings::ANNOTATION,
                "unknown-annotation" => Warnings::UNKNOWN_ANNOTATION,
                "pedantic" => Warnings::PEDANTIC,
                "error" => Warnings::ERROR,
                "help" => todo!(),
                _ => {
                    warn!("unknown warning '{}'", format!("-W{arg}").yellow());
                    continue;
                }
            };

            if is_negated {
                bits.unset(bit);
            } else {
                bits.set(bit);
            }
        }
        Ok(bits)
    }
}
