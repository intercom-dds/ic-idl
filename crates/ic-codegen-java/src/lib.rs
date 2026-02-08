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

mod codegen;

use ic_cli::Command;
use ic_emit::File;
use ic_emit::case::Case;
use ic_hir_xform::rename::{self, Convention, Target};

#[rustfmt::skip]
pub(crate) const KEYWORDS: &[&str] = &[
    "_", "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
    "clone", "const", "continue", "default", "do", "double", "else", "enum", "equals", "extends",
    "false", "final", "finalize", "finally", "float", "for", "getClass", "goto", "hashCode", "if",
    "implements", "import", "instanceof", "int", "interface", "long", "native", "new", "notify",
    "notifyAll", "null", "package", "private", "protected", "public", "return", "short", "static",
    "strictfp", "super", "switch", "synchronized", "this", "throw", "throws", "toString",
    "transient", "true", "try", "void", "volatile", "wait", "while",
];

const JAVA_CONVENTION: Convention = Convention {
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
    member: Some(Case::Camel),
    variant: Some(Case::Camel),
    enumerator: Some(Case::UpperSnake),
    bit_flag: Some(Case::UpperSnake),
    bitset_field: Some(Case::Camel),
    constant: Some(Case::UpperSnake),
    module: Some(Case::Snake),
    operation: Some(Case::Camel),
    attribute: Some(Case::Camel),
    parameter: Some(Case::Camel),
    name_preprocessor: Some(rename::strip_common_suffixes),
    strip_enum_prefix: true,
};

#[derive(Command, Clone, Debug, Default)]
pub struct JavaOptions {
    /// Do not rename types to Java conventions
    #[option(long)]
    pub no_rename: bool,

    /// Use Java package prefix
    #[option(long)]
    pub package_prefix: Option<String>,
}

#[must_use]
pub fn codegen_java(hir: &ic_hir::ResolvedGraph, options: JavaOptions) -> Vec<File> {
    let convention = if options.no_rename {
        Convention::default()
    } else {
        JAVA_CONVENTION
    };

    let target = Target {
        convention,
        keyword_escape: Some(|ctx| {
            if KEYWORDS.contains(&ctx.name) {
                Some(format!("{}_", ctx.name))
            } else {
                None
            }
        }),
        ..Target::default()
    };

    let hir = rename::transform(hir.clone(), &target);
    let generator = codegen::JavaGen::new(&hir, options);
    generator.generate()
}
