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

use std::assert_matches;

use ic_parse::from_str;
use ic_syntax::Item;

#[test]
fn parse_simple_module() {
    let result = from_str("module Foo { struct Bar {}; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Module(def) => {
            assert_eq!(def.name.name, "Foo");
            assert_eq!(def.items.len(), 1);
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parse_escaped_identifiers() {
    let result = from_str("struct _struct { long _long; long ordinary; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.name.name, "struct");
            assert_matches!(
                &def.fields[0].declarators[0],
                ic_syntax::Declarator::Name(ident) if ident.name == "long"
            );
            assert_matches!(
                &def.fields[1].declarators[0],
                ic_syntax::Declarator::Name(ident) if ident.name == "ordinary"
            );
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn preserve_invalid_escaped_identifiers() {
    let result = from_str("struct _123 {}; struct _ {};");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    assert_matches!(
        &result.tree[0],
        Item::Struct(def) if def.name.name == "_123"
    );
    assert_matches!(
        &result.tree[1],
        Item::Struct(def) if def.name.name == "_"
    );
}

#[test]
fn parse_nested_modules() {
    let result = from_str("module Parent { module Child { struct S {}; }; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Module(parent) => {
            assert_eq!(parent.name.name, "Parent");
            assert_eq!(parent.items.len(), 1);

            match &parent.items[0] {
                Item::Module(child) => {
                    assert_eq!(child.name.name, "Child");
                    assert_eq!(child.items.len(), 1);
                }
                _ => panic!("expected nested module"),
            }
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parse_annotated_module() {
    let result = from_str("@version(1) module Foo { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Module(def) => {
            assert_eq!(def.meta.annotations.len(), 1);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "version");
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parse_simple_struct() {
    let result = from_str("struct Point { long x; long y; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.name.name, "Point");
            assert_eq!(def.fields.len(), 2);
            assert!(def.parent.is_none());
        }
        _ => panic!("expected struct, got {:?}", result.tree[0]),
    }
}

#[test]
fn parse_empty_struct() {
    let result = from_str("struct Empty {};");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.name.name, "Empty");
            assert!(def.fields.is_empty());
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_forward_declaration() {
    let result = from_str("struct Forward;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Decl(decl) => {
            assert_eq!(decl.name.name, "Forward");
        }
        _ => panic!("expected forward declaration"),
    }
}

#[test]
fn parse_struct_with_inheritance() {
    let result = from_str("struct Point3D : Point { long z; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.name.name, "Point3D");
            assert!(def.parent.is_some());
            let parent = def.parent.as_ref().unwrap();
            assert_eq!(parent.segments.len(), 1);
            assert_eq!(parent.segments[0].name, "Point");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_struct_with_annotation() {
    let result = from_str("@final struct Point { long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.name.name, "Point");
            assert_eq!(def.meta.annotations.len(), 1);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "final");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_struct_with_annotated_member() {
    let result = from_str("struct Point { @key long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.fields.len(), 1);
            assert_eq!(def.fields[0].meta.annotations.len(), 1);
            assert_eq!(
                def.fields[0].meta.annotations[0].path.segments[0].name,
                "key"
            );
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_annotation_with_args() {
    let result = from_str("@range(min = 0, max = 100) struct Bounded { long value; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.meta.annotations.len(), 1);
            let ann = &def.meta.annotations[0];
            assert_eq!(ann.path.segments[0].name, "range");
            assert_eq!(ann.arguments.len(), 2);
            assert_eq!(ann.arguments[0].name.as_ref().unwrap().name, "min");
            assert_eq!(ann.arguments[1].name.as_ref().unwrap().name, "max");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_multiple_annotations() {
    let result = from_str("@final @mutable struct Point { long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.meta.annotations.len(), 2);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "final");
            assert_eq!(def.meta.annotations[1].path.segments[0].name, "mutable");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_array_member() {
    let result = from_str("struct Arrays { long values[10]; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Struct(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    assert_eq!(def.fields.len(), 1);
    let ic_syntax::Declarator::Array(arr) = &def.fields[0].declarators[0] else {
        panic!("expected array declarator")
    };
    assert_eq!(arr.name.name, "values");
    assert_eq!(arr.bounds.len(), 1);
}

#[test]
fn parse_multiple_declarators() {
    let result = from_str("struct Point { long x, y, z; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.fields.len(), 1);
            assert_eq!(def.fields[0].declarators.len(), 3);
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_annotation_with_keyword_name() {
    // @default is a keyword but valid as annotation name
    let result = from_str("struct Point { @default(123) long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.fields[0].meta.annotations.len(), 1);
            assert_eq!(
                def.fields[0].meta.annotations[0].path.segments[0].name,
                "default"
            );
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_annotation_between_struct_and_name() {
    let result = from_str("struct @foo Point { long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);
}

#[test]
fn parse_multiple_structs() {
    let result = from_str(
        r"
        struct Point { long x; long y; };
        struct Point3D : Point { long z; };
        ",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 2);
}

#[test]
fn parse_annotations_in_multiple_positions() {
    // Annotations can appear after type and after declarator
    let result = from_str("struct foo { int32 @baz value @bar; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.fields.len(), 1);
            let field = &def.fields[0];
            // Both @baz and @bar should be collected
            assert_eq!(field.meta.annotations.len(), 2);
            assert_eq!(field.meta.annotations[0].path.segments[0].name, "baz");
            assert_eq!(field.meta.annotations[1].path.segments[0].name, "bar");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_trailing_annotation_after_brace() {
    // Annotation after closing brace: `} @boom;`
    let result = from_str("struct foo { long x; } @boom;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.meta.annotations.len(), 1);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "boom");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_annotations_everywhere() {
    // All positions: before struct, after brace, in members
    let result = from_str("@pre struct foo { int32 @mid value @post; } @trailing;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Struct(def) => {
            // @pre and @trailing on struct
            assert_eq!(def.meta.annotations.len(), 2);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "pre");
            assert_eq!(def.meta.annotations[1].path.segments[0].name, "trailing");

            // @mid and @post on field
            assert_eq!(def.fields[0].meta.annotations.len(), 2);
            assert_eq!(
                def.fields[0].meta.annotations[0].path.segments[0].name,
                "mid"
            );
            assert_eq!(
                def.fields[0].meta.annotations[1].path.segments[0].name,
                "post"
            );
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_recovers_after_error() {
    let result = from_str(
        r"
        struct Bad { invalid syntax };
        struct Good { long x; };
        ",
    );
    // Should have errors
    assert!(!result.errors.is_empty());
    // But should still parse the good struct
    assert_eq!(result.tree.len(), 1);
    match &result.tree[0] {
        Item::Struct(def) => {
            assert_eq!(def.name.name, "Good");
        }
        _ => panic!("expected struct"),
    }
}

#[test]
fn parse_recovers_multiple_errors() {
    let result = from_str(
        r"
        struct A { bad };
        struct B { also bad };
        struct C { long x; };
        ",
    );
    // Should have multiple errors
    assert!(result.errors.len() >= 2);
    // But should still parse the good struct
    assert_eq!(result.tree.len(), 1);
}
