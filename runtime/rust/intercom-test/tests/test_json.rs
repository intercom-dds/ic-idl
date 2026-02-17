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

use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use intercom_cts::decode::{Deserializer, MapDeserializer};
use intercom_cts::encode::{MapSerializer, Serializer};
use intercom_cts::json::{self, Number, Result, Value, to_string};
use intercom_cts::{Marshal, Unmarshal, value};
use intercom_test::*;

fn roundtrip<T>(value: &T) -> Result<T>
where
    T: Marshal + Unmarshal + Default + std::fmt::Debug,
{
    // T -> str -> T -> Value -> T
    let str = json::to_string(value, false)?;
    let value: T = json::from_str(&str)?;
    let value = json::to_value(&value)?;
    let value: T = json::from_value(value)?;

    // T -> pretty str -> Value -> T
    let str = json::to_string(&value, true)?;
    let value: Value = json::from_str(&str)?;
    json::from_value(value)
}

#[allow(clippy::needless_pass_by_value)]
fn from_str<T: ToString>(input: T) -> Result<Value> {
    let str = input.to_string();
    json::from_str(&str)
}

#[test]
fn test_trailing_chars() {
    assert!(from_str("[1]]").is_err());
    assert!(from_str("[1,]").is_err());
    assert!(from_str("[1],").is_err());
    assert!(from_str(r#""abc"""#).is_err());
    assert!(from_str(r#"{"key":123}}"#).is_err());

    // trailing whitespace is fine
    assert!(from_str("[1]    ").is_ok());
}

#[test]
fn test_primitive() {
    let value = from_str("null").unwrap();
    assert_eq!(value, Value::Null);

    let value = from_str("true").unwrap();
    assert_eq!(value, Value::Bool(true));

    let value = from_str("false").unwrap();
    assert_eq!(value, Value::Bool(false));

    let value = from_str(123).unwrap();
    assert_eq!(value.as_u64(), Some(123));

    let value = from_str(-456).unwrap();
    assert_eq!(value.as_i64(), Some(-456));

    let value = from_str(std::f64::consts::PI).unwrap();
    assert_eq!(value.as_f64(), Some(std::f64::consts::PI));

    let value = from_str(r#""foo bar 123""#).unwrap();
    assert_eq!(value.as_str(), Some("foo bar 123"));

    let value = from_str("[1, 2, 3]").unwrap();
    assert_eq!(
        value,
        Value::Array(vec![
            Value::Number(Number::Unsigned(1)),
            Value::Number(Number::Unsigned(2)),
            Value::Number(Number::Unsigned(3)),
        ])
    );

    let value = from_str(r#"[true,1,"foo",1.5]"#).unwrap();
    assert_eq!(
        value,
        Value::Array(vec![
            Value::Bool(true),
            Value::Number(Number::Unsigned(1)),
            Value::String("foo".to_string()),
            Value::Number(Number::Float(1.5)),
        ])
    );

    let value = from_str("[[1],[2],[[3,4]]]").unwrap();
    assert_eq!(
        value,
        Value::Array(vec![
            Value::Array(vec![Value::Number(Number::Unsigned(1))]),
            Value::Array(vec![Value::Number(Number::Unsigned(2))]),
            Value::Array(vec![Value::Array(vec![
                Value::Number(Number::Unsigned(3)),
                Value::Number(Number::Unsigned(4)),
            ]),]),
        ])
    );

    let value = from_str(r#"{"key": "value", "foo": 123, "baz": false}"#).unwrap();
    assert_eq!(
        value,
        Value::Object(BTreeMap::from([
            ("key".to_string(), Value::String("value".to_string())),
            ("foo".to_string(), Value::Number(Number::Unsigned(123))),
            ("baz".to_string(), Value::Bool(false)),
        ]))
    );
}

#[test]
fn test_unterminated() {
    // strings
    assert!(from_str("\"foo").is_err());
    assert!(from_str(r#"{"key":"value}"#).is_err());
    assert!(from_str(r#"{"key:"value}"#).is_err());

    // arrays
    assert!(from_str("[1,2").is_err());
    assert!(from_str("[[1,2]").is_err());
    assert!(from_str(r#"{"key": [1,2}"#).is_err());

    // objects
    assert!(from_str(r#"{"foo":"bar""#).is_err());
    assert!(from_str(r#"{"foo":{}"#).is_err());
}

#[test]
#[allow(clippy::float_cmp)]
fn test_marshal_primitive() {
    macro_rules! bounds {
        ($($type:ty)*) => {
            $(
                assert_eq!(json::from_str::<$type>(&<$type>::MIN.to_string()).unwrap(), <$type>::MIN);
                assert_eq!(json::from_str::<$type>(&<$type>::MAX.to_string()).unwrap(), <$type>::MAX);
                assert_eq!(
                    json::from_str::<$type>(&json::to_string(&<$type>::MIN, false).unwrap()).unwrap(),
                    <$type>::MIN,
                );
                assert_eq!(
                    json::from_str::<$type>(&json::to_string(&<$type>::MAX, false).unwrap()).unwrap(),
                    <$type>::MAX,
                );
            )*
        };
    }

    bounds! {
        i8 u8 i16 u16 i32 u32 i64 u64 // f32 f64 isize usize
    }
    assert!(json::from_str::<bool>("true").unwrap());
    assert!(!json::from_str::<bool>("false").unwrap());
    assert_eq!(json::from_str::<char>("\"c\"").unwrap(), 'c');
    assert_eq!(json::from_str::<String>("\"foobar\"").unwrap(), "foobar");
}

#[test]
fn test_option() {
    let value: Option<i32> = None;
    assert_eq!("null", json::to_string(&value, false).unwrap());
    assert_eq!(value, roundtrip(&value).unwrap());

    let value = Some(123);
    assert_eq!("123", json::to_string(&value, false).unwrap());
    assert_eq!(value, roundtrip(&value).unwrap());
}

#[test]
fn test_nan() {
    assert_eq!("null", json::to_string(&f32::NAN, false).unwrap());
    assert_eq!("null", json::to_string(&f64::NAN, false).unwrap());
}

#[test]
fn test_inf() {
    assert_eq!("null", json::to_string(&f32::INFINITY, false).unwrap());
    assert_eq!("null", json::to_string(&f32::NEG_INFINITY, false).unwrap());
    assert_eq!("null", json::to_string(&f64::INFINITY, false).unwrap());
    assert_eq!("null", json::to_string(&f64::NEG_INFINITY, false).unwrap());
}

#[test]
fn test_char() {
    assert_eq!("\"f\"", json::to_string(&'f', false).unwrap());
    assert_eq!('f', roundtrip(&'f').unwrap());
}

#[test]
fn test_unmarshal_seq() {
    let input = vec![15, 30, 50];
    assert_eq!(to_string(&input, false).unwrap(), "[15,30,50]");
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
    assert_eq!(to_string(&input, false).unwrap(), r#"["foo","bar","baz"]"#);
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_unmarshal_array() {
    let input = [15, 30, 50];
    assert_eq!(to_string(&input, false).unwrap(), "[15,30,50]");
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = ["foo".to_string(), "bar".to_string(), "baz".to_string()];
    assert_eq!(to_string(&input, false).unwrap(), r#"["foo","bar","baz"]"#);
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_unmarshal_map() {
    let input = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
        ("key3".to_string(), "value3".to_string()),
    ]);
    assert_eq!(
        to_string(&input, false).unwrap(),
        r#"{"key1":"value1","key2":"value2","key3":"value3"}"#
    );
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_marshal_map_integer_key() {
    let input = BTreeMap::from([
        (-1, "value1".to_string()),
        (2, "value2".to_string()),
        (3, "value3".to_string()),
    ]);
    assert_eq!(
        to_string(&input, false).unwrap(),
        r#"{"-1":"value1","2":"value2","3":"value3"}"#
    );
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_marshal_map_bool_key() {
    let input = BTreeMap::from([(false, "value1".to_string()), (true, "value2".to_string())]);
    assert_eq!(
        to_string(&input, false).unwrap(),
        r#"{"false":"value1","true":"value2"}"#
    );
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_marshal_map_enum_key() {
    let input = BTreeMap::from([
        (SimpleEnum::Zero, "value1".to_string()),
        (SimpleEnum::One, "value2".to_string()),
        (SimpleEnum::NinetyNine, "value3".to_string()),
    ]);
    assert_eq!(
        to_string(&input, false).unwrap(),
        r#"{"ZERO":"value1","ONE":"value2","NINETY_NINE":"value3"}"#
    );
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_simple_struct() {
    let input = SimpleStruct {
        x: 1,
        y: std::f32::consts::PI,
        z: true,
        str: "abc".to_string(),
        seq: vec![123, 456],
    };
    assert_eq!(
        r#"{"x":1,"y":3.1415927,"z":true,"str":"abc","seq":[123,456]}"#,
        json::to_string(&input, false).unwrap()
    );
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_simple_union() {
    let input = SimpleUnion::Foo(3);
    assert_eq!(
        json::to_string(&input, false).unwrap(),
        r#"{"$discriminator":"ZERO","foo":3}"#,
    );
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = SimpleUnion::Bar(vec![1, 2, 3]);
    assert_eq!(
        json::to_string(&input, false).unwrap(),
        r#"{"$discriminator":"ONE","bar":[1,2,3]}"#,
    );
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = SimpleUnion::Baz("my string".to_string());
    assert_eq!(
        json::to_string(&input, false).unwrap(),
        r#"{"$discriminator":"NINETY_NINE","baz":"my string"}"#,
    );
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_enum() {
    let input = SimpleEnum::Zero;
    assert_eq!("\"ZERO\"", json::to_string(&input, false).unwrap());
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = SimpleEnum::One;
    assert_eq!("\"ONE\"", json::to_string(&input, false).unwrap());
    assert_eq!(input, roundtrip(&input).unwrap());

    let input = SimpleEnum::NinetyNine;
    assert_eq!("\"NINETY_NINE\"", json::to_string(&input, false).unwrap());
    assert_eq!(input, roundtrip(&input).unwrap());
}

#[test]
fn test_enum_integer() {
    assert_eq!(SimpleEnum::Zero, json::from_str("0").unwrap());
    assert_eq!(SimpleEnum::One, json::from_str("1").unwrap());
    assert_eq!(SimpleEnum::NinetyNine, json::from_str("99").unwrap());
}

#[test]
fn test_array_missing_values() {
    assert!(json::from_str::<[u8; 5]>("[0, 128, 255]").is_err());
}

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
fn test_value_macro() {
    assert_eq!(value!(null), Value::Null);
    assert_eq!(value!(true), Value::Bool(true));
    assert_eq!(value!(123_u32), Value::Number(Number::Unsigned(123)));
    assert_eq!(value!(-123), Value::Number(Number::Signed(-123)));
    assert_eq!(value!(1.5), Value::Number(Number::Float(1.5)));
    assert_eq!(value!("foo"), Value::String("foo".to_string()));
    assert_eq!(
        value!([1, 2, 3]),
        Value::Array(vec![
            Value::Number(Number::Signed(1)),
            Value::Number(Number::Signed(2)),
            Value::Number(Number::Signed(3)),
        ])
    );

    let obj = value!({
        "key": "value",
        "number": 3,
        "foo": {
            "bar": true
        },
        "null": null,
        "bool": false,
    });

    assert_eq!(
        obj,
        Value::Object(BTreeMap::from([
            ("key".to_string(), Value::String("value".to_string())),
            ("number".to_string(), Value::Number(Number::Signed(3))),
            (
                "foo".to_string(),
                Value::Object(BTreeMap::from([("bar".to_string(), Value::Bool(true))]))
            ),
            ("null".to_string(), Value::Null),
            ("bool".to_string(), Value::Bool(false)),
        ]))
    );
}

#[test]
fn test_unmarshal_value() {
    let expected = SimpleStruct {
        x: 1,
        y: std::f32::consts::PI,
        z: true,
        str: "abc".to_string(),
        seq: vec![123, 456],
    };

    let json = json::to_value(&expected).unwrap();
    let value: Value = json::from_value(json).unwrap();
    let value: SimpleStruct = json::from_value(value).unwrap();
    assert_eq!(value, expected);
}

// #[test]
// fn test_unmarshal_partial_value() {
//     #[derive(Default, Debug, Marshal, Unmarshal, PartialEq)]
//     struct HasValue {
//         name: String,
//         alive: bool,
//         values: Value,
//     }
//
//     let expected = HasValue {
//         name: "foobar".to_string(),
//         alive: true,
//         values: json::value!({
//             "b": "c",
//             "bool": false,
//         }),
//     };
//
//     let json = json::to_value(&expected).unwrap();
//     let value: HasValue = json::from_value(json).unwrap();
//     assert_eq!(value, expected);
// }

#[test]
fn test_error_from() {
    let err = json::Error::from("foo".parse::<bool>().err().unwrap());
    assert!(!err.to_string().is_empty());

    let err = json::Error::from(u8::try_from(256).err().unwrap());
    assert!(!err.to_string().is_empty());

    let err = json::Error::from("1.2.3".parse::<f32>().err().unwrap());
    assert!(!err.to_string().is_empty());

    let err = json::Error::from("a".parse::<i32>().err().unwrap());
    assert!(!err.to_string().is_empty());
}

#[derive(Default)]
struct KeyedMap<T>(PhantomData<T>);

impl<T: Default + Marshal> Marshal for KeyedMap<T> {
    fn marshal<'a, S>(&self, ar: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer<'a>,
    {
        let mut state = ar.encode_map(1)?;
        state.encode_pair(&T::default(), &123)?;
        state.end()
    }
}

impl<T: Default + Unmarshal> Unmarshal for KeyedMap<T> {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> std::result::Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let mut state = archive.decode_map()?;
        let mut key = T::default();
        let mut value = 0;
        state.decode_pair(&mut key, &mut value)?;
        Ok(())
    }
}

#[test]
fn test_invalid_key_type() {
    fn valid_key<T: Default + Marshal + Unmarshal>() -> bool {
        let res = json::to_string(&KeyedMap::<T>::default(), false);
        if let Ok(str) = res {
            json::from_str::<KeyedMap<T>>(&str).is_ok()
        } else {
            let mut dummy = HashMap::new();
            dummy.insert("foo".to_string(), 123);
            let value = json::to_value(&dummy).unwrap();
            json::from_value::<KeyedMap<T>>(value).is_ok()
        }
    }

    assert!(valid_key::<char>());
    assert!(valid_key::<i8>());
    assert!(valid_key::<u8>());
    assert!(valid_key::<i16>());
    assert!(valid_key::<u16>());
    assert!(valid_key::<i32>());
    assert!(valid_key::<u32>());
    assert!(valid_key::<i64>());
    assert!(valid_key::<u64>());
    assert!(valid_key::<SimpleEnum>());
    assert!(valid_key::<String>());

    assert!(!valid_key::<f32>());
    assert!(!valid_key::<f64>());
    assert!(!valid_key::<[u8; 3]>());
    assert!(!valid_key::<Vec<u8>>());
    assert!(!valid_key::<Option<i32>>());
    assert!(!valid_key::<SimpleStruct>());
    assert!(!valid_key::<SimpleUnion>());
    assert!(!valid_key::<HashMap<i32, i32>>());
}

#[test]
fn test_value_is() {
    let value = Value::Null;
    assert!(value.is_null());

    let value = json::to_value(&123).unwrap();
    assert!(!value.is_null());
    assert!(value.is_integer());

    let value = json::to_value(&"foo").unwrap();
    assert!(value.is_string());

    let value = json::to_value(&[0, 1]).unwrap();
    assert!(value.is_array());

    let value = json::to_value(&1.5).unwrap();
    assert!(value.is_float());
    assert!(!value.is_bool());

    let value = json::to_value(&true).unwrap();
    assert!(value.is_bool());
    assert!(!value.is_integer());
    assert!(!value.is_float());
    assert!(!value.is_array());
    assert!(!value.is_string());
    assert!(!value.is_object());
    assert!(!value.is_null());
}

#[test]
fn test_value_as() {
    let value = Value::Null;
    assert!(value.as_bool().is_none());
    assert!(value.as_f64().is_none());
    assert!(value.as_i64().is_none());
    assert!(value.as_u64().is_none());
    assert!(value.as_str().is_none());
    assert!(value.as_array().is_none());
    assert!(value.as_object().is_none());

    let value = json::to_value(&true).unwrap();
    assert!(value.as_bool().is_some());

    let value = json::to_value(&123).unwrap();
    assert!(value.as_i64().is_some());

    let value = json::to_value(&123_u32).unwrap();
    assert!(value.as_u64().is_some());

    let value = json::to_value(&"foo").unwrap();
    assert!(value.as_str().is_some());

    let value = json::to_value(&[0, 1]).unwrap();
    assert!(value.as_array().is_some());

    let value = json::to_value(&1.5).unwrap();
    assert!(value.as_f64().is_some());

    let value = json::to_value(&SimpleStruct::default()).unwrap();
    assert!(value.as_object().is_some());
}

#[test]
fn test_value_from() {
    assert!(Value::from(true).is_bool());
    assert!(Value::from(1.5).is_float());
    assert!(Value::from(None::<bool>).is_null());
    assert!(Value::from("foo".to_string()).is_string());
    assert!(Value::from("foo").is_string());

    assert!(Value::from(Number::from(0_i8)).is_integer());
    assert!(Value::from(Number::from(0_u8)).is_integer());
    assert!(Value::from(Number::from(0_u16)).is_integer());
    assert!(Value::from(Number::from(0_i16)).is_integer());
    assert!(Value::from(Number::from(0_u32)).is_integer());
    assert!(Value::from(Number::from(0_i32)).is_integer());
    assert!(Value::from(Number::from(0_u64)).is_integer());
    assert!(Value::from(Number::from(0_i64)).is_integer());
    assert!(Value::from(Number::from(0_usize)).is_integer());
    assert!(Value::from(Number::from(0_isize)).is_integer());
    assert!(Value::from(Number::from(0_f32)).is_float());
    assert!(Value::from(Number::from(0_f64)).is_float());
}

#[test]
fn test_encode_nan() {
    let nan = json::to_value(&f64::NAN).unwrap();
    assert!(nan.is_null());
    assert_eq!(nan.to_string(), "null");
}

#[test]
fn test_wstring() {
    let value = WStringStruct {
        my_wchar: 'f',
        my_wstr: "foobar".to_string(),
        my_map: BTreeMap::from([("key".to_string(), "value".to_string())]),
    };
    assert_eq!(value, roundtrip(&value).unwrap());
}

#[test]
fn test_hashmap() {
    let mut value = HashMap::new();
    value.insert('v', "foo".to_string());
    assert_eq!(value, roundtrip(&value).unwrap());
}

#[test]
fn test_string_escaping_serialization() {
    let input = "Hello \"World\"\n\t\\";
    let _serialized = json::to_string(&input, false).unwrap();

    let input_unicode = "😀";
    let serialized_unicode = json::to_string(&input_unicode, false).unwrap();

    assert!(!serialized_unicode.contains("\\u{"));
}

#[test]
fn test_string_parsing_escapes() {
    let json = "\"Hello \\\"World\\\"\\n\\t\\\\\"";
    let parsed = json::from_str::<Value>(json);
    match parsed {
        Ok(Value::String(s)) => {
            assert_eq!(s, "Hello \"World\"\n\t\\");
        }
        _ => assert!(false),
    }

    let json_unicode = "\"\\uD83D\\uDE00\"";
    let parsed_unicode = json::from_str::<Value>(json_unicode);
    match parsed_unicode {
        Ok(Value::String(s)) => {
            assert_eq!(s, "😀");
        }
        _ => {
            assert!(false);
        }
    }
}

#[test]
fn test_number_parsing_scientific() {
    let json = "1.23e5";
    let parsed = json::from_str::<Value>(json);
    match parsed {
        Ok(Value::Number(n)) => match n {
            Number::Float(f) => assert!((f - 123000.0).abs() < 0.001),
            _ => assert!(false),
        },
        _ => assert!(false),
    }

    let json2 = "1e-10";
    let parsed2 = json::from_str::<Value>(json2);
    match parsed2 {
        Ok(Value::Number(n)) => match n {
            Number::Float(f) => assert!((f - 1e-10).abs() < 1e-12),
            _ => assert!(false),
        },
        _ => assert!(false),
    }
}
