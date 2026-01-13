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
mod types;
mod writer;

use ic_cli::Command;
use ic_emit::File;
use ic_emit::case::Case;
use ic_hir_xform::{Convention, Target, rename};

const KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
];

const PYTHON_CONVENTION: Convention = Convention {
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
    variant: Some(Case::Snake),
    enumerator: Some(Case::UpperSnake),
    bit_flag: Some(Case::UpperSnake),
    bitset_field: Some(Case::Snake),
    constant: Some(Case::UpperSnake),
    module: Some(Case::Snake),
    operation: Some(Case::Snake),
    attribute: Some(Case::Snake),
    parameter: Some(Case::Snake),
    name_preprocessor: None,
    strip_enum_prefix: false,
};

#[derive(Command, Debug, Default, Clone)]
pub struct PythonOptions {
    #[option(long)]
    pub use_pep8: bool,
}

#[must_use]
pub fn codegen_python(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: PythonOptions,
) -> Vec<File> {
    let target = Target {
        convention: if options.use_pep8 {
            PYTHON_CONVENTION
        } else {
            Convention::default()
        },
        keyword_escape: Some(|ctx| {
            if KEYWORDS.contains(&ctx.name) {
                Some(format!("_{}", ctx.name))
            } else {
                None
            }
        }),
        ..Target::default()
    };

    let hir = rename::transform(hir.clone(), &target);
    let generator = codegen::PyGen::new(&hir, source_map, options);
    generator.generate()
}
