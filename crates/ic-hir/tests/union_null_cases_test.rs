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

mod common;

#[test]
fn test_union_with_null_case() {
    let input = r"
        enum MessageType {
            REQUEST,
            RESPONSE,
            ERROR,
            HEARTBEAT
        };
        
        union Message switch (MessageType) {
            case REQUEST:
                string request_data;
            case RESPONSE:
                long response_code;
            case ERROR:
                string error_message;
            case HEARTBEAT:
                null;  // No data for heartbeat messages
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union definition
    let union_def = graph
        .order
        .iter()
        .find_map(|id| {
            let def = graph.context.type_of(*id);
            if def.ident.name == "Message" {
                if let DefKind::Union(union_ty) = &def.kind {
                    Some(union_ty)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Message union not found");

    // Check that we have 4 variants
    assert_eq!(union_def.variants.len(), 4);

    // Find the null variant
    let null_variant = union_def
        .variants
        .iter()
        .find(|v| v.ident.name == "_null_case_3")
        .expect("Null variant not found");

    // Check that it has a null type
    assert!(matches!(null_variant.ty.kind, TyKind::Null));
}

#[test]
fn test_union_with_multiple_null_cases() {
    let input = r"
        enum Status {
            ACTIVE,
            INACTIVE,
            PENDING,
            UNKNOWN
        };
        
        union StatusInfo switch (Status) {
            case ACTIVE:
                long active_since;
            case INACTIVE:
                null;
            case PENDING:
                string pending_reason;
            case UNKNOWN:
                null;
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union definition
    let union_def = graph
        .order
        .iter()
        .find_map(|id| {
            let def = graph.context.type_of(*id);
            if def.ident.name == "StatusInfo" {
                if let DefKind::Union(union_ty) = &def.kind {
                    Some(union_ty)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("StatusInfo union not found");

    // Check that we have 4 variants
    assert_eq!(union_def.variants.len(), 4);

    // Count null variants
    let null_count = union_def
        .variants
        .iter()
        .filter(|v| matches!(v.ty.kind, TyKind::Null))
        .count();

    assert_eq!(null_count, 2, "Expected 2 null variants");
}

#[test]
fn test_union_with_default_null_case() {
    let input = r"
        union OptionalData switch (long) {
            case 1:
                string text;
            case 2:
                long number;
            default:
                null;
        };
    ";

    let (graph, _, _) = common::parse_and_resolve(input);
    assert!(graph.errors.is_empty());

    // Find the union definition
    let union_def = graph
        .order
        .iter()
        .find_map(|id| {
            let def = graph.context.type_of(*id);
            if def.ident.name == "OptionalData" {
                if let DefKind::Union(union_ty) = &def.kind {
                    Some(union_ty)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("OptionalData union not found");

    // Find the default variant
    let default_variant = union_def
        .variants
        .iter()
        .find(|v| v.is_default)
        .expect("Default variant not found");

    // Check that the default variant has a null type
    assert!(matches!(default_variant.ty.kind, TyKind::Null));
}
