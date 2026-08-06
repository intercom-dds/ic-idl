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

use crate::typedef_types;

#[test]
fn primitive_typedef_values() {
    let i: typedef_types::Integer = 42;
    assert_eq!(i, 42);

    let ui: typedef_types::UnsignedInteger = 100;
    assert_eq!(ui, 100);

    let r: typedef_types::Real = 3.14;
    assert_approx!(r, 3.14, f64::EPSILON);

    let t: typedef_types::Text = "hello".into();
    assert_eq!(t, "hello");

    let f: typedef_types::Flag = true;
    assert!(f);

    let b: typedef_types::Byte = 255;
    assert_eq!(b, 255);
}

#[test]
fn sequence_typedef_values() {
    let il: typedef_types::IntList = vec![1, 2, 3, 4, 5];
    assert_eq!(il.len(), 5);
    assert_eq!(il[0], 1);
    assert_eq!(il[4], 5);

    let sl: typedef_types::StringList = vec!["one".into(), "two".into(), "three".into()];
    assert_eq!(sl.len(), 3);
    assert_eq!(sl[0], "one");
    assert_eq!(sl[2], "three");

    let rl: typedef_types::RealList = vec![1.1, 2.2, 3.3];
    assert_eq!(rl.len(), 3);
    assert_approx!(rl[0], 1.1, f64::EPSILON);
    assert_approx!(rl[2], 3.3, f64::EPSILON);
}

#[test]
fn nested_typedef_values() {
    let c: typedef_types::Count = 42;
    assert_eq!(c, 42);

    let l: typedef_types::Label = "test_label".into();
    assert_eq!(l, "test_label");
}

#[test]
fn map_typedef_values() {
    let sim = typedef_types::StringIntMap::from([("one".into(), 1), ("two".into(), 2)]);
    assert_eq!(sim.len(), 2);
    assert_eq!(sim["one"], 1);
    assert_eq!(sim["two"], 2);

    let ssm = typedef_types::StringStringMap::from([
        ("key1".into(), "value1".into()),
        ("key2".into(), "value2".into()),
    ]);
    assert_eq!(ssm.len(), 2);
    assert_eq!(ssm["key1"], "value1");
}

#[test]
fn array_typedef_value() {
    let la: typedef_types::LongArray = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(la.len(), 10);
    assert_eq!(la[0], 1);
    assert_eq!(la[9], 10);
}

#[test]
fn struct_with_typedef_fields() {
    let p = typedef_types::Point { x: 10.5, y: 20.5 };
    assert_approx!(p.x, 10.5, f64::EPSILON);
    assert_approx!(p.y, 20.5, f64::EPSILON);
}

#[test]
fn struct_with_typedef_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Point::new().x),
        std::any::type_name::<typedef_types::Real>(),
        "x should be Real"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Point::new().y),
        std::any::type_name::<typedef_types::Real>(),
        "y should be Real"
    );
}

#[test]
fn person_struct_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Person::new().name),
        std::any::type_name::<typedef_types::Text>(),
        "name should be Text"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Person::new().age),
        std::any::type_name::<typedef_types::Integer>(),
        "age should be Integer"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Person::new().active),
        std::any::type_name::<typedef_types::Flag>(),
        "active should be Flag"
    );
}

#[test]
fn person_struct_values() {
    let person = typedef_types::Person {
        name: "Alice".into(),
        age: 30,
        active: true,
    };
    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 30);
    assert!(person.active);
}

#[test]
fn container_struct_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Container::new().numbers),
        std::any::type_name::<typedef_types::IntList>(),
        "numbers should be IntList"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Container::new().labels),
        std::any::type_name::<typedef_types::StringList>(),
        "labels should be StringList"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Container::new().lookup),
        std::any::type_name::<typedef_types::StringIntMap>(),
        "lookup should be StringIntMap"
    );
}

#[test]
fn container_struct_values() {
    let nums: typedef_types::IntList = vec![1, 2, 3];
    let labs: typedef_types::StringList = vec!["a".into(), "b".into(), "c".into()];
    let lup = typedef_types::StringIntMap::from([("x".into(), 10), ("y".into(), 20)]);
    let container = typedef_types::Container {
        numbers: nums,
        labels: labs,
        lookup: lup,
    };

    assert_eq!(container.numbers.len(), 3);
    assert_eq!(container.labels.len(), 3);
    assert_eq!(container.lookup.len(), 2);
    assert_eq!(container.numbers[0], 1);
    assert_eq!(container.labels[1], "b");
    assert_eq!(container.lookup["x"], 10);
}

#[test]
fn nested_typedef_in_struct() {
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Measurement::new().name),
        std::any::type_name::<typedef_types::Label>(),
        "name should be Label"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::Measurement::new().value),
        std::any::type_name::<typedef_types::Count>(),
        "value should be Count"
    );
}

#[test]
fn nested_typedef_struct_values() {
    let m = typedef_types::Measurement {
        name: "temperature".into(),
        value: 42,
    };
    assert_eq!(m.name, "temperature");
    assert_eq!(m.value, 42);
}

#[test]
fn array_typedef_in_struct() {
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::WithArrayTypedef::new().values),
        std::any::type_name::<typedef_types::LongArray>(),
        "values should be LongArray"
    );
}

#[test]
fn array_typedef_struct_values() {
    let arr: typedef_types::LongArray = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let wat = typedef_types::WithArrayTypedef { values: arr };
    assert_eq!(wat.values.len(), 10);
    assert_eq!(wat.values[0], 1);
    assert_eq!(wat.values[9], 10);
}

#[test]
fn deep_typedef_chain_values() {
    let l1: typedef_types::Level1 = 100;
    let l2: typedef_types::Level2 = 100;
    let l3: typedef_types::Level3 = 100;
    let l4: typedef_types::Level4 = 100;
    let l5: typedef_types::Level5 = 100;

    assert_eq!(l1, 100);
    assert_eq!(l2, 100);
    assert_eq!(l3, 100);
    assert_eq!(l4, 100);
    assert_eq!(l5, 100);
}

#[test]
fn deep_sequence_typedef_chain() {
    let sl1: typedef_types::SeqLevel1 = vec![1, 2, 3];
    let sl2: typedef_types::SeqLevel2 = vec![4, 5, 6];
    let sl3: typedef_types::SeqLevel3 = vec![7, 8, 9];

    assert_eq!(sl1.len(), 3);
    assert_eq!(sl2.len(), 3);
    assert_eq!(sl3.len(), 3);
}

#[test]
fn deep_map_typedef_chain() {
    let ml1 = typedef_types::MapLevel1::from([("a".into(), 1)]);
    let ml2 = typedef_types::MapLevel2::from([("b".into(), 2)]);
    let ml3 = typedef_types::MapLevel3::from([("c".into(), 3)]);

    assert_eq!(ml1.len(), 1);
    assert_eq!(ml2.len(), 1);
    assert_eq!(ml3.len(), 1);
    assert_eq!(ml1["a"], 1);
    assert_eq!(ml2["b"], 2);
    assert_eq!(ml3["c"], 3);
}

#[test]
fn deep_chain_struct_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::DeepChainStruct::new().deep_int),
        std::any::type_name::<typedef_types::Level5>(),
        "deep_int should be Level5"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::DeepChainStruct::new().deep_seq),
        std::any::type_name::<typedef_types::SeqLevel3>(),
        "deep_seq should be SeqLevel3"
    );
    assert_eq!(
        std::any::type_name_of_val(&typedef_types::DeepChainStruct::new().deep_map),
        std::any::type_name::<typedef_types::MapLevel3>(),
        "deep_map should be MapLevel3"
    );
}

#[test]
fn deep_chain_struct_values() {
    let di: typedef_types::Level5 = 999;
    let ds: typedef_types::SeqLevel3 = vec![1, 2, 3, 4, 5];
    let dm = typedef_types::MapLevel3::from([("key1".into(), 100), ("key2".into(), 200)]);
    let dcs = typedef_types::DeepChainStruct {
        deep_int: di,
        deep_seq: ds,
        deep_map: dm,
    };

    assert_eq!(dcs.deep_int, 999);
    assert_eq!(dcs.deep_seq.len(), 5);
    assert_eq!(dcs.deep_map.len(), 2);
    assert_eq!(dcs.deep_seq[0], 1);
    assert_eq!(dcs.deep_map["key1"], 100);
}

#[test]
fn typedef_type_compatibility() {
    let i: typedef_types::Integer = 42;
    let c: typedef_types::Count = i;
    assert_eq!(c, 42);

    let t: typedef_types::Text = "hello".into();
    let l: typedef_types::Label = t;
    assert_eq!(l, "hello");
}
