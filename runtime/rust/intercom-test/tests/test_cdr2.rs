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

#![allow(clippy::needless_pass_by_value)]

use intercom_cts::type_info::TypeDescriptor;
use intercom_cts::{Marshal, TypeKind, Unmarshal};
use intercom_test::cdr2;
use intercom_test::cdr2_corner_cases::*;

fn roundtrip_expected<T, U>(input: T, expected: U)
where
    T: Marshal + Unmarshal + std::fmt::Debug + Default + PartialEq,
    U: AsRef<[u8]>,
{
    let bytes = intercom_cts::cdr2::to_le_bytes(&input).unwrap();
    assert_eq!(bytes.as_slice(), expected.as_ref(), "serialization differs");

    let roundtrip: T = intercom_cts::cdr2::from_le_bytes(&bytes).unwrap();
    assert_eq!(roundtrip, input, "roundtrip failed");

    let bytes = intercom_cts::cdr2::to_be_bytes(&roundtrip).unwrap();
    let roundtrip: T = intercom_cts::cdr2::from_be_bytes(&bytes).unwrap();
    assert_eq!(roundtrip, input, "roundtrip failed");
}

fn roundtrip<T>(input: T)
where
    T: Marshal + Unmarshal + std::fmt::Debug + PartialEq + Default,
{
    let bytes = intercom_cts::cdr2::to_le_bytes(&input).unwrap();
    let roundtrip: T = intercom_cts::cdr2::from_le_bytes(&bytes).unwrap();
    assert_eq!(roundtrip, input, "roundtrip failed");

    let bytes = intercom_cts::cdr2::to_be_bytes(&roundtrip).unwrap();
    let roundtrip: T = intercom_cts::cdr2::from_be_bytes(&bytes).unwrap();
    assert_eq!(roundtrip, input, "roundtrip failed");
}

// TODO: test final unions, bitmasks, enums, wchar/wstrings
#[test]
fn test_final_primitive_struct() {
    roundtrip_expected(
        cdr2::final_::PrimitiveStruct::default(),
        cdr2::final_::PRIMITIVE_STRUCT_EXPECTED,
    );
}

#[test]
fn test_final_primitive_array() {
    roundtrip_expected(
        cdr2::final_::PrimitiveArray::default(),
        cdr2::final_::PRIMITIVE_ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_array() {
    roundtrip_expected(
        cdr2::final_::ArrayType::default(),
        cdr2::final_::ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_sequence() {
    roundtrip_expected(
        cdr2::final_::SeqType::default(),
        cdr2::final_::SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_primitive_sequence() {
    roundtrip_expected(
        cdr2::final_::PrimitiveSeqType::default(),
        cdr2::final_::PRIMITIVE_SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_map() {
    roundtrip_expected(
        cdr2::final_::MapType::default(),
        cdr2::final_::MAP_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_primitive_map() {
    roundtrip_expected(
        cdr2::final_::PrimitiveMap::default(),
        cdr2::final_::PRIMITIVE_MAP_EXPECTED,
    );
}

#[test]
fn test_final_optional_member() {
    let input = cdr2::final_::OptionalMember {
        is_present: Some(123),
        ..Default::default()
    };

    roundtrip_expected(input, cdr2::final_::OPTIONAL_MEMBER_EXPECTED);
}

#[test]
fn test_appendable_primitive_struct() {
    roundtrip_expected(
        cdr2::appendable::PrimitiveStruct::default(),
        cdr2::appendable::PRIMITIVE_STRUCT_EXPECTED,
    );
}

#[test]
fn test_appendable_array() {
    roundtrip_expected(
        cdr2::appendable::ArrayType::default(),
        cdr2::appendable::ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_primitive_array() {
    roundtrip_expected(
        cdr2::appendable::PrimitiveArray::default(),
        cdr2::appendable::PRIMITIVE_ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_sequence() {
    roundtrip_expected(
        cdr2::appendable::SeqType::default(),
        cdr2::appendable::SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_primitive_sequence() {
    roundtrip_expected(
        cdr2::appendable::PrimitiveSeqType::default(),
        cdr2::appendable::PRIMITIVE_SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_map() {
    roundtrip_expected(
        cdr2::appendable::MapType::default(),
        cdr2::appendable::MAP_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_primitive_map() {
    roundtrip_expected(
        cdr2::appendable::PrimitiveMap::default(),
        cdr2::appendable::PRIMITIVE_MAP_EXPECTED,
    );
}

#[test]
fn test_appendable_optional_member() {
    let input = cdr2::appendable::OptionalMember {
        is_present: Some(123),
        ..Default::default()
    };

    roundtrip_expected(input, cdr2::appendable::OPTIONAL_MEMBER_EXPECTED);
}

#[test]
fn test_appendable_optional_long_pl_member() {
    let input = cdr2::appendable::OptionalLongPlMember {
        is_present: Some(123),
        ..Default::default()
    };

    roundtrip_expected(input, cdr2::appendable::OPTIONAL_LONG_PL_MEMBER);
}

#[test]
fn test_mutable_primitive_struct() {
    roundtrip_expected(
        cdr2::mutable::PrimitiveStruct::default(),
        cdr2::mutable::PRIMITIVE_STRUCT_EXPECTED,
    );
}

#[test]
fn test_mutable_primitive_array() {
    roundtrip_expected(
        cdr2::mutable::PrimitiveArray::default(),
        cdr2::mutable::PRIMITIVE_ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_array() {
    roundtrip_expected(
        cdr2::mutable::ArrayType::default(),
        cdr2::mutable::ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_primitive_sequence() {
    roundtrip_expected(
        cdr2::mutable::PrimitiveSeqType::default(),
        cdr2::mutable::PRIMITIVE_SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_sequence() {
    roundtrip_expected(
        cdr2::mutable::SeqType::default(),
        cdr2::mutable::SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_primitive_map() {
    roundtrip_expected(
        cdr2::mutable::PrimitiveMapType::default(),
        cdr2::mutable::PRIMITIVE_MAP_EXPECTED,
    );
}

#[test]
fn test_mutable_map() {
    roundtrip_expected(
        cdr2::mutable::MapType::default(),
        cdr2::mutable::MAP_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_optional_member() {
    let input = cdr2::mutable::OptionalMember {
        is_present: Some(123),
        ..Default::default()
    };
    roundtrip_expected(input, cdr2::mutable::OPTIONAL_MEMBER_EXPECTED);
}

#[test]
fn test_mutable_long_pl() {
    roundtrip_expected(
        cdr2::mutable::LongPlStruct::default(),
        cdr2::mutable::LONG_PL_STRUCT_EXPECTED,
    );
}

#[test]
fn test_mutable_optional_long_pl() {
    let input = cdr2::mutable::OptionalLongPlMember {
        is_present: Some(123),
        ..Default::default()
    };
    roundtrip_expected(input, cdr2::mutable::OPTIONAL_LONG_PL_MEMBER);
}

#[test]
fn test_mutable_union() {
    let input = cdr2::mutable::UnionType::Value1("foo".to_string());
    roundtrip_expected(input, cdr2::mutable::UNION_TYPE_EXPECTED);
}

#[test]
fn test_mutable_unordered() {
    roundtrip_expected(
        cdr2::mixed::MutableMiddle::default(),
        cdr2::mixed::MUTABLE_MIDDLE_EXPECTED,
    );
}

#[test]
fn test_mixed_final() {
    roundtrip_expected(
        cdr2::mixed::TopFinal::default(),
        cdr2::mixed::TOP_FINAL_EXPECTED,
    );
}

#[test]
fn test_mixed_appendable() {
    roundtrip_expected(
        cdr2::mixed::TopAppendable::default(),
        cdr2::mixed::TOP_APPENDABLE_EXPECTED,
    );
}

#[test]
fn test_mixed_mutable() {
    roundtrip_expected(
        cdr2::mixed::TopMutable::default(),
        cdr2::mixed::TOP_MUTABLE_EXPECTED,
    );
}

#[test]
fn test_invalid_dheader_bounds() {
    let data = &[0x40, 0x7a, 0x02, 0x0a];
    let result = intercom_cts::cdr2::from_le_bytes::<cdr2::mutable::SeqType>(data);
    assert!(
        result.is_err(),
        "Should reject DHEADER that exceeds buffer bounds"
    );
}

#[test]
fn test_truncated_emheader() {
    let data = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let result = intercom_cts::cdr2::from_le_bytes::<cdr2::mutable::SeqType>(data);
    assert!(result.is_err(), "Should reject truncated EMHEADER data");
}

#[test]
fn test_type_descriptor_primitives() {
    assert_eq!(bool::TYPE_INFO.name, "bool");
    assert_eq!(i32::TYPE_INFO.name, "i32");
    assert_eq!(u64::TYPE_INFO.name, "u64");
    assert_eq!(String::TYPE_INFO.name, "string");
}

#[test]
fn test_type_descriptor_generic_vec() {
    // Vec<i32> should have i32 as its element type
    assert_eq!(Vec::<i32>::TYPE_INFO.name, "sequence");
    assert_eq!(Vec::<i32>::TYPE_INFO.element_info.unwrap().name, "i32");

    // Vec<u32> should have u32 as its element type
    assert_eq!(Vec::<u32>::TYPE_INFO.element_info.unwrap().name, "u32");

    // Nested Vec<Vec<i32>>
    assert_eq!(
        Vec::<Vec<i32>>::TYPE_INFO.element_info.unwrap().name,
        "sequence",
    );
    assert_eq!(
        Vec::<Vec<i32>>::TYPE_INFO
            .element_info
            .unwrap()
            .element_info
            .unwrap()
            .kind,
        TypeKind::I32,
    );
}

#[test]
fn test_type_descriptor_array() {
    assert_eq!(<[i32; 10]>::TYPE_INFO.element_info.unwrap().name, "i32");
    assert_eq!(<[bool; 5]>::TYPE_INFO.element_info.unwrap().name, "bool");
}

#[test]
fn test_type_descriptor_option() {
    assert_eq!(Option::<i32>::TYPE_INFO.name, "i32");
    assert_eq!(Option::<String>::TYPE_INFO.name, "string");
}

#[test]
fn test_type_descriptor_struct() {
    assert_eq!(
        cdr2::final_::PrimitiveStruct::TYPE_INFO.name,
        "cdr2::final::PrimitiveStruct"
    );
}

#[test]
fn test_nested_appendable() {
    let input = NestedAppendable {
        x: 1,
        inner: InnerAppendable {
            a: 2,
            b: 0x1234_5678,
            c: 3,
        },
        y: 0x4567,
    };
    roundtrip(input);
}

#[test]
fn test_seq_of_appendable() {
    let input = WithSeqOfAppendable {
        prefix: 0x1234,
        elements: vec![
            SeqElement {
                id: 1,
                value: 0x123_4567_89AB_CDEF,
            },
            SeqElement {
                id: 2,
                value: 0x7ED_CBA9_8765_4321,
            },
            SeqElement {
                id: 3,
                value: 0x7AAA_BBBB_CCCC_DDDD,
            },
        ],
        suffix: 0xFF,
    };
    roundtrip(input);
}

#[test]
fn test_mixed_final_appendable_seq() {
    let input = MixedSeq {
        finals: vec![FinalInSeq { a: 1, b: 100 }, FinalInSeq { a: 2, b: 200 }],
        appendables: vec![
            AppendableInSeq { a: 3, b: 300 },
            AppendableInSeq { a: 4, b: 400 },
        ],
    };
    roundtrip(input);
}

#[test]
fn test_optional_alignment_present() {
    let input = OptionalAlignment {
        maybe_byte: Some(42),
        maybe_long: Some(0x123_4567_89AB_CDEF),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_optional_alignment_absent() {
    let input = OptionalAlignment {
        maybe_byte: None,
        maybe_long: None,
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_optional_alignment_partial() {
    let input = OptionalAlignment {
        maybe_byte: Some(42),
        maybe_long: None,
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_deeply_nested_mutable() {
    let input = OuterMutable {
        x: 0x1234,
        inner: InnerMutable {
            a: 5,
            b: 0x7EAD_BEEF_CAFE_BABE,
        },
        y: 77,
    };
    roundtrip(input);
}

#[test]
fn test_string_sequence() {
    let input = StringSequence {
        marker: 0xAA,
        strings: vec!["hello".to_string(), "world".to_string(), "test".to_string()],
        trailer: 0x5678,
    };
    roundtrip(input);
}

#[test]
fn test_complex_map() {
    let mut entries = std::collections::BTreeMap::new();
    entries.insert(
        MapKey { id: 1 },
        MapValue {
            data: 10,
            timestamp: 1_000_000,
        },
    );
    entries.insert(
        MapKey { id: 2 },
        MapValue {
            data: 20,
            timestamp: 2_000_000,
        },
    );

    let input = ComplexMap {
        prefix_seq: vec![1, 2, 3, 4],
        entries,
        suffix: 0xBB,
    };
    roundtrip(input);
}

#[test]
fn test_array_vs_seq() {
    let input = ArrayVsSeq {
        header: 0x11,
        fixed_array: [
            InnerAppendable {
                a: 1,
                b: 10,
                c: 100,
            },
            InnerAppendable {
                a: 2,
                b: 20,
                c: 200,
            },
            InnerAppendable {
                a: 3,
                b: 30,
                c: 250,
            },
        ],
        variable: vec![
            InnerAppendable {
                a: 4,
                b: 40,
                c: 150,
            },
            InnerAppendable {
                a: 5,
                b: 50,
                c: 175,
            },
        ],
    };
    roundtrip(input);
}

#[test]
fn test_optional_seq_present() {
    let input = OptionalSeq {
        maybe_seq: Some(vec![
            InnerAppendable {
                a: 1,
                b: 10,
                c: 100,
            },
            InnerAppendable {
                a: 2,
                b: 20,
                c: 200,
            },
        ]),
        marker: 0x1234_5678,
    };
    roundtrip(input);
}

#[test]
fn test_optional_seq_absent() {
    let input = OptionalSeq {
        maybe_seq: None,
        marker: 0x1234_5678,
    };
    roundtrip(input);
}

#[test]
fn test_long_param_list() {
    let input = LongParamList {
        f1: 1,
        f2: 2,
        f3: 3,
        f4: 4,
        f5: 5.0,
        f6: 6.0,
        f7: true,
        f8: "test".to_string(),
    };
    roundtrip(input);
}

#[test]
fn test_alignment_stress() {
    let input = AlignmentStress {
        b1: 1,
        ll1: 0x1111_1111_1111_1111,
        b2: 2,
        l1: 0x2222_2222,
        b3: 3,
        s1: 0x3333,
        b4: 4,
    };
    roundtrip(input);
}

#[test]
fn test_empty_collections() {
    let input = EmptyCollections {
        empty_seq: vec![],
        missing: None,
        empty_strings: vec![],
    };
    roundtrip(input);
}

#[test]
fn test_optional_nested_present() {
    let input = OptionalNested {
        maybe_inner: Some(InnerAppendable { a: 1, b: 2, c: 3 }),
        maybe_outer: Some(OuterMutable {
            x: 4,
            inner: InnerMutable { a: 5, b: 6 },
            y: 7,
        }),
    };
    roundtrip(input);
}

#[test]
fn test_optional_nested_absent() {
    let input = OptionalNested {
        maybe_inner: None,
        maybe_outer: None,
    };
    roundtrip(input);
}

#[test]
fn test_string_alignment() {
    let input = StringAlignment {
        text: "hello world".to_string(),
        value: 0x123_4567_89AB_CDEF,
    };
    roundtrip(input);
}

#[test]
fn test_string_alignment_short() {
    let input = StringAlignment {
        text: "hi".to_string(),
        value: 0x7ED_CBA9_8765_4321,
    };
    roundtrip(input);
}

#[test]
fn test_nested_sequences() {
    let input = NestedSequences {
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
        marker: 0xCC,
    };
    roundtrip(input);
}

#[test]
fn test_final_outer_with_appendable() {
    let input = FinalOuter {
        x: 1,
        appendable_member: InnerAppendable { a: 2, b: 3, c: 4 },
        y: 5,
    };
    roundtrip(input);
}

#[test]
fn test_large_gap() {
    let input = LargeGap {
        tiny: 1,
        big: 0x7AAA_AAAA_BBBB_BBBB,
        tiny2: 2,
        big2: 0x7CCC_CCCC_DDDD_DDDD,
    };
    roundtrip(input);
}

#[test]
fn test_map_primitive_key() {
    let mut entries = std::collections::BTreeMap::new();
    entries.insert(
        1,
        InnerAppendable {
            a: 10,
            b: 100,
            c: 200,
        },
    );
    entries.insert(
        2,
        InnerAppendable {
            a: 20,
            b: 200,
            c: 250,
        },
    );

    let input = MapPrimitiveKey { entries };
    roundtrip(input);
}

#[test]
fn test_union_byte_val() {
    let input = WithUnion {
        choice: SimpleUnion::ByteVal(42),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_union_long_val() {
    let input = WithUnion {
        choice: SimpleUnion::LongVal(0x123_4567_89AB_CDEF),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_union_str_val() {
    let input = WithUnion {
        choice: SimpleUnion::StrVal("union string".to_string()),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_optional_at_end_present() {
    let input = OptionalAtEnd {
        a: 0x1234_5678,
        b: 0x1234,
        optional_last: Some(0x7ED_CBA9_8765_4321),
    };
    roundtrip(input);
}

#[test]
fn test_optional_at_end_absent() {
    let input = OptionalAtEnd {
        a: 0x1234_5678,
        b: 0x1234,
        optional_last: None,
    };
    roundtrip(input);
}

#[test]
fn test_final_union_byte() {
    let input = WithFinalUnion {
        choice: FinalUnion::ByteVal(42),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_final_union_long() {
    let input = WithFinalUnion {
        choice: FinalUnion::LongVal(0x123_4567_89AB_CDEF),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_final_union_str() {
    let input = WithFinalUnion {
        choice: FinalUnion::StrVal("final union".to_string()),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_mutable_union_byte() {
    let input = WithMutableUnion {
        choice: MutableUnion::ByteVal(42),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_mutable_union_long() {
    let input = WithMutableUnion {
        choice: MutableUnion::LongVal(0x123_4567_89AB_CDEF),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_mutable_union_str() {
    let input = WithMutableUnion {
        choice: MutableUnion::StrVal("mutable union".to_string()),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_final_struct_with_union() {
    let input = FinalWithUnion {
        choice: SimpleUnion::ByteVal(42),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_mutable_struct_with_union() {
    let input = MutableWithUnion {
        choice: SimpleUnion::LongVal(0x123_4567_89AB_CDEF),
        after: 99,
    };
    roundtrip(input);
}

#[test]
fn test_mutable_appendable_evolution() {
    {
        let gen1 = cdr2::mixed::MutableAppendableGen1::default();
        let bytes = intercom_cts::cdr2::to_le_bytes(&gen1).unwrap();
        let _: cdr2::mixed::MutableAppendableGen2 =
            intercom_cts::cdr2::from_le_bytes(&bytes).unwrap();
    }

    {
        let gen2 = cdr2::mixed::MutableAppendableGen2::default();
        let bytes = intercom_cts::cdr2::to_le_bytes(&gen2).unwrap();
        let _: cdr2::mixed::MutableAppendableGen1 =
            intercom_cts::cdr2::from_le_bytes(&bytes).unwrap();
    }
}
