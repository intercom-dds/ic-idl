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

use crate::{annotation_types, autoid_hash_types};

#[test]
fn autoid_hash_member_ids() {
    let members = intercom_cts::member_info::<autoid_hash_types::ModuleHash>();
    assert_eq!(members[0].name, "camelCase");
    assert_eq!(
        members.iter().map(|member| member.member_id).collect::<Vec<_>>(),
        [96462948, 37920031, 42, 57943011]
    );
    assert!(
        intercom_cts::type_info::<autoid_hash_types::ModuleHash>()
            .flags
            .contains(intercom_cts::TypeFlag::IS_AUTOID_HASH)
    );

    let members = intercom_cts::member_info::<autoid_hash_types::SequentialOverride>();
    assert_eq!(
        members.iter().map(|member| member.member_id).collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(
        !intercom_cts::type_info::<autoid_hash_types::SequentialOverride>()
            .flags
            .contains(intercom_cts::TypeFlag::IS_AUTOID_HASH)
    );

    let members = intercom_cts::member_info::<autoid_hash_types::HashUnion>();
    assert_eq!(
        members.iter().map(|member| member.member_id).collect::<Vec<_>>(),
        [239_892_167, 256_044_424]
    );
}

#[test]
fn keyed_struct_exists() {
    let ks = annotation_types::KeyedStruct {
        id: 1,
        name: "test".into(),
        value: 3.14,
    };

    assert_eq!(ks.id, 1);
    assert_eq!(ks.name, "test");
    assert_approx!(ks.value, 3.14, f64::EPSILON);
}

#[test]
fn multi_key_struct() {
    let mks = annotation_types::MultiKeyStruct {
        namespace: "namespace1".into(),
        id: 42,
        data: "data".into(),
    };

    assert_eq!(mks.namespace, "namespace1");
    assert_eq!(mks.id, 42);
    assert_eq!(mks.data, "data");
}

#[test]
fn optional_fields_default_none() {
    let os = annotation_types::OptionalStruct::new();

    assert!(os.optional_int.is_none());
    assert!(os.optional_string.is_none());
    assert!(os.optional_seq.is_none());
}

#[test]
fn optional_fields_can_be_set() {
    let mut os = annotation_types::OptionalStruct::new();
    os.optional_int = Some(42);
    os.optional_string = Some("test".into());
    os.optional_seq = Some(vec![1, 2, 3]);

    assert_eq!(os.optional_int, Some(42));
    assert_eq!(os.optional_string, Some("test".into()));
    assert_eq!(os.optional_seq, Some(vec![1, 2, 3]));
}

#[test]
fn optional_type_annotations() {
    let os = annotation_types::OptionalStruct::new();

    assert_eq!(
        std::any::type_name_of_val(&os.optional_int),
        std::any::type_name::<Option<i32>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&os.optional_string),
        std::any::type_name::<Option<String>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&os.optional_seq),
        std::any::type_name::<Option<Vec<i32>>>()
    );
}

#[test]
fn nested_struct() {
    let ns = annotation_types::NestedStruct { x: 10, y: 20 };

    assert_eq!(ns.x, 10);
    assert_eq!(ns.y, 20);
}

#[test]
fn shared_refs_struct() {
    let ns = annotation_types::NestedStruct { x: 5, y: 10 };
    let sr = annotation_types::SharedRefs {
        shared_string: Box::new("shared".into()),
        shared_struct: Box::new(ns),
    };

    assert_eq!(sr.shared_string.as_str(), "shared");
    assert_eq!(sr.shared_struct.x, 5);
    assert_eq!(sr.shared_struct.y, 10);
    assert_eq!(
        std::any::type_name_of_val(&sr.shared_string),
        std::any::type_name::<Box<String>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&sr.shared_struct),
        std::any::type_name::<Box<annotation_types::NestedStruct>>()
    );
}

#[test]
fn combined_annotations() {
    let ca = annotation_types::CombinedAnnotations {
        id: 99,
        maybe_shared_name: Some("combined".into()),
    };

    assert_eq!(ca.id, 99);
    assert_eq!(
        ca.maybe_shared_name,
        Some("combined".into())
    );
    assert_eq!(
        std::any::type_name_of_val(&ca.maybe_shared_name),
        std::any::type_name::<Option<String>>()
    );
}

#[test]
fn custom_shared_annotation_does_not_box() {
    let value = annotation_types::CustomSharedName { value: 42 };

    assert_eq!(value.value, 42);
    assert_eq!(
        std::any::type_name_of_val(&value.value),
        std::any::type_name::<i32>()
    );
}

#[test]
fn annotated_interface_exists() {
    struct _I;
    impl annotation_types::AnnotatedInterface for _I {
        fn fire_and_forget(&mut self, _message: &str) {
            todo!()
        }

        fn get_value(&mut self) -> i32 {
            todo!()
        }

        fn set_value(&mut self, _value: i32) {
            todo!()
        }
    }
}

#[test]
fn topic_struct() {
    let tm = annotation_types::TopicMessage {
        message_id: 1,
        payload: "payload".into(),
        timestamp: 123456,
    };

    assert_eq!(tm.message_id, 1);
    assert_eq!(tm.payload, "payload");
    assert_eq!(tm.timestamp, 123456);
}

#[test]
fn mutable_struct() {
    let ms = annotation_types::MutableStruct {
        version: 1,
        data: "data".into(),
    };

    assert_eq!(ms.version, 1);
    assert_eq!(ms.data, "data");
}

#[test]
fn final_struct() {
    let fs = annotation_types::FinalStruct { fixed_field: 42 };

    assert_eq!(fs.fixed_field, 42);
}

#[test]
fn optional_assignment() {
    let mut os = annotation_types::OptionalStruct {
        optional_int: Some(100),
        ..Default::default()
    };
    assert_eq!(os.optional_int, Some(100));

    os.optional_int = None;

    assert!(os.optional_int.is_none());
}
