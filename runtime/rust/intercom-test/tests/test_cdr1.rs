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

use intercom_cts::cdr1::{from_le_bytes, to_le_bytes};
use intercom_cts::{Marshal, Unmarshal};
use intercom_test::cdr1;

#[track_caller]
#[allow(clippy::needless_pass_by_value)]
fn roundtrip<T, U>(input: T, expected: U)
where
    T: Marshal + Unmarshal + std::fmt::Debug + Default + PartialEq,
    U: AsRef<[u8]>,
{
    let bytes = to_le_bytes(&input).unwrap();
    assert_eq!(bytes.as_slice(), expected.as_ref(), "serialization differs");

    let roundtrip: T = from_le_bytes(&bytes).unwrap();
    assert_eq!(roundtrip, input, "roundtrip failed");
}

// TODO: test final unions, bitmasks, enums, wchar/wstrings
#[test]
fn test_final_primitive_struct() {
    roundtrip(
        cdr1::final_::PrimitiveStruct::default(),
        cdr1::final_::PRIMITIVE_STRUCT_EXPECTED,
    );
}

#[test]
fn test_final_primitive_array() {
    roundtrip(
        cdr1::final_::PrimitiveArray::default(),
        cdr1::final_::PRIMITIVE_ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_array() {
    roundtrip(
        cdr1::final_::ArrayType::default(),
        cdr1::final_::ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_sequence() {
    roundtrip(
        cdr1::final_::SeqType::default(),
        cdr1::final_::SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_primitive_sequence() {
    roundtrip(
        cdr1::final_::PrimitiveSeqType::default(),
        cdr1::final_::PRIMITIVE_SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_map() {
    roundtrip(
        cdr1::final_::MapType::default(),
        cdr1::final_::MAP_TYPE_EXPECTED,
    );
}

#[test]
fn test_final_primitive_map() {
    roundtrip(
        cdr1::final_::PrimitiveMap::default(),
        cdr1::final_::PRIMITIVE_MAP_EXPECTED,
    );
}

#[test]
fn test_final_optional_member() {
    let input = cdr1::final_::OptionalMember {
        is_null: None,
        is_present: Some(123),
    };

    roundtrip(input, cdr1::final_::OPTIONAL_MEMBER_EXPECTED);
}

#[test]
fn test_appendable_primitive_struct() {
    roundtrip(
        cdr1::appendable::PrimitiveStruct::default(),
        cdr1::appendable::PRIMITIVE_STRUCT_EXPECTED,
    );
}

#[test]
fn test_appendable_array() {
    roundtrip(
        cdr1::appendable::ArrayType::default(),
        cdr1::appendable::ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_primitive_array() {
    roundtrip(
        cdr1::appendable::PrimitiveArray::default(),
        cdr1::appendable::PRIMITIVE_ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_sequence() {
    roundtrip(
        cdr1::appendable::SeqType::default(),
        cdr1::appendable::SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_primitive_sequence() {
    roundtrip(
        cdr1::appendable::PrimitiveSeqType::default(),
        cdr1::appendable::PRIMITIVE_SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_map() {
    roundtrip(
        cdr1::appendable::MapType::default(),
        cdr1::appendable::MAP_TYPE_EXPECTED,
    );
}

#[test]
fn test_appendable_primitive_map() {
    roundtrip(
        cdr1::appendable::PrimitiveMap::default(),
        cdr1::appendable::PRIMITIVE_MAP_EXPECTED,
    );
}

#[test]
fn test_appendable_optional_member() {
    let input = cdr1::final_::OptionalMember {
        is_null: None,
        is_present: Some(123),
    };

    roundtrip(input, cdr1::appendable::OPTIONAL_MEMBER_EXPECTED);
}

#[test]
fn test_appendable_optional_long_pl_member() {
    let input = cdr1::appendable::OptionalLongPlMember {
        is_null: None,
        is_present: Some(123),
    };

    roundtrip(input, cdr1::appendable::OPTIONAL_LONG_PL_MEMBER);
}

#[test]
fn test_appendable_evolution() {
    let default = cdr1::appendable::Gen3 {
        value1: 1,
        value2: 2,
        value3: 3,
    };
    {
        let gen1 = cdr1::appendable::Gen1 { value1: 123 };
        let bytes = to_le_bytes(&gen1).unwrap();

        // gen1 -> gen2
        let gen2: cdr1::appendable::Gen2 = from_le_bytes(&bytes).unwrap();
        assert_eq!(gen2.value1, gen1.value1);
        assert_eq!(gen2.value2, default.value2);

        // gen1 -> gen3
        let gen3: cdr1::appendable::Gen3 = from_le_bytes(&bytes).unwrap();
        assert_eq!(gen3.value1, gen1.value1);
        assert_eq!(gen3.value2, default.value2);
        assert_eq!(gen3.value3, default.value3);
    }
    {
        let gen2 = cdr1::appendable::Gen2 {
            value1: 123,
            value2: 456,
        };
        let bytes = to_le_bytes(&gen2).unwrap();

        // gen2 -> gen1
        let gen1: cdr1::appendable::Gen1 = from_le_bytes(&bytes).unwrap();
        assert_eq!(gen1.value1, gen2.value1);

        // gen2 -> gen3
        let gen3: cdr1::appendable::Gen3 = from_le_bytes(&bytes).unwrap();
        assert_eq!(gen3.value1, gen2.value1);
        assert_eq!(gen3.value2, gen2.value2);
        assert_eq!(gen3.value3, default.value3);
    }
    {
        let input = cdr1::appendable::Gen3 {
            value1: 123,
            value2: 456,
            value3: 789,
        };
        let bytes = to_le_bytes(&input).unwrap();

        // gen3 -> gen1
        let gen1: cdr1::appendable::Gen1 = from_le_bytes(&bytes).unwrap();
        assert_eq!(gen1.value1, input.value1);

        // gen3 -> gen2
        let gen2: cdr1::appendable::Gen2 = from_le_bytes(&bytes).unwrap();
        assert_eq!(gen2.value1, input.value1);
        assert_eq!(gen2.value2, input.value2);
    }
    {
        let input = cdr1::appendable::EvolvedMemberGen1 {
            value1: cdr1::appendable::Gen1 { value1: 1 },
            value2: 99,
        };
        let bytes = to_le_bytes(&input).unwrap();
        let res = from_le_bytes::<cdr1::appendable::EvolvedMemberGen3>(&bytes);
        assert!(res.is_err());
    }
}

#[test]
fn test_mutable_primitive_struct() {
    let input = cdr1::mutable::PrimitiveStruct {
        value1: 255,
        value2: 128,
        value3: 128,
        value4: 128,
        value5: 1.5,
        value6: 1.5,
    };
    assert_eq!(
        to_le_bytes(&input).unwrap(),
        cdr1::mutable::PRIMITIVE_STRUCT_EXPECTED,
    );
}

#[test]
fn test_mutable_primitive_array() {
    roundtrip(
        cdr1::mutable::PrimitiveArray::default(),
        cdr1::mutable::PRIMITIVE_ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_array() {
    roundtrip(
        cdr1::mutable::ArrayType::default(),
        cdr1::mutable::ARRAY_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_primitive_sequence() {
    roundtrip(
        cdr1::mutable::PrimitiveSeqType::default(),
        cdr1::mutable::PRIMITIVE_SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_sequence() {
    roundtrip(
        cdr1::mutable::SeqType::default(),
        cdr1::mutable::SEQ_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_primitive_map() {
    roundtrip(
        cdr1::mutable::PrimitiveMapType::default(),
        cdr1::mutable::PRIMITIVE_MAP_EXPECTED,
    );
}

#[test]
fn test_mutable_map() {
    roundtrip(
        cdr1::mutable::MapType::default(),
        cdr1::mutable::MAP_TYPE_EXPECTED,
    );
}

#[test]
fn test_mutable_optional_member() {
    let input = cdr1::mutable::OptionalMember {
        is_null: None,
        is_present: Some(123),
    };

    roundtrip(input, cdr1::mutable::OPTIONAL_MEMBER_EXPECTED);
}

#[test]
fn test_mutable_long_pl() {
    roundtrip(
        cdr1::mutable::LongPlStruct::default(),
        cdr1::mutable::LONG_PL_STRUCT_EXPECTED,
    );
}

#[test]
fn test_mutable_optional_long_pl() {
    let input = cdr1::mutable::OptionalLongPlMember {
        is_null: None,
        is_present: Some(123),
    };

    roundtrip(input, cdr1::mutable::OPTIONAL_LONG_PL_MEMBER);
}

#[test]
fn test_mutable_union() {
    let input = cdr1::mutable::UnionType::Value1("foo".to_string());
    roundtrip(input, cdr1::mutable::UNION_TYPE_EXPECTED);
}

#[test]
fn test_mutable_unordered() {
    roundtrip(
        cdr1::mutable::MutableUnordered::default(),
        cdr1::mutable::MUTABLE_UNORDERED_EXPECTED,
    );

    // Verifies that members can appear out of order, and that we correctly
    // rewind the cursor if we've already advanced past a member.
    let value = from_le_bytes(&cdr1::mutable::MUTABLE_UNORDERED_REV_EXPECTED).unwrap();
    assert_eq!(cdr1::mutable::MutableUnordered::default(), value);
}

#[test]
fn test_mutable_missing_sentinel() {
    let value = from_le_bytes::<cdr1::mutable::MutableUnordered>(
        &cdr1::mutable::MUTABLE_UNORDERED_MISSING_SENTINEL,
    );
    assert!(value.is_err());

    let value =
        from_le_bytes::<cdr1::mutable::UnionType>(&cdr1::mutable::UNION_TYPE_MISSING_SENTINEL);
    assert!(value.is_err());
}

#[test]
fn test_mixed_final() {
    roundtrip(
        cdr1::mixed::TopFinal::default(),
        cdr1::mixed::TOP_FINAL_EXPECTED,
    );
}

#[test]
fn test_mixed_appendable() {
    roundtrip(
        cdr1::mixed::TopAppendable::default(),
        cdr1::mixed::TOP_APPENDABLE_EXPECTED,
    );
}

#[test]
fn test_mixed_mutable() {
    roundtrip(
        cdr1::mixed::TopMutable::default(),
        cdr1::mixed::TOP_MUTABLE_EXPECTED,
    );
}

#[test]
fn test_mixed_mutable_middle() {
    roundtrip(
        cdr1::mixed::MutableMiddle::default(),
        cdr1::mixed::MUTABLE_MIDDLE_EXPECTED,
    );
}

#[test]
fn test_mixed_switcheroo() {
    roundtrip(
        cdr1::mixed::Switcheroo::default(),
        cdr1::mixed::SWITCHEROO_EXPECTED,
    );
}

#[test]
fn test_mutable_appendable_evolution() {
    {
        let gen1 = cdr1::mixed::MutableAppendableGen1::default();
        let bytes = to_le_bytes(&gen1).unwrap();
        let _: cdr1::mixed::MutableAppendableGen2 = from_le_bytes(&bytes).unwrap();
    }

    {
        let gen2 = cdr1::mixed::MutableAppendableGen2::default();
        let bytes = to_le_bytes(&gen2).unwrap();
        let _: cdr1::mixed::MutableAppendableGen1 = from_le_bytes(&bytes).unwrap();
    }
}
