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

#![allow(clippy::unused_self)]

use ic_cli::Command;
use ic_emit::File;
use ic_hir_xform::rename::{self, IdentifierKind, MemberKind, Target};

mod codegen;
mod deps;
mod scalars;
mod structs;
mod type_info;
mod unions;
mod valuetypes;

#[rustfmt::skip]
const KEYWORDS: &[&str] = &[
    "alignas", "alignof", "and", "and_eq", "asm", "auto", "bitand", "bitor", "bool", "break",
    "case", "catch", "char", "char16_t", "char32_t", "char8_t", "class", "co_await", "co_return",
    "co_yield", "compl", "concept", "const", "const_cast", "consteval", "constexpr", "constinit",
    "continue", "contract_assert", "decltype", "default", "delete", "do", "double", "dynamic_cast",
    "else", "enum", "explicit", "export", "extern", "false", "float", "for", "friend", "goto",
    "if", "import", "inline", "int", "long", "module", "mutable", "namespace", "new", "noexcept",
    "not", "not_eq", "nullptr", "operator", "or", "or_eq", "private", "protected", "public",
    "register", "reinterpret_cast", "requires", "return", "short", "signed", "sizeof", "static",
    "static_assert", "static_cast", "struct", "switch", "template", "this", "thread_local",
    "throw", "true", "try", "typedef", "typeid", "typename", "union", "unsigned", "using",
    "virtual", "void", "volatile", "wchar_t", "while", "xor", "xor_eq",

    // not keywords(?), operators, "identifiers with meaning", etc
    "typeof", "assert",
];

#[derive(Command, Debug, Default, Clone)]
pub struct CppOptions {
    /// Generate scoped enums
    #[option(long)]
    pub scoped_enums: bool,

    /// Use access functions instead of direct member access
    #[option(long)]
    pub access_functions: bool,

    /// Do not generate code for included files
    #[option(long)]
    pub no_header_follow: bool,

    /// Do not generate ostream operators for serialization
    #[option(long)]
    pub no_stream_op: bool,

    /// Use const char* instead of `std::string_view` for constants
    #[option(long)]
    pub char_ptr_constants: bool,

    /// Generate formatting specializations for fmtlib
    #[option(long)]
    pub use_fmt: bool,

    /// Use <sym> as dllexport symbol
    #[option(long, arg = "sym")]
    pub dll_export: Option<String>,

    /// Use <ext> as file extension for C++ headers
    #[option(long, arg = "ext")]
    pub header_ext: Option<String>,

    /// Store header files inside a subdirectory
    #[option(long, arg = "dir")]
    pub header_subdir: Option<String>,
}

fn escape_cpp_keyword(ctx: rename::RenameContext) -> Option<String> {
    if KEYWORDS.contains(&ctx.name) {
        Some(format!("{}_", ctx.name))
    } else {
        if ctx.kind == IdentifierKind::Member(MemberKind::Exception) && ctx.name == "what" {
            return Some("what_".into());
        }

        None
    }
}

/// # Panics
///
/// May panic if some of the passed string parameters contain a NUL byte.
#[must_use]
pub fn codegen_cpp(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: CppOptions,
) -> Vec<File> {
    let target = Target {
        keyword_escape: Some(escape_cpp_keyword),
        ..Target::default()
    };

    let hir = ic_hir_xform::rename::transform(hir.clone(), &target);
    codegen::CppGen::new(&hir, source_map, options).generate()
}
