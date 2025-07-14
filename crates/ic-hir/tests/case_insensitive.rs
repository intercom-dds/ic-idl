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

use ic_parse::{SourceMap, from_file};
use ic_preproc::ProcArgs;

#[test]
fn test_case_insensitive_type_resolution() {
    let idl = r"
        struct MyStruct {
            long value;
        };
        
        struct OtherStruct {
            MYSTRUCT s1;     // Uppercase
            mystruct s2;     // Lowercase
            MyStruct s3;     // Original case
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let hir = ic_hir::from_ast(ast.tree);

    // Should have no errors - all references should resolve
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_case_insensitive_module_paths() {
    let idl = r"
        module DDS {
            module XTypes {
                struct TypeIdentifier {
                    long id;
                };
            };
        };
        
        struct Container {
            dds::xtypes::typeidentifier t1;  // All lowercase
            DDS::Xtypes::TypeIdentifier t2;  // Mixed case
            DDS::XTYPES::TYPEIDENTIFIER t3;  // All uppercase
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let hir = ic_hir::from_ast(ast.tree);

    // Should have no errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_case_insensitive_primitive_types() {
    let idl = r"
        struct PrimitiveTypes {
            OCTET o1;
            octet o2;
            Octet o3;
            
            LONG l1;
            long l2;
            Long l3;
            
            UNSIGNED SHORT us1;
            unsigned short us2;
            Unsigned Short us3;
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let hir = ic_hir::from_ast(ast.tree);

    // Should have no errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_case_insensitive_enum_references() {
    let idl = r"
        enum Color {
            RED,
            GREEN,
            BLUE
        };
        
        struct Item {
            COLOR c1;  // Uppercase enum name
            color c2;  // Lowercase enum name
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let hir = ic_hir::from_ast(ast.tree);

    // Should have no errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_case_sensitive_same_module() {
    // Within the same module, names must still be unique when case-folded
    let idl = r"
        module Test {
            struct foo {
                long x;
            };
            
            struct FOO {  // Should be an error - duplicate definition
                long y;
            };
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);
    let hir = ic_hir::from_ast(ast.tree);

    // Should have an error about duplicate definition
    assert!(!hir.errors.is_empty());
}
