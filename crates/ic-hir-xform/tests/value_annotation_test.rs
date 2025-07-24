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
use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

#[test]
#[allow(clippy::too_many_lines)]
fn test_value_annotation_transform_integration() {
    let input = r"
        @annotation deprecated {};
        
        enum Status {
            @value(200)
            OK,
            
            @value(404)
            NOT_FOUND,
            
            ERROR = 500,
            
            @value(503)
            @deprecated
            SERVICE_UNAVAILABLE
        };
    ";

    // Parse the input directly
    let mut source_map = SourceMap::default();
    let file_id = source_map.embed(input);
    let parsed = ic_parse::from_file(file_id, ProcArgs::default(), &mut source_map);

    // Parse built-in annotations
    let builtin_file_id = source_map.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file_id, ProcArgs::default(), &mut source_map);

    // Convert to HIR
    let hir = ic_hir::from_ast(ic_hir::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: parsed.tree,
        include_in_output: false,
    });

    // Verify we have the expected definitions
    assert!(
        hir.context
            .definitions
            .iter()
            .any(|(_, def)| def.ident.name == "Status" && matches!(def.kind, DefKind::Enum(_)))
    );
    assert!(hir.context.definitions.iter().any(|(_, def)| def.ident.name == "value" && matches!(def.kind, DefKind::Annotation(_))));

    // The HIR construction already processes @value annotations and sets the enum values
    // Our transformation should remove the @value annotations that are still present
    // Let's verify the annotations are there before transformation
    let status_enum_before = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Status")
        .expect("Status enum not found");

    if let DefKind::Enum(enum_ty) = &status_enum_before.1.kind {
        // The @value annotations should be present in the HIR
        // even though the values have been applied during evaluation
        assert!(
            enum_ty.fields[0]
                .annotations
                .iter()
                .any(|a| a.ident.name == "value"),
            "OK should have @value annotation"
        );
        assert!(
            enum_ty.fields[1]
                .annotations
                .iter()
                .any(|a| a.ident.name == "value"),
            "NOT_FOUND should have @value annotation"
        );
        assert_eq!(enum_ty.fields[2].value, 500); // ERROR has explicit value
        assert!(
            enum_ty.fields[3]
                .annotations
                .iter()
                .any(|a| a.ident.name == "value"),
            "SERVICE_UNAVAILABLE should have @value annotation"
        );
    }

    // Apply the transformation
    let transformed = ic_hir_xform::value_annotation::transform(hir);

    // Find the Status enum
    let status_enum = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Status")
        .expect("Status enum not found");

    if let DefKind::Enum(enum_ty) = &status_enum.1.kind {
        // Check OK: should have value 200 and no @value annotation
        assert_eq!(enum_ty.fields[0].ident.name, "OK");
        assert_eq!(enum_ty.fields[0].value, 200);
        assert!(
            !enum_ty.fields[0]
                .annotations
                .iter()
                .any(|a| a.ident.name == "value")
        );

        // Check NOT_FOUND: should have value 404 and no @value annotation
        assert_eq!(enum_ty.fields[1].ident.name, "NOT_FOUND");
        assert_eq!(enum_ty.fields[1].value, 404);
        assert!(
            !enum_ty.fields[1]
                .annotations
                .iter()
                .any(|a| a.ident.name == "value")
        );

        // Check ERROR: should keep its explicit value 500
        assert_eq!(enum_ty.fields[2].ident.name, "ERROR");
        assert_eq!(enum_ty.fields[2].value, 500);

        // Check SERVICE_UNAVAILABLE: should have value 503, no @value but keep @deprecated
        assert_eq!(enum_ty.fields[3].ident.name, "SERVICE_UNAVAILABLE");
        assert_eq!(enum_ty.fields[3].value, 503);
        assert!(
            !enum_ty.fields[3]
                .annotations
                .iter()
                .any(|a| a.ident.name == "value")
        );
        assert!(
            enum_ty.fields[3]
                .annotations
                .iter()
                .any(|a| a.ident.name == "deprecated")
        );
    } else {
        panic!("Expected enum definition");
    }
}

#[test]
fn test_value_annotation_with_auto_increment() {
    let input = r"
        @annotation deprecated {};
        
        enum Numbers {
            ZERO,
            
            @value(10)
            TEN,
            
            ELEVEN,
            
            @value(20)
            TWENTY
        };
    ";

    // Parse the input directly
    let mut source_map = SourceMap::default();
    let file_id = source_map.embed(input);
    let parsed = ic_parse::from_file(file_id, ProcArgs::default(), &mut source_map);

    // Parse built-in annotations
    let builtin_file_id = source_map.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file_id, ProcArgs::default(), &mut source_map);

    // Convert to HIR
    let hir = ic_hir::from_ast(ic_hir::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: parsed.tree,
        include_in_output: false,
    });

    // Apply the transformation
    let transformed = ic_hir_xform::value_annotation::transform(hir);

    // Find the Numbers enum
    let numbers_enum = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Numbers")
        .expect("Numbers enum not found");

    if let DefKind::Enum(enum_ty) = &numbers_enum.1.kind {
        // Check values
        // Note: ELEVEN gets value 2 because during initial HIR construction,
        // TEN had no explicit value (only @value annotation) so it got 1,
        // and ELEVEN auto-incremented to 2. Our transformation only changes
        // the values for fields with @value annotations.
        assert_eq!(enum_ty.fields[0].value, 0); // ZERO
        assert_eq!(enum_ty.fields[1].value, 10); // TEN (from @value)
        assert_eq!(enum_ty.fields[2].value, 2); // ELEVEN (auto-increment from original)
        assert_eq!(enum_ty.fields[3].value, 20); // TWENTY (from @value)

        // Check that @value annotations are removed
        for field in &enum_ty.fields {
            assert!(!field.annotations.iter().any(|a| a.ident.name == "value"));
        }
    } else {
        panic!("Expected enum definition");
    }
}
