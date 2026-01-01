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

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefKind, Numeric};
use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

/// Helper to parse IDL with builtin annotations
fn parse_with_builtins(input: &str) -> ResolvedGraph {
    let mut source_map = SourceMap::default();
    let file_id = source_map.embed(input);
    let parsed = ic_parse::from_file(file_id, ProcArgs::default(), &mut source_map);

    let builtin_file_id = source_map.embed_with_name(
        "<builtin-annotations>",
        include_str!("../../ic-idl/idl/annotations.idl"),
    );
    let builtin_parsed = ic_parse::from_file(builtin_file_id, ProcArgs::default(), &mut source_map);

    ic_hir_lower::from_ast(ic_hir_lower::AstInput::WithBuiltins {
        builtins: builtin_parsed.tree,
        user: parsed.tree,
        include_in_output: false,
    })
}

/// Helper to check a bitmask flag value
fn check_flag(
    transformed: &ResolvedGraph,
    flag_id: ic_hir::hir::DefId,
    expected_name: &str,
    expected_value: u64,
) {
    let flag_def = transformed.context.definitions.get(flag_id);
    assert_eq!(flag_def.ident.name, expected_name);
    if let DefKind::Const(c) = &flag_def.kind {
        assert_eq!(c.value, Numeric::UInt64(expected_value));
    } else {
        panic!("Expected const definition for flag");
    }
}

#[test]
fn test_position_annotation_transform_integration() {
    let input = r"
        @annotation deprecated {};
        
        bitmask Permissions {
            READ,
            WRITE,
            
            @position(7)
            EXECUTE,
            
            @position(15)
            @deprecated
            ADMIN
        };
    ";

    let hir = parse_with_builtins(input);

    // Verify expected definitions exist
    assert!(
        hir.context
            .definitions
            .iter()
            .any(|(_, def)| def.ident.name == "Permissions"
                && matches!(def.kind, DefKind::Bitmask(_)))
    );
    assert!(
        hir.context
            .definitions
            .iter()
            .any(|(_, def)| def.ident.name == "position"
                && matches!(def.kind, DefKind::Annotation(_)))
    );

    // Verify annotations are there before transformation
    let permissions_before = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Permissions")
        .expect("Permissions bitmask not found");

    if let DefKind::Bitmask(bitmask_ty) = &permissions_before.1.kind {
        // The @position annotations should be present in the HIR
        let execute_def = hir.context.definitions.get(bitmask_ty.flags[2]);
        assert!(
            execute_def
                .annotations
                .iter()
                .any(|a| a.ident.name == "position"),
            "EXECUTE should have @position annotation"
        );
        let admin_def = hir.context.definitions.get(bitmask_ty.flags[3]);
        assert!(
            admin_def
                .annotations
                .iter()
                .any(|a| a.ident.name == "position"),
            "ADMIN should have @position annotation"
        );
    }

    // Apply the transformation
    let transformed = ic_hir_xform::position_annotation::transform(hir);

    // Find the Permissions bitmask
    let permissions = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Permissions")
        .expect("Permissions bitmask not found");

    if let DefKind::Bitmask(bitmask_ty) = &permissions.1.kind {
        // Check flag values
        check_flag(&transformed, bitmask_ty.flags[0], "READ", 1);
        check_flag(&transformed, bitmask_ty.flags[1], "WRITE", 2);
        check_flag(&transformed, bitmask_ty.flags[2], "EXECUTE", 128); // 1 << 7
        check_flag(&transformed, bitmask_ty.flags[3], "ADMIN", 32768); // 1 << 15

        // Check EXECUTE: no @position annotation
        let execute_def = transformed.context.definitions.get(bitmask_ty.flags[2]);
        assert!(
            !execute_def
                .annotations
                .iter()
                .any(|a| a.ident.name == "position")
        );

        // Check ADMIN: no @position but keep @deprecated
        let admin_def = transformed.context.definitions.get(bitmask_ty.flags[3]);
        assert!(
            !admin_def
                .annotations
                .iter()
                .any(|a| a.ident.name == "position")
        );
        assert!(
            admin_def
                .annotations
                .iter()
                .any(|a| a.ident.name == "deprecated")
        );
    } else {
        panic!("Expected bitmask definition");
    }
}

#[test]
fn test_position_annotation_mixed_values() {
    let input = r"
        bitmask Options {
            @position(0)
            ENABLED,
            
            @position(3)
            VERBOSE,
            
            DEBUG,
            
            @position(8)
            ADMIN
        };
    ";

    let hir = parse_with_builtins(input);

    // Apply the transformation
    let transformed = ic_hir_xform::position_annotation::transform(hir);

    // Find the Options bitmask
    let options_bitmask = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Options")
        .expect("Options bitmask not found");

    if let DefKind::Bitmask(bitmask_ty) = &options_bitmask.1.kind {
        // Check values
        check_flag(&transformed, bitmask_ty.flags[0], "ENABLED", 1); // 1 << 0
        check_flag(&transformed, bitmask_ty.flags[1], "VERBOSE", 8); // 1 << 3
        check_flag(&transformed, bitmask_ty.flags[2], "DEBUG", 4); // auto-incremented
        check_flag(&transformed, bitmask_ty.flags[3], "ADMIN", 256); // 1 << 8

        // Check that @position annotations are removed
        for &flag_id in &bitmask_ty.flags {
            let flag_def = transformed.context.definitions.get(flag_id);
            assert!(
                !flag_def
                    .annotations
                    .iter()
                    .any(|a| a.ident.name == "position")
            );
        }
    } else {
        panic!("Expected bitmask definition");
    }
}
