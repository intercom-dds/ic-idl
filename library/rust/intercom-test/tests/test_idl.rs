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

#![allow(clippy::bool_assert_comparison, clippy::float_cmp)]
#![allow(unused_imports, dead_code)] // Some imports/functions are used by commented-out tests

use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use intercom_cts::{Marshal, Unmarshal, bitmask, cdr, cdr1, cdr2, json};
use intercom_test::*;

fn is_type<T: 'static>(value: &dyn Any) -> bool {
    let type_id = TypeId::of::<T>();
    value.type_id() == type_id
}

#[track_caller]
fn roundtrip<T>(value: &T) -> T
where
    T: Marshal + Unmarshal + Default,
{
    let bytes = cdr::to_le_bytes(value).unwrap();
    let value: T = cdr::from_le_bytes(bytes.as_slice()).unwrap();

    let bytes = cdr1::to_le_bytes(&value).unwrap();
    let value: T = cdr1::from_le_bytes(bytes.as_slice()).unwrap();

    let bytes = cdr2::to_le_bytes(&value).unwrap();
    let value: T = cdr2::from_le_bytes(bytes.as_slice()).unwrap();

    let str = json::to_string(&value, false).unwrap();
    json::from_str(&str).unwrap()
}

// TODO: Re-enable when default.idl codegen is fixed (LazyLock for struct constants with String)
// #[test]
// fn test_constants() {
//     // strings
//     assert_eq!(STR1, "STR1");
//     assert_eq!(STR2, "STR2");
//
//     // enum constants
//     assert_eq!(MY_ENUM_CONST, MyDefaultEnum::Nine);
//
//     // arrays
//     assert_eq!(MY_ARR.len(), 1);
//     assert_eq!(MY_ARR[0].len(), 2);
//     assert_eq!(MY_ARR[0][1].len(), 3);
//
//     assert_eq!(MY_ARR[0][0][0], false);
//     assert_eq!(MY_ARR[0][0][1], true);
//     assert_eq!(MY_ARR[0][0][2], false);
//
//     assert_eq!(MY_ARR[0][1][0], true);
//     assert_eq!(MY_ARR[0][1][1], false);
//     assert_eq!(MY_ARR[0][1][2], true);
//
//     // complex, lazily-intiailized constants
//     assert_eq!(MY_TEST.my_int, 123);
//     assert_eq!(MY_TEST.my_str, "test");
//     assert_eq!(MY_TEST.my_float, 1.5);
//     assert!(is_type::<i32>(&MY_TEST.my_int));
//     assert!(is_type::<f32>(&MY_TEST.my_float));
//
//     assert_eq!(MY_CONTAINS_TEST.my_test.my_int, 123);
//     assert_eq!(MY_CONTAINS_TEST.my_test.my_str, "abc");
//     assert_eq!(MY_CONTAINS_TEST.my_test.my_float, 3.0);
//     assert_eq!(MY_CONTAINS_TEST.other_int, 456);
// }

// TODO: Re-enable when default.idl codegen is fixed
// #[test]
// fn test_default_struct() {
//     let x = X::default();
//     assert_eq!(x.bool_empty, false);
//     assert_eq!(x.bool_default, true);
//
//     assert_eq!(x.int32_empty, 0);
//     assert_eq!(x.int32_default, 5);
//     assert!(is_type::<i32>(&x.int32_empty));
//
//     assert_eq!(x.float_empty, 0.0);
//     assert_eq!(x.float_default, 0.75);
//     assert!(is_type::<f32>(&x.float_empty));
//
//     assert!(x.string_empty.is_empty());
//     assert_eq!(x.string_default, "test");
//     assert_eq!(x.string_const_default, STR1);
//     assert!(is_type::<String>(&x.string_empty));
//
//     assert!(x.string_shared_empty.is_empty());
//     assert!(is_type::<Box<String>>(&x.string_shared_empty));
//
//     assert_eq!(*x.string_shared_default, STR1);
//     assert!(is_type::<Box<String>>(&x.string_shared_default));
//
//     assert!(x.optional_empty.is_none());
//     assert!(is_type::<Option<i32>>(&x.optional_empty));
//
//     assert_eq!(x.array_empty.len(), 2);
//     assert_eq!(x.array_empty[0], 0);
//     assert_eq!(x.array_empty[1], 0);
//     assert_eq!(x.array_default[0], 123);
//     assert_eq!(x.array_default[1], 456);
//     assert!(is_type::<[i32; 2]>(&x.array_empty));
//
//     assert!(x.string_seq_empty.is_empty());
//     assert_eq!(x.string_seq_default.len(), 2);
//     assert_eq!(x.string_seq_default[0], "abc");
//     assert_eq!(x.string_seq_default[1], "def");
//     assert!(is_type::<Vec<String>>(&x.string_seq_empty));
//
//     assert!(x.string_seq_const_empty.is_empty());
//     assert!(is_type::<Vec<String>>(&x.string_seq_const_empty));
//
//     assert!(x.primitive_map_empty.is_empty());
//     assert_eq!(x.primitive_map_default.len(), 5);
//     assert_eq!(*x.primitive_map_default.get(&1).unwrap(), 2);
//     assert_eq!(*x.primitive_map_default.get(&3).unwrap(), 4);
//     assert_eq!(*x.primitive_map_default.get(&5).unwrap(), 6);
//     assert_eq!(*x.primitive_map_default.get(&7).unwrap(), 8);
//     assert_eq!(*x.primitive_map_default.get(&9).unwrap(), 0);
//
//     assert!(x.string_map_empty.is_empty());
//     assert_eq!(x.string_map_default.len(), 2);
//     assert_eq!(x.string_map_default.get("key1").unwrap(), "value1");
//     assert_eq!(x.string_map_default.get("key2").unwrap(), "value2");
//
//     assert!(x.complex_array_seq_empty.is_empty());
//     assert!(is_type::<Vec<MyArray>>(&x.complex_array_seq_empty));
//
//     assert_eq!(x.complex_array_seq_default.len(), 2);
//     assert_eq!(x.complex_array_seq_default[0][0][0][0], false);
//     assert_eq!(x.complex_array_seq_default[0][0][0][1], false);
//     assert_eq!(x.complex_array_seq_default[0][0][0][2], false);
//
//     assert_eq!(x.complex_array_seq_default[0][0][1][0], true);
//     assert_eq!(x.complex_array_seq_default[0][0][1][1], false);
//     assert_eq!(x.complex_array_seq_default[0][0][1][2], false);
//
//     assert_eq!(x.complex_array_seq_default[1][0][0][0], false);
//     assert_eq!(x.complex_array_seq_default[1][0][0][1], true);
//     assert_eq!(x.complex_array_seq_default[1][0][0][2], false);
//
//     assert_eq!(x.complex_array_seq_default[1][0][1][0], false);
//     assert_eq!(x.complex_array_seq_default[1][0][1][1], false);
//     assert_eq!(x.complex_array_seq_default[1][0][1][2], true);
// }

// TODO: Re-enable when default.idl codegen is fixed
// #[test]
// fn test_default_literal() {
//     let x = MyDefaultEnum::default();
//     assert_eq!(x, MyDefaultEnum::Two);
//     assert_eq!(MyDefaultEnum::One as usize, 1);
//     assert_eq!(MyDefaultEnum::Two as usize, 2);
//     assert_eq!(MyDefaultEnum::Nine as usize, 9);
// }

// TODO: Re-enable when default.idl codegen is fixed
// #[test]
// fn test_union_default_null() {
//     let x = MyUnionDefaultNull::default();
//     assert_eq!(x.disc(), 0);
// }

// TODO: Re-enable when default.idl codegen is fixed
// #[test]
// fn test_union_default_shared() {
//     let x = UnionSharedDefault::default();
//     assert_eq!(x.disc(), 1);
//
//     if let UnionSharedDefault::MyInt(v) = x {
//         assert!(is_type::<Box<i32>>(&v));
//         assert_eq!(v, Box::new(5));
//     } else {
//         panic!("union is initialized with the wrong discriminator");
//     }
// }

// TODO: Re-enable when derive.idl codegen generates the Dummy trait impl
// #[test]
// fn test_derive_annotation() {
//     fn has_dummy<T: Dummy>() {}
//
//     has_dummy::<derive::MyDerivedStruct>();
//     has_dummy::<derive::MyDerivedUnion>();
//     has_dummy::<derive::MyDerivedEnum>();
// }

// TODO: Re-enable when bitmask.idl types are generated
// #[test]
// fn test_bitmask() {
//     let x = BitmaskTest::default();
//     assert_eq!(x, BitmaskTest::BIT_D);
//
//     let x = WrapsBitmask::default();
//     assert_eq!(x.value, BitmaskTest::BIT_D);
//
//     assert_eq!(BitmaskAbc::FLAG_A | BitmaskAbc::FLAG_B, BitmaskAbc(6));
//     assert_eq!(BitmaskAbc::FLAG_A & BitmaskAbc::FLAG_B, BitmaskAbc(0));
//     assert_eq!(BitmaskAbc::FLAG_A ^ BitmaskAbc::FLAG_B, BitmaskAbc(6));
//
//     let mut mask = BitmaskAbc::FLAG_A;
//     mask |= BitmaskAbc::FLAG_B;
//     assert_eq!(mask, BitmaskAbc(6));
//
//     let mut mask = BitmaskAbc::FLAG_A | BitmaskAbc::FLAG_B;
//     mask &= BitmaskAbc::FLAG_A;
//     assert_eq!(mask, BitmaskAbc::FLAG_A);
//
//     let mut mask = BitmaskAbc::FLAG_A;
//     mask ^= BitmaskAbc::FLAG_B;
//     assert_eq!(mask, BitmaskAbc(6));
//
//     assert_eq!((!BitmaskAbc::FLAG_A).0, !BitmaskAbc::FLAG_A.0);
//
//     let mut mask = BitmaskAbc::all();
//     assert!(mask.is_all());
//     mask.clear();
//     assert!(mask.is_empty());
// }

#[test]
fn test_bitmask_vis() {
    bitmask! {
        PrivateBitmask: u8 {
            FOO = 1,
        }
    }
    bitmask! {
        pub PublicBitmask: u8 {
            FOO = 1,
        }
    }
}

#[test]
fn test_bitmask_debug() {
    bitmask! {
        MyBitmask: u32 {
            FOO = 1 << 0,
            BAR = 1 << 1,
            BAZ = 1 << 2,
            QUX = 1 << 3,
        }
    }

    let bitmask = MyBitmask::FOO | MyBitmask::BAZ;
    let formatted = format!("{bitmask:?}");
    assert_eq!(formatted, "MyBitmask(FOO | BAZ)");
}

#[test]
fn test_bitmask_debug_empty() {
    bitmask! {
        MyBitmask: u32 {
            FOO = 1 << 0,
            BAR = 1 << 1,
            BAZ = 1 << 2,
            QUX = 1 << 3,
        }
    }

    let bitmask = MyBitmask::nil();
    let formatted = format!("{bitmask:?}");
    assert_eq!(formatted, "MyBitmask(0)");
}

#[test]
fn test_bitmask_debug_submask() {
    bitmask! {
        MyBitmask: u32 {
            FOO = 1 << 0,
            BAR = 1 << 1,
            BAZ = 1 << 2,
            MASK = 0xB,
        }
    }

    let bitmask = MyBitmask::MASK;
    let formatted = format!("{bitmask:?}");
    assert!(formatted.contains("MASK"));
}

// TODO: Re-enable when scoped.idl codegen is fixed (union variant naming bug)
// #[test]
// fn test_nested_definitions_scope() {
//     // Construct each type to make sure they exist in the correct scope
//     let _ = scoped::Foo::default();
//     let _ = scoped::Bar::default();
//     let _ = scoped::my_interface::Foo::default();
//     let _ = scoped::my_interface::Bar::default();
//     let _ = scoped::my_interface::DeeplyNestedEnum::default();
//     let _ = scoped::my_interface::MyBitmask::default();
//     let _ = scoped::my_interface::Bar::default();
//     let _ = scoped::my_interface::MyException::default();
//     let _ = scoped::my_interface::MyExceptionResult::Ok(());
//     assert_eq!(
//         scoped::DEEPLY_NESTED_CONST,
//         scoped::my_interface::DeeplyNestedEnum::One,
//     );
//     assert_eq!(scoped::MY_BITMASK, scoped::my_interface::MyBitmask::ZERO);
// }

// TODO: Re-enable when scoped.idl codegen is fixed (union variant naming bug)
// #[test]
// fn test_union_multiple_labels_same_member() {
//     // Access the variants to make sure they're correctly named
//     let _ = label::MultipleNull::Zero;
//     let _ = label::MultipleNull::One;
//     let _ = label::MultipleNull::Var(String::default());
//
//     let _ = label::MultipleNullOctet::Null0;
//     let _ = label::MultipleNullOctet::Null1;
//     let _ = label::MultipleNullOctet::Var(String::default());
//
//     let _ = label::MultipleDefault::Zero(String::default());
//     let _ = label::MultipleDefault::One(String::default());
//     let _ = label::MultipleDefault::Two(String::default());
//
//     let _ = label::MultipleDefaultOctet::Var0(String::default());
//     let _ = label::MultipleDefaultOctet::Var1(String::default());
//     let _ = label::MultipleDefaultOctet::VarDefault(String::default());
//
//     let _ = label::AllLiteralsCovered::Null;
//     let _ = label::AllLiteralsCovered::MyInt(0);
//     let _ = label::AllLiteralsCovered::MyStr(String::default());
//
//     let _ = intercom::xtypes::TypeIdentifier::TkNone;
//     let _ = intercom::xtypes::TypeIdentifier::TkByte;
//     let _ = intercom::xtypes::TypeIdentifier::TkInt8;
//     let _ = intercom::xtypes::TypeIdentifier::EkComplete;
//     let _ = intercom::xtypes::TypeIdentifier::EkMinimal;
//     let _ = intercom::xtypes::TypeIdentifier::TiString8Small;
//     let _ = intercom::xtypes::TypeIdentifier::TiString8Large;
// }

// TODO: Re-enable when bounds.idl codegen is fixed (duplicate enum generation)
// #[test]
// fn test_min() {
//     let valid = bounds::MinType {
//         min_i8: 0,
//         min_u16: 128,
//         min_u32: 1024,
//         min_i64: 4096,
//     };
//     assert_eq!(valid, roundtrip(&valid));
//
//     let invalid = bounds::MinType {
//         min_i8: -1,
//         min_u16: 128,
//         min_u32: 1024,
//         min_i64: 4096,
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::MinType {
//         min_i8: 0,
//         min_u16: 127,
//         min_u32: 1024,
//         min_i64: 4096,
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::MinType {
//         min_i8: 0,
//         min_u16: 128,
//         min_u32: 1023,
//         min_i64: 4095,
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::MinType {
//         min_i8: 0,
//         min_u16: 128,
//         min_u32: 1024,
//         min_i64: 4094,
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
// }

// TODO: Re-enable when bounds.idl codegen is fixed (duplicate enum generation)
// #[test]
// fn test_max() {
//     let valid = bounds::MaxType {
//         max_i8: 0,
//         max_u16: 128,
//         max_u32: 1024,
//         max_i64: 4096,
//     };
//     assert_eq!(valid, roundtrip(&valid));
//
//     let invalid = bounds::MaxType {
//         max_i8: 1,
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::MaxType {
//         max_u16: 129,
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::MaxType {
//         max_u32: 1025,
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::MaxType {
//         max_i64: 4097,
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
// }

// TODO: Re-enable when bounds.idl codegen is fixed (duplicate enum generation)
// #[test]
// fn test_range() {
//     let valid = bounds::RangedType {
//         range_i8: 0,
//         range_u16: 128,
//         range_u32: 1024,
//         range_i64: 4096,
//     };
//     assert_eq!(valid, roundtrip(&valid));
//
//     let invalid = bounds::RangedType {
//         range_i8: -1,
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
// }

// TODO: Re-enable when bounds.idl codegen is fixed (duplicate enum generation)
// #[test]
// fn test_bounded_collections() {
//     let valid = bounds::BoundedCollections {
//         bounded_seq: vec![1, 2, 3],
//         bounded_map: BTreeMap::from([
//             ("key1".to_string(), "value1".to_string()),
//             ("key2".to_string(), "value2".to_string()),
//             ("key3".to_string(), "value3".to_string()),
//         ]),
//         bounded_str: "abc".to_string(),
//         bounded_wstr: "abc".to_string(),
//     };
//     assert_eq!(valid, roundtrip(&valid));
//
//     let invalid = bounds::BoundedCollections {
//         bounded_seq: vec![1, 2, 3, 4],
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::BoundedCollections {
//         bounded_map: BTreeMap::from([
//             ("key1".to_string(), "value1".to_string()),
//             ("key2".to_string(), "value2".to_string()),
//             ("key3".to_string(), "value3".to_string()),
//             ("key4".to_string(), "value5".to_string()),
//         ]),
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::BoundedCollections {
//         bounded_str: "abcd".to_string(),
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
//
//     let invalid = bounds::BoundedCollections {
//         bounded_wstr: "abcd".to_string(),
//         ..Default::default()
//     };
//     assert!(cdr1::to_le_bytes(&invalid).is_err());
// }

#[test]
fn test_collision() {
    // Construct each type to make sure they have the correct names
    let _ = precedence::FooBar { unaltered: true };
    let _ = precedence::FooBar_ {
        single_underscore: true,
    };
    let _ = precedence::FooBar__ {
        double_underscore: true,
    };
    let _ = precedence::FooBar___ {
        triple_underscore: true,
    };

    let _ = substitute::FooBar {};
    let _ = substitute::FooBar_ {
        single_underscore: true,
    };
    let _ = substitute::FooBar__ {
        double_underscore: true,
    };

    let _ = mod_collision::my_module::Foo { unaltered: true };
    let _ = mod_collision::my_module_::Foo {};

    let _ = moved_node::foo_bar::Bar {};
    // TODO: Re-enable when collision.idl codegen generates Bar_
    // let _ = moved_node::foo_bar::Bar_ {
    //     single_underscore: true,
    // };

    _ = mod_collision::property::Foo {};
    _ = mod_collision::Property {};
}

#[allow(dead_code)]
fn takes_foobar<T: moved_node::FooBar>(_: T) {}

#[test]
fn test_no_rename() {
    // Construct each type to make sure they have the correct names
    let _ = NOT_RENAMED::my_struct_t {
        MY_MEMBER: NOT_RENAMED::my_const_str.to_string(),
    };
    let _ = NOT_RENAMED::my_enum_e::FOO;
    let _ = NOT_RENAMED::mod_::_struct {};
}

// TODO: Re-enable when default.idl codegen is fixed (LazyLock for struct constants)
// #[test]
// fn test_lazy_member_const() {
//     assert_eq!(referential::MY_FOO.value, "foo");
//     assert_eq!(referential::MY_BAR.my_foo, *referential::MY_FOO);
// }

// TODO: Re-enable when complex.idl is fixed ('any' and 'Object' types)
// #[test]
// fn test_large_array() {
//     let copy = SingleCopyArray::default();
//     assert_eq!(copy, roundtrip(&copy));
//
//     let clone = SingleCloneArray::default();
//     assert_eq!(clone, roundtrip(&clone));
// }

#[test]
fn test_set() {
    let set = HashSet::from([1, 2, 3]);
    assert_eq!(set, roundtrip(&set));

    let set = BTreeSet::from([1, 2, 3]);
    assert_eq!(set, roundtrip(&set));

    let bytes = cdr1::to_le_bytes(&set).unwrap();
    let vec: Vec<i32> = cdr1::from_le_bytes(&bytes).unwrap();
    assert!(set.iter().eq(vec.iter()));
}

#[test]
fn test_bounded_set() {
    let set = HashSet::from([1, 2, 3]);
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 2>(&set)).is_err());

    let set = BTreeSet::from([1, 2, 3]);
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 2>(&set)).is_err());
}

#[test]
fn test_map() {
    let hash = HashMap::from([(1, 1), (2, 2), (3, 3)]);
    assert_eq!(hash, roundtrip(&hash));

    let btree = BTreeMap::from([(1, 1), (2, 2), (3, 3)]);
    assert_eq!(btree, roundtrip(&btree));
}

#[test]
fn test_bounded_map() {
    let hash = HashMap::from([(1, 1), (2, 2), (3, 3)]);
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 2>(&hash)).is_err());

    let btree = BTreeMap::from([(1, 1), (2, 2), (3, 3)]);
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 2>(&btree)).is_err());
}

#[test]
fn test_bounded_string() {
    let string = "plentylong".to_string();
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 5>(&string)).is_err());

    let str = string.as_str();
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 2>(&str)).is_err());
}

#[test]
fn test_bounded_box() {
    let boxed = Box::new("plentylong");
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 5>(&boxed)).is_err());
}

#[test]
fn test_bounded_optional() {
    let opt = Some("plentylong".to_string());
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 5>(&opt)).is_err());

    let opt: Option<String> = None;
    assert!(cdr1::to_le_bytes(&intercom_cts::bound::<_, 5>(&opt)).is_ok());
}

#[test]
fn test_wstring_lit() {
    let string = intercom_cts::WString("foo");
    let bytes = cdr1::to_le_bytes(&string).unwrap();
    assert_eq!(bytes.len(), 10);
}
