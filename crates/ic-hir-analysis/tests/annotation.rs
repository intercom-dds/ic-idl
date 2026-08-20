// Copyright 2026 KONGSBERG
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

#[path = "../../ic-hir/tests/common/mod.rs"]
mod common;

use ic_hir::hir::{Def, DefKind};
use ic_hir_analysis::annotation::{
    Extensibility, bit_bound, default_value, doc, extensibility, is_external, is_key,
    is_must_understand, is_nested, is_newtype, is_non_serialized, is_optional,
};

fn def<'a>(hir: &'a ic_hir::ResolvedGraph, name: &str) -> &'a Def {
    hir.context
        .definitions
        .iter()
        .find_map(|(_, def)| (def.ident.name == name).then_some(def))
        .unwrap()
}

#[test]
fn resolves_definition_properties() {
    let hir = common::parse_and_resolve_successfully(
        r"
        @default_nested
        module Outer {
            struct InheritedNested {};
            @nested(FALSE) struct NotNested {};
        };

        @FINAL struct FinalType {};
        @extensibility(MUTABLE) struct MutableType {};
        @appendable struct AppendableType {};
        struct DefaultType {};
        ",
    );

    assert!(is_nested(&hir.context, def(&hir, "InheritedNested")));
    assert!(!is_nested(&hir.context, def(&hir, "NotNested")));
    assert_eq!(
        extensibility(&hir.context, def(&hir, "FinalType")),
        Extensibility::Final
    );
    assert_eq!(
        extensibility(&hir.context, def(&hir, "MutableType")),
        Extensibility::Mutable
    );
    assert_eq!(
        extensibility(&hir.context, def(&hir, "AppendableType")),
        Extensibility::Appendable
    );
    assert_eq!(
        extensibility(&hir.context, def(&hir, "DefaultType")),
        Extensibility::Appendable
    );
}

#[test]
fn resolves_annotation_properties() {
    let hir = common::parse_and_resolve_successfully(
        r#"
        struct Properties {
            @key long key_value;
            @key(FALSE) long non_key_value;
            @optional long optional_value;
            @optional(FALSE) long required_value;
            @shared long shared_value;
            @shared(FALSE) long direct_value;
            @external long external_value;
            @non_serialized long non_serialized_value;
            @non_serialized(FALSE) long serialized_value;
            @must_understand long understood_value;
            @must_understand(FALSE) long ignored_value;
            @default(42) long defaulted_value;
        };

        @BIT_BOUND(16)
        enum Bounded {
            FIRST,
            SECOND
        };

        @EXTERNAL typedef long ExternalAlias;
        @external(FALSE) typedef long DirectAlias;

        @EXT::NEWTYPE typedef long NewtypeAlias;
        @ext::newtype(FALSE) typedef long PlainAlias;

        const string DOC_TEXT = "Built-in documentation";
        @DOC(DOC_TEXT) struct Documented {};

        module Custom {
            @annotation doc {
                string text;
            };
        };

        @Custom::doc("Custom documentation") struct CustomDocumented {};

        union Choice switch (@key long) {
            case 0: @optional long value;
        };
        "#,
    );

    let DefKind::Struct(properties) = &def(&hir, "Properties").kind else {
        panic!("Properties is not a struct");
    };
    assert!(is_key(&hir.context, &properties.members[0]));
    assert!(!is_key(&hir.context, &properties.members[1]));
    assert!(is_optional(&hir.context, &properties.members[2]));
    assert!(!is_optional(&hir.context, &properties.members[3]));
    assert!(is_external(&hir.context, &properties.members[4]));
    assert!(!is_external(&hir.context, &properties.members[5]));
    assert!(is_external(&hir.context, &properties.members[6]));
    assert!(is_non_serialized(&hir.context, &properties.members[7]));
    assert!(!is_non_serialized(&hir.context, &properties.members[8]));
    assert!(is_must_understand(&hir.context, &properties.members[9]));
    assert!(!is_must_understand(&hir.context, &properties.members[10]));
    assert_eq!(
        default_value(&hir.context, &properties.members[11])
            .map(|value| hir.context.unsigned_value(value)),
        Some(42)
    );
    assert_eq!(
        bit_bound(&hir.context, def(&hir, "Bounded"))
            .map(|value| hir.context.unsigned_value(value)),
        Some(16)
    );
    assert!(is_external(&hir.context, def(&hir, "ExternalAlias")));
    assert!(!is_external(&hir.context, def(&hir, "DirectAlias")));
    assert!(is_newtype(&hir.context, def(&hir, "NewtypeAlias")));
    assert!(!is_newtype(&hir.context, def(&hir, "PlainAlias")));
    assert_eq!(
        doc(&hir.context, &def(&hir, "Documented").annotations[0]).as_deref(),
        Some("Built-in documentation")
    );
    assert_eq!(
        doc(&hir.context, &def(&hir, "CustomDocumented").annotations[0]),
        None
    );

    let DefKind::Union(choice) = &def(&hir, "Choice").kind else {
        panic!("Choice is not a union");
    };
    assert!(is_key(&hir.context, &choice.disc));
    assert!(is_optional(&hir.context, &choice.variants[0]));
}
