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
use ic_syntax::{Expr, Item, LiteralValue};

#[test]
fn parse_simple_enum() {
    let result = from_str("enum Color { RED, GREEN, BLUE };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::EnumValue(def) => {
            assert_eq!(def.ident.name, "Color");
            assert_eq!(def.fields.len(), 3);
            assert_eq!(def.fields[0].ident.name, "RED");
            assert_eq!(def.fields[1].ident.name, "GREEN");
            assert_eq!(def.fields[2].ident.name, "BLUE");
        }
        _ => panic!("expected enum, got {:?}", result.tree[0]),
    }
}

#[test]
fn parse_empty_enum() {
    let result = from_str("enum Empty {};");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::EnumValue(def) => {
            assert_eq!(def.ident.name, "Empty");
            assert!(def.fields.is_empty());
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parse_enum_trailing_comma_rejected() {
    let result = from_str("enum Color { RED, GREEN, BLUE, };");
    assert!(
        !result.errors.is_empty(),
        "trailing comma should be rejected"
    );
}

#[test]
fn parse_enum_with_values() {
    let result = from_str("enum Priority { LOW = 0, MEDIUM = 5, HIGH = 10 };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::EnumValue(def) => {
            assert_eq!(def.fields.len(), 3);
            assert_eq!(def.fields[0].ident.name, "LOW");
            assert!(def.fields[0].value.is_some());
            assert_eq!(def.fields[1].ident.name, "MEDIUM");
            assert!(def.fields[1].value.is_some());
            assert_eq!(def.fields[2].ident.name, "HIGH");
            assert!(def.fields[2].value.is_some());

            // Check actual values
            match &def.fields[0].value {
                Some(Expr::Literal(lit)) => {
                    assert_eq!(lit.value, LiteralValue::Int(0));
                }
                _ => panic!("expected literal value"),
            }
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parse_enum_with_annotation() {
    let result = from_str("@extensibility(FINAL) enum Color { RED, GREEN, BLUE };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::EnumValue(def) => {
            assert_eq!(def.annotations.len(), 1);
            assert_eq!(def.annotations[0].ident.segments[0].name, "extensibility");
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parse_enum_with_annotated_values() {
    let result = from_str(
        r"enum Status {
            @value(0) PENDING,
            @value(1) ACTIVE,
            @value(2) COMPLETED
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::EnumValue(def) => {
            assert_eq!(def.fields.len(), 3);
            assert_eq!(def.fields[0].annotations.len(), 1);
            assert_eq!(def.fields[0].annotations[0].ident.segments[0].name, "value");
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parse_enum_in_module() {
    let result = from_str("module Foo { enum Color { RED }; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::ModuleValue(module) => {
            assert_eq!(module.definitions.len(), 1);
            match &module.definitions[0] {
                Item::EnumValue(def) => {
                    assert_eq!(def.ident.name, "Color");
                }
                _ => panic!("expected enum in module"),
            }
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parse_simple_bitmask() {
    let result = from_str("bitmask Permissions { READ, WRITE, EXECUTE };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::BitmaskValue(def) => {
            assert_eq!(def.ident.name, "Permissions");
            assert_eq!(def.bits.len(), 3);
            assert_eq!(def.bits[0].ident.name, "READ");
            assert_eq!(def.bits[1].ident.name, "WRITE");
            assert_eq!(def.bits[2].ident.name, "EXECUTE");
        }
        _ => panic!("expected bitmask, got {:?}", result.tree[0]),
    }
}

#[test]
fn parse_empty_bitmask() {
    let result = from_str("bitmask Empty {};");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::BitmaskValue(def) => {
            assert_eq!(def.ident.name, "Empty");
            assert!(def.bits.is_empty());
        }
        _ => panic!("expected bitmask"),
    }
}

#[test]
fn parse_bitmask_trailing_comma_rejected() {
    let result = from_str("bitmask Flags { A, B, C, };");
    assert!(
        !result.errors.is_empty(),
        "trailing comma should be rejected"
    );
}

#[test]
fn parse_bitmask_with_values() {
    let result = from_str("bitmask Flags { NONE = 0, FLAG_A = 1, FLAG_B = 2 };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::BitmaskValue(def) => {
            assert_eq!(def.bits.len(), 3);
            assert!(def.bits[0].value.is_some());
            assert!(def.bits[1].value.is_some());
            assert!(def.bits[2].value.is_some());
        }
        _ => panic!("expected bitmask"),
    }
}

#[test]
fn parse_bitmask_with_annotation() {
    let result = from_str("@bit_bound(8) bitmask Flags { A, B };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::BitmaskValue(def) => {
            assert_eq!(def.annotations.len(), 1);
            assert_eq!(def.annotations[0].ident.segments[0].name, "bit_bound");
        }
        _ => panic!("expected bitmask"),
    }
}

#[test]
fn parse_bitmask_with_annotated_bits() {
    let result = from_str(
        r"bitmask Permissions {
            @position(0) READ,
            @position(1) WRITE,
            @position(2) EXECUTE
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::BitmaskValue(def) => {
            assert_eq!(def.bits.len(), 3);
            assert_eq!(def.bits[0].annotations.len(), 1);
            assert_eq!(
                def.bits[0].annotations[0].ident.segments[0].name,
                "position"
            );
        }
        _ => panic!("expected bitmask"),
    }
}

#[test]
fn parse_bitmask_in_module() {
    let result = from_str("module Foo { bitmask Flags { A }; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::ModuleValue(module) => {
            assert_eq!(module.definitions.len(), 1);
            match &module.definitions[0] {
                Item::BitmaskValue(def) => {
                    assert_eq!(def.ident.name, "Flags");
                }
                _ => panic!("expected bitmask in module"),
            }
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parse_mixed_definitions() {
    let result = from_str(
        r"
        enum Color { RED, GREEN, BLUE };
        bitmask Flags { A, B };
        struct Point { long x; long y; };
        ",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 3);

    assert!(matches!(&result.tree[0], Item::EnumValue(_)));
    assert!(matches!(&result.tree[1], Item::BitmaskValue(_)));
    assert!(matches!(&result.tree[2], Item::StructValue(_)));
}

#[test]
fn parse_enum_trailing_annotation() {
    // Annotation after closing brace
    let result = from_str("enum Color { RED } @foo;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::EnumValue(def) => {
            assert_eq!(def.annotations.len(), 1);
            assert_eq!(def.annotations[0].ident.segments[0].name, "foo");
        }
        _ => panic!("expected enum"),
    }
}

#[test]
fn parse_bitmask_trailing_annotation() {
    // Annotation after closing brace
    let result = from_str("bitmask Flags { A } @bar;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::BitmaskValue(def) => {
            assert_eq!(def.annotations.len(), 1);
            assert_eq!(def.annotations[0].ident.segments[0].name, "bar");
        }
        _ => panic!("expected bitmask"),
    }
}
