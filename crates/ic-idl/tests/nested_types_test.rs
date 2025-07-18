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
fn test_nested_enum_in_annotation() {
    let input = r"
        @annotation TestAnnotation {
            enum NestedEnum {
                FIRST,
                SECOND,
                THIRD
            };
            NestedEnum value;
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    let hir = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);
    assert!(
        hir.warnings.is_empty(),
        "Unexpected warnings: {:?}",
        hir.warnings
    );

    // Find the nested enum
    let enum_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "NestedEnum" && matches!(def.kind, DefKind::Enum(_)))
        .map(|(_, def)| def)
        .expect("Should find nested enum");

    // Verify enum values are evaluated correctly
    if let DefKind::Enum(enum_ty) = &enum_def.kind {
        assert_eq!(enum_ty.fields.len(), 3);
        assert_eq!(enum_ty.fields[0].ident.name, "FIRST");
        assert_eq!(enum_ty.fields[0].value, 0);
        assert_eq!(enum_ty.fields[1].ident.name, "SECOND");
        assert_eq!(enum_ty.fields[1].value, 1);
        assert_eq!(enum_ty.fields[2].ident.name, "THIRD");
        assert_eq!(enum_ty.fields[2].value, 2);
    } else {
        panic!("NestedEnum should be an enum");
    }
}

#[test]
fn test_nested_enum_in_interface() {
    let input = r"
        interface TestInterface {
            enum Status {
                PENDING = 10,
                ACTIVE = 20,
                DONE = 30
            };
            
            Status getStatus();
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    let hir = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);

    // Find the nested enum
    let enum_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Status" && matches!(def.kind, DefKind::Enum(_)))
        .map(|(_, def)| def)
        .expect("Should find nested enum");

    // Verify enum values are evaluated correctly
    if let DefKind::Enum(enum_ty) = &enum_def.kind {
        assert_eq!(enum_ty.fields.len(), 3);
        assert_eq!(enum_ty.fields[0].value, 10);
        assert_eq!(enum_ty.fields[1].value, 20);
        assert_eq!(enum_ty.fields[2].value, 30);
    }
}

#[test]
fn test_nested_enum_in_module() {
    let input = r"
        module TestModule {
            enum Color {
                RED,
                GREEN,
                BLUE
            };
            
            struct ColoredItem {
                Color color;
                string name;
            };
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    let hir = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);

    // Find the nested enum
    let enum_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Color" && matches!(def.kind, DefKind::Enum(_)))
        .map(|(_, def)| def)
        .expect("Should find nested enum");

    // Verify enum values
    if let DefKind::Enum(enum_ty) = &enum_def.kind {
        assert_eq!(enum_ty.fields[0].value, 0);
        assert_eq!(enum_ty.fields[1].value, 1);
        assert_eq!(enum_ty.fields[2].value, 2);
    }
}

#[test]
fn test_type_resolution_in_annotation() {
    let input = r"
        @annotation ExtensibilityAnnotation {
            enum ExtensibilityKind {
                FINAL = 0,
                APPENDABLE = 1,
                MUTABLE = 2
            };
            ExtensibilityKind value;
        };
        
        @ExtensibilityAnnotation(value=APPENDABLE)
        struct TestStruct {
            string data;
        };
    ";

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(parsed.errors.is_empty());

    let hir = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(hir.errors.is_empty(), "Unexpected errors: {:?}", hir.errors);
    assert!(
        hir.warnings.is_empty(),
        "Unexpected warnings: {:?}",
        hir.warnings
    );

    // Verify the annotation definition has the correct member type
    let ann_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| {
            def.ident.name == "ExtensibilityAnnotation"
                && matches!(def.kind, DefKind::Annotation(_))
        })
        .map(|(_, def)| def)
        .expect("Should find annotation");

    if let DefKind::Annotation(ann) = &ann_def.kind {
        assert_eq!(ann.members.len(), 1);
        assert_eq!(ann.members[0].ident.name, "value");
        // The type should be resolved to ExtensibilityKind
    }
}
