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

use std::fs;

use ic_hir::hir::DefKind;
use ic_idl::{Compiler, CompilerOptions};

#[test]
fn test_position_annotation_transform_integration() {
    let input = r"
        // Built-in annotations
        @annotation position {
            int32 value;
        };
        
        @annotation deprecated {
        };
        
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

    // Create a temporary directory and file
    let temp_dir = format!("/tmp/ic_hir_xform_pos_test_{}", std::process::id());
    fs::create_dir_all(&temp_dir).unwrap();
    let input_file = format!("{temp_dir}/test.idl");
    fs::write(&input_file, input).unwrap();

    // Set up compiler options
    let mut options = CompilerOptions::default();
    options.files.push(input_file.into());

    // Compile to get HIR
    let mut compiler = Compiler::new(options);
    let (hir, _diagnostics) = compiler.compile_hir().expect("Compilation failed");

    // Debug: print all definitions
    println!("\nAll definitions:");
    for (id, def) in hir.context.definitions.iter() {
        println!(
            "  {:?}: {} ({})",
            id,
            def.ident.name,
            match &def.kind {
                DefKind::Bitmask(_) => "bitmask",
                DefKind::Annotation(_) => "annotation",
                _ => "other",
            }
        );
    }

    // Verify annotations are there before transformation
    let permissions_before = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Permissions")
        .expect("Permissions bitmask not found");

    if let DefKind::Bitmask(bitmask_ty) = &permissions_before.1.kind {
        println!("\nBefore transformation:");
        for flag in &bitmask_ty.flags {
            println!(
                "  {}: value={}, annotations={:?}",
                flag.ident.name,
                flag.value,
                flag.annotations
                    .iter()
                    .map(|a| &a.ident.name)
                    .collect::<Vec<_>>()
            );
        }

        // The @position annotations should be present in the HIR
        assert!(
            bitmask_ty.flags[2]
                .annotations
                .iter()
                .any(|a| a.ident.name == "position"),
            "EXECUTE should have @position annotation"
        );
        assert!(
            bitmask_ty.flags[3]
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
        println!("\nAfter transformation:");
        for flag in &bitmask_ty.flags {
            println!(
                "  {}: value={}, annotations={:?}",
                flag.ident.name,
                flag.value,
                flag.annotations
                    .iter()
                    .map(|a| &a.ident.name)
                    .collect::<Vec<_>>()
            );
        }

        // Check READ: should have value 1 (default)
        assert_eq!(bitmask_ty.flags[0].ident.name, "READ");
        assert_eq!(bitmask_ty.flags[0].value, 1);

        // Check WRITE: should have value 2 (auto-increment)
        assert_eq!(bitmask_ty.flags[1].ident.name, "WRITE");
        assert_eq!(bitmask_ty.flags[1].value, 2);

        // Check EXECUTE: should have value 1 << 7 = 128 and no @position annotation
        assert_eq!(bitmask_ty.flags[2].ident.name, "EXECUTE");
        assert_eq!(bitmask_ty.flags[2].value, 128); // 1 << 7
        assert!(
            !bitmask_ty.flags[2]
                .annotations
                .iter()
                .any(|a| a.ident.name == "position")
        );

        // Check ADMIN: should have value 1 << 15 = 32768, no @position but keep @deprecated
        assert_eq!(bitmask_ty.flags[3].ident.name, "ADMIN");
        assert_eq!(bitmask_ty.flags[3].value, 32768); // 1 << 15
        assert!(
            !bitmask_ty.flags[3]
                .annotations
                .iter()
                .any(|a| a.ident.name == "position")
        );
        assert!(
            bitmask_ty.flags[3]
                .annotations
                .iter()
                .any(|a| a.ident.name == "deprecated")
        );
    } else {
        panic!("Expected bitmask definition");
    }

    // Clean up
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_position_annotation_mixed_values() {
    let input = r"
        // Built-in annotations
        @annotation position {
            int32 value;
        };
        
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

    // Create a temporary directory and file
    let temp_dir = format!("/tmp/ic_hir_xform_pos_test2_{}", std::process::id());
    fs::create_dir_all(&temp_dir).unwrap();
    let input_file = format!("{temp_dir}/test.idl");
    fs::write(&input_file, input).unwrap();

    // Set up compiler options
    let mut options = CompilerOptions::default();
    options.files.push(input_file.into());

    // Compile to get HIR
    let mut compiler = Compiler::new(options);
    let (hir, _diagnostics) = compiler.compile_hir().expect("Compilation failed");

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
        assert_eq!(bitmask_ty.flags[0].value, 1); // ENABLED: 1 << 0 = 1
        assert_eq!(bitmask_ty.flags[1].value, 8); // VERBOSE: 1 << 3 = 8
        assert_eq!(bitmask_ty.flags[2].value, 4); // DEBUG: auto-incremented (original)
        assert_eq!(bitmask_ty.flags[3].value, 256); // ADMIN: 1 << 8 = 256

        // Check that @position annotations are removed
        for flag in &bitmask_ty.flags {
            assert!(!flag.annotations.iter().any(|a| a.ident.name == "position"));
        }
    } else {
        panic!("Expected bitmask definition");
    }

    // Clean up
    fs::remove_dir_all(&temp_dir).ok();
}