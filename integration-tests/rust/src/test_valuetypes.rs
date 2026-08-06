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

use crate::valuetype_types;

#[test]
fn valuetype_instantiation() {
    let sv = valuetype_types::SimpleValue {
        id: 42,
        name: "test".into(),
    };
    assert_eq!(sv.id, 42);
    assert_eq!(sv.name, "test");
}

#[test]
fn valuetype_defaults() {
    let sv = valuetype_types::SimpleValue::new();
    assert_eq!(sv.id, 0);
    assert_eq!(sv.name, "");
}

#[test]
fn valuetype_inheritance() {
    let dv = valuetype_types::DerivedValue {
        id: 1,
        name: "base".into(),
        description: "derived".into(),
    };
    assert_eq!(dv.id, 1);
    assert_eq!(dv.name, "base");
    assert_eq!(dv.description, "derived");

    // TODO: Add inheritance test for SimpleValue if ever an inheritance into/cast is added
}

#[test]
fn valuetype_empty() {
    let e = valuetype_types::Empty::new();
    assert_eq!(e, e);
}

#[test]
fn valuetype_with_sequence() {
    let nums = vec![1, 2, 3, 4, 5];
    let names = vec!["a".into(), "b".into(), "c".into()];
    let ws = valuetype_types::WithSequence {
        numbers: nums,
        names,
    };

    assert_eq!(ws.numbers.len(), 5);
    assert_eq!(ws.names.len(), 3);
    assert_eq!(ws.numbers[0], 1);
    assert_eq!(ws.names[1], "b");
}

#[test]
fn valuetype_equality() {
    let v1 = valuetype_types::SimpleValue {
        id: 10,
        name: "test".into(),
    };
    let v2 = valuetype_types::SimpleValue {
        id: 10,
        name: "test".into(),
    };
    let v3 = valuetype_types::SimpleValue {
        id: 20,
        name: "other".into(),
    };

    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

#[test]
fn valuetype_supports_interface() {
    let iv = valuetype_types::IdentifiableValue {
        id: 123,
        data: "data".into(),
    };
    assert_eq!(iv.id, 123);
    assert_eq!(iv.data, "data");
}

#[test]
fn valuetype_supports_named() {
    let nv = valuetype_types::NamedValue {
        name: "test_name".into(),
        value: 456,
    };
    assert_eq!(nv.name, "test_name");
    assert_eq!(nv.value, 456);
}

#[test]
fn valuetype_inheritance_and_supports() {
    let fv = valuetype_types::FullValue {
        id: 1,
        name: "name".into(),
        extra: "extra".into(),
    };
    assert_eq!(fv.id, 1);
    assert_eq!(fv.name, "name");
    assert_eq!(fv.extra, "extra");

    // TODO: Add inheritance test for SimpleValue if ever an inheritance into/cast is added
}

#[test]
fn valuetype_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&valuetype_types::SimpleValue::new().id),
        std::any::type_name::<i32>(),
        "id should be i32"
    );
    assert_eq!(
        std::any::type_name_of_val(&valuetype_types::SimpleValue::new().name),
        std::any::type_name::<String>(),
        "name should be String"
    );
}

#[test]
fn valuetype_sequence_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&valuetype_types::WithSequence::new().numbers),
        std::any::type_name::<Vec<i32>>(),
        "numbers should be Vec<i32>"
    );
    assert_eq!(
        std::any::type_name_of_val(&valuetype_types::WithSequence::new().names),
        std::any::type_name::<Vec<String>>(),
        "names should be Vec<String>"
    );
}

#[test]
fn valuetype_derived_field_types() {
    assert_eq!(
        std::any::type_name_of_val(&valuetype_types::DerivedValue::new().description),
        std::any::type_name::<String>(),
        "description should be String"
    );

    // TODO: add inheritance test for SimpleValue if into/casting/is is implemented
}

#[test]
fn valuetype_comparison_operators() {
    let v1 = valuetype_types::SimpleValue {
        id: 10,
        name: "test".into(),
    };
    let v2 = valuetype_types::SimpleValue {
        id: 10,
        name: "test".into(),
    };
    let v3 = valuetype_types::SimpleValue {
        id: 5,
        name: "other".into(),
    };
    let v4 = valuetype_types::SimpleValue {
        id: 10,
        name: "zzz".into(),
    };

    assert_eq!(v1, v2);
    assert!(!(v1 == v3));
    assert!(v1 != v3);
    assert!(v3 < v1);
    assert!(v1 > v3);
    assert!(v1 < v4);
    assert!(v1 <= v2);
    assert!(v1 >= v2);
}
