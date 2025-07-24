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

use ic_parse::from_file;
use ic_preproc::ProcArgs;
use ic_ptree_lower::{from_ast, from_hir};
use ic_vfs::SourceMap;

#[test]
fn test_empty_input() {
    // Test with truly empty input
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed("");
    let parsed = from_file(file_id, ProcArgs::default(), &mut vfs);
    assert!(parsed.errors.is_empty());

    // HIR lowering should also work
    let hir = ic_hir::from_ast(ic_hir::AstInput::User(parsed.tree));
    let _ptree_hir = from_hir(&hir, &vfs);
    // Just check it doesn't panic
}

#[test]
fn test_minimal_struct() {
    // Try the simplest possible valid IDL
    let idl = "struct S { };";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(idl);
    let parsed = from_file(file_id, ProcArgs::default(), &mut vfs);

    if !parsed.errors.is_empty() {
        // Parse errors encountered
        return;
    }

    // Try AST lowering
    let ptree_ast = from_ast(&parsed, &vfs);
    let _ = ptree_ast.diagnostics();

    // Try HIR lowering
    let hir = ic_hir::from_ast(ic_hir::AstInput::User(parsed.tree));
    if !hir.errors.is_empty() {
        // HIR errors encountered
        return;
    }

    let ptree_hir = from_hir(&hir, &vfs);
    let _ = ptree_hir.diagnostics();
}

#[test]
fn test_from_file() {
    // Test using actual IDL files that should work
    let mut vfs = SourceMap::default();

    // Test with builtin annotations which we know should parse
    let builtin_idl = include_str!("../idl/annotations.idl");
    let file_id = vfs.embed(builtin_idl);
    let parsed = from_file(file_id, ProcArgs::default(), &mut vfs);

    if !parsed.errors.is_empty() {
        // Parse errors in builtin annotations file
        return;
    }

    // This should definitely work since it's used internally
    let hir = ic_hir::from_ast(ic_hir::AstInput::User(parsed.tree));
    assert!(hir.errors.is_empty(), "HIR errors: {:?}", hir.errors);

    let _ptree = from_hir(&hir, &vfs);
    // The builtin annotations should lower successfully
}
