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

use ic_hir::hir::{DefKind, Numeric};
use ic_hir_xform::default_annotation;

mod common;

fn parse_and_transform(idl: &str) -> ic_hir::ResolvedGraph {
    let parsed = common::parse_with_builtins(idl);
    default_annotation::transform(parsed)
}

fn get_member_default<'a>(
    hir: &'a ic_hir::ResolvedGraph,
    struct_name: &str,
    member_name: &str,
) -> &'a Numeric {
    let struct_def = hir
        .iter()
        .find(|def| def.ident.name == struct_name)
        .unwrap();

    let DefKind::Struct(struct_ty) = &struct_def.kind else {
        panic!("Expected struct")
    };

    let member = struct_ty
        .members
        .iter()
        .find(|m| m.ident.name == member_name)
        .unwrap();

    let default_ann = member
        .annotations
        .iter()
        .find(|a| a.ident.name == "default")
        .unwrap();

    &default_ann.args[0].value
}

#[test]
fn sequence_to_array() {
    let idl = r"
        struct Test {
            @default({1, 2, 3})
            long my_array[3];
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_array");

    assert!(
        matches!(default_val, Numeric::Array { .. }),
        "Expected Array, got {default_val:?}"
    );
}

#[test]
fn sequence_to_map() {
    let idl = r"
        struct Test {
            @default({{1, 10}, {2, 20}})
            map<long, long> my_map;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_map");

    let Numeric::Map { entries, .. } = default_val else {
        panic!("Expected Map, got {default_val:?}")
    };
    assert_eq!(entries.len(), 2);
}

#[test]
fn double_to_float() {
    let idl = r"
        struct Test {
            @default(3.14)
            float my_float;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_float");

    assert!(
        matches!(default_val, Numeric::Float(_)),
        "Expected Float, got {default_val:?}"
    );
}

#[test]
fn int_to_enum() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        struct Test {
            @default(1)
            Color my_color;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_color");

    let Numeric::Const(def_id) = default_val else {
        panic!("Expected Const, got {default_val:?}")
    };
    let const_def = hir.context.type_of(*def_id);
    assert_eq!(const_def.ident.name, "GREEN");
}

#[test]
fn int_to_enum_through_typedef() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        typedef Color MyColor;
        struct Test {
            @default(2)
            MyColor my_color;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_color");

    let Numeric::Const(def_id) = default_val else {
        panic!("Expected Const, got {default_val:?}")
    };
    let const_def = hir.context.type_of(*def_id);
    assert_eq!(const_def.ident.name, "BLUE");
}

#[test]
fn enum_const_unchanged() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        struct Test {
            @default(GREEN)
            Color my_color;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_color");

    let Numeric::Const(def_id) = default_val else {
        panic!("Expected Const, got {default_val:?}")
    };
    let const_def = hir.context.type_of(*def_id);
    assert_eq!(const_def.ident.name, "GREEN");
}

#[test]
fn nested_sequence_coercion() {
    let idl = r"
        struct Test {
            @default({{1, 2}, {3, 4}})
            long my_array[2][2];
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_array");

    let Numeric::Array { values, .. } = default_val else {
        panic!("Expected Array, got {default_val:?}")
    };
    assert_eq!(values.len(), 2);
    assert!(matches!(&values[0], Numeric::Array { .. }));
}

#[test]
fn sequence_stays_sequence() {
    let idl = r"
        struct Test {
            @default({1, 2, 3})
            sequence<long> my_seq;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_seq");

    assert!(
        matches!(default_val, Numeric::Sequence { .. }),
        "Expected Sequence, got {default_val:?}"
    );
}

#[test]
fn sequence_to_struct() {
    let idl = r"
        struct Duration {
            long long sec;
            unsigned long long nanosec;
        };

        struct Test {
            @default({10, 30})
            Duration my_duration;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_duration");

    let Numeric::Struct { fields, .. } = default_val else {
        panic!("Expected Struct, got {default_val:?}")
    };
    assert_eq!(fields.len(), 2);
}

fn get_default_value(annotations: &[ic_hir::hir::Ann]) -> &Numeric {
    &annotations
        .iter()
        .find(|a| a.ident.name == "default")
        .unwrap()
        .args[0]
        .value
}

fn get_disc_default<'a>(hir: &'a ic_hir::ResolvedGraph, union_name: &str) -> &'a Numeric {
    let union_def = hir.iter().find(|def| def.ident.name == union_name).unwrap();

    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    get_default_value(&union_ty.disc.annotations)
}

fn get_ann_param_default<'a>(
    hir: &'a ic_hir::ResolvedGraph,
    ann_name: &str,
    param_name: &str,
) -> &'a Numeric {
    let ann_def = hir.iter().find(|def| def.ident.name == ann_name).unwrap();

    let DefKind::Annotation(ann_ty) = &ann_def.kind else {
        panic!("Expected annotation")
    };

    let param = ann_ty
        .params
        .iter()
        .find(|p| p.ident.name == param_name)
        .unwrap();

    get_default_value(&param.annotations)
}

fn get_variant_default<'a>(
    hir: &'a ic_hir::ResolvedGraph,
    union_name: &str,
    variant_name: &str,
) -> &'a Numeric {
    let union_def = hir.iter().find(|def| def.ident.name == union_name).unwrap();

    let DefKind::Union(union_ty) = &union_def.kind else {
        panic!("Expected union")
    };

    let variant = union_ty
        .variants
        .iter()
        .find(|v| v.ident.name == variant_name)
        .unwrap();

    get_default_value(&variant.annotations)
}

fn get_alias_default<'a>(hir: &'a ic_hir::ResolvedGraph, alias_name: &str) -> &'a Numeric {
    let alias_def = hir.iter().find(|def| def.ident.name == alias_name).unwrap();

    get_default_value(&alias_def.annotations)
}

#[test]
fn named_union_variant_default() {
    let idl = r#"
        struct Foo {
            string value;
        };

        union Bar switch (long) {
            case 1:
                @default({ value = "foo" })
                Foo my_foo;
        };
    "#;

    let hir = parse_and_transform(idl);
    let default_val = get_variant_default(&hir, "Bar", "my_foo");

    let Numeric::Struct { fields, .. } = default_val else {
        panic!("Expected Struct, got {default_val:?}")
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0], Numeric::String("foo".to_string()));
}

#[test]
fn named_typedef_default() {
    let idl = r#"
        struct Foo {
            string value;
        };

        @default({ value = "foo" })
        typedef Foo FooAlias;
    "#;

    let hir = parse_and_transform(idl);
    let default_val = get_alias_default(&hir, "FooAlias");

    let Numeric::Struct { fields, .. } = default_val else {
        panic!("Expected Struct, got {default_val:?}")
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0], Numeric::String("foo".to_string()));
}

#[test]
fn int_to_enum_on_union_variant() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };

        union Test switch (long) {
            case 1:
                @default(1)
                Color my_color;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_variant_default(&hir, "Test", "my_color");

    assert!(
        matches!(default_val, Numeric::Const(_)),
        "Expected Const, got {default_val:?}"
    );
}

#[test]
fn int_to_enum_on_union_discriminator() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };

        union Test switch (@default(1) Color) {
            case RED:
                long my_long;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_disc_default(&hir, "Test");

    assert!(
        matches!(default_val, Numeric::Const(_)),
        "Expected Const, got {default_val:?}"
    );
}

#[test]
fn int_to_enum_on_typedef() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };

        @default(1)
        typedef Color MyColor;
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_alias_default(&hir, "MyColor");

    assert!(
        matches!(default_val, Numeric::Const(_)),
        "Expected Const, got {default_val:?}"
    );
}

#[test]
fn int_to_enum_on_annotation_member() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };

        @annotation palette {
            @default(1)
            Color shade;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_ann_param_default(&hir, "palette", "shade");

    assert!(
        matches!(default_val, Numeric::Const(_)),
        "Expected Const, got {default_val:?}"
    );
}

#[test]
fn double_to_float_on_union_variant() {
    let idl = r"
        union Test switch (long) {
            case 1:
                @default(3.14)
                float my_float;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_variant_default(&hir, "Test", "my_float");

    assert!(
        matches!(default_val, Numeric::Float(_)),
        "Expected Float, got {default_val:?}"
    );
}

#[test]
fn named_valuetype_member_default() {
    let idl = r#"
        struct Foo {
            string value;
        };

        valuetype Bar {
            @default({ value = "foo" })
            public Foo my_foo;
        };
    "#;

    let hir = parse_and_transform(idl);

    let value_def = hir.iter().find(|def| def.ident.name == "Bar").unwrap();
    let DefKind::Valuetype(value_ty) = &value_def.kind else {
        panic!("Expected valuetype")
    };
    let member = value_ty
        .members
        .iter()
        .find(|m| m.ident.name == "my_foo")
        .unwrap();
    let default_val = get_default_value(&member.annotations);

    let Numeric::Struct { fields, .. } = default_val else {
        panic!("Expected Struct, got {default_val:?}")
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0], Numeric::String("foo".to_string()));
}

#[test]
fn named_struct_default() {
    let idl = r#"
        struct Foo {
            string value;
        };

        struct Bar {
            @default({ value = "foo" })
            Foo my_foo;
        };
    "#;

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Bar", "my_foo");

    let Numeric::Struct { fields, .. } = default_val else {
        panic!("Expected Struct, got {default_val:?}")
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0], Numeric::String("foo".to_string()));
}

#[test]
fn sequence_to_derived_struct() {
    let idl = r#"
        struct Base {
            long id;
            string name;
        };

        struct Middle : Base {
            long count;
        };

        struct Derived : Middle {
            long total;
        };

        struct Test {
            @default({10, "base", 30, 40})
            Derived value;
        };
    "#;

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "value");

    let Numeric::Struct { fields, .. } = default_val else {
        panic!("Expected Struct, got {default_val:?}")
    };
    assert_eq!(fields.len(), 4);
    assert!(matches!(&fields[0], Numeric::Int32(10)));
    assert!(matches!(&fields[1], Numeric::String(value) if value == "base"));
    assert!(matches!(&fields[2], Numeric::Int32(30)));
    assert!(matches!(&fields[3], Numeric::Int32(40)));
}

#[test]
fn no_coercion_for_invalid_enum_int() {
    let idl = r"
        enum Color { RED, GREEN };
        struct Test {
            @default(99)
            Color my_color;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_color");

    assert!(
        matches!(default_val, Numeric::Int32(99)),
        "Expected unchanged Int32(99), got {default_val:?}"
    );
}

#[test]
fn const_referencing_enum_unchanged() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        const Color MY_COLOR = RED;
        struct Test {
            @default(MY_COLOR)
            Color my_color;
        };
    ";

    let hir = parse_and_transform(idl);
    let default_val = get_member_default(&hir, "Test", "my_color");

    let Numeric::Const(def_id) = default_val else {
        panic!("Expected Const, got {default_val:?}")
    };
    let const_def = hir.context.type_of(*def_id);
    assert_eq!(const_def.ident.name, "MY_COLOR");
}
