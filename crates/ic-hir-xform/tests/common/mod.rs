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

#![allow(dead_code)]

use ic_hir::ResolvedGraph;
use ic_hir_lower::AstInput;
use ic_vfs::SourceMap;

/// Parse IDL input and return the HIR
#[track_caller]
pub fn parse_and_resolve(input: &str) -> ResolvedGraph {
    let mut source_map = SourceMap::default();
    let file = source_map.embed_with_name("test.idl", input);
    let parsed = ic_parse::from_file(file, &source_map);

    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir_lower::from_ast(AstInput::User(parsed.tree));
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);
    result
}

/// Parse IDL input with builtin annotations and return the HIR
#[track_caller]
pub fn parse_with_builtins(input: &str) -> ResolvedGraph {
    let mut source_map = SourceMap::default();
    let file = source_map.embed_with_name("test.idl", input);
    let parsed = ic_parse::from_file(file, &source_map);

    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let builtin_file = source_map.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file, &source_map);

    assert!(
        builtin_parsed.errors.is_empty(),
        "Builtin parse errors: {:?}",
        builtin_parsed.errors
    );

    let result = ic_hir_lower::from_ast(AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: parsed.tree,
        include_in_output: false,
    });
    assert!(result.errors.is_empty(), "HIR errors: {:?}", result.errors);
    result
}
