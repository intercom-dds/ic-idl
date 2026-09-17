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

mod common;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind};
use ic_hir_xform::{modularize_root, normalize};

fn root_definitions(hir: &ResolvedGraph) -> impl Iterator<Item = &Def> {
    hir.iter().filter(|def| def.parent.is_none())
}

fn module_definitions<'a>(
    hir: &'a ResolvedGraph,
    qualified_name: &str,
) -> impl Iterator<Item = &'a Def> {
    hir.context
        .definitions
        .into_iter()
        .filter(move |(id, _)| hir.context.qualified_name(*id) == qualified_name)
        .flat_map(|(_, def)| match &def.kind {
            DefKind::Module(m) => m.definitions.clone(),
            _ => vec![],
        })
        .map(|id| hir.context.type_of(id))
}

#[test]
fn non_module_root_definitions_are_modularized() {
    let (hir, source_map) = common::parse_and_resolve_with_source_map(
        "test.idl",
        r"
        struct RootStruct {};
        typedef long RootAlias;
        const RootAlias RootConst = 123;

        module A {
            struct ModuleStruct {};
            typedef long ModuleAlias;
            const ModuleAlias ModuleConst = 123;

            module A2 {
                struct SubModuleStruct {};
            };
        };

        module B {
            module B2 {
            };
        };
        ",
    );

    let (transformed, moved_defs) = modularize_root::transform(hir, &source_map, "_IDL_FILE");
    normalize::normalize(&transformed);
    assert_eq!(
        moved_defs.len(),
        3,
        "Three definitions were moved into a module"
    );

    assert!(
        root_definitions(&transformed).all(|def| matches!(def.kind, DefKind::Module(_))),
        "All root-level definitions are modules"
    );

    assert_eq!(
        root_definitions(&transformed)
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["test_IDL_FILE", "A", "B"]
    );

    assert_eq!(
        module_definitions(&transformed, "test_IDL_FILE")
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["RootStruct", "RootAlias", "RootConst"]
    );

    assert_eq!(
        module_definitions(&transformed, "A")
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["ModuleStruct", "ModuleAlias", "ModuleConst", "A2"]
    );

    assert_eq!(
        module_definitions(&transformed, "A::A2")
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["SubModuleStruct"]
    );

    assert_eq!(
        module_definitions(&transformed, "B")
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["B2"]
    );
}

#[test]
fn dots_in_file_names_are_modularized() {
    let (hir, source_map) = common::parse_and_resolve_with_source_map(
        "test.ing.idl",
        r"
        struct RootStruct {};
        ",
    );
    let (transformed, moved_defs) = modularize_root::transform(hir, &source_map, "_IDL_FILE");
    normalize::normalize(&transformed);

    assert_eq!(moved_defs.len(), 1);
    assert_eq!(
        root_definitions(&transformed)
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["test_ing_IDL_FILE"]
    );
}

#[test]
fn prefixed_numbers_in_file_names_are_modularized() {
    let (hir, source_map) = common::parse_and_resolve_with_source_map(
        "1test.idl",
        r"
        struct RootStruct {};
        ",
    );
    let (transformed, moved_defs) = modularize_root::transform(hir, &source_map, "_IDL_FILE");
    normalize::normalize(&transformed);

    assert_eq!(moved_defs.len(), 1);
    assert_eq!(
        root_definitions(&transformed)
            .map(|ty| ty.ident.name.clone())
            .collect::<Vec<_>>(),
        vec!["IDL_1test_IDL_FILE"]
    );
}
