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
mod group;

use ic_emit::File;
use ic_hir_xform::rename::{self, Convention, Target};

use crate::codegen::ProtoGen;

#[rustfmt::skip]
const PROTO_KEYWORDS: &[&str] = &[
    "bool", "bytes", "double", "edition", "enum", "extend", "extensions", "fixed32", "fixed64",
    "float", "group", "import", "inf", "int32", "int64", "map", "max", "message", "nan", "oneof",
    "option", "optional", "package", "public", "repeated", "required", "reserved", "returns",
    "rpc", "service", "sfixed32", "sfixed64", "sint32", "sint64", "stream", "string", "syntax",
    "to", "uint32", "uint64", "weak",
];

fn escape_kw(ctx: ic_hir_xform::rename::RenameContext) -> Option<String> {
    if PROTO_KEYWORDS.contains(&ctx.name) {
        Some(format!("{}_", ctx.name))
    } else {
        None
    }
}

#[must_use]
pub fn codegen_proto(hir: &ic_hir::ResolvedGraph) -> Vec<File> {
    // Move nested types into modules
    let (hir, moved_defs) = ic_hir_xform::move_nested::transform(hir.clone());

    // Escape keywords
    let target = Target {
        moved_defs,
        convention: Convention::default(),
        keyword_escape: Some(escape_kw),
    };
    let hir = rename::transform(hir, &target);

    let generator = ProtoGen::new(&hir);
    generator.generate()
}
