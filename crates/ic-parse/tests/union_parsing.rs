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

use ic_parse::from_str;
use ic_syntax::{Item, Label, UnionElement};

#[test]
fn parse_simple_union() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 0: long intVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.ident.name, "MyUnion");
            assert_eq!(def.fields.len(), 1);
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_multiple_cases() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 0: long intVal;
            case 1: string strVal;
            case 2: float floatVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.fields.len(), 3);
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_fallthrough_cases() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 1:
            case 2:
            case 3: string strVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.fields.len(), 1);
            let field = &def.fields[0];
            assert_eq!(field.labels.len(), 3);
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_default() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 0: long intVal;
            default: string defaultVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::UnionValue(def) = &result.tree[0] else {
        panic!("expected union")
    };
    assert_eq!(def.fields.len(), 2);
    let Label::Default(_) = &def.fields[1].labels[0] else {
        panic!("expected default label")
    };
}

#[test]
fn parse_union_default_only() {
    let result = from_str(
        "union MyUnion switch (long) {
            default: string defaultVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::UnionValue(def) = &result.tree[0] else {
        panic!("expected union")
    };
    assert_eq!(def.fields.len(), 1);
    let Label::Default(_) = &def.fields[0].labels[0] else {
        panic!("expected default label")
    };
}

#[test]
fn parse_union_with_short_discriminator() {
    let result = from_str(
        "union MyUnion switch (short) {
            case 0: long val;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => match &def.disc.ty {
            ic_syntax::Type::Path(path) => {
                assert_eq!(path.segments[0].name, "int16");
            }
            _ => panic!("expected path type"),
        },
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_boolean_discriminator() {
    let result = from_str(
        "union MyUnion switch (boolean) {
            case TRUE: long trueVal;
            case FALSE: string falseVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => match &def.disc.ty {
            ic_syntax::Type::Path(path) => {
                assert_eq!(path.segments[0].name, "boolean");
            }
            _ => panic!("expected path type"),
        },
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_enum_discriminator() {
    let result = from_str(
        "union MyUnion switch (MyEnum) {
            case VALUE1: long val1;
            case VALUE2: string val2;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => match &def.disc.ty {
            ic_syntax::Type::Path(path) => {
                assert_eq!(path.segments[0].name, "MyEnum");
            }
            _ => panic!("expected path type"),
        },
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_forward_declaration() {
    let result = from_str("union Forward;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::DeclValue(decl) => {
            assert_eq!(decl.ident.name, "Forward");
        }
        _ => panic!("expected forward declaration"),
    }
}

#[test]
fn parse_union_with_sequence_member() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 0: sequence<long> seqVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::UnionValue(def) = &result.tree[0] else {
        panic!("expected union")
    };
    let UnionElement::Member(member) = &def.fields[0].field else {
        panic!("expected member")
    };
    assert!(matches!(member.ty.as_ref(), ic_syntax::Type::Sequence(_)));
}

#[test]
fn parse_union_with_array_member() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 0: long arrVal[10];
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::UnionValue(def) = &result.tree[0] else {
        panic!("expected union")
    };
    let UnionElement::Member(member) = &def.fields[0].field else {
        panic!("expected member")
    };
    let ic_syntax::Declarator::Array(arr) = &member.decl else {
        panic!("expected array declarator")
    };
    assert_eq!(arr.ident.name, "arrVal");
    assert_eq!(arr.bounds.len(), 1);
}

#[test]
fn parse_union_with_annotation() {
    let result = from_str(
        "@custom union MyUnion switch (long) {
            case 0: long val;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.annotations.len(), 1);
            assert_eq!(def.annotations[0].ident.segments[0].name, "custom");
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_annotated_discriminator() {
    let result = from_str(
        "union MyUnion switch (@key long) {
            case 0: long val;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.disc.annotations.len(), 1);
            assert_eq!(def.disc.annotations[0].ident.segments[0].name, "key");
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_negative_case() {
    let result = from_str(
        "union MyUnion switch (long) {
            case -1: long negVal;
            case 0: long zeroVal;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.fields.len(), 2);
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_with_expression_case() {
    let result = from_str(
        "union MyUnion switch (long) {
            case 1 + 2: long val;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::UnionValue(def) => {
            assert_eq!(def.fields.len(), 1);
        }
        _ => panic!("expected union"),
    }
}

#[test]
fn parse_union_in_module() {
    let result = from_str(
        "module MyModule {
            union MyUnion switch (long) {
                case 0: long val;
            };
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::ModuleValue(module) => {
            assert_eq!(module.definitions.len(), 1);
            assert!(matches!(&module.definitions[0], Item::UnionValue(_)));
        }
        _ => panic!("expected module"),
    }
}
