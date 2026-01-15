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

#![allow(clippy::float_cmp)]

use std::collections::{BTreeMap, HashMap};

use intercom_cts::buf::endian::{Big, Endian, Little};
use intercom_cts::cdr::{
    Error, from_be_bytes, from_bytes_mut, from_le_bytes, to_be_bytes, to_le_bytes,
};
use intercom_cts::encode::Serializer;
use intercom_cts::{Marshal, Unmarshal, json};
use intercom_test::*;

fn roundtrip<T>(value: &T) -> Result<T, Error>
where
    T: Marshal + Unmarshal + Default,
{
    let bytes = to_be_bytes(value)?;
    let value: T = from_be_bytes(bytes.as_slice())?;
    let bytes = to_le_bytes(&value)?;
    from_le_bytes::<T>(bytes.as_slice())
}

#[test]
fn test_roundtrip_primitives() {
    macro_rules! roundtrip {
        ($($type:ty)*) => {
            $(
                assert_eq!(<$type>::MIN, roundtrip(&<$type>::MIN).unwrap());
                assert_eq!(<$type>::MAX, roundtrip(&<$type>::MAX).unwrap());
            )*
        };
    }

    roundtrip! {
        i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 isize usize
    }
}

#[test]
fn test_roundtrip_complex() {
    let vec = vec![1, 2, 3, 4];
    assert_eq!(vec, roundtrip(&vec).unwrap());

    let map = HashMap::from([(0, 1), (1, 2), (2, 3)]);
    assert_eq!(map, roundtrip(&map).unwrap());
}

#[test]
fn test_invalid_char() {
    let value: char = '🤠';
    assert_eq!(to_le_bytes(&value), Err(Error::InvalidChar));
    assert_eq!(to_be_bytes(&value), Err(Error::InvalidChar));

    let value = char::from_u32(256).unwrap();
    assert_eq!(to_le_bytes(&value), Err(Error::InvalidChar));
    assert_eq!(to_be_bytes(&value), Err(Error::InvalidChar));

    let value = WCharStruct { my_wchar: '🤠' };
    assert_eq!(to_le_bytes(&value), Err(Error::InvalidChar));
    assert_eq!(to_be_bytes(&value), Err(Error::InvalidChar));
}

#[test]
fn test_float() {
    let input = vec![0xdb, 0xf, 0x49, 0x40];
    let expected = std::f32::consts::PI;
    assert_eq!(expected, from_le_bytes(input.as_slice()).unwrap());

    let input = vec![0xdb, 0xf, 0x49, 0xc0];
    let expected = -std::f32::consts::PI;
    assert_eq!(expected, from_le_bytes(input.as_slice()).unwrap());
}

#[test]
fn test_double() {
    let input = vec![0x18, 0x2d, 0x44, 0x54, 0xfb, 0x21, 0x09, 0x40];
    let expected = std::f64::consts::PI;
    assert_eq!(expected, from_le_bytes(input.as_slice()).unwrap());

    let input = vec![0x18, 0x2d, 0x44, 0x54, 0xfb, 0x21, 0x09, 0xc0];
    let expected = -std::f64::consts::PI;
    assert_eq!(expected, from_le_bytes(input.as_slice()).unwrap());
}

#[test]
fn test_option() {
    let input = [0x01, 0x00, 0x00, 0x00, 0x15, 0xcd, 0x5b, 0x07];
    let expected = Some(123_456_789);
    assert_eq!(expected, from_le_bytes(input.as_slice()).unwrap());
    assert_eq!(expected, roundtrip(&expected).unwrap());

    let input = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let expected: Option<u32> = None;
    assert_eq!(expected, from_le_bytes(input.as_slice()).unwrap());
    assert_eq!(expected, roundtrip(&expected).unwrap());
}

#[test]
fn test_seq() {
    let input = vec![0, 0, 0, 0];
    let expected: Vec<i32> = vec![];
    assert_eq!(
        expected,
        from_le_bytes::<Vec<i32>>(input.as_slice()).unwrap()
    );

    let input = [
        0x04, 0x00, 0x00, 0x00, // length
        0x01, 0x00, 0x00, 0x00, // elem 1
        0x02, 0x00, 0x00, 0x00, // elem 2
        0x03, 0x00, 0x00, 0x00, // elem 3
        0x04, 0x00, 0x00, 0x00, // elem 4
    ];
    let expected = vec![1, 2, 3, 4];
    assert_eq!(
        expected,
        from_le_bytes::<Vec<i32>>(input.as_slice()).unwrap()
    );
    assert_eq!(expected, roundtrip(&expected).unwrap());
}

#[test]
fn test_array() {
    let input = [
        0x01, 0x00, 0x00, 0x00, // elem 1
        0x02, 0x00, 0x00, 0x00, // elem 2
        0x03, 0x00, 0x00, 0x00, // elem 3
        0x04, 0x00, 0x00, 0x00, // elem 4
    ];
    let expected: [u32; 4] = [1, 2, 3, 4];
    assert_eq!(expected, from_le_bytes::<[u32; 4]>(&input).unwrap());
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_array_too_short() {
    let too_short: [u32; 2] = [1, 2];
    let bytes = to_le_bytes(&too_short).unwrap();
    assert!(from_le_bytes::<[u32; 4]>(&bytes).is_err());
}

#[test]
fn test_str() {
    let input = vec![0, 0, 0, 0];
    let expected = String::new();
    assert_eq!(expected, from_le_bytes::<String>(input.as_slice()).unwrap());
    assert_eq!(expected, roundtrip(&expected).unwrap());

    let input = [
        0x04, 0x00, 0x00, 0x00, // length
        0x61, // 'a'
        0x62, // 'b'
        0x63, // 'c'
        0x00, // '\0'
    ];
    let expected = "abc".to_string();
    assert_eq!(expected, from_le_bytes::<String>(input.as_slice()).unwrap());
    assert_eq!(expected, roundtrip(&expected).unwrap());
}

#[test]
fn test_map() {
    let input = vec![0, 0, 0, 0];
    let expected = HashMap::<u8, u8>::new();
    assert_eq!(
        expected,
        from_le_bytes::<HashMap::<u8, u8>>(input.as_slice()).unwrap()
    );

    let input = [
        0x03, 0x00, 0x00, 0x00, // len
        0x00, 0x01, // (0, 1)
        0x01, 0x02, // (1, 2)
        0x02, 0x03, // (2, 3)
    ];
    let expected = HashMap::from([(0, 1), (1, 2), (2, 3)]);
    assert_eq!(
        expected,
        from_le_bytes::<HashMap::<u8, u8>>(input.as_slice()).unwrap()
    );
    assert_eq!(expected, roundtrip(&expected).unwrap());
}

#[test]
fn test_map_complex() {
    let input = vec![0, 0, 0, 0];
    let expected = HashMap::<String, String>::new();
    assert_eq!(
        expected,
        from_le_bytes::<HashMap::<String, String>>(input.as_slice()).unwrap()
    );

    let input = [
        0x02, 0x00, 0x00, 0x00, // map len
        0x05, 0x00, 0x00, 0x00, // string len
        0x6b, 0x65, 0x79, 0x31, 0x00, // 'key1'
        0x00, 0x00, 0x00, // padding
        0x07, 0x00, 0x00, 0x00, // string len
        0x76, 0x61, 0x6c, 0x75, 0x65, 0x31, 0x00, // 'value1'
        0x00, // padding
        0x05, 0x00, 0x00, 0x00, // string len
        0x6b, 0x65, 0x79, 0x32, 0x00, // 'key2'
        0x00, 0x00, 0x00, // padding
        0x07, 0x00, 0x00, 0x00, // string len
        0x76, 0x61, 0x6c, 0x75, 0x65, 0x32, 0x00, // 'value2'
    ];
    let expected = HashMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);
    assert_eq!(
        expected,
        from_le_bytes::<HashMap::<String, String>>(input.as_slice()).unwrap()
    );
    assert_eq!(expected, roundtrip(&expected).unwrap());
}

#[test]
fn test_simple_struct() {
    let input = SimpleStruct {
        x: 1,
        y: std::f32::consts::PI,
        z: true,
        str: "abc".to_string(),
        seq: vec![1, 2, 3],
    };
    let expected = vec![
        0x01, 0x00, 0x00, 0x00, // x
        0xdb, 0xf, 0x49, 0x40, // y
        0x01, // z
        0x00, 0x00, 0x00, // alignment
        0x04, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x00, // str
        0x03, 0x00, 0x00, 0x00, // vec len
        0x01, 0x00, 0x00, 0x00, // vec[0]
        0x02, 0x00, 0x00, 0x00, // vec[1]
        0x03, 0x00, 0x00, 0x00, // vec[2]
    ];
    assert_eq!(expected, to_le_bytes(&input).unwrap());
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_simple_union() {
    let input = SimpleUnion::Foo(333);
    let expected = [0x00, 0x00, 0x00, 0x00, 0x4d, 0x01, 0x00, 0x00];
    assert_eq!(to_le_bytes(&input).unwrap(), expected);
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = SimpleUnion::Bar(vec![1, 2, 3]);
    let expected = [
        0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x00, 0x03, 0x00, 0x00, 0x00,
    ];
    assert_eq!(to_le_bytes(&input).unwrap(), expected);
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = SimpleUnion::Baz("abc".to_string());
    let expected = [
        0x63, 0x00, 0x00, 0x00, // 99
        0x04, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x00, // "abc"
    ];
    assert_eq!(to_le_bytes(&input).unwrap(), expected);
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_enum() {
    let value = SimpleEnum::NinetyNine;
    let bytes = to_le_bytes(&value).unwrap();
    assert_eq!(bytes, [0x63, 0x00, 0x00, 0x00]);
    assert_eq!(
        std::mem::discriminant(&value),
        std::mem::discriminant(&roundtrip(&value).unwrap())
    );
}

// TODO: re-enable
// #[test]
// fn test_unions() {
//     let some_data = SomeData {
//         str: "some string".into(),
//         data: vec![1, 2, 3, 4],
//     };
//
//     let my_data = ComplexUnion::MyData(some_data.clone());
//     assert_eq!(my_data.disc(), 0);
//     assert_eq!(my_data, roundtrip(&my_data).unwrap());
//
//     let my_str = ComplexUnion::MyStr("abcdef123".into());
//     assert_eq!(my_str.disc(), 1);
//     assert_eq!(my_str, roundtrip(&my_str).unwrap());
//
//     let my_int = ComplexUnion::MyInt(123);
//     assert_eq!(my_int.disc(), 2);
//     assert_eq!(my_int, roundtrip(&my_int).unwrap());
//
//     let my_other_data = ComplexUnion::MyOtherData(SomeOtherData {
//         id: 123,
//         data: some_data,
//     });
//     assert_eq!(my_other_data.disc(), 4);
//     assert_eq!(my_other_data, roundtrip(&my_other_data).unwrap());
//
//     let my_struct = WrapsComplexUnion {
//         name: "name".into(),
//         coords: 1.5,
//         data: my_other_data,
//     };
//     assert_eq!(my_struct, roundtrip(&my_struct).unwrap());
// }

// TODO: re-enable
// #[test]
// fn test_recursive_union() {
//     let empty = RecursiveUnion::RecursiveUnion(WrapsRecursiveUnion {
//         data: Box::default(),
//     });
//     assert_eq!(empty, roundtrip(&empty).unwrap());
//
//     let recursive = RecursiveUnion::RecursiveUnion(WrapsRecursiveUnion {
//         data: Box::new(empty),
//     });
//     assert_eq!(recursive, roundtrip(&recursive).unwrap());
// }

#[test]
fn test_wstring_union() {
    let value = WStringStruct {
        my_wchar: 'f',
        my_wstr: "foobar".to_string(),
        my_map: BTreeMap::from([("key".to_string(), "value".to_string())]),
    };
    assert_eq!(value, roundtrip(&value).unwrap());

    let value = WStringUnion::MyWchar('c');
    assert_eq!(value, roundtrip(&value).unwrap());

    let value = WStringUnion::MyWstr("foo".to_string());
    assert_eq!(value, roundtrip(&value).unwrap());
}

// TODO: re-enable
// #[test]
// fn test_any_object() {
//     let val = CorbaKw {
//         any_value: (),
//         obj_value: (),
//         str_value: "foo".to_string(),
//     };
//     assert_eq!(val, roundtrip(&val).unwrap());
// }

#[test]
fn test_error_display() {
    let err = Error::Eof;
    assert_eq!(format!("{err}"), err.to_string());

    let err = Error::InvalidUtf8;
    assert_eq!(format!("{err}"), err.to_string());

    let err = Error::InvalidChar;
    assert_eq!(format!("{err}"), err.to_string());

    let err = Error::InvalidLen;
    assert_eq!(format!("{err}"), err.to_string());

    let err = Error::Unknown("foo".to_string());
    assert_eq!(format!("{err}"), "foo");
}

#[test]
fn test_endian() {
    {
        let mut buf = [0u8; size_of::<u8>()];
        Little::write_u8(u8::MAX, &mut buf);
        assert_eq!(buf[0], u8::MAX);

        let mut buf = [0u8; size_of::<u16>()];
        Little::write_u16(u16::MAX, &mut buf);
        assert_eq!(buf, u16::MAX.to_be_bytes());

        let mut buf = [0u8; size_of::<u32>()];
        Little::write_u32(u32::MAX, &mut buf);
        assert_eq!(buf, u32::MAX.to_be_bytes());

        let mut buf = [0u8; size_of::<u64>()];
        Little::write_u64(u64::MAX, &mut buf);
        assert_eq!(buf, u64::MAX.to_be_bytes());
    }
    {
        let mut buf = [0u8; size_of::<u8>()];
        Big::write_u8(u8::MAX, &mut buf);
        assert_eq!(buf[0], u8::MAX);

        let mut buf = [0u8; size_of::<u16>()];
        Big::write_u16(u16::MAX, &mut buf);
        assert_eq!(buf, u16::MAX.to_be_bytes());

        let mut buf = [0u8; size_of::<u32>()];
        Big::write_u32(u32::MAX, &mut buf);
        assert_eq!(buf, u32::MAX.to_be_bytes());

        let mut buf = [0u8; size_of::<u64>()];
        Big::write_u64(u64::MAX, &mut buf);
        assert_eq!(buf, u64::MAX.to_be_bytes());
    }
}

#[test]
fn test_peek() {
    let value = SimpleStruct::default();
    let bytes = to_le_bytes(&value).unwrap();

    // Deserialization of a `json::Value` requires the serializer to support
    // `peek`, which CDR does not.
    let res = from_le_bytes::<json::Value>(&bytes);
    assert!(res.is_err());
}

#[test]
fn test_broken_container() {
    struct BrokenContainer;

    impl Marshal for BrokenContainer {
        fn marshal<'a, S>(&self, archive: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer<'a>,
        {
            archive
                .encode_sequence(usize::MAX)
                .and_then(intercom_cts::encode::SeqSerializer::end)
        }
    }

    assert!(to_le_bytes(&BrokenContainer).is_err());
}

#[test]
fn test_invalid_seq_len() {
    let bytes = u32::MAX.to_le_bytes();
    assert_eq!(from_le_bytes::<String>(&bytes), Err(Error::InvalidLen));
}

// TODO: re-enable
#[test]
#[ignore]
fn test_invalid_utf8() {
    let bytes = [0x03, 0x00, 0x00, 0x00, 0xE2, 0x80, 0xBF, 0x00];
    assert_eq!(from_le_bytes::<String>(&bytes), Err(Error::InvalidUtf8));

    let bytes = [0x02, 0x00, 0x00, 0x00, 0x00, 0xD8, 0x00, 0x00];
    assert_eq!(
        from_le_bytes::<OnlyWString>(&bytes),
        Err(Error::InvalidUtf8),
    );
}

#[test]
fn test_mut_ref() {
    let bytes = to_le_bytes("foo").unwrap();
    let mut buf = String::new();
    let mut str = &mut buf;

    from_bytes_mut::<_, Little>(&bytes, &mut str).unwrap();
    assert_eq!(buf, "foo");
}
