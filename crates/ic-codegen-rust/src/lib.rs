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

mod codegen;
mod helpers;
mod literals;
mod marshal;

use ic_cli::Command;
use ic_emit::File;
use ic_emit::case::Case;
use ic_hir_xform::rename::{self, Convention};

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

const RUST_CONVENTION: rename::Convention = rename::Convention {
    struct_type: Some(Case::Pascal),
    union_type: Some(Case::Pascal),
    enum_type: Some(Case::Pascal),
    interface: Some(Case::Pascal),
    valuetype: Some(Case::Pascal),
    alias: Some(Case::Pascal),
    bitmask: Some(Case::Pascal),
    bitset: Some(Case::Pascal),
    exception: Some(Case::Pascal),
    annotation: Some(Case::Pascal),
    member: Some(Case::Snake),
    variant: Some(Case::Pascal),
    enumerator: Some(Case::Pascal),
    bit_flag: Some(Case::UpperSnake),
    bitset_field: Some(Case::Snake),
    constant: Some(Case::UpperSnake),
    module: Some(Case::Snake),
    operation: Some(Case::Snake),
    attribute: Some(Case::Snake),
    parameter: Some(Case::Snake),
    name_preprocessor: Some(rename::strip_common_suffixes),
    strip_enum_prefix: true,
};

fn escape_rust_keyword(ctx: rename::RenameContext) -> Option<String> {
    if RUST_KEYWORDS.contains(&ctx.name) {
        Some(format!("{}_", ctx.name))
    } else {
        None
    }
}

#[derive(Copy, Clone, Command, Debug, Default)]
pub struct RustOptions {
    /// Do not rename generated types
    #[option(long)]
    pub no_rename: bool,

    /// Annotate all types with `#[must_use]`
    #[option(long)]
    pub must_use: bool,
}

#[must_use]
pub fn codegen_rust(hir: &ic_hir::ResolvedGraph, options: RustOptions) -> Vec<File> {
    let (hir, original_hir) = prepare_hir(hir, options);
    codegen::RustGen::new(&hir, &original_hir, options).generate()
}

#[must_use]
pub fn codegen_rust_inline(hir: &ic_hir::ResolvedGraph, options: RustOptions) -> String {
    let (hir, original_hir) = prepare_hir(hir, options);
    codegen::RustGen::new(&hir, &original_hir, options).generate_inline()
}

fn prepare_hir(
    hir: &ic_hir::ResolvedGraph,
    options: RustOptions,
) -> (ic_hir::ResolvedGraph, ic_hir::ResolvedGraph) {
    // Clone HIR for Rust-specific transformations
    let hir = hir.clone();

    // Move nested types into modules. Keep track of the moved nodes to
    // properly escape their names later on to ensure the correct node gets
    // precedence.
    let (hir, moved_defs) = ic_hir_xform::move_nested::transform(hir);

    // Squash reopened modules into single definitions
    let hir = ic_hir_xform::squash_modules::transform(hir);

    // Keep a copy of the HIR before renaming to preserve original names
    let original_hir = hir.clone();

    // Rename `DDS::XTypes` to `DDS::xtypes`
    let hir = ic_hir_xform::rename_xtypes::transform(hir);

    // Rename all nodes to conform to Rust's naming convention
    let convention = if options.no_rename {
        Convention::default()
    } else {
        RUST_CONVENTION
    };

    let hir = ic_hir_xform::rename::transform(
        hir,
        &rename::Target {
            convention,
            keyword_escape: Some(escape_rust_keyword),
            moved_defs,
        },
    );

    (hir, original_hir)
}
