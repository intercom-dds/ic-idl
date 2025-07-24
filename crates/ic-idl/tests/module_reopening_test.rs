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
fn test_module_reopening_preserves_annotations() {
    let input = r"
        // First, reopen the intercom module (empty)
        module intercom {};
        
        // Now try to use annotations - they should still work
        struct Foo { 
            @key string id;
            @optional boolean flag;
            @range(min=0, max=100) long value;
        };
        
        // Reopen intercom again with content
        module intercom {
            struct Bar {
                @id(123) unsigned long id;
            };
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
    let hir = ic_hir::from_ast(ic_hir::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: parsed.tree,
        include_in_output: false,
    });

    // Should have no errors or warnings
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);
    assert!(
        hir.warnings.is_empty(),
        "Unexpected warnings: {:?}",
        hir.warnings
    );

    // Verify struct Foo has annotations resolved
    let foo_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Foo" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .expect("Should find struct Foo");

    if let DefKind::Struct(s) = &foo_def.kind {
        assert_eq!(s.members.len(), 3);

        // Check @key annotation on id field
        assert_eq!(s.members[0].ident.name, "id");
        assert_eq!(s.members[0].annotations.len(), 1);
        assert_eq!(s.members[0].annotations[0].ident.name, "key");

        // Check @optional annotation on flag field
        assert_eq!(s.members[1].ident.name, "flag");
        assert_eq!(s.members[1].annotations.len(), 1);
        assert_eq!(s.members[1].annotations[0].ident.name, "optional");

        // Check @range annotation on value field
        assert_eq!(s.members[2].ident.name, "value");
        assert_eq!(s.members[2].annotations.len(), 1);
        assert_eq!(s.members[2].annotations[0].ident.name, "range");
    } else {
        panic!("Foo should be a struct");
    }
}

#[test]
fn test_nested_module_reopening() {
    let input = r"
        module A {
            module B {
                struct First {};
            };
        };
        
        // Reopen module A
        module A {};
        
        // Reopen module A again and add to B
        module A {
            module B {
                struct Second {};
            };
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    let hir = ic_hir::from_ast(ic_hir::AstInput::User(parsed.tree));

    // Should have no errors
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);

    // Find all module A definitions
    let module_a_defs: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter(|(_, def)| def.ident.name == "A" && matches!(def.kind, DefKind::Module(_)))
        .collect();

    // Should have created 3 separate module A definitions
    assert_eq!(module_a_defs.len(), 3, "Should have 3 module A definitions");
}
