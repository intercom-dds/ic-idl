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
use ic_hir_analysis::annotation::builtin_annotation;
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
fn integer_defaults_match_member_types() {
    let hir = parse_and_transform(
        r"
        struct Test {
            @default(12) int8 signed8;
            @default(12) uint8 unsigned8;
            @default(12) short signed16;
            @default(12) unsigned short unsigned16;
            @default(12) long signed32;
            @default(12) unsigned long unsigned32;
            @default(12) long long signed64;
            @default(12) unsigned long long unsigned64;
        };
        ",
    );

    assert_eq!(
        get_member_default(&hir, "Test", "signed8"),
        &Numeric::Int8(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned8"),
        &Numeric::UInt8(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "signed16"),
        &Numeric::Int16(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned16"),
        &Numeric::UInt16(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "signed32"),
        &Numeric::Int32(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned32"),
        &Numeric::UInt32(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "signed64"),
        &Numeric::Int64(12)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned64"),
        &Numeric::UInt64(12)
    );
}

#[test]
fn integer_defaults_preserve_boundaries() {
    let hir = parse_and_transform(
        r"
        struct Test {
            @default(-32768) short minimum;
            @default(32767) short maximum;
            @default(18446744073709551615) unsigned long long largest;
            @default(32768) short overflow;
            @default(-32769) short underflow;
            @default(-1) unsigned short negative;
            @default(18446744073709551615) long long unsigned_overflow;
        };
        ",
    );

    assert_eq!(
        get_member_default(&hir, "Test", "minimum"),
        &Numeric::Int16(i16::MIN)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "maximum"),
        &Numeric::Int16(i16::MAX)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "largest"),
        &Numeric::UInt64(u64::MAX)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "overflow"),
        &Numeric::Int32(32768)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "underflow"),
        &Numeric::Int32(-32769)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "negative"),
        &Numeric::Int32(-1)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned_overflow"),
        &Numeric::UInt64(u64::MAX),
    );
}

#[test]
fn integer_default_const_unchanged() {
    let hir = parse_and_transform(
        r"
        const short VALUE = 12;
        struct Test {
            @default(VALUE) short value;
        };
        ",
    );

    let Numeric::Const(def_id) = get_member_default(&hir, "Test", "value") else {
        panic!("Expected Const")
    };
    assert_eq!(hir.context.type_of(*def_id).ident.name, "VALUE");
}

#[test]
fn integer_defaults_on_every_target() {
    let hir = parse_and_transform(
        r"
        @default(12)
        typedef short Small;
        typedef Small Smaller;

        struct Test {
            @default(12) Smaller value;
        };

        union Choice switch (@default(12) short) {
            case 0:
                @default(12) short value;
        };

        @annotation custom {
            @default(12) short value;
        };
        ",
    );

    assert_eq!(get_alias_default(&hir, "Small"), &Numeric::Int16(12));
    assert_eq!(
        get_member_default(&hir, "Test", "value"),
        &Numeric::Int16(12)
    );
    assert_eq!(get_disc_default(&hir, "Choice"), &Numeric::Int16(12));
    assert_eq!(
        get_variant_default(&hir, "Choice", "value"),
        &Numeric::Int16(12)
    );
    assert_eq!(
        get_ann_param_default(&hir, "custom", "value"),
        &Numeric::Int16(12)
    );
}

#[test]
fn bounds_match_member_types() {
    let hir = parse_and_transform(
        r"
        struct Test {
            @min(-12) @max(12) short small;
            @range(max = 24, min = 12) unsigned short bounded;
            @min(1.5) @max(2.5) float real;
            @range(min = 1.5, max = 2.5) float real_range;
        };
        ",
    );

    let DefKind::Struct(test) = &hir
        .iter()
        .find(|def| def.ident.name == "Test")
        .unwrap()
        .kind
    else {
        panic!("Expected struct")
    };
    let small = &test.members[0].annotations;
    let bounded = &test.members[1].annotations;
    let real = &test.members[2].annotations;
    let real_range = &test.members[3].annotations;
    let minimum = builtin_annotation(&hir.context, small, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, small, "max").unwrap();
    let range = builtin_annotation(&hir.context, bounded, "range").unwrap();
    let float_minimum = builtin_annotation(&hir.context, real, "min").unwrap();
    let float_maximum = builtin_annotation(&hir.context, real, "max").unwrap();
    let float_range = builtin_annotation(&hir.context, real_range, "range").unwrap();

    assert_eq!(minimum.args[0].value, Numeric::Int16(-12));
    assert_eq!(maximum.args[0].value, Numeric::Int16(12));
    assert_eq!(range.args[0].value, Numeric::UInt16(12));
    assert_eq!(range.args[1].value, Numeric::UInt16(24));
    assert_eq!(minimum.args[0].ty, Some(test.members[0].ty.clone()));
    assert_eq!(maximum.args[0].ty, Some(test.members[0].ty.clone()));
    assert_eq!(range.args[0].ty, Some(test.members[1].ty.clone()));
    assert_eq!(range.args[1].ty, Some(test.members[1].ty.clone()));
    assert_eq!(float_minimum.args[0].value, Numeric::Float(1.5));
    assert_eq!(float_maximum.args[0].value, Numeric::Float(2.5));
    assert_eq!(float_range.args[0].value, Numeric::Float(1.5));
    assert_eq!(float_range.args[1].value, Numeric::Float(2.5));
}

#[test]
fn bounds_on_typedefs_and_variants() {
    let hir = parse_and_transform(
        r"
        @min(-12) @max(12) @range(min = -12, max = 12)
        typedef short Small;
        typedef Small Smaller;

        struct Test {
            @range(min = -12, max = 12) Smaller value;
        };

        union Choice switch (long) {
            case 0: @min(-12) @max(12) short value;
        };

        @annotation custom {
            @range(min = -12, max = 12) short value;
        };
        ",
    );

    let alias = hir.iter().find(|def| def.ident.name == "Small").unwrap();
    let minimum = builtin_annotation(&hir.context, &alias.annotations, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, &alias.annotations, "max").unwrap();
    let range = builtin_annotation(&hir.context, &alias.annotations, "range").unwrap();
    assert_eq!(minimum.args[0].value, Numeric::Int16(-12));
    assert_eq!(maximum.args[0].value, Numeric::Int16(12));
    assert_eq!(range.args[0].value, Numeric::Int16(-12));
    assert_eq!(range.args[1].value, Numeric::Int16(12));

    let DefKind::Struct(test) = &hir
        .iter()
        .find(|def| def.ident.name == "Test")
        .unwrap()
        .kind
    else {
        panic!("Expected struct")
    };
    let range = builtin_annotation(&hir.context, &test.members[0].annotations, "range").unwrap();
    assert_eq!(range.args[0].value, Numeric::Int16(-12));
    assert_eq!(range.args[1].value, Numeric::Int16(12));

    let DefKind::Union(choice) = &hir
        .iter()
        .find(|def| def.ident.name == "Choice")
        .unwrap()
        .kind
    else {
        panic!("Expected union")
    };
    let minimum = builtin_annotation(&hir.context, &choice.variants[0].annotations, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, &choice.variants[0].annotations, "max").unwrap();
    assert_eq!(minimum.args[0].value, Numeric::Int16(-12));
    assert_eq!(maximum.args[0].value, Numeric::Int16(12));

    let DefKind::Annotation(custom) = &hir
        .iter()
        .find(|def| def.ident.name == "custom")
        .unwrap()
        .kind
    else {
        panic!("Expected annotation")
    };
    let range = builtin_annotation(&hir.context, &custom.params[0].annotations, "range").unwrap();
    assert_eq!(range.args[0].value, Numeric::Int16(-12));
    assert_eq!(range.args[1].value, Numeric::Int16(12));
}

#[test]
fn bounds_preserve_constants_and_invalid_values() {
    let hir = parse_and_transform(
        r"
        const short LOW = 1;
        const short HIGH = 12;
        module Custom {
            @annotation min { any value; };
        };
        struct Test {
            @min(LOW) @max(HIGH) @range(min = LOW, max = HIGH) short constants;
            @min(-32769) @max(32768) @range(min = -32769, max = 32768) short invalid;
            @Custom::min(12) short custom;
        };
        ",
    );

    let low = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "LOW")
        .unwrap()
        .0;
    let high = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "HIGH")
        .unwrap()
        .0;
    let DefKind::Struct(test) = &hir
        .iter()
        .find(|def| def.ident.name == "Test")
        .unwrap()
        .kind
    else {
        panic!("Expected struct")
    };
    let minimum = builtin_annotation(&hir.context, &test.members[0].annotations, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, &test.members[0].annotations, "max").unwrap();
    let range = builtin_annotation(&hir.context, &test.members[0].annotations, "range").unwrap();
    assert_eq!(minimum.args[0].value, Numeric::Const(low));
    assert_eq!(maximum.args[0].value, Numeric::Const(high));
    assert_eq!(range.args[0].value, Numeric::Const(low));
    assert_eq!(range.args[1].value, Numeric::Const(high));

    let minimum = builtin_annotation(&hir.context, &test.members[1].annotations, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, &test.members[1].annotations, "max").unwrap();
    let range = builtin_annotation(&hir.context, &test.members[1].annotations, "range").unwrap();
    assert_eq!(minimum.args[0].value, Numeric::Int32(-32769));
    assert_eq!(maximum.args[0].value, Numeric::Int32(32768));
    assert_eq!(range.args[0].value, Numeric::Int32(-32769));
    assert_eq!(range.args[1].value, Numeric::Int32(32768));
    assert_eq!(
        test.members[2].annotations[0].args[0].value,
        Numeric::Int32(12)
    );
}

#[test]
fn integer_to_float_defaults() {
    let hir = parse_and_transform(
        r"
        typedef float Real;
        struct Test {
            @default(23) float single;
            @default(-23) double wide;
            @default(23) long double extended;
            @default(23) Real alias;
            @default(18446744073709551615) double large;
        };
        ",
    );

    assert_eq!(
        get_member_default(&hir, "Test", "single"),
        &Numeric::Float(23.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "wide"),
        &Numeric::Double(-23.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "extended"),
        &Numeric::Double(23.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "alias"),
        &Numeric::Float(23.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "large"),
        &Numeric::Double(18_446_744_073_709_551_616.0)
    );
}

#[test]
fn float_to_integer_defaults() {
    let hir = parse_and_transform(
        r"
        struct Test {
            @default(23.75) short positive;
            @default(-23.75) short negative;
            @default(23.75) unsigned short unsigned_value;
            @default(32768.0) short overflow;
            @default(-1.0) unsigned short below_zero;
            @default(9223372036854775808.0) long long signed_overflow;
            @default(18446744073709551616.0) unsigned long long unsigned_overflow;
        };
        ",
    );

    assert_eq!(
        get_member_default(&hir, "Test", "positive"),
        &Numeric::Int16(23)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "negative"),
        &Numeric::Int16(-23)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned_value"),
        &Numeric::UInt16(23)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "overflow"),
        &Numeric::Double(32768.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "below_zero"),
        &Numeric::Double(-1.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "signed_overflow"),
        &Numeric::Double(9_223_372_036_854_775_808.0)
    );
    assert_eq!(
        get_member_default(&hir, "Test", "unsigned_overflow"),
        &Numeric::Double(18_446_744_073_709_551_616.0)
    );
}

#[test]
fn cross_category_bounds() {
    let hir = parse_and_transform(
        r"
        struct Test {
            @min(1) @max(23) @range(min = 1, max = 23) float real;
            @min(-1.75) @max(23.75) @range(min = -1.75, max = 23.75) short integer;
        };
        ",
    );

    let DefKind::Struct(test) = &hir
        .iter()
        .find(|def| def.ident.name == "Test")
        .unwrap()
        .kind
    else {
        panic!("Expected struct")
    };
    let minimum = builtin_annotation(&hir.context, &test.members[0].annotations, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, &test.members[0].annotations, "max").unwrap();
    let range = builtin_annotation(&hir.context, &test.members[0].annotations, "range").unwrap();
    assert_eq!(minimum.args[0].value, Numeric::Float(1.0));
    assert_eq!(maximum.args[0].value, Numeric::Float(23.0));
    assert_eq!(range.args[0].value, Numeric::Float(1.0));
    assert_eq!(range.args[1].value, Numeric::Float(23.0));

    let minimum = builtin_annotation(&hir.context, &test.members[1].annotations, "min").unwrap();
    let maximum = builtin_annotation(&hir.context, &test.members[1].annotations, "max").unwrap();
    let range = builtin_annotation(&hir.context, &test.members[1].annotations, "range").unwrap();
    assert_eq!(minimum.args[0].value, Numeric::Int16(-1));
    assert_eq!(maximum.args[0].value, Numeric::Int16(23));
    assert_eq!(range.args[0].value, Numeric::Int16(-1));
    assert_eq!(range.args[1].value, Numeric::Int16(23));
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
