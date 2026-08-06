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

use crate::default_types;

#[test]
fn const_string_values() {
    assert_eq!(default_types::DEFAULT_NAME, "unnamed");
    assert_eq!(default_types::DEFAULT_COUNT, 100);
    assert_approx!(default_types::DEFAULT_RATE, 0.5, f64::EPSILON);
}

#[test]
fn struct_const_initializer() {
    assert_eq!(default_types::DEFAULT_INNER.x, 10);
    assert_eq!(default_types::DEFAULT_INNER.y, "default");
    assert_eq!(default_types::NESTED_INNER.x, 99);
    assert_eq!(default_types::NESTED_INNER.y, "nested");
}

#[test]
fn optional_fields_are_none() {
    let opt = default_types::OptionalFields::new();
    assert!(opt.maybe_int.is_none());
    assert!(opt.maybe_string.is_none());
    assert!(opt.maybe_struct.is_none());
}

#[test]
fn optional_fields_type_annotations() {
    let opt = default_types::OptionalFields::new();

    assert_eq!(
        std::any::type_name_of_val(&opt.maybe_int),
        std::any::type_name::<Option<i32>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&opt.maybe_string),
        std::any::type_name::<Option<String>>()
    );
    assert_eq!(
        std::any::type_name_of_val(&opt.maybe_struct),
        std::any::type_name::<Option<default_types::Inner>>()
    );
}

#[test]
fn optional_fields_can_be_set() {
    let mut opt = default_types::OptionalFields::new();
    opt.maybe_int = Some(42);
    opt.maybe_string = Some("test".into());
    opt.maybe_struct = Some(default_types::Inner {
        x: 10,
        y: "hello".into(),
    });
    assert!(opt.maybe_int.is_some());
    assert_eq!(opt.maybe_int, Some(42));
    assert!(opt.maybe_string.is_some());
    assert_eq!(opt.maybe_string, Some("test".into()));
    assert!(opt.maybe_struct.is_some());
    assert_eq!(opt.maybe_struct.as_ref().unwrap().x, 10);
    assert_eq!(opt.maybe_struct.as_ref().unwrap().y, "hello");
}

#[test]
fn enum_default_literal_exists() {
    assert_eq!(default_types::Priority::Low as i32, 0);
    assert_eq!(default_types::Priority::Medium as i32, 1);
    assert_eq!(default_types::Priority::High as i32, 2);
}

#[test]
fn primitive_bool_default() {
    let p = default_types::PrimitiveDefaults::new();
    assert_eq!(p.bool_empty, false);
    assert_eq!(p.bool_true, true);
    assert_eq!(p.bool_false, false);
}

#[test]
fn primitive_int_default() {
    let p = default_types::PrimitiveDefaults::new();
    assert_eq!(p.int_empty, 0);
    assert_eq!(p.int_value, 42);
    assert_eq!(p.int_negative, -100);
}

#[test]
fn primitive_float_default() {
    let p = default_types::PrimitiveDefaults::new();
    assert_approx!(p.float_empty, 0.0, f64::EPSILON);
    assert_approx!(p.float_value, 3.14159, 0.00001);
    assert_approx!(p.float_negative, -0.5, f64::EPSILON);
}

#[test]
fn primitive_string_default() {
    let p = default_types::PrimitiveDefaults::new();
    assert_eq!(p.string_empty, "");
    assert_eq!(p.string_value, "hello");
    assert_eq!(p.string_from_const, "unnamed");
}

#[test]
fn array_default_values() {
    let a = default_types::ArrayDefaults::new();
    assert_eq!(a.array_empty.len(), 3);
    assert_eq!(a.array_empty[0], 0);
    assert_eq!(a.array_empty[1], 0);
    assert_eq!(a.array_empty[2], 0);
    assert_eq!(a.array_values.len(), 3);
    assert_eq!(a.array_values[0], 1);
    assert_eq!(a.array_values[1], 2);
    assert_eq!(a.array_values[2], 3);
    assert_eq!(a.array_partial.len(), 2);
    assert_eq!(a.array_partial[0], 10);
    assert_eq!(a.array_partial[1], 20);
    assert_eq!(a.string_array_empty.len(), 2);
    assert_eq!(a.string_array_empty[0], "");
    assert_eq!(a.string_array_empty[1], "");
    assert_eq!(a.string_array_values.len(), 2);
    assert_eq!(a.string_array_values[0], "foo");
    assert_eq!(a.string_array_values[1], "bar");
}

#[test]
fn sequence_default_values() {
    let s = default_types::SequenceDefaults::new();
    assert_eq!(s.seq_empty.len(), 0);
    assert_eq!(s.seq_values.len(), 5);
    assert_eq!(s.seq_values[0], 1);
    assert_eq!(s.seq_values[1], 2);
    assert_eq!(s.seq_values[2], 3);
    assert_eq!(s.seq_values[3], 4);
    assert_eq!(s.seq_values[4], 5);
    assert_eq!(s.string_seq_empty.len(), 0);
    assert_eq!(s.string_seq_values.len(), 3);
    assert_eq!(s.string_seq_values[0], "a");
    assert_eq!(s.string_seq_values[1], "b");
    assert_eq!(s.string_seq_values[2], "c");
}

#[test]
fn map_default_values() {
    let m = default_types::MapDefaults::new();
    assert_eq!(m.map_empty.len(), 0);
    assert_eq!(m.map_values.len(), 2);
    assert_eq!(m.map_values.get("one"), Some(&1));
    assert_eq!(m.map_values.get("two"), Some(&2));
    assert_eq!(m.reverse_map_empty.len(), 0);
    assert_eq!(m.reverse_map_values.len(), 2);
    assert_eq!(m.reverse_map_values.get(&1), Some(&"one".into()));
    assert_eq!(m.reverse_map_values.get(&2), Some(&"two".into()));
}

#[test]
fn enum_field_default() {
    let e = default_types::EnumDefaults::new();
    assert_eq!(e.priority_high, default_types::Priority::High);
}
