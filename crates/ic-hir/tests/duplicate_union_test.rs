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

use ic_hir::hir::{DefKind, TyKind};

mod common;
use common::parse_and_resolve;

#[test]
fn test_duplicate_union_detected() {
    let idl = r"
        union Foo switch (long) {
        case 0: long x;
        };
        union Foo switch (long) {
        case 0: long y;
        };
    ";
    let (result, _, diagnostics) = parse_and_resolve(idl);
    assert!(
        !result.errors.is_empty(),
        "Expected duplicate definition error"
    );
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_duplicate_union_keeps_original_resolvable() {
    let idl = r"
        union Foo switch (long) {
        case 0: long x;
        };
        union Foo switch (long) {
        case 0: long y;
        };
        typedef Foo Alias;
    ";
    let (result, _, _) = parse_and_resolve(idl);
    assert!(
        !result.errors.is_empty(),
        "Expected duplicate definition error"
    );

    let alias_def = result
        .context
        .definitions
        .iter()
        .map(|(_, d)| d)
        .find(|d| d.ident.name == "Alias")
        .expect("Alias definition not found");
    let DefKind::Alias(alias_ty) = &alias_def.kind else {
        panic!("Alias is not an alias definition");
    };
    let TyKind::Adt(target_id) = &alias_ty.ty.kind else {
        panic!("Alias target is not an ADT");
    };
    let target_def = result.context.definitions.get(*target_id);
    let DefKind::Union(union_ty) = &target_def.kind else {
        panic!("Alias target is not a union");
    };
    assert_eq!(
        union_ty.variants.len(),
        1,
        "alias should resolve to the first (valid) Foo, not the rejected duplicate"
    );
    assert_eq!(union_ty.variants[0].ident.name, "x");
}
