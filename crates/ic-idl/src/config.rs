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

use ic_cli::Command;

#[derive(Command, Default)]
struct PpOptions {
    /// Only preprocess the files
    #[option(short = 'E', long)]
    preprocessor_only: bool,

    /// Do not preprocess the files
    #[option(short = 'X', long)]
    preprocessor_skip: bool,

    /// Add directory to include search paths
    #[option(short = 'I', long, arg = "dir")]
    include: Vec<PathBuf>,

    /// Define <def> to <val> (or 1 if <val> is omitted)
    #[option(short = 'D', long, arg = "def>=<val")]
    define: Vec<String>,
}

#[derive(Command, Default)]
struct ParseOptions {
    /// Do not generate code for included files
    #[option(short = 'H', long)]
    no_header_follow: bool,

    /// Enable specified warning
    #[option(short = 'W', long, arg = "lint")]
    warn: Vec<String>,

    #[option(positional)]
    _files: Vec<PathBuf>,
}

intercom_cts::bitmask! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub Warnings: u32 {
        DEPRECATED = 1 << 0,
        ANNOTATION = 1 << 1,
        UNKNOWN_ANNOTATION = 1 << 2,
        PEDANTIC = 1 << 3,
        ERROR = 1 << 4,
        HELP = 1 << 5,
    }
}

#[derive(Command, Default)]
#[allow(clippy::struct_field_names)]
struct CodegenOptions {
    /// Generate C++ files
    #[option(long, arg = "dir")]
    cpp_out: PathBuf,

    /// Generate Rust files
    #[option(long, arg = "dir")]
    rust_out: PathBuf,

    /// Generate Protobuf files
    #[option(long, arg = "dir")]
    proto_out: PathBuf,

    /// Generate Python files
    #[option(long, arg = "dir")]
    python_out: PathBuf,

    /// Generate IDL files
    #[option(long, arg = "dir")]
    idl_out: PathBuf,

    /// Generate JSON files
    #[option(long, arg = "dir")]
    json_out: PathBuf,

    /// Generate JSON Schema files
    #[option(long, arg = "dir")]
    schema_out: PathBuf,

    /// Generate TypeScript files
    #[option(long, arg = "dir")]
    ts_out: PathBuf,

    /// Generate XML schema files
    #[option(long, arg = "dir")]
    xml_out: PathBuf,
}

#[derive(Command, Default)]
struct GlobalOptions {
    // #[option(merge)]
    // preprocessor: PpOptions,

    // #[option(merge)]
    // parser: ParseOptions,

    // #[option(merge)]
    // codegen: CodegenOptions,
    /// Output list of files to be generated
    #[option(short, long)]
    list: bool,

    /// Empty output directories before emitting code
    #[option(long)]
    purge_dirs: bool,

    /// Display version information
    #[option(short = 'V', long)]
    version: bool,

    /// Unstable flags, see `-Z help` for details
    // TODO: trait Parseable? So that we can manually impl that for UnstableFlags
    // and make the parser accept T: Parseable types.
    #[option(short = 'Z', arg = "flag")]
    unstable: Vec<String>,
}

/// Generic IDL code generator
#[derive(Command, Default)]
pub struct Options {
    /// Only preprocess the files
    #[option(short = 'E', long)]
    pub preprocessor_only: bool,

    /// Do not preprocess the files
    #[option(short = 'X', long)]
    pub preprocessor_skip: bool,

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

    /// Enable specified warning
    #[option(short = 'W', long, arg = "lint")]
    pub warn: Vec<String>,

    /// Unstable flags, see `-Z help` for details
    #[option(short = 'Z', arg = "flag")]
    pub unstable: Vec<String>,

    /// Dump out the AST exactly as it was parsed
    #[option(long)]
    pub ast_dump: bool,

    /// Dump out the type-resolved IR
    #[option(long)]
    pub hir_dump: bool,

    /// Display version information
    #[option(short = 'V', long)]
    pub version: bool,

    /// Generate Protobuf files in <dir>
    #[option(long, arg = "dir")]
    pub proto_out: Option<PathBuf>,

    /// Generate Java files in <dir>
    #[option(long, arg = "dir")]
    pub java_out: Option<PathBuf>,

    /// Generate C# files in <dir>
    #[option(long, arg = "dir")]
    pub csharp_out: Option<PathBuf>,

    /// Generate C++ files in <dir>
    #[option(long, arg = "dir")]
    pub cpp_out: Option<PathBuf>,

    /// Generate C++11 files in <dir>
    #[option(long, arg = "dir")]
    pub cpp11_out: Option<PathBuf>,

    /// Generate Python files in <dir>
    #[option(long, arg = "dir")]
    pub python_out: Option<PathBuf>,

    /// Generate Rust files in <dir>
    #[option(long, arg = "dir")]
    pub rust_out: Option<PathBuf>,

    /// Generate IDL files in <dir>
    #[option(long, arg = "dir")]
    pub idl_out: Option<PathBuf>,

    #[option(positional)]
    pub files: Vec<PathBuf>,
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
