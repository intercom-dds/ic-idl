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

use crate::struct_types;

#[test]
fn point_instantiation() {
    let p = struct_types::Point { x: 10, y: 20 };
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn point_defaults() {
    let p = struct_types::Point::new();
    assert_eq!(p.x, 0);
    assert_eq!(p.y, 0);
}

#[test]
fn point_field_modification() {
    let mut p = struct_types::Point { x: 5, y: 10 };
    p.x = 100;
    p.y = 200;
    assert_eq!(p.x, 100);
    assert_eq!(p.y, 200);
}

#[test]
fn point3d_inheritance() {
    let p3d = struct_types::Point3D { x: 1, y: 2, z: 3 };
    assert_eq!(p3d.x, 1);
    assert_eq!(p3d.y, 2);
    assert_eq!(p3d.z, 3);

    let p = &p3d;
    assert_eq!(p.x, 1);
    assert_eq!(p.y, 2);
}

#[test]
fn nested_struct() {
    let tl = struct_types::Point { x: 0, y: 0 };
    let br = struct_types::Point { x: 100, y: 100 };
    let rect = struct_types::Rectangle {
        top_left: tl,
        bottom_right: br,
    };
    assert_eq!(rect.top_left.x, 0);
    assert_eq!(rect.top_left.y, 0);
    assert_eq!(rect.bottom_right.x, 100);
    assert_eq!(rect.bottom_right.y, 100);
}

#[test]
fn all_primitives() {
    let p = struct_types::AllPrimitives {
        bool_val: true,
        byte_val: 255,
        short_val: -100,
        ushort_val: 1000,
        long_val: -50000,
        ulong_val: 100000,
        longlong_val: -9999999999i64,
        ulonglong_val: 9999999999u64,
        float_val: 3.14f32,
        double_val: 2.71828f64,
        string_val: "hello".into(),
    };
    assert_eq!(p.bool_val, true);
    assert_eq!(p.byte_val, 255);
    assert_eq!(p.short_val, -100);
    assert_eq!(p.ushort_val, 1000);
    assert_eq!(p.long_val, -50000);
    assert_eq!(p.ulong_val, 100000u32);
    assert_eq!(p.longlong_val, -9999999999i64);
    assert_eq!(p.ulonglong_val, 9999999999u64);
    assert_approx!(p.float_val, 3.14f32, f32::EPSILON);
    assert_approx!(p.double_val, 2.71828f64, f64::EPSILON);
    assert_eq!(p.string_val, "hello");
}

#[test]
fn struct_with_sequence() {
    let s = struct_types::WithSequence {
        numbers: vec![1, 2, 3],
        names: vec!["a".into(), "b".into()],
    };

    assert_eq!(s.numbers.len(), 3);
    assert_eq!(s.numbers[0], 1);
    assert_eq!(s.numbers[1], 2);
    assert_eq!(s.numbers[2], 3);
    assert_eq!(s.names.len(), 2);
    assert_eq!(s.names[0], "a");
    assert_eq!(s.names[1], "b");
}

#[test]
fn struct_with_array() {
    let w = struct_types::WithArray {
        fixed_numbers: [1, 2, 3, 4, 5],
    };
    assert_eq!(w.fixed_numbers.len(), 5);
    assert_eq!(w.fixed_numbers[0], 1);
    assert_eq!(w.fixed_numbers[4], 5);
}

#[test]
fn struct_with_map() {
    let mut map = std::collections::BTreeMap::new();
    map.insert("one".into(), 1);
    map.insert("two".into(), 2);

    let m = struct_types::WithMap { string_to_int: map };
    assert_eq!(m.string_to_int.len(), 2);
    assert_eq!(m.string_to_int.get("one"), Some(&1));
    assert_eq!(m.string_to_int.get("two"), Some(&2));
}

#[test]
fn multi_level_inheritance() {
    let p4d = struct_types::Point4D {
        x: 1,
        y: 2,
        z: 3,
        w: 4,
    };
    assert_eq!(p4d.x, 1);
    assert_eq!(p4d.y, 2);
    assert_eq!(p4d.z, 3);
    assert_eq!(p4d.w, 4);
}

/* */
#[test]
fn empty_struct() {
    let _e = struct_types::Empty {};
}

#[test]
fn all_primitives_defaults() {
    let p = struct_types::AllPrimitives::new();
    assert_eq!(p.bool_val, false);
    assert_eq!(p.byte_val, 0);
    assert_eq!(p.short_val, 0);
    assert_eq!(p.ushort_val, 0);
    assert_eq!(p.long_val, 0);
    assert_eq!(p.ulong_val, 0);
    assert_eq!(p.longlong_val, 0);
    assert_eq!(p.ulonglong_val, 0);
    assert_approx!(p.float_val, 0.0f32, f32::EPSILON);
    assert_approx!(p.double_val, 0.0f64, f64::EPSILON);
    assert_eq!(p.string_val, "");
}

#[test]
fn point_copy_constructor() {
    let mut p1 = struct_types::Point { x: 10, y: 20 };
    let p2 = p1;
    assert_eq!(p2.x, 10);
    assert_eq!(p2.y, 20);

    p1.x = 30;
    assert_eq!(p1.x, 30);
    assert_eq!(p2.x, 10);
}

#[test]
fn point_assignment() {
    let mut p1 = struct_types::Point { x: 10, y: 20 };
    let p2;
    p2 = p1;
    assert_eq!(p2.x, 10);
    assert_eq!(p2.y, 20);

    p1.x = 30;
    assert_eq!(p1.x, 30);
    assert_eq!(p2.x, 10);
}

#[test]
fn point_equality() {
    let p1 = struct_types::Point { x: 10, y: 20 };
    let p2 = struct_types::Point { x: 10, y: 20 };
    let p3 = struct_types::Point { x: 10, y: 30 };

    assert!(p1 == p2);
    assert!(!(p1 == p3));
}

#[test]
fn point_inequality() {
    let p1 = struct_types::Point { x: 10, y: 20 };
    let p2 = struct_types::Point { x: 10, y: 20 };
    let p3 = struct_types::Point { x: 10, y: 30 };

    assert!(!(p1 != p2));
    assert!(p1 != p3);
}

#[test]
fn point_less_than() {
    let p1 = struct_types::Point { x: 5, y: 10 };
    let p2 = struct_types::Point { x: 5, y: 20 };
    let p3 = struct_types::Point { x: 10, y: 5 };
    let p4 = struct_types::Point { x: 5, y: 10 };

    assert!(p1 < p2);
    assert!(p1 < p3);
    assert!(!(p2 < p1));
    assert!(!(p1 < p4));
}

#[test]
fn struct_move_semantics() {
    let r1 = struct_types::Rectangle {
        top_left: struct_types::Point { x: 0, y: 0 },
        bottom_right: struct_types::Point { x: 100, y: 100 },
    };

    let r2 = r1;
    assert_eq!(r2.top_left.x, 0);
    assert_eq!(r2.bottom_right.x, 100);

    let r3;
    r3 = r2;
    assert_eq!(r3.top_left.x, 0);
    assert_eq!(r3.bottom_right.x, 100);
}

#[test]
fn nested_struct_deep_copy() {
    let tl = struct_types::Point { x: 0, y: 0 };
    let br = struct_types::Point { x: 100, y: 100 };
    let mut r1 = struct_types::Rectangle {
        top_left: tl,
        bottom_right: br,
    };

    let r2 = r1;

    r1.top_left.x = 50;
    assert_eq!(r1.top_left.x, 50);
    assert_eq!(r2.top_left.x, 0);
}

#[test]
fn sequence_field_operations() {
    let mut s = struct_types::WithSequence::new();

    s.numbers.push(1);
    s.numbers.push(2);
    s.numbers.push(3);
    assert_eq!(s.numbers.len(), 3);
    assert_eq!(s.numbers[0], 1);
    assert_eq!(s.numbers[2], 3);

    s.numbers.remove(1);
    assert_eq!(s.numbers.len(), 2);
    assert_eq!(s.numbers[0], 1);
    assert_eq!(s.numbers[1], 3);

    s.names.push("hello".into());
    s.names.push("world".into());
    assert_eq!(s.names.len(), 2);
    assert_eq!(s.names[0], "hello");
    assert_eq!(s.names[1], "world");
}

#[test]
fn map_field_operations() {
    let mut m = struct_types::WithMap::new();

    m.string_to_int.insert("one".into(), 1);
    m.string_to_int.insert("two".into(), 2);
    m.string_to_int.insert("three".into(), 3);

    assert_eq!(m.string_to_int.len(), 3);
    assert_eq!(m.string_to_int["one"], 1);
    assert_eq!(m.string_to_int["three"], 3);

    m.string_to_int.remove("two");
    assert_eq!(m.string_to_int.len(), 2);
    assert!(!m.string_to_int.contains_key("two"));
}

#[test]
fn constructor_with_all_fields() {
    let p = struct_types::Point { x: 42, y: 84 };
    assert_eq!(p.x, 42);
    assert_eq!(p.y, 84);

    let p3d = struct_types::Point3D { x: 1, y: 2, z: 3 };
    assert_eq!(p3d.x, 1);
    assert_eq!(p3d.y, 2);
    assert_eq!(p3d.z, 3);

    let p4d = struct_types::Point4D {
        x: 10,
        y: 20,
        z: 30,
        w: 40,
    };
    assert_eq!(p4d.x, 10);
    assert_eq!(p4d.y, 20);
    assert_eq!(p4d.z, 30);
    assert_eq!(p4d.w, 40);
}

#[test]
fn struct_with_defaults() {
    let w = struct_types::WithDefaults::new();
    assert_eq!(w.count, 0);
    assert_eq!(w.name, "");
    assert_approx!(w.value, 0.0, f64::EPSILON);

    let w2 = struct_types::WithDefaults {
        count: 42,
        name: "test".into(),
        value: 3.14,
    };
    assert_eq!(w2.count, 42);
    assert_eq!(w2.name, "test");
    assert_approx!(w2.value, 3.14, f64::EPSILON);
}

#[test]
fn struct_swap() {
    let mut p1 = struct_types::Point { x: 10, y: 20 };
    let mut p2 = struct_types::Point { x: 30, y: 40 };

    std::mem::swap(&mut p1, &mut p2);

    assert_eq!(p1.x, 30);
    assert_eq!(p1.y, 40);
    assert_eq!(p2.x, 10);
    assert_eq!(p2.y, 20);
}

#[test]
fn struct_swap_nested() {
    let mut r1 = struct_types::Rectangle {
        top_left: struct_types::Point { x: 0, y: 0 },
        bottom_right: struct_types::Point { x: 10, y: 10 },
    };
    let mut r2 = struct_types::Rectangle {
        top_left: struct_types::Point { x: 20, y: 20 },
        bottom_right: struct_types::Point { x: 30, y: 30 },
    };

    std::mem::swap(&mut r1, &mut r2);

    assert_eq!(r1.top_left.x, 20);
    assert_eq!(r1.top_left.y, 20);
    assert_eq!(r1.bottom_right.x, 30);
    assert_eq!(r1.bottom_right.y, 30);

    assert_eq!(r2.top_left.x, 0);
    assert_eq!(r2.top_left.y, 0);
    assert_eq!(r2.bottom_right.x, 10);
    assert_eq!(r2.bottom_right.y, 10);
}
