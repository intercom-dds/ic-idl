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
mod deps;

use ic_cli::Command;
use ic_emit::File;
use ic_hir::keywords::IDL_KEYWORDS;
use ic_hir_xform::Target;

#[derive(Command, Copy, Clone, Debug, Default)]
pub struct IdlOptions {
    /// Output Doxygen-compatible IDL files
    #[option(long)]
    pub idl_doxygen: bool,

    /// Emit IDL compatible with older parsers
    #[option(long)]
    pub idl_legacy: bool,
}

#[must_use]
pub fn codegen_idl(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: IdlOptions,
) -> Vec<File> {
    let target = Target {
        keywords: IDL_KEYWORDS.iter().copied().collect(),
        keyword_escape_fn: |name| format!("_{name}"),
        ..Target::default()
    };

    let hir = ic_hir_xform::rename::transform(hir.clone(), &target);
    let generator = codegen::IdlGen::new(&hir, source_map, options);
    generator.generate()
}
