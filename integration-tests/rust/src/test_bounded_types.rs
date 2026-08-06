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

use crate::bounded_types;

#[test]
fn bounded_string_typedef_maps_to_str() {
    let short_str: bounded_types::ShortString = "Hello".into();
    assert_eq!(short_str, "Hello");
    assert_eq!(
        std::any::type_name::<bounded_types::ShortString>(),
        std::any::type_name::<String>()
    );

    let medium_str: bounded_types::MediumString = "This is a medium length string".into();
    assert_eq!(medium_str.len(), 30);
    assert_eq!(
        std::any::type_name::<bounded_types::MediumString>(),
        std::any::type_name::<String>()
    );

    let long_str: bounded_types::LongString =
        "This is a very long string that could contain a lot of text".into();
    assert!(long_str.len() > 0);
    assert_eq!(
        std::any::type_name::<bounded_types::LongString>(),
        std::any::type_name::<String>()
    );
}

#[test]
fn bounded_sequence_typedef_maps_to_list() {
    let small_list: bounded_types::SmallIntList = vec![1, 2, 3, 4, 5];
    assert_eq!(small_list.len(), 5);
    assert_eq!(
        std::any::type_name::<bounded_types::SmallIntList>(),
        std::any::type_name::<Vec<i32>>()
    );

    let string_list: bounded_types::StringList100 =
        vec!["one".into(), "two".into(), "three".into()];
    assert_eq!(string_list.len(), 3);
    assert_eq!(
        std::any::type_name::<bounded_types::StringList100>(),
        std::any::type_name::<Vec<String>>()
    );

    let double_list: bounded_types::LargeDoubleList = vec![1.1, 2.2, 3.3];
    assert_eq!(double_list.len(), 3);
    assert_eq!(
        std::any::type_name::<bounded_types::LargeDoubleList>(),
        std::any::type_name::<Vec<f64>>()
    );
}

#[test]
fn bounded_fields_struct() {
    let bf = bounded_types::BoundedFields {
        name: "test".into(),
        description: "description".into(),
        values: vec![1, 2, 3],
        tags: vec!["tag1".into(), "tag2".into()],
    };

    assert_eq!(bf.name, "test");
    assert_eq!(bf.description, "description");
    assert_eq!(bf.values.len(), 3);
    assert_eq!(bf.tags.len(), 2);
    assert_eq!(bf.values[0], 1);
    assert_eq!(bf.tags[0], "tag1");
}

#[test]
fn bounded_fields_annotations() {
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::BoundedFields::new().name),
        std::any::type_name::<String>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::BoundedFields::new().description),
        std::any::type_name::<String>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::BoundedFields::new().values),
        std::any::type_name::<Vec<i32>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::BoundedFields::new().tags),
        std::any::type_name::<Vec<String>>()
    );
}

#[test]
fn nested_bounded_struct() {
    let matrix: Vec<Vec<i32>> = vec![vec![1, 2], vec![3, 4]];
    let indexed_lists = BTreeMap::from([("key".to_string(), vec![5, 6])]);

    let nb = bounded_types::NestedBounded {
        matrix,
        indexed_lists,
    };

    assert_eq!(nb.matrix.len(), 2);
    assert_eq!(nb.matrix[0][1], 2);
    assert_eq!(nb.indexed_lists["key"][0], 5);
}

#[test]
fn nested_bounded_annotations() {
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::NestedBounded::new().matrix),
        std::any::type_name::<Vec<Vec<i32>>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::NestedBounded::new().indexed_lists),
        std::any::type_name::<BTreeMap<String, Vec<i32>>>()
    );
}

#[test]
fn typedef_chain_with_bounds() {
    let name: bounded_types::Name = "Alice".into();
    assert_eq!(name, "Alice");
    assert_eq!(
        std::any::type_name::<bounded_types::Name>(),
        std::any::type_name::<String>()
    );

    let names: bounded_types::NameList = vec!["Alice".into(), "Bob".into(), "Charlie".into()];
    assert_eq!(names.len(), 3);
    assert_eq!(
        std::any::type_name::<bounded_types::NameList>(),
        std::any::type_name::<Vec<bounded_types::Name>>()
    );

    let name_map: bounded_types::NameMap =
        bounded_types::NameMap::from([("group1".into(), vec!["Alice".into(), "Bob".into()])]);

    assert_eq!(name_map.len(), 1);
    assert_eq!(
        std::any::type_name::<bounded_types::NameMap>(),
        std::any::type_name::<BTreeMap<String, bounded_types::NameList>>()
    );
}

#[test]
fn mixed_bounds_struct() {
    let mb = bounded_types::MixedBounds {
        bounded_string: "bounded".into(),
        unbounded_string: "unbounded".into(),
        bounded_seq: vec![1, 2],
        unbounded_seq: vec![3, 4, 5],
    };

    assert_eq!(mb.bounded_string, "bounded");
    assert_eq!(mb.unbounded_string, "unbounded");
    assert_eq!(mb.bounded_seq.len(), 2);
    assert_eq!(mb.unbounded_seq.len(), 3);
}

#[test]
fn mixed_bounds_annotations() {
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::MixedBounds::new().bounded_string),
        std::any::type_name::<String>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::MixedBounds::new().unbounded_string),
        std::any::type_name::<String>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::MixedBounds::new().unbounded_seq),
        std::any::type_name::<Vec<i32>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&bounded_types::MixedBounds::new().bounded_seq),
        std::any::type_name::<Vec<i32>>()
    );
}

#[test]
fn bounds_not_enforced_at_runtime() {
    let str: bounded_types::ShortString =
        "This string is longer than 32 characters and should not be truncated at runtime".into();
    assert!(str.len() > 32);

    let mut list = bounded_types::SmallIntList::new();
    for i in 0..20 {
        list.push(i);
    }
    assert_eq!(list.len(), 20);
}
