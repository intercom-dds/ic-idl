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

use ic_hir::hir::{DefKind, Numeric};
use ic_hir::merge::merge_hir_trees;

#[test]
fn test_merge_constant_struct_value_defid_update() {
    let input1 = r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
    ";

    let input2 = r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        
        const Point MY_POINT = {1, 2, 3};
    ";

    let parsed1 = ic_parse::from_str(input1);
    assert!(parsed1.errors.is_empty());
    let result1 = ic_hir::from_ast(parsed1.tree);
    assert!(result1.errors.is_empty());

    let parsed2 = ic_parse::from_str(input2);
    assert!(parsed2.errors.is_empty());
    let result2 = ic_hir::from_ast(parsed2.tree);
    assert!(result2.errors.is_empty());

    let merged = merge_hir_trees(&[result1, result2]);
    assert!(merged.errors.is_empty());

    let my_point = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "MY_POINT")
        .expect("MY_POINT constant not found");

    match &my_point.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Struct { ty, fields } => {
                let point_def = merged.context.definitions.get(*ty);
                assert_eq!(point_def.ident.name, "Point");
                assert!(matches!(&point_def.kind, DefKind::Struct(_)));

                assert_eq!(fields.len(), 3);
                match (&fields[0].1, &fields[1].1, &fields[2].1) {
                    (Numeric::Int32(1), Numeric::Int32(2), Numeric::Int32(3)) => {}
                    _ => panic!("Expected field values to be 1, 2, 3"),
                }
            }
            _ => panic!("Expected struct initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}

#[test]
fn test_merge_constant_references_correct_type() {
    let input = r"
        struct Point {
            int32 x;
            int32 y;
            int32 z;
        };
        
        const Point MY_POINT = {1, 2, 3};
    ";

    let parsed1 = ic_parse::from_str("");
    assert!(parsed1.errors.is_empty());
    let result1 = ic_hir::from_ast(parsed1.tree);
    assert!(result1.errors.is_empty());

    let parsed2 = ic_parse::from_str(input);
    assert!(parsed2.errors.is_empty());
    let result2 = ic_hir::from_ast(parsed2.tree);
    assert!(result2.errors.is_empty());

    let merged = merge_hir_trees(&[result1, result2]);
    assert!(merged.errors.is_empty());

    let my_point = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "MY_POINT")
        .expect("MY_POINT constant not found");

    match &my_point.1.kind {
        DefKind::Const(const_ty) => match &const_ty.value {
            Numeric::Struct { ty, .. } => {
                let ty_def = merged.context.definitions.get(*ty);
                assert_eq!(
                    ty_def.ident.name, "Point",
                    "Constant should reference Point struct"
                );
                assert!(
                    matches!(&ty_def.kind, DefKind::Struct(_)),
                    "Referenced type should be a struct, not {:?}",
                    ty_def.kind
                );
            }
            _ => panic!("Expected struct initialization"),
        },
        _ => panic!("Expected constant definition"),
    }
}
