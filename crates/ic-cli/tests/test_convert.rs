// Copyright 2024 KONGSBERG
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

use std::collections::HashSet;
use std::path::PathBuf;

use ic_cli::convert::{Convert, ConvertError};

#[test]
fn test_convert_char() {
    let result = char::from_result(&["a".to_string()]).unwrap();
    assert_eq!(result, 'a');

    let result = char::from_result(&["hello".to_string()]).unwrap();
    assert_eq!(result, 'h'); // Takes first char

    let result = char::from_result(&["first".to_string(), "last".to_string()]).unwrap();
    assert_eq!(result, 'l'); // Takes last value's first char
}

#[test]
fn test_convert_string() {
    let result = String::from_result(&["hello".to_string()]).unwrap();
    assert_eq!(result, "hello");

    let result = String::from_result(&["first".to_string(), "last".to_string()]).unwrap();
    assert_eq!(result, "last"); // Takes last value
}

#[test]
fn test_convert_pathbuf() {
    let result = PathBuf::from_result(&["/tmp/file.txt".to_string()]).unwrap();
    assert_eq!(result, PathBuf::from("/tmp/file.txt"));

    let result = PathBuf::from_result(&["file1".to_string(), "file2".to_string()]).unwrap();
    assert_eq!(result, PathBuf::from("file2")); // Takes last value
}

#[test]
fn test_convert_option() {
    let result = Option::<String>::from_result(&["value".to_string()]).unwrap();
    assert_eq!(result, Some("value".to_string()));

    let result = Option::<i32>::from_result(&["42".to_string()]).unwrap();
    assert_eq!(result, Some(42));
}

#[test]
fn test_convert_vec() {
    let result =
        Vec::<String>::from_result(&["a".to_string(), "b".to_string(), "c".to_string()]).unwrap();
    assert_eq!(result, vec!["a", "b", "c"]);

    let result =
        Vec::<i32>::from_result(&["1".to_string(), "2".to_string(), "3".to_string()]).unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_convert_hashset() {
    let result =
        HashSet::<String>::from_result(&["a".to_string(), "b".to_string(), "a".to_string()])
            .unwrap();
    assert_eq!(result.len(), 2); // Duplicate "a" removed
    assert!(result.contains("a"));
    assert!(result.contains("b"));
}

#[test]
fn test_convert_bool() {
    // Test various true values
    assert!(bool::from_result(&["true".to_string()]).unwrap());
    assert!(bool::from_result(&["True".to_string()]).unwrap());
    assert!(bool::from_result(&["TRUE".to_string()]).unwrap());
    assert!(bool::from_result(&["yes".to_string()]).unwrap());
    assert!(bool::from_result(&["YES".to_string()]).unwrap());
    assert!(bool::from_result(&["y".to_string()]).unwrap());
    assert!(bool::from_result(&["Y".to_string()]).unwrap());
    assert!(bool::from_result(&["1".to_string()]).unwrap());

    // Test various false values
    assert!(!bool::from_result(&["false".to_string()]).unwrap());
    assert!(!bool::from_result(&["False".to_string()]).unwrap());
    assert!(!bool::from_result(&["FALSE".to_string()]).unwrap());
    assert!(!bool::from_result(&["no".to_string()]).unwrap());
    assert!(!bool::from_result(&["NO".to_string()]).unwrap());
    assert!(!bool::from_result(&["n".to_string()]).unwrap());
    assert!(!bool::from_result(&["N".to_string()]).unwrap());
    assert!(!bool::from_result(&["0".to_string()]).unwrap());

    // Test invalid value
    let result = bool::from_result(&["invalid".to_string()]);
    assert!(matches!(result, Err(ConvertError::InvalidValue(_))));
}

#[test]
fn test_convert_integers() {
    assert_eq!(u8::from_result(&["255".to_string()]).unwrap(), 255);
    assert_eq!(i8::from_result(&["-128".to_string()]).unwrap(), -128);
    assert_eq!(u16::from_result(&["65535".to_string()]).unwrap(), 65535);
    assert_eq!(i16::from_result(&["-32768".to_string()]).unwrap(), -32768);
    assert_eq!(
        u32::from_result(&["4294967295".to_string()]).unwrap(),
        4_294_967_295
    );
    assert_eq!(
        i32::from_result(&["-2147483648".to_string()]).unwrap(),
        -2_147_483_648
    );
    assert_eq!(
        u64::from_result(&["18446744073709551615".to_string()]).unwrap(),
        18_446_744_073_709_551_615
    );
    assert_eq!(
        i64::from_result(&["-9223372036854775808".to_string()]).unwrap(),
        -9_223_372_036_854_775_808
    );
    assert_eq!(usize::from_result(&["12345".to_string()]).unwrap(), 12345);
    assert_eq!(isize::from_result(&["-12345".to_string()]).unwrap(), -12345);
}

#[test]
fn test_convert_floats() {
    // Use epsilon comparison for floats
    let pi = f32::from_result(&["3.14".to_string()]).unwrap();
    assert!((pi - std::f32::consts::PI).abs() < 0.01);

    let e = f64::from_result(&["2.718281828".to_string()]).unwrap();
    assert!((e - std::f64::consts::E).abs() < 0.000_000_001);

    let neg = f32::from_result(&["-1.5".to_string()]).unwrap();
    assert!((neg - (-1.5)).abs() < f32::EPSILON);

    let exp = f64::from_result(&["1e10".to_string()]).unwrap();
    assert!((exp - 1e10).abs() < f64::EPSILON);
}

#[test]
fn test_convert_error_invalid_integer() {
    let result = i32::from_result(&["not_a_number".to_string()]);
    assert!(matches!(result, Err(ConvertError::InvalidValue(_))));

    let result = u8::from_result(&["256".to_string()]); // Too large for u8
    assert!(matches!(result, Err(ConvertError::InvalidValue(_))));
}

#[test]
fn test_convert_error_invalid_float() {
    let result = f32::from_result(&["not_a_float".to_string()]);
    assert!(matches!(result, Err(ConvertError::InvalidValue(_))));
}

#[test]
fn test_convert_error_display() {
    let err = ConvertError::InvalidValue("test message".to_string());
    let display = format!("{err}");
    assert_eq!(display, "invalid value: test message");
}

#[test]
fn test_convert_vec_with_error() {
    let result =
        Vec::<i32>::from_result(&["1".to_string(), "not_a_number".to_string(), "3".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_convert_option_with_error() {
    let result = Option::<i32>::from_result(&["not_a_number".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_convert_hashset_with_error() {
    let result = HashSet::<i32>::from_result(&["1".to_string(), "not_a_number".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_convert_multiple_values_takes_last() {
    // All single-value types should take the last value when given multiple
    assert_eq!(
        String::from_result(&[
            "first".to_string(),
            "second".to_string(),
            "last".to_string()
        ])
        .unwrap(),
        "last"
    );
    assert_eq!(
        i32::from_result(&["1".to_string(), "2".to_string(), "3".to_string()]).unwrap(),
        3
    );
    assert!(!bool::from_result(&["true".to_string(), "false".to_string()]).unwrap());
}
