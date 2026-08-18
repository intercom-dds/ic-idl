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
fn test_annotation_boolean_default() {
    let input = r"
        @annotation sample {
            boolean enable default TRUE;
            boolean debug default FALSE;
        };
    ";

    let hir = common::parse_and_resolve_successfully(input);

    // Find the annotation definition
    let ann_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "sample")
        .expect("annotation not found");

    if let DefKind::Annotation(ann) = &ann_def.1.kind {
        assert_eq!(ann.params.len(), 2);

        // Check first member
        assert_eq!(ann.params[0].ident.name, "enable");
        assert_eq!(ann.params[0].default, Some(Numeric::Bool(true)));

        // Check second member
        assert_eq!(ann.params[1].ident.name, "debug");
        assert_eq!(ann.params[1].default, Some(Numeric::Bool(false)));
    } else {
        panic!("Expected annotation definition");
    }
}

#[test]
fn test_annotation_numeric_defaults() {
    let input = r"
        @annotation numeric_test {
            long value default 42;
            float ratio default 3.15;
            uint8 priority default 5;
        };
    ";

    let hir = common::parse_and_resolve_successfully(input);

    // Find the annotation definition
    let ann_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "numeric_test")
        .expect("annotation not found");

    if let DefKind::Annotation(ann) = &ann_def.1.kind {
        assert_eq!(ann.params.len(), 3);

        // Check members
        assert_eq!(ann.params[0].ident.name, "value");
        assert_eq!(ann.params[0].default, Some(Numeric::Int32(42)));

        assert_eq!(ann.params[1].ident.name, "ratio");
        assert!(
            matches!(ann.params[1].default, Some(Numeric::Float(f)) if (f - 3.15).abs() < f32::EPSILON)
        );

        assert_eq!(ann.params[2].ident.name, "priority");
        assert_eq!(ann.params[2].default, Some(Numeric::UInt8(5)));
    } else {
        panic!("Expected annotation definition");
    }
}

#[test]
fn test_annotation_string_default() {
    let input = r#"
        @annotation meta {
            string description default "No description";
        };
    "#;

    let hir = common::parse_and_resolve_successfully(input);

    // Find the annotation definition
    let ann_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "meta")
        .expect("annotation not found");

    if let DefKind::Annotation(ann) = &ann_def.1.kind {
        assert_eq!(ann.params.len(), 1);
        assert_eq!(ann.params[0].ident.name, "description");
        assert_eq!(
            ann.params[0].default,
            Some(Numeric::String("No description".to_string()))
        );
    } else {
        panic!("Expected annotation definition");
    }
}

#[test]
fn test_annotation_no_default() {
    let input = r"
        @annotation required {
            string name;
            boolean enabled default TRUE;
        };
    ";

    let hir = common::parse_and_resolve_successfully(input);

    // Find the annotation definition
    let ann_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "required")
        .expect("annotation not found");

    if let DefKind::Annotation(ann) = &ann_def.1.kind {
        assert_eq!(ann.params.len(), 2);

        // First member has no default
        assert_eq!(ann.params[0].ident.name, "name");
        assert_eq!(ann.params[0].default, None);

        // Second member has default
        assert_eq!(ann.params[1].ident.name, "enabled");
        assert_eq!(ann.params[1].default, Some(Numeric::Bool(true)));
    } else {
        panic!("Expected annotation definition");
    }
}
