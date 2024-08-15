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

#![allow(unused)]

use std::path::{Path, PathBuf};

use ic_cli::{convert, Command};

intercom_cts::bitmask! {
    #[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub WarningBits: u32 {
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

    /// Output list of files to be generated
    #[option(short, long)]
    pub list: bool,

    /// Erase output directories before emitting code
    #[option(long)]
    pub purge_dirs: bool,

    /// Add directory to include search paths
    #[option(short = 'I', long, arg = "dir")]
    pub include: Vec<String>,

    /// Define <def> to <val> (or 1 if <val> is omitted)
    #[option(short = 'D', long, arg = "def>=<val")]
    pub define: Vec<String>,

    /// Enable the specified warning, see `-W help` for details
    #[option(short = 'W', long, arg = "lint")]
    pub warn: Vec<String>,

    /// Unstable flags, see `-Z help` for details
    #[option(short = 'Z', arg = "flag")]
    pub unstable: Vec<String>,

    /// Dump out the IDL tokens
    #[option(long)]
    pub token_dump: bool,

    /// Dump out the AST exactly as it was parsed
    #[option(long)]
    pub ast_dump: bool,

    /// Dump out the type-resolved IR
    #[option(long)]
    pub hir_dump: bool,

    /// Display version information
    #[option(short = 'V', long)]
    pub version: bool,

    #[option(positional)]
    pub files: Vec<PathBuf>,

    // #[section = "backends"]
    pub codegen: CodegenOptions,
}

#[derive(Command, Default)]
#[allow(clippy::struct_field_names)]
pub struct CodegenOptions {
    /// Generate C++ files in <dir>
    #[option(long, arg = "dir")]
    pub cpp_out: Option<PathBuf>,

    /// Generate C++11 files in <dir>
    #[option(long, arg = "dir")]
    pub cpp11_out: Option<PathBuf>,

    /// Generate Rust files in <dir>
    #[option(long, arg = "dir")]
    pub rust_out: Option<PathBuf>,

    /// Generate Java files in <dir>
    #[option(long, arg = "dir")]
    pub java_out: Option<PathBuf>,

    /// Generate C# files in <dir>
    #[option(long, arg = "dir")]
    pub csharp_out: Option<PathBuf>,

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

    /// Generate XML files in <dir>
    #[option(long, arg = "dir")]
    pub xml_out: Option<PathBuf>,
}

#[derive(Command, Default)]
pub struct Unstable {
    /// Print the HIR in a tree-like format
    #[option(long)]
    pub hir_pretty: bool,

    /// Dump out the AST exactly as it was parsed
    #[option(long)]
    pub ast_dump: bool,

    /// Print the ptree in a tree-like format
    #[option(long)]
    pub ptree_dump: bool,

    /// Dump the ptree as JSON
    #[option(long)]
    pub ptree_json: bool,
}

impl convert::Convert for WarningBits {
    fn from_result(input: &[String]) -> convert::Result<Self> {
        let mut bits = WarningBits::nil();

        for arg in input {
            let (arg, is_negated) = if let Some(arg) = arg.strip_prefix("no-") {
                (arg, true)
            } else {
                (arg.as_str(), false)
            };

            let bit = match arg {
                "deprecated" => WarningBits::DEPRECATED,
                "annotation" => WarningBits::ANNOTATION,
                "unknown-annotation" => WarningBits::UNKNOWN_ANNOTATION,
                "pedantic" => WarningBits::PEDANTIC,
                "ERROR" => WarningBits::ERROR,
                _ => Err(convert::ConvertError::InvalidValue(format!(
                    "unknown warning '{arg}'"
                )))?,
            };

            if is_negated {
                bits.set(bit);
            } else {
                bits.unset(bit);
            }
        }
        Ok(bits)
    }
}
