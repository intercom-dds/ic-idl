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
use ic_parse::SourceMap;

#[test]
fn test_ext_annotation_requires_qualification() {
    let input = r"
        struct TestStruct { 
            // This should produce a warning - no_serializer needs ext:: prefix
            @no_serializer string field1;
            
            // This should work - properly qualified
            @ext::no_serializer string field2;
            
            // Regular annotations work without qualification
            @key string field3;
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    // Parse built-in annotations
    let builtin_annotations = include_str!("../idl/annotations.idl");
    let builtin_file_id = source_map.embed_with_name("<builtin-annotations>", builtin_annotations);
    let builtin_parsed = ic_parse::from_file(
        builtin_file_id,
        ic_preproc::ProcArgs::default(),
        &mut source_map,
    );
    assert!(builtin_parsed.errors.is_empty());

    // Convert to HIR with built-ins
    let hir = ic_hir::from_ast_with_builtins(builtin_parsed.tree, parsed.tree);

    // Should have no errors
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);

    // Should have exactly one warning for the unqualified no_serializer
    assert_eq!(
        hir.warnings.len(),
        1,
        "Expected exactly 1 warning, got {}: {:?}",
        hir.warnings.len(),
        hir.warnings
    );
    assert!(hir.warnings[0].to_string().contains("no_serializer"));
    assert!(hir.warnings[0].to_string().contains("annotation not found"));

    // Verify struct has correct annotations resolved
    let struct_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestStruct" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .expect("Should find struct TestStruct");

    if let DefKind::Struct(s) = &struct_def.kind {
        assert_eq!(s.members.len(), 3);

        // field1 should have no annotations (unresolved)
        assert_eq!(s.members[0].ident.name, "field1");
        assert_eq!(s.members[0].annotations.len(), 0);

        // field2 should have ext::no_serializer annotation
        assert_eq!(s.members[1].ident.name, "field2");
        assert_eq!(s.members[1].annotations.len(), 1);
        assert_eq!(s.members[1].annotations[0].ident.name, "ext::no_serializer");

        // field3 should have key annotation
        assert_eq!(s.members[2].ident.name, "field3");
        assert_eq!(s.members[2].annotations.len(), 1);
        assert_eq!(s.members[2].annotations[0].ident.name, "key");
    } else {
        panic!("TestStruct should be a struct");
    }
}

#[test]
fn test_other_qualified_annotations() {
    let input = r#"
        struct TestStruct {
            // Test various ext annotations
            @ext::doc(text="Hello") string field1;
            @ext::suppress boolean field2;
            @ext::rename(name="new_name") long field3;
        };
    "#;

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    // Parse built-in annotations
    let builtin_annotations = include_str!("../idl/annotations.idl");
    let builtin_file_id = source_map.embed_with_name("<builtin-annotations>", builtin_annotations);
    let builtin_parsed = ic_parse::from_file(
        builtin_file_id,
        ic_preproc::ProcArgs::default(),
        &mut source_map,
    );
    assert!(builtin_parsed.errors.is_empty());

    // Convert to HIR with built-ins
    let hir = ic_hir::from_ast_with_builtins(builtin_parsed.tree, parsed.tree);

    // Should have no errors or warnings
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);
    assert!(
        hir.warnings.is_empty(),
        "Unexpected warnings: {:?}",
        hir.warnings
    );

    // Verify all ext:: annotations were resolved
    let struct_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestStruct" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .expect("Should find struct TestStruct");

    if let DefKind::Struct(s) = &struct_def.kind {
        assert_eq!(s.members.len(), 3);

        // All fields should have their ext:: annotations resolved
        assert_eq!(s.members[0].annotations[0].ident.name, "ext::doc");
        assert_eq!(s.members[1].annotations[0].ident.name, "ext::suppress");
        assert_eq!(s.members[2].annotations[0].ident.name, "ext::rename");
    }
}
