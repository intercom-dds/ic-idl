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

use ic_hir::hir::{DefKind, TyKind};
use ic_parse::{SourceMap, from_file};
use ic_preproc::ProcArgs;

#[test]
fn test_array_bounds_in_struct() {
    let idl = r"
        struct MyStruct {
            long array1[5];
            long array2[10];
            long array3[2 + 3];
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);

    // Check parse errors
    assert_eq!(ast.errors.len(), 0, "Parse errors: {:?}", ast.errors);

    let hir = ic_hir::from_ast(ast.tree);

    // Should have no HIR errors
    assert_eq!(hir.errors.len(), 0, "HIR errors: {:?}", hir.errors);

    // Find the struct and check array bounds
    for def in hir.iter() {
        if def.ident.name == "MyStruct" {
            if let DefKind::Struct(s) = &def.kind {
                // array1[5]
                if let TyKind::Array { ty: _, len } = &s.members[0].ty.kind {
                    assert_eq!(*len, 5);
                }
                // array2[10]
                if let TyKind::Array { ty: _, len } = &s.members[1].ty.kind {
                    assert_eq!(*len, 10);
                }
                // array3[2 + 3]
                if let TyKind::Array { ty: _, len } = &s.members[2].ty.kind {
                    assert_eq!(*len, 5);
                }
            }
        }
    }
}

#[test]
fn test_array_bounds_in_union() {
    let idl = r"
        union MyUnion switch (long) {
            case 1:
                char buffer[256];
            case 2:
                long data[4 * 2];
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);

    // Check parse errors
    assert_eq!(ast.errors.len(), 0, "Parse errors: {:?}", ast.errors);

    let hir = ic_hir::from_ast(ast.tree);

    // Should have no HIR errors
    assert_eq!(hir.errors.len(), 0, "HIR errors: {:?}", hir.errors);

    // Find the union and check array bounds
    for def in hir.iter() {
        if def.ident.name == "MyUnion" {
            if let DefKind::Union(u) = &def.kind {
                // buffer[256]
                if let TyKind::Array { ty: _, len } = &u.variants[0].ty.kind {
                    assert_eq!(*len, 256);
                }
                // data[4 * 2]
                if let TyKind::Array { ty: _, len } = &u.variants[1].ty.kind {
                    assert_eq!(*len, 8);
                }
            }
        }
    }
}

#[test]
fn test_array_bounds_in_exception() {
    let idl = r"
        exception MyException {
            string messages[3];
            octet codes[1 << 3];  // 8
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);

    // Check parse errors
    assert_eq!(ast.errors.len(), 0, "Parse errors: {:?}", ast.errors);

    let hir = ic_hir::from_ast(ast.tree);

    // Should have no HIR errors
    assert_eq!(hir.errors.len(), 0, "HIR errors: {:?}", hir.errors);

    // Find the exception and check array bounds
    for def in hir.iter() {
        if def.ident.name == "MyException" {
            if let DefKind::Except(e) = &def.kind {
                // messages[3]
                if let TyKind::Array { ty: _, len } = &e.members[0].ty.kind {
                    assert_eq!(*len, 3);
                }
                // codes[1 << 3]
                if let TyKind::Array { ty: _, len } = &e.members[1].ty.kind {
                    assert_eq!(*len, 8);
                }
            }
        }
    }
}

#[test]
fn test_multidimensional_arrays() {
    let idl = r"
        struct Matrix {
            long matrix2d[3][4];
            long matrix3d[2][3][4];
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);

    // Check parse errors
    assert_eq!(ast.errors.len(), 0, "Parse errors: {:?}", ast.errors);

    let hir = ic_hir::from_ast(ast.tree);

    // Should have no HIR errors
    assert_eq!(hir.errors.len(), 0, "HIR errors: {:?}", hir.errors);

    // Find the struct and check nested array bounds
    for def in hir.iter() {
        if def.ident.name == "Matrix" {
            if let DefKind::Struct(s) = &def.kind {
                // matrix2d[3][4] - In HIR, arrays are nested with innermost dimension first
                if let TyKind::Array { ty, len } = &s.members[0].ty.kind {
                    assert_eq!(*len, 3); // First dimension
                    if let TyKind::Array { ty: _, len } = &ty.kind {
                        assert_eq!(*len, 4); // Second dimension
                    }
                }
                // matrix3d[2][3][4]
                if let TyKind::Array { ty, len } = &s.members[1].ty.kind {
                    assert_eq!(*len, 2); // First dimension
                    if let TyKind::Array { ty, len } = &ty.kind {
                        assert_eq!(*len, 3); // Second dimension
                        if let TyKind::Array { ty: _, len } = &ty.kind {
                            assert_eq!(*len, 4); // Third dimension
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_array_bounds_with_enum_values() {
    let idl = r"
        enum Sizes {
            SMALL = 4,
            MEDIUM = 8,
            LARGE = 16
        };
        
        struct Storage {
            octet small_buffer[Sizes::SMALL];
            octet large_buffer[Sizes::LARGE];
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", idl);
    let ast = from_file(file_id, ProcArgs::default(), &mut vfs);

    // Check parse errors
    assert_eq!(ast.errors.len(), 0, "Parse errors: {:?}", ast.errors);

    let hir = ic_hir::from_ast(ast.tree);

    // Should have no HIR errors
    assert_eq!(hir.errors.len(), 0, "HIR errors: {:?}", hir.errors);

    // Find the struct and check array bounds
    for def in hir.iter() {
        if def.ident.name == "Storage" {
            if let DefKind::Struct(s) = &def.kind {
                // small_buffer[Sizes::SMALL] = 4
                if let TyKind::Array { ty: _, len } = &s.members[0].ty.kind {
                    assert_eq!(*len, 4);
                }
                // large_buffer[Sizes::LARGE] = 16
                if let TyKind::Array { ty: _, len } = &s.members[1].ty.kind {
                    assert_eq!(*len, 16);
                }
            }
        }
    }
}
