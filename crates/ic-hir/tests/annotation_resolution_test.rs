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

mod common;

#[test]
fn test_simple_annotation() {
    let input = r"
        @annotation custom {};
        
        @custom
        struct S {
            long field;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);
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
    let ann_def_id = struct_def.annotations[0].def_id.unwrap();
    let ann_def = result.context.definitions.get(ann_def_id);
    assert!(matches!(ann_def.kind, DefKind::Annotation(_)));
}

#[test]
fn test_annotation_in_module() {
    let input = r"
        @annotation custom {};
        
        module M {
            @custom
            struct S {
                long field;
            };
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

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
    let input = r"
        module M {
            @annotation custom {};
        };
        
        @M::custom
        struct S {
            long field;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

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
    let ann_def_id = struct_def.annotations[0].def_id.unwrap();
    let ann_def = result.context.definitions.get(ann_def_id);
    assert!(matches!(ann_def.kind, DefKind::Annotation(_)));
    assert_eq!(ann_def.ident.name, "custom");
}

#[test]
fn test_unknown_annotation_warning() {
    let input = r"
        @unknown
        struct S {
            long field;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Struct should have the unknown annotation with None def_id
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S")
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "unknown");
    assert_eq!(struct_def.annotations[0].def_id, None);
}

#[test]
fn test_annotation_with_arguments() {
    let input = r"
        @annotation value_range {
            long min;
            long max;
        };
        
        @value_range(min = 0, max = 100)
        struct S {
            long value;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "S")
        .map(|(_, def)| def)
        .unwrap();

    assert_eq!(struct_def.annotations.len(), 1);
    assert_eq!(struct_def.annotations[0].ident.name, "value_range");
    assert_eq!(struct_def.annotations[0].args.len(), 2);

    // Check arguments
    let args = &struct_def.annotations[0].args;
    assert_eq!(args[0].ident.name, "min");
    assert_eq!(args[1].ident.name, "max");
}

#[test]
fn test_member_annotations() {
    let input = r"
        @annotation deprecated {};
        
        struct S {
            @deprecated
            long old_field;
            
            long new_field;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

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
    let input = r"
        @annotation enumval {};
        
        enum E {
            @enumval
            A,
            B,
            @enumval
            C
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

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
        let field_a = result.context.definitions.get(e.fields[0]);
        assert_eq!(field_a.ident.name, "A");
        assert_eq!(field_a.annotations.len(), 1);

        // B should have no annotation
        let field_b = result.context.definitions.get(e.fields[1]);
        assert_eq!(field_b.ident.name, "B");
        assert_eq!(field_b.annotations.len(), 0);

        // C should have annotation
        let field_c = result.context.definitions.get(e.fields[2]);
        assert_eq!(field_c.ident.name, "C");
        assert_eq!(field_c.annotations.len(), 1);
    } else {
        panic!("Expected enum");
    }
}

#[test]
fn test_nested_module_annotation_resolution() {
    let input = r"
        module Outer {
            module Inner {
                @annotation custom {};
            };
        };
        
        @Outer::Inner::custom
        struct S {};
    ";

    let result = common::parse_and_resolve_successfully(input);

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
    let input = r"
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
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Check that all user-defined types have the annotation
    let expected_defs = ["S", "U", "E", "Ex", "I", "C", "T"];
    for expected in &expected_defs {
        let def = result
            .context
            .definitions
            .iter()
            .find(|(_, def)| def.ident.name == *expected)
            .map(|(_, def)| def)
            .unwrap();

        assert_eq!(
            def.annotations.len(),
            1,
            "Definition {} should have 1 annotation",
            def.ident.name
        );
        assert_eq!(def.annotations[0].ident.name, "mark");
    }
}
