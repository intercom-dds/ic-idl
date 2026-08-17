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

mod codegen;
mod deps;

use ic_emit::File;
use ic_hir_xform::rename::{self, Target};

#[rustfmt::skip]
const KEYWORDS: &[&str] = &[
    "alignas", "extern", "signed", "alignof", "false", "sizeof", "auto",
    "float", "static", "bool", "for", "static_assert", "break", "goto",
    "struct", "case", "if", "switch", "char", "inline", "thread_local",
    "const", "int", "true", "constexpr", "long", "typedef", "continue",
    "nullptr", "typeof", "default", "register", "typeof_unqual", "do",
    "restrict", "union", "double", "return", "unsigned", "else", "short",
    "void", "enum", "volatile", "while", "_Alignas", "_Decimal32", "_Noreturn",
    "_Alignof", "_Decimal64", "_Static_assert", "_Atomic", "_Decimal128",
    "_Thread_local", "_BitInt", "_Generic", "_Bool", "_Imaginary", "_Complex",
];

const RESERVED_PARAMETERS: &[&str] = &["_self", "_result", "_value", "_error"];

fn escape_c_keyword(ctx: rename::RenameContext) -> Option<String> {
    let reserved = matches!(ctx.kind, rename::IdentifierKind::Parameter)
        && RESERVED_PARAMETERS.contains(&ctx.name);

    if KEYWORDS.contains(&ctx.name) || reserved {
        Some(format!("{}_", ctx.name))
    } else {
        None
    }
}

#[must_use]
pub fn codegen_c(hir: &ic_hir::ResolvedGraph, source_map: &ic_vfs::SourceMap) -> Vec<File> {
    let flattened = ic_hir_xform::flatten::transform(hir.clone(), "_");
    let target = Target {
        keyword_escape: Some(escape_c_keyword),
        moved_defs: flattened.moved_defs,
        ..Target::default()
    };

    let hir = rename::transform(flattened.hir, &target);
    codegen::CGen::new(&hir, source_map).generate()
}
