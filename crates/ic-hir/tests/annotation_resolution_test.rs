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
use ic_parse::{SourceMap, from_file};
use ic_preproc::ProcArgs;

#[test]
fn test_simple_annotation() {
    let input = r#"
        @annotation custom {};
        
        @custom
        struct S {
            long field;
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );
    assert_eq!(result.order.len(), 2);

    // Find the struct
    let struct_id = result
        .order
        .iter()
        .find(|&&id| matches!(result.context.definitions.get(id).kind, DefKind::Struct(_)))
        .unwrap();

    let struct_def = result.context.definitions.get(*struct_id);
    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "custom");

    // Verify the annotation resolves to the correct definition
    let ann_def_id = struct_def.annotations[0].def_id;
    let ann_def = result.context.definitions.get(ann_def_id);
    assert!(matches!(ann_def.kind, DefKind::Annotation(_)));
}

#[test]
fn test_annotation_in_module() {
    let input = r#"
        @annotation custom {};
        
        module M {
            @custom
            struct S {
                long field;
            };
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find the struct (it's nested in module)
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "custom");
}

#[test]
fn test_qualified_annotation_path() {
    let input = r#"
        module M {
            @annotation custom {};
        };
        
        @M::custom
        struct S {
            long field;
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Find the struct
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S" && matches!(def.kind, DefKind::Struct(_)))
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "M::custom");

    // Verify it resolves to the annotation inside module M
    let ann_def_id = struct_def.annotations[0].def_id;
    let ann_def = result.context.definitions.get(ann_def_id);
    assert!(matches!(ann_def.kind, DefKind::Annotation(_)));
    assert_eq!(ann_def.ident.name, "custom");
}

#[test]
fn test_unknown_annotation_warning() {
    let input = r#"
        @unknown
        struct S {
            long field;
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    // Should have a warning about unknown annotation
    assert_eq!(result.warnings.len(), 1);
    let warning_msg = format!("{}", result.warnings[0]);
    assert!(warning_msg.contains("unknown"));

    // Struct should have no annotations (unknown ones are filtered out)
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S")
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 0);
}

#[test]
fn test_annotation_with_arguments() {
    let input = r#"
        @annotation range {
            long min;
            long max;
        };
        
        @range(min = 0, max = 100)
        struct S {
            long value;
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S")
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "range");
    assert_eq!(struct_def.annotations[0].args.len(), 2);

    // Check arguments
    let args = &struct_def.annotations[0].args;
    assert_eq!(args[0].ident.as_ref().unwrap().name, "min");
    assert_eq!(args[1].ident.as_ref().unwrap().name, "max");
}

#[test]
fn test_member_annotations() {
    let input = r#"
        @annotation deprecated {};
        
        struct S {
            @deprecated
            long old_field;
            
            long new_field;
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S")
        .map(|(_, def)| def)
        .unwrap();

    if let DefKind::Struct(s) = &struct_def.kind {
        assert_eq!(s.members.len(), 2);

        // old_field should have annotation
        assert_eq!(s.members[0].ident.name, "old_field");
        assert_eq!(s.members[0].annotations.len(), 1);
        assert_eq!(s.members[0].annotations[0].ident.name, "deprecated");

        // new_field should have no annotations
        assert_eq!(s.members[1].ident.name, "new_field");
        assert_eq!(s.members[1].annotations.len(), 0);
    } else {
        panic!("Expected struct");
    }
}

#[test]
fn test_enum_field_annotations() {
    let input = r#"
        @annotation enumval {};
        
        enum E {
            @enumval
            A,
            B,
            @enumval
            C
        };
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    let enum_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "E")
        .map(|(_, def)| def)
        .unwrap();

    if let DefKind::Enum(e) = &enum_def.kind {
        assert_eq!(e.fields.len(), 3);

        // A should have annotation
        assert_eq!(e.fields[0].ident.name, "A");
        assert_eq!(e.fields[0].annotations.len(), 1);

        // B should have no annotation
        assert_eq!(e.fields[1].ident.name, "B");
        assert_eq!(e.fields[1].annotations.len(), 0);

        // C should have annotation
        assert_eq!(e.fields[2].ident.name, "C");
        assert_eq!(e.fields[2].annotations.len(), 1);
    } else {
        panic!("Expected enum");
    }
}

#[test]
fn test_nested_module_annotation_resolution() {
    let input = r#"
        module Outer {
            module Inner {
                @annotation custom {};
            };
        };
        
        @Outer::Inner::custom
        struct S {};
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S")
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "Outer::Inner::custom");
}

#[test]
fn test_annotation_on_all_definition_types() {
    let input = r#"
        @annotation mark {};
        
        @mark
        struct S {};
        
        @mark
        union U switch (long) {
            case 1: long a;
        };
        
        @mark
        enum E { A };
        
        @mark
        exception Ex {};
        
        @mark
        interface I {};
        
        @mark
        const long C = 42;
        
        @mark
        typedef long T;
    "#;

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let result = ic_hir::from_ast(ast.tree);

    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Check that all types have the annotation
    for (_, def) in result.context.definitions.iter() {
        if def.ident.name != "mark" {
            // Skip the annotation definition itself
            assert_eq!(
                def.annotations.len(),
                1,
                "Definition {} should have 1 annotation",
                def.ident.name
            );
            assert_eq!(def.annotations[0].ident.name, "mark");
        }
    }
}
