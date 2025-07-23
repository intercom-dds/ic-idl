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

use ic_hir::hir::DefKind;
use ic_idl::ast_to_hir;
use ic_lint::LintConfig;
use ic_parse::SourceMap;

#[test]
fn test_builtin_value_annotation() {
    let input = r"
        enum E {
            @value(10)
            A,
            B,
            @value(20)  
            C
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);

    assert!(parsed.errors.is_empty());

    // We expect the HIR conversion to succeed, though it may have warnings
    // about duplicate enum values in the built-in annotations
    let hir = match ast_to_hir(parsed.tree, &source_map, &LintConfig::default()) {
        Ok(hir) => hir,
        Err(e) => {
            // If it's just warnings about built-in annotations, that's OK
            // The important thing is that user annotations like @value are resolved
            panic!("HIR conversion failed: {e:?}");
        }
    };

    // Check that only user types are in the order (no intercom module)
    for def_id in &hir.order {
        let def = hir.context.definitions.get(*def_id);
        assert!(
            !def.ident.name.starts_with("intercom"),
            "Built-in definition {} should not be in order",
            def.ident.name
        );
    }
}

#[test]
fn test_builtin_key_annotation() {
    let input = r"
        struct S {
            @key
            long id;
            
            string name;
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);

    assert!(parsed.errors.is_empty());

    // Parse built-in annotations
    let builtin_annotations = include_str!("../../ic-idl/idl/annotations.idl");

    let builtin_file_id = source_map.embed_with_name("<builtin-annotations>", builtin_annotations);
    let builtin_parsed = ic_parse::from_file(
        builtin_file_id,
        ic_preproc::ProcArgs::default(),
        &mut source_map,
    );
    assert!(builtin_parsed.errors.is_empty());

    // We need to use from_ast_with_builtins to include built-in annotations
    let hir = ic_hir::from_ast_with_builtins(builtin_parsed.tree, parsed.tree);

    // Check for errors
    assert!(
        hir.errors.is_empty(),
        "HIR conversion should succeed without errors"
    );
    assert!(
        hir.warnings.is_empty(),
        "HIR conversion should succeed without warnings"
    );

    // Verify the struct has the @key annotation resolved correctly
    let struct_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .expect("Should find struct S");

    if let DefKind::Struct(s) = &struct_def.kind {
        assert_eq!(s.members.len(), 2);
        // The id field should have the @key annotation
        assert_eq!(s.members[0].ident.name, "id");
        assert_eq!(s.members[0].annotations.len(), 1);
        assert_eq!(s.members[0].annotations[0].ident.name, "key");
    }
}
