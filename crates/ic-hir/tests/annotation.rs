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

use ic_hir::annotation::*;
use ic_hir::hir::{Ann, AnnArg, Ident, Numeric};

fn make_ann(name: &str, args: Vec<AnnArg>) -> Ann {
    Ann {
        ident: Ident {
            name: name.to_string(),
            span: ic_syntax::Span::default(),
        },
        def_id: None,
        args,
    }
}

fn make_arg(name: Option<&str>, value: Numeric) -> AnnArg {
    AnnArg {
        ident: name.map_or_else(
            || Ident {
                name: "value".to_string(),
                span: ic_syntax::Span::default(),
            },
            |n| Ident {
                name: n.to_string(),
                span: ic_syntax::Span::default(),
            },
        ),
        value,
        ty: None,
    }
}

#[test]
fn test_cts_annotation_error_display() {
    let err = CtsAnnotationError::WrongAnnotationType {
        expected: "range",
        actual: "optional".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "Expected annotation @range but found @optional"
    );

    let err = CtsAnnotationError::DeserializationError("test error".to_string());
    assert_eq!(err.to_string(), "Deserialization error: test error");

    let err = CtsAnnotationError::FieldNotFound("value".to_string());
    assert_eq!(err.to_string(), "Field 'value' not found");

    let err = CtsAnnotationError::TypeConversionError {
        field: "value".to_string(),
        expected: "i32",
    };
    assert_eq!(
        err.to_string(),
        "Field 'value' has wrong type, expected i32"
    );
}

#[test]
fn test_cts_annotation_error_custom() {
    let err: CtsAnnotationError = intercom_cts::error::Error::custom("custom error");
    assert!(matches!(err, CtsAnnotationError::DeserializationError(_)));
}

#[test]
fn test_optional_with_all_numeric_types() {
    // Test that Optional only accepts bool values
    let ann = make_ann(
        "optional",
        vec![make_arg(Some("value"), Numeric::Bool(true))],
    );
    let optional: Optional = ann.unmarshal("optional").unwrap();
    assert!(optional.value);

    // Test with wrong numeric types - these should fail during field decoding
    let ann = make_ann(
        "optional",
        vec![make_arg(Some("value"), Numeric::Int32(42))],
    );
    let result: Result<Optional, _> = ann.unmarshal("optional");
    assert!(result.is_err());

    let ann = make_ann(
        "optional",
        vec![make_arg(Some("value"), Numeric::String("true".to_string()))],
    );
    let result: Result<Optional, _> = ann.unmarshal("optional");
    assert!(result.is_err());
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_range_with_all_numeric_types() {
    // Test with i8
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Int8(-128)),
            make_arg(Some("max"), Numeric::Int8(127)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(-128));
    assert_eq!(range.max, Some(127));

    // Test with i16
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Int16(-32768)),
            make_arg(Some("max"), Numeric::Int16(32767)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(-32768));
    assert_eq!(range.max, Some(32767));

    // Test with i32
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Int32(i32::MIN)),
            make_arg(Some("max"), Numeric::Int32(i32::MAX)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(i64::from(i32::MIN)));
    assert_eq!(range.max, Some(i64::from(i32::MAX)));

    // Test with i64
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Int64(i64::MIN)),
            make_arg(Some("max"), Numeric::Int64(i64::MAX)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(i64::MIN));
    assert_eq!(range.max, Some(i64::MAX));

    // Test with unsigned types
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Octet(0)),
            make_arg(Some("max"), Numeric::Octet(255)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, Some(255));

    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::UInt16(0)),
            make_arg(Some("max"), Numeric::UInt16(65535)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, Some(65535));

    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::UInt32(0)),
            make_arg(Some("max"), Numeric::UInt32(u32::MAX)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, Some(i64::from(u32::MAX)));

    // Test with UInt64 that fits in i64
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::UInt64(0)),
            make_arg(Some("max"), Numeric::UInt64(i64::MAX as u64)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, Some(i64::MAX));

    // Test with UInt64 that doesn't fit in i64 - should fail
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::UInt64(0)),
            make_arg(Some("max"), Numeric::UInt64(u64::MAX)),
        ],
    );
    let result: Result<Range, _> = ann.unmarshal("range");
    assert!(result.is_err());

    // Test with wrong types
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Float(1.5)),
            make_arg(Some("max"), Numeric::Double(10.5)),
        ],
    );
    let result: Result<Range, _> = ann.unmarshal("range");
    assert!(result.is_err());

    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Char('a')),
            make_arg(Some("max"), Numeric::Char('z')),
        ],
    );
    let result: Result<Range, _> = ann.unmarshal("range");
    assert!(result.is_err());
}

#[test]
fn test_default_value_with_wrong_types() {
    // DefaultValue expects a string
    let ann = make_ann(
        "default",
        vec![make_arg(None, Numeric::String("test".to_string()))],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "test");

    // Test with non-string types
    let ann = make_ann("default", vec![make_arg(None, Numeric::Int32(42))]);
    let result: Result<DefaultValue, _> = ann.unmarshal("default");
    assert!(result.is_err());

    let ann = make_ann("default", vec![make_arg(None, Numeric::Bool(true))]);
    let result: Result<DefaultValue, _> = ann.unmarshal("default");
    assert!(result.is_err());

    let ann = make_ann("default", vec![make_arg(None, Numeric::Float(1.234))]);
    let result: Result<DefaultValue, _> = ann.unmarshal("default");
    assert!(result.is_err());
}

#[test]
fn test_mode_annotation_invalid_string() {
    // Test invalid mode string
    let ann = make_ann(
        "mode",
        vec![make_arg(None, Numeric::String("invalid_mode".to_string()))],
    );
    let result: Result<ModeAnnotation, _> = ann.unmarshal("mode");
    assert!(result.is_err());

    // Test with non-string type
    let ann = make_ann("mode", vec![make_arg(None, Numeric::Int32(1))]);
    let result: Result<ModeAnnotation, _> = ann.unmarshal("mode");
    assert!(result.is_err());
}

#[test]
fn test_mode_all_variants() {
    // Test ReadWrite
    let ann = make_ann(
        "mode",
        vec![make_arg(None, Numeric::String("read_write".to_string()))],
    );
    let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
    assert_eq!(mode.value, Mode::ReadWrite);

    // Test ReadOnly
    let ann = make_ann(
        "mode",
        vec![make_arg(None, Numeric::String("read_only".to_string()))],
    );
    let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
    assert_eq!(mode.value, Mode::ReadOnly);

    // Test WriteOnly
    let ann = make_ann(
        "mode",
        vec![make_arg(None, Numeric::String("write_only".to_string()))],
    );
    let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
    assert_eq!(mode.value, Mode::WriteOnly);
}

#[test]
fn test_find_annotation_multiple() {
    let annotations = vec![
        make_ann("optional", vec![]),
        make_ann("range", vec![make_arg(Some("min"), Numeric::Int32(0))]),
        make_ann(
            "default",
            vec![make_arg(None, Numeric::String("test".to_string()))],
        ),
        make_ann(
            "mode",
            vec![make_arg(None, Numeric::String("read_only".to_string()))],
        ),
    ];

    // Find optional
    let optional: Optional = find_annotation(&annotations, "optional").unwrap().unwrap();
    assert!(optional.value); // Default is true

    // Find range
    let range: Range = find_annotation(&annotations, "range").unwrap().unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, None);

    // Find default
    let default: DefaultValue = find_annotation(&annotations, "default").unwrap().unwrap();
    assert_eq!(default.value, "test");

    // Find mode
    let mode: ModeAnnotation = find_annotation(&annotations, "mode").unwrap().unwrap();
    assert_eq!(mode.value, Mode::ReadOnly);

    // Try to find non-existent
    let result: Option<Result<Optional, _>> = find_annotation(&annotations, "nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_empty_annotations() {
    let annotations = vec![];

    let result: Option<Result<Optional, _>> = find_annotation(&annotations, "optional");
    assert!(result.is_none());
}

#[test]
fn test_duplicate_annotations() {
    // If there are duplicates, find_annotation returns the first one
    let annotations = vec![
        make_ann("range", vec![make_arg(Some("min"), Numeric::Int32(0))]),
        make_ann("range", vec![make_arg(Some("min"), Numeric::Int32(10))]),
    ];

    let range: Range = find_annotation(&annotations, "range").unwrap().unwrap();
    assert_eq!(range.min, Some(0)); // First one
}

#[test]
fn test_positional_vs_named_arguments() {
    // Test positional argument (no name)
    let ann = make_ann(
        "default",
        vec![make_arg(None, Numeric::String("positional".to_string()))],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "positional");

    // Test named argument
    let ann = make_ann(
        "default",
        vec![make_arg(
            Some("value"),
            Numeric::String("named".to_string()),
        )],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "named");

    // Test with wrong field name - value is still used if it's not found by name
    let ann = make_ann(
        "default",
        vec![make_arg(
            Some("wrong_name"),
            Numeric::String("wrong".to_string()),
        )],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "wrong"); // Gets the value even with wrong name
}

#[test]
fn test_complex_annotation_scenarios() {
    // Range with only min
    let ann = make_ann("range", vec![make_arg(Some("min"), Numeric::Int32(-10))]);
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(-10));
    assert_eq!(range.max, None);

    // Range with only max
    let ann = make_ann("range", vec![make_arg(Some("max"), Numeric::Int32(100))]);
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, Some(100));

    // Range with reversed order (max first, min second)
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("max"), Numeric::Int32(100)),
            make_arg(Some("min"), Numeric::Int32(0)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, Some(100));
}

#[test]
fn test_edge_case_numeric_values() {
    // Test with edge case values
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Int8(i8::MIN)),
            make_arg(Some("max"), Numeric::Int8(i8::MAX)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(-128));
    assert_eq!(range.max, Some(127));

    // Test with maximum unsigned values
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::UInt32(0)),
            make_arg(Some("max"), Numeric::UInt32(u32::MAX)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    assert_eq!(range.min, Some(0));
    assert_eq!(range.max, Some(4_294_967_295));
}

#[test]
fn test_mixed_argument_types() {
    // Test with mix of positional and named arguments
    let ann = make_ann(
        "range",
        vec![
            make_arg(None, Numeric::Int32(5)), // Positional - should map to "min" if it's first field
            make_arg(Some("max"), Numeric::Int32(10)), // Named
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    // Positional arguments don't work as expected for Range
    assert_eq!(range.min, None);
    assert_eq!(range.max, Some(10));
}

#[test]
fn test_special_string_values() {
    // Test with empty string
    let ann = make_ann(
        "default",
        vec![make_arg(None, Numeric::String(String::new()))],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "");

    // Test with special characters
    let ann = make_ann(
        "default",
        vec![make_arg(
            None,
            Numeric::String("hello\nworld\t!".to_string()),
        )],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "hello\nworld\t!");

    // Test with unicode
    let ann = make_ann(
        "default",
        vec![make_arg(
            None,
            Numeric::String("😀 Unicode 世界".to_string()),
        )],
    );
    let default: DefaultValue = ann.unmarshal("default").unwrap();
    assert_eq!(default.value, "😀 Unicode 世界");
}

#[test]
fn test_char_values() {
    // While Range doesn't support char, we can test the error handling
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Char('a')),
            make_arg(Some("max"), Numeric::Char('z')),
        ],
    );
    let result: Result<Range, _> = ann.unmarshal("range");
    assert!(result.is_err());
}

#[test]
fn test_float_double_values() {
    // Test float/double rejection for Range
    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Float(1.5)),
            make_arg(Some("max"), Numeric::Float(10.5)),
        ],
    );
    let result: Result<Range, _> = ann.unmarshal("range");
    assert!(result.is_err());

    let ann = make_ann(
        "range",
        vec![
            make_arg(Some("min"), Numeric::Double(1.5)),
            make_arg(Some("max"), Numeric::Double(10.5)),
        ],
    );
    let result: Result<Range, _> = ann.unmarshal("range");
    assert!(result.is_err());
}

#[test]
fn test_optional_edge_cases() {
    // Test optional with no arguments - should use default (true)
    let ann = make_ann("optional", vec![]);
    let optional: Optional = ann.unmarshal("optional").unwrap();
    assert!(optional.value);

    // Test with multiple arguments - only first named "value" should be used
    let ann = make_ann(
        "optional",
        vec![
            make_arg(Some("other"), Numeric::Bool(true)),
            make_arg(Some("value"), Numeric::Bool(false)),
            make_arg(Some("value"), Numeric::Bool(true)), // Duplicate - should be ignored
        ],
    );
    let optional: Optional = ann.unmarshal("optional").unwrap();
    assert!(!optional.value); // Should use the first "value" field

    // Test positional bool
    let ann = make_ann("optional", vec![make_arg(None, Numeric::Bool(false))]);
    let optional: Optional = ann.unmarshal("optional").unwrap();
    assert!(!optional.value);
}

#[test]
fn test_mode_default_behavior() {
    // Mode with no arguments - should use default (ReadWrite)
    let ann = make_ann("mode", vec![]);
    let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
    assert_eq!(mode.value, Mode::ReadWrite);

    // Mode with wrong field name - still gets the value
    let ann = make_ann(
        "mode",
        vec![make_arg(
            Some("wrong"),
            Numeric::String("read_only".to_string()),
        )],
    );
    let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
    assert_eq!(mode.value, Mode::ReadOnly); // Gets the value even with wrong name
}

#[test]
fn test_error_propagation() {
    // Test that errors contain useful information
    let ann = make_ann(
        "optional",
        vec![make_arg(Some("value"), Numeric::Int32(42))],
    );
    let result: Result<Optional, _> = ann.unmarshal("optional");
    match result {
        Err(CtsAnnotationError::TypeConversionError { field, expected }) => {
            assert_eq!(field, "value");
            assert_eq!(expected, "bool");
        }
        _ => panic!("Expected TypeConversionError"),
    }

    // Test wrong annotation type
    let ann = make_ann("wrong", vec![]);
    let result: Result<Optional, _> = ann.unmarshal("optional");
    match result {
        Err(CtsAnnotationError::WrongAnnotationType { expected, actual }) => {
            assert_eq!(expected, "optional");
            assert_eq!(actual, "wrong");
        }
        _ => panic!("Expected WrongAnnotationType"),
    }
}

#[test]
fn test_multiple_positional_arguments() {
    // For Range, positional arguments should work if structured correctly
    // First positional goes to first field "min" if it's named "value"
    let ann = make_ann(
        "range",
        vec![
            make_arg(None, Numeric::Int32(10)),
            make_arg(None, Numeric::Int32(20)),
        ],
    );
    let range: Range = ann.unmarshal("range").unwrap();
    // Positional arguments don't work for Range
    assert_eq!(range.min, None);
    assert_eq!(range.max, None);
}
