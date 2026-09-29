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

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::Result;
use jsonschema::{Registry, Validator};
use serde_json::{Value, json};

use crate::{
    annotation_types, any_types, bitmask_types, bounded_types, circular_types, deep_generic_types,
    default_types, enum_types, interface_types, large_integer_types, module_a, module_b,
    nested_module_types, serialization_types, struct_types, typedef_types, union_types,
};

fn read_schema_files(schema_dir: &Path, schemas: &mut Vec<(String, Value)>) -> Result<()> {
    for entry in fs::read_dir(schema_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_schema_files(&path, schemas)?;
        } else if let Some(ext) = path.extension()
            && ext == "json"
        {
            let content = fs::read_to_string(&path)?;
            let object: serde_json::Map<String, Value> = serde_json::from_str(&content)?;
            if let Some(val) = object.get("$id")
                && let Value::String(id) = val
            {
                schemas.push((String::from(id), Value::Object(object)))
            } else {
                anyhow::bail!("schema file has no $id: {}", path.display());
            }
        }
    }
    Ok(())
}

fn get_schemas() -> Result<Vec<(String, Value)>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let mut schema_dir = std::path::PathBuf::from(out_dir);
    schema_dir.push("jsonschema");
    let mut schemas = Vec::new();
    read_schema_files(&schema_dir, &mut schemas)?;
    Ok(schemas)
}

fn registry() -> &'static Registry<'static> {
    static REGISTRY: OnceLock<Registry<'static>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let schemas = get_schemas().expect("read generated schemas");
        Registry::new()
            .extend(schemas)
            .expect("register generated schemas")
            .prepare()
            .expect("resolve schema references")
    })
}

fn validator_for(schema_file: &str, type_name: &str) -> Result<Validator> {
    let schema_ref = format!("file:///{schema_file}#/$defs/{type_name}");
    let schema = json!({"$ref": schema_ref});
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .with_registry(registry())
        .build(&schema)?;
    Ok(validator)
}

fn to_json<T>(value: &T) -> Result<Value>
where
    T: ?Sized + intercom_cts::Marshal,
{
    let s = intercom_cts::json::to_string(value, false)?;
    let json: Value = serde_json::from_str(&s)?;
    Ok(json)
}

fn from_json<T>(json: &Value) -> Result<T>
where
    T: intercom_cts::Unmarshal + Default,
{
    let s = json.to_string();
    let value: T = intercom_cts::json::from_str(&s)?;
    Ok(value)
}

#[track_caller]
fn assert_valid(validator: &Validator, value: &Value) {
    if let Err(error) = validator.validate(value) {
        panic!("expected {value} to validate, but it did not: {error}");
    }
}

#[track_caller]
fn assert_invalid(validator: &Validator, value: &Value) {
    assert!(
        !validator.is_valid(value),
        "expected {value} to be rejected, but it validated"
    );
}

fn roundtrip<T>(schema_file: &str, type_name: &str, value: &T) -> Result<Value>
where
    T: intercom_cts::Marshal + intercom_cts::Unmarshal + Default + PartialEq + std::fmt::Debug,
{
    let validator = validator_for(schema_file, type_name)?;
    let json = to_json(value)?;
    assert_valid(&validator, &json);
    let decoded: T = from_json(&json)?;
    assert_eq!(*value, decoded);
    Ok(json)
}

#[test]
fn struct_point_roundtrip() -> Result<()> {
    roundtrip(
        "structs.json",
        "struct_types.Point",
        &struct_types::Point { x: 10, y: 20 },
    )?;
    Ok(())
}

#[test]
fn struct_point_rejects_wrong_field_type() -> Result<()> {
    let validator = validator_for("structs.json", "struct_types.Point")?;
    let mut point = to_json(&struct_types::Point { x: 10, y: 20 })?;
    assert_valid(&validator, &point);

    *point.get_mut("x").unwrap() = json!("10");
    assert_invalid(&validator, &point);
    Ok(())
}

#[test]
fn struct_point_rejects_missing_field() -> Result<()> {
    let validator = validator_for("structs.json", "struct_types.Point")?;
    let mut point = to_json(&struct_types::Point { x: 10, y: 20 })?;

    point.as_object_mut().unwrap().remove("y");
    assert_invalid(&validator, &point);
    Ok(())
}

#[test]
fn struct_all_primitives_roundtrip() -> Result<()> {
    roundtrip(
        "structs.json",
        "struct_types.AllPrimitives",
        &struct_types::AllPrimitives {
            bool_val: true,
            byte_val: 255,
            short_val: -100,
            ushort_val: 1000,
            long_val: -50000,
            ulong_val: 100000,
            longlong_val: -9999999999i64,
            ulonglong_val: 9999999999u64,
            float_val: 3.25f32,
            double_val: 2.5f64,
            string_val: "hello".into(),
        },
    )?;
    Ok(())
}

#[test]
fn struct_inheritance_requires_inherited_fields() -> Result<()> {
    let validator = validator_for("structs.json", "struct_types.Point4D")?;
    let mut point = roundtrip(
        "structs.json",
        "struct_types.Point4D",
        &struct_types::Point4D {
            x: 1,
            y: 2,
            z: 3,
            w: 4,
        },
    )?;

    point.as_object_mut().unwrap().remove("x");
    assert_invalid(&validator, &point);
    Ok(())
}

#[test]
fn struct_fixed_array_requires_exact_length() -> Result<()> {
    let validator = validator_for("structs.json", "struct_types.WithArray")?;
    let mut array = roundtrip(
        "structs.json",
        "struct_types.WithArray",
        &struct_types::WithArray {
            fixed_numbers: [1, 2, 3, 4, 5],
        },
    )?;

    array["fixed_numbers"] = json!([1, 2, 3, 4]);
    assert_invalid(&validator, &array);

    array["fixed_numbers"] = json!([1, 2, 3, 4, 5, 6]);
    assert_invalid(&validator, &array);
    Ok(())
}

#[test]
fn struct_sequence_and_map_roundtrip() -> Result<()> {
    roundtrip(
        "structs.json",
        "struct_types.WithSequence",
        &struct_types::WithSequence {
            numbers: vec![1, 2, 3],
            names: vec!["a".into(), "b".into()],
        },
    )?;

    let validator = validator_for("structs.json", "struct_types.WithMap")?;
    let mut map = roundtrip(
        "structs.json",
        "struct_types.WithMap",
        &struct_types::WithMap {
            string_to_int: BTreeMap::from([("one".to_string(), 1), ("two".to_string(), 2)]),
        },
    )?;

    map["string_to_int"]["one"] = json!("1");
    assert_invalid(&validator, &map);
    Ok(())
}

#[test]
fn struct_empty_and_nested_roundtrip() -> Result<()> {
    roundtrip(
        "structs.json",
        "struct_types.Empty",
        &struct_types::Empty::new(),
    )?;

    let validator = validator_for("structs.json", "struct_types.Rectangle")?;
    let mut rect = roundtrip(
        "structs.json",
        "struct_types.Rectangle",
        &struct_types::Rectangle {
            top_left: struct_types::Point { x: 0, y: 0 },
            bottom_right: struct_types::Point { x: 100, y: 100 },
        },
    )?;

    rect["bottom_right"].as_object_mut().unwrap().remove("y");
    assert_invalid(&validator, &rect);
    Ok(())
}

#[test]
fn enum_variants_serialize_to_schema_names() -> Result<()> {
    for color in [
        enum_types::Color::Red,
        enum_types::Color::Green,
        enum_types::Color::Blue,
    ] {
        assert_eq!(
            roundtrip("enums.json", "enum_types.Color", &color)?,
            json!(match color {
                enum_types::Color::Red => "RED",
                enum_types::Color::Green => "GREEN",
                enum_types::Color::Blue => "BLUE",
            })
        );
    }

    for status in [
        enum_types::Status::Ok,
        enum_types::Status::Warning,
        enum_types::Status::Error,
    ] {
        roundtrip("enums.json", "enum_types.Status", &status)?;
    }

    for gapped in [
        enum_types::GappedEnum::First,
        enum_types::GappedEnum::Second,
        enum_types::GappedEnum::Third,
        enum_types::GappedEnum::Fourth,
    ] {
        roundtrip("enums.json", "enum_types.GappedEnum", &gapped)?;
    }
    Ok(())
}

#[test]
fn enum_rejects_unknown_name_and_ordinal() -> Result<()> {
    let validator = validator_for("enums.json", "enum_types.Color")?;
    assert_invalid(&validator, &json!("PURPLE"));
    assert_invalid(&validator, &json!(0));

    let negative = validator_for("enums.json", "enum_types.NegativeEnum")?;
    assert_valid(&negative, &json!("NEG_TWO"));
    assert_invalid(&negative, &json!(-2));
    Ok(())
}

#[test]
fn bitmask_named_flags_roundtrip() -> Result<()> {
    let flags = bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE;
    assert_eq!(
        roundtrip("bitmasks.json", "bitmask_types.Permissions", &flags)?,
        json!("READ|WRITE")
    );

    roundtrip(
        "bitmasks.json",
        "bitmask_types.BitBoundFlags",
        &bitmask_types::BitBoundFlags::THREE,
    )?;
    Ok(())
}

#[test]
fn bitmask_accepts_integer_form() -> Result<()> {
    let validator = validator_for("bitmasks.json", "bitmask_types.Permissions")?;
    assert_valid(&validator, &json!(0));
    assert_valid(&validator, &json!(3));
    assert_invalid(&validator, &json!(-1));
    Ok(())
}

#[test]
fn bitmask_rejects_malformed_flag_string() -> Result<()> {
    let validator = validator_for("bitmasks.json", "bitmask_types.Permissions")?;
    assert_invalid(&validator, &json!("READ|"));
    assert_invalid(&validator, &json!("1READ"));
    assert_invalid(&validator, &json!("READ WRITE"));
    Ok(())
}

#[test]
fn bitmask_in_struct_roundtrip() -> Result<()> {
    let validator = validator_for("bitmasks.json", "bitmask_types.FileInfo")?;
    let mut info = roundtrip(
        "bitmasks.json",
        "bitmask_types.FileInfo",
        &bitmask_types::FileInfo {
            path: "/tmp/file".into(),
            perms: bitmask_types::Permissions::READ | bitmask_types::Permissions::EXECUTE,
        },
    )?;

    info["perms"] = json!("READ|");
    assert_invalid(&validator, &info);
    Ok(())
}

#[test]
fn bitmask_empty_value_roundtrip() -> Result<()> {
    roundtrip(
        "bitmasks.json",
        "bitmask_types.Permissions",
        &bitmask_types::Permissions::nil(),
    )?;
    Ok(())
}

#[test]
fn bounded_string_length_enforced() -> Result<()> {
    let validator = validator_for("bounded_types.json", "bounded_types.BoundedFields")?;
    let mut fields = roundtrip(
        "bounded_types.json",
        "bounded_types.BoundedFields",
        &bounded_types::BoundedFields {
            name: "n".repeat(64),
            description: "d".repeat(256),
            values: vec![1, 2, 3],
            tags: vec!["tag".into()],
        },
    )?;

    fields["name"] = json!("n".repeat(65));
    assert_invalid(&validator, &fields);
    Ok(())
}

#[test]
fn bounded_sequence_length_enforced() -> Result<()> {
    let validator = validator_for("bounded_types.json", "bounded_types.BoundedFields")?;
    let mut fields = to_json(&bounded_types::BoundedFields {
        name: "n".into(),
        description: "d".into(),
        values: vec![0; 50],
        tags: vec!["t".into(); 10],
    })?;
    assert_valid(&validator, &fields);

    fields["tags"] = json!(vec!["t"; 11]);
    assert_invalid(&validator, &fields);

    fields["tags"] = json!(vec!["t"; 10]);
    fields["values"] = json!(vec![0; 51]);
    assert_invalid(&validator, &fields);
    Ok(())
}

#[test]
fn bounded_typedef_alias_enforced() -> Result<()> {
    let validator = validator_for("bounded_types.json", "bounded_types.NameList")?;
    let names: bounded_types::NameList = vec!["name".to_string(); 50];
    assert_valid(&validator, &to_json(&names)?);

    assert_invalid(&validator, &json!(vec!["name"; 51]));
    assert_invalid(&validator, &json!(vec!["n".repeat(101)]));
    Ok(())
}

#[test]
fn bounded_mixed_bounds_roundtrip() -> Result<()> {
    let validator = validator_for("bounded_types.json", "bounded_types.MixedBounds")?;
    let mut mixed = roundtrip(
        "bounded_types.json",
        "bounded_types.MixedBounds",
        &bounded_types::MixedBounds {
            bounded_string: "b".repeat(100),
            unbounded_string: "u".repeat(10000),
            bounded_seq: vec![1; 10],
            unbounded_seq: vec![1; 1000],
        },
    )?;

    mixed["bounded_string"] = json!("b".repeat(101));
    assert_invalid(&validator, &mixed);
    Ok(())
}

#[test]
fn union_integer_discriminator_roundtrip() -> Result<()> {
    assert_eq!(
        roundtrip(
            "unions.json",
            "union_types.IntOrString",
            &union_types::IntOrString::IntVal(42),
        )?,
        json!({"$discriminator": 1, "int_val": 42})
    );

    roundtrip(
        "unions.json",
        "union_types.IntOrString",
        &union_types::IntOrString::StrVal("hello".into()),
    )?;
    roundtrip(
        "unions.json",
        "union_types.IntOrString",
        &union_types::IntOrString::DefaultVal(true),
    )?;
    Ok(())
}

#[test]
fn union_rejects_field_discriminator_mismatch() -> Result<()> {
    let validator = validator_for("unions.json", "union_types.IntOrString")?;
    assert_invalid(&validator, &json!({"$discriminator": 1, "str_val": "x"}));
    assert_invalid(&validator, &json!({"int_val": 1}));
    Ok(())
}

#[test]
fn union_default_case_rejects_declared_discriminator() -> Result<()> {
    let validator = validator_for("unions.json", "union_types.IntOrString")?;
    assert_invalid(
        &validator,
        &json!({"$discriminator": 2, "default_val": true}),
    );
    assert_valid(
        &validator,
        &json!({"$discriminator": 7, "default_val": true}),
    );
    Ok(())
}

#[test]
fn union_bool_discriminator_roundtrip() -> Result<()> {
    assert_eq!(
        roundtrip(
            "unions.json",
            "union_types.BoolSwitch",
            &union_types::BoolSwitch::TrueVal(100),
        )?,
        json!({"$discriminator": true, "true_val": 100})
    );
    roundtrip(
        "unions.json",
        "union_types.BoolSwitch",
        &union_types::BoolSwitch::FalseVal("false branch".into()),
    )?;
    Ok(())
}

#[test]
fn union_enum_discriminator_roundtrip() -> Result<()> {
    roundtrip(
        "unions.json",
        "union_types.TypedValue",
        &union_types::TypedValue::StringValue("test".into()),
    )?;
    Ok(())
}

#[test]
fn typedef_chain_resolves_to_primitive() -> Result<()> {
    let validator = validator_for("typedefs.json", "typedef_types.DeepChainStruct")?;
    let mut chain = roundtrip(
        "typedefs.json",
        "typedef_types.DeepChainStruct",
        &typedef_types::DeepChainStruct {
            deep_int: 7,
            deep_seq: vec![1, 2, 3],
            deep_map: BTreeMap::from([("k".to_string(), 1)]),
        },
    )?;

    chain["deep_int"] = json!("7");
    assert_invalid(&validator, &chain);
    Ok(())
}

#[test]
fn typedef_container_roundtrip() -> Result<()> {
    roundtrip(
        "typedefs.json",
        "typedef_types.Container",
        &typedef_types::Container {
            numbers: vec![1, 2],
            labels: vec!["a".into()],
            lookup: BTreeMap::from([("k".to_string(), 3)]),
        },
    )?;

    roundtrip(
        "typedefs.json",
        "typedef_types.Measurement",
        &typedef_types::Measurement {
            name: "temperature".into(),
            value: 42,
        },
    )?;
    Ok(())
}

#[test]
fn typedef_fixed_array_requires_exact_length() -> Result<()> {
    let validator = validator_for("typedefs.json", "typedef_types.WithArrayTypedef")?;
    let mut with_array = roundtrip(
        "typedefs.json",
        "typedef_types.WithArrayTypedef",
        &typedef_types::WithArrayTypedef {
            values: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        },
    )?;

    with_array["values"] = json!([1, 2, 3]);
    assert_invalid(&validator, &with_array);
    Ok(())
}

#[test]
fn optional_fields_may_be_omitted() -> Result<()> {
    let validator = validator_for("defaults.json", "default_types.OptionalFields")?;
    assert_valid(&validator, &json!({}));

    let empty = roundtrip(
        "defaults.json",
        "default_types.OptionalFields",
        &default_types::OptionalFields {
            maybe_int: None,
            maybe_string: None,
            maybe_struct: None,
        },
    )?;
    assert_eq!(empty, json!({}));

    roundtrip(
        "defaults.json",
        "default_types.OptionalFields",
        &default_types::OptionalFields {
            maybe_int: Some(1),
            maybe_string: Some("x".into()),
            maybe_struct: Some(default_types::Inner {
                x: 2,
                y: "y".into(),
            }),
        },
    )?;
    Ok(())
}

#[test]
fn default_map_and_enum_roundtrip() -> Result<()> {
    roundtrip(
        "defaults.json",
        "default_types.MapDefaults",
        &default_types::MapDefaults {
            map_empty: BTreeMap::new(),
            map_values: BTreeMap::from([("a".to_string(), 1)]),
            reverse_map_empty: BTreeMap::new(),
            reverse_map_values: BTreeMap::from([(1, "a".to_string())]),
        },
    )?;

    roundtrip(
        "defaults.json",
        "default_types.EnumDefaults",
        &default_types::EnumDefaults {
            priority_empty: default_types::Priority::Low,
            priority_high: default_types::Priority::High,
        },
    )?;
    Ok(())
}

#[test]
fn annotation_optional_struct_roundtrip() -> Result<()> {
    let sparse = roundtrip(
        "annotations.json",
        "annotation_types.OptionalStruct",
        &annotation_types::OptionalStruct {
            required_field: 1,
            optional_int: None,
            optional_string: None,
            optional_seq: None,
        },
    )?;
    assert_eq!(sparse, json!({"required_field": 1}));

    let validator = validator_for("annotations.json", "annotation_types.OptionalStruct")?;
    assert_invalid(&validator, &json!({}));
    Ok(())
}

#[test]
fn annotation_final_struct_rejects_unknown_property() -> Result<()> {
    let validator = validator_for("annotations.json", "annotation_types.FinalStruct")?;
    let mut final_struct = roundtrip(
        "annotations.json",
        "annotation_types.FinalStruct",
        &annotation_types::FinalStruct { fixed_field: 7 },
    )?;

    final_struct["extra"] = json!(1);
    assert_invalid(&validator, &final_struct);

    let mutable = validator_for("annotations.json", "annotation_types.MutableStruct")?;
    assert_valid(&mutable, &json!({"version": 1, "data": "d", "extra": 1}));
    Ok(())
}

#[test]
fn circular_tree_node_roundtrip() -> Result<()> {
    let validator = validator_for("circular_types.json", "circular_types.TreeNode")?;
    let mut tree = roundtrip(
        "circular_types.json",
        "circular_types.TreeNode",
        &circular_types::TreeNode {
            value: 1,
            children: vec![
                circular_types::TreeNode {
                    value: 2,
                    children: vec![circular_types::TreeNode {
                        value: 3,
                        children: vec![],
                    }],
                },
                circular_types::TreeNode {
                    value: 4,
                    children: vec![],
                },
            ],
        },
    )?;

    tree["children"][0]["children"][0]["value"] = json!("3");
    assert_invalid(&validator, &tree);
    Ok(())
}

#[test]
fn circular_map_self_ref_roundtrip() -> Result<()> {
    roundtrip(
        "circular_types.json",
        "circular_types.MapSelfRef",
        &circular_types::MapSelfRef {
            id: "root".into(),
            children_by_name: BTreeMap::from([(
                "child".to_string(),
                circular_types::MapSelfRef {
                    id: "leaf".into(),
                    children_by_name: BTreeMap::new(),
                },
            )]),
        },
    )?;
    Ok(())
}

#[test]
fn deep_generic_four_level_roundtrip() -> Result<()> {
    let validator = validator_for("deep_generics.json", "deep_generic_types.FourLevelDeep")?;
    let mut deep = roundtrip(
        "deep_generics.json",
        "deep_generic_types.FourLevelDeep",
        &deep_generic_types::FourLevelDeep {
            hypercube: vec![vec![vec![vec![1, 2], vec![3]]], vec![]],
        },
    )?;

    deep["hypercube"] = json!([[[1, 2]]]);
    assert_invalid(&validator, &deep);
    Ok(())
}

#[test]
fn deep_generic_fixed_array_element_length() -> Result<()> {
    let validator = validator_for("deep_generics.json", "deep_generic_types.SeqOfArray")?;
    let mut seq = roundtrip(
        "deep_generics.json",
        "deep_generic_types.SeqOfArray",
        &deep_generic_types::SeqOfArray {
            fixed_triples: vec![[1, 2, 3], [4, 5, 6]],
        },
    )?;

    seq["fixed_triples"][0] = json!([1, 2]);
    assert_invalid(&validator, &seq);
    Ok(())
}

#[test]
fn deep_generic_map_of_points_roundtrip() -> Result<()> {
    let validator = validator_for("deep_generics.json", "deep_generic_types.MapOfPoints")?;
    let mut map = roundtrip(
        "deep_generics.json",
        "deep_generic_types.MapOfPoints",
        &deep_generic_types::MapOfPoints {
            named_points: BTreeMap::from([(
                "origin".to_string(),
                deep_generic_types::Point { x: 0, y: 0 },
            )]),
        },
    )?;

    map["named_points"]["origin"]
        .as_object_mut()
        .unwrap()
        .remove("y");
    assert_invalid(&validator, &map);
    Ok(())
}

#[test]
fn any_field_accepts_any_json() -> Result<()> {
    let validator = validator_for("any_type.json", "any_types.ContainsAny")?;
    for payload in [
        json!(null),
        json!(1),
        json!("text"),
        json!([1, 2]),
        json!({"nested": true}),
    ] {
        assert_valid(&validator, &json!({ "value": payload }));
    }
    assert_invalid(&validator, &json!({}));

    roundtrip(
        "any_type.json",
        "any_types.ContainsAny",
        &any_types::ContainsAny { value: () },
    )?;
    Ok(())
}

#[test]
fn nested_module_qualified_defs_roundtrip() -> Result<()> {
    roundtrip(
        "nested_modules.json",
        "nested_module_types.level1.level2.Level2Struct",
        &nested_module_types::level1::level2::Level2Struct {
            name: "l2".into(),
            level1_ref: nested_module_types::level1::Level1Struct {
                data: 1,
                parent_ref: nested_module_types::TopLevelStruct { value: 2 },
            },
            top_ref: nested_module_types::TopLevelStruct { value: 3 },
        },
    )?;

    let validator = validator_for(
        "nested_modules.json",
        "nested_module_types.level1.Level1Enum",
    )?;
    assert_valid(&validator, &json!("A"));
    assert_invalid(&validator, &json!("D"));
    Ok(())
}

#[test]
fn multi_module_defs_share_schema_file() -> Result<()> {
    roundtrip(
        "multi_module.json",
        "module_a.StructA3",
        &module_a::StructA3 {
            flag: true,
            a1: module_a::StructA1 { value: 1 },
            a2: module_a::StructA2 {
                data: 2.5,
                ref_to_a1: module_a::StructA1 { value: 3 },
            },
        },
    )?;

    roundtrip(
        "multi_module.json",
        "module_b.StructB2",
        &module_b::StructB2 {
            id: 4,
            ref_to_b1: module_b::StructB1 { name: "b".into() },
        },
    )?;
    Ok(())
}

#[test]
fn interface_typedefs_validate() -> Result<()> {
    let validator = validator_for("interfaces.json", "interface_types.LongArray")?;
    let values: interface_types::LongArray = [1, 2, 3, 4];
    assert_valid(&validator, &to_json(&values)?);
    assert_invalid(&validator, &json!([1, 2, 3]));

    let lookup = validator_for("interfaces.json", "interface_types.Lookup")?;
    assert_valid(&lookup, &json!({"a": 1}));
    assert_invalid(&lookup, &json!({"a": "1"}));

    let payload = validator_for("interfaces.json", "interface_types.Payload")?;
    assert_valid(&payload, &json!({"anything": [1, "two"]}));
    Ok(())
}

#[test]
fn large_integers_validate_as_numbers() -> Result<()> {
    roundtrip(
        "constants.json",
        "large_integer_types.LargeIntFields",
        &large_integer_types::LargeIntFields {
            big_signed: i64::MIN,
            big_unsigned: u64::MAX,
        },
    )?;

    roundtrip(
        "serialization.json",
        "serialization_types.JsonSafeNumbers",
        &serialization_types::JsonSafeNumbers {
            a: 9007199254740993,
            b: -9007199254740993,
            c: 1,
            d: BTreeMap::from([(1, 2)]),
        },
    )?;

    let validator = validator_for("constants.json", "constant_types.Priority")?;
    assert_valid(&validator, &json!("MEDIUM"));
    assert_invalid(&validator, &json!(50));
    Ok(())
}
