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

mod common;

#[test]
fn test_union_with_enum_case_labels() {
    let input = r"
        enum Color {
            RED,
            GREEN,
            BLUE
        };
        
        union ColorData switch (Color) {
            case RED: long r;
            case GREEN: long g;
            case BLUE: long b;
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union
    let union_def = graph
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ColorData" && matches!(def.kind, DefKind::Union(_)))
        .expect("Should find ColorData union");

    if let DefKind::Union(union_ty) = &union_def.1.kind {
        assert_eq!(union_ty.variants.len(), 3);

        // Check that each variant has a numeric label with the enum value
        for variant in &union_ty.variants {
            assert_eq!(variant.labels.len(), 1);

            // The label should be a Const reference to the enum constant
            assert!(matches!(variant.labels[0].value, Numeric::Const(_)));
        }
    } else {
        panic!("ColorData should be a union");
    }
}

#[test]
fn test_union_with_const_case_labels() {
    let input = r"
        const long STATUS_OK = 200;
        const long STATUS_ERROR = 500;
        
        union Response switch (long) {
            case STATUS_OK: string message;
            case STATUS_ERROR: string error;
            default: octet data[100];
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union
    let union_def = graph
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Response" && matches!(def.kind, DefKind::Union(_)))
        .expect("Should find Response union");

    if let DefKind::Union(union_ty) = &union_def.1.kind {
        assert_eq!(union_ty.variants.len(), 3);

        // First variant should have the value of STATUS_OK (200)
        assert_eq!(union_ty.variants[0].labels.len(), 1);
        // Should be a Const reference to STATUS_OK
        assert!(matches!(
            union_ty.variants[0].labels[0].value,
            Numeric::Const(_)
        ));

        // Second variant should have the value of STATUS_ERROR (500)
        assert_eq!(union_ty.variants[1].labels.len(), 1);
        // Should be a Const reference to STATUS_ERROR
        assert!(matches!(
            union_ty.variants[1].labels[0].value,
            Numeric::Const(_)
        ));

        // Third variant is default
        assert!(union_ty.variants[2].is_default);
        assert_eq!(union_ty.variants[2].labels.len(), 0);
    } else {
        panic!("Response should be a union");
    }
}

#[test]
fn test_union_with_numeric_case_labels() {
    let input = r"
        union NumberData switch (long) {
            case 1: string one;
            case 2: string two;
            case 100: string hundred;
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union
    let union_def = graph
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "NumberData" && matches!(def.kind, DefKind::Union(_)))
        .expect("Should find NumberData union");

    if let DefKind::Union(union_ty) = &union_def.1.kind {
        assert_eq!(union_ty.variants.len(), 3);

        // Check that numeric literals are stored as Int32 values, not Const
        assert_eq!(union_ty.variants[0].labels.len(), 1);
        assert_eq!(union_ty.variants[0].labels[0].value, Numeric::Int32(1));
        assert_eq!(union_ty.variants[1].labels.len(), 1);
        assert_eq!(union_ty.variants[1].labels[0].value, Numeric::Int32(2));
        assert_eq!(union_ty.variants[2].labels.len(), 1);
        assert_eq!(union_ty.variants[2].labels[0].value, Numeric::Int32(100));
    } else {
        panic!("NumberData should be a union");
    }
}

#[test]
fn test_union_with_mixed_case_labels() {
    let input = r"
        enum Status { PENDING, ACTIVE, DONE };
        const long SPECIAL_CODE = 999;
        
        union Data switch (long) {
            case PENDING: string pending_msg;
            case ACTIVE: string active_msg;
            case SPECIAL_CODE: string special_msg;
            case 42: string answer;
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union
    let union_def = graph
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Data" && matches!(def.kind, DefKind::Union(_)))
        .expect("Should find Data union");

    if let DefKind::Union(union_ty) = &union_def.1.kind {
        assert_eq!(union_ty.variants.len(), 4);

        // First two should have Const references to enum values
        assert!(matches!(
            union_ty.variants[0].labels[0].value,
            Numeric::Const(_)
        )); // PENDING
        assert!(matches!(
            union_ty.variants[1].labels[0].value,
            Numeric::Const(_)
        )); // ACTIVE

        // Third should have Const reference to SPECIAL_CODE
        assert!(matches!(
            union_ty.variants[2].labels[0].value,
            Numeric::Const(_)
        )); // SPECIAL_CODE

        // Fourth should be numeric literal
        assert_eq!(union_ty.variants[3].labels[0].value, Numeric::Int32(42));
    } else {
        panic!("Data should be a union");
    }
}
