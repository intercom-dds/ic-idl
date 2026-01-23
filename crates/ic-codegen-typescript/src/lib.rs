// Copyright 2025 KONGSBERG
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

use ic_cli::Command;
use ic_emit::File;
use ic_hir_xform::{Convention, Target, rename};

#[rustfmt::skip]
const KEYWORDS: &[&str] = &[
    // Hard reserved words
    "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete", "do",
    "else", "enum", "export", "extends", "false", "finally", "for", "function", "if", "import",
    "in", "instanceof", "new", "null", "return", "super", "switch", "this", "throw", "true", "try",
    "typeof", "var", "void", "while", "with",

    // Strict mode reserved words
    "arguments", "eval", "implements", "interface", "let", "package", "private", "protected",
    "public", "static", "yield",

    // Contextual keywords
    "any", "as", "async", "await", "boolean", "constructor", "declare", "get", "infer", "is",
    "keyof", "module", "namespace", "never", "number", "readonly", "require", "set", "string",
    "symbol", "type", "unique", "unknown",
];

#[derive(Command, Clone, Debug, Default)]
pub struct TypeScriptOptions {
    /// Use bigint for 64-bit integers
    #[option(long)]
    pub use_bigint: bool,
}

#[must_use]
pub fn codegen_typescript(hir: &ic_hir::ResolvedGraph, options: TypeScriptOptions) -> Vec<File> {
    // Squash reopened modules into single definitions
    let hir = ic_hir_xform::squash_modules::transform(hir.clone());

    let target = Target {
        convention: Convention::default(),
        keyword_escape: Some(|ctx| {
            if KEYWORDS.contains(&ctx.name) {
                Some(format!("{}_", ctx.name))
            } else {
                None
            }
        }),
        ..Target::default()
    };

    let hir = rename::transform(hir, &target);
    let generator = codegen::TsGen::new(&hir, options);
    generator.generate()
}
