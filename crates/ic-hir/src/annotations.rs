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

//! Type-safe annotation downcasting system for HIR annotations.
//!
//! This module provides a trait-based system for converting HIR annotations
//! to strongly-typed Rust structures, enabling type-safe access to annotation
//! values.

use std::collections::HashMap;

use crate::hir::{Ann, Numeric};

/// Error type for annotation downcasting
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationError {
    /// The annotation name doesn't match the expected type
    WrongAnnotationType {
        expected: &'static str,
        actual: String,
    },
    /// Required argument is missing
    MissingArgument { name: &'static str },
    /// Argument has wrong type
    InvalidArgumentType {
        name: &'static str,
        expected: &'static str,
    },
    /// Unexpected argument provided
    UnexpectedArgument { name: String },
    /// Value is out of valid range
    ValueOutOfRange { name: &'static str, value: String },
}

impl std::fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongAnnotationType { expected, actual } => {
                write!(f, "Expected annotation @{expected} but found @{actual}")
            }
            Self::MissingArgument { name } => {
                write!(f, "Missing required argument '{name}'")
            }
            Self::InvalidArgumentType { name, expected } => {
                write!(f, "Argument '{name}' has wrong type, expected {expected}")
            }
            Self::UnexpectedArgument { name } => {
                write!(f, "Unexpected argument '{name}'")
            }
            Self::ValueOutOfRange { name, value } => {
                write!(f, "Argument '{name}' value {value} is out of range")
            }
        }
    }
}

impl std::error::Error for AnnotationError {}

/// Trait for strongly-typed annotation representations
pub trait Annotation: Sized {
    /// The name of the annotation (without @ prefix)
    const NAME: &'static str;

    /// Convert from HIR annotation to typed representation
    ///
    /// # Errors
    ///
    /// Returns `AnnotationError` if:
    /// - The annotation name doesn't match the expected type
    /// - Required arguments are missing
    /// - Arguments have wrong types
    /// - Values are out of valid range
    fn from_hir(ann: &Ann) -> Result<Self, AnnotationError>;
}

/// Helper to extract arguments from an annotation
fn extract_args(ann: &Ann) -> HashMap<String, &Numeric> {
    let mut args = HashMap::new();
    for (idx, arg) in ann.args.iter().enumerate() {
        let key = arg
            .ident
            .as_ref()
            .map_or_else(|| idx.to_string(), |id| id.name.clone());
        args.insert(key, &arg.value);
    }
    args
}

/// Helper to get a boolean argument with default
#[allow(dead_code)]
fn get_bool_arg(
    args: &HashMap<String, &Numeric>,
    name: &'static str,
    default: bool,
) -> Result<bool, AnnotationError> {
    match args.get(name) {
        None => Ok(default),
        Some(Numeric::Bool(b)) => Ok(*b),
        Some(_) => Err(AnnotationError::InvalidArgumentType {
            name,
            expected: "boolean",
        }),
    }
}

/// Helper to get an optional numeric argument
fn get_numeric_arg<T>(
    args: &HashMap<String, &Numeric>,
    name: &'static str,
) -> Result<Option<T>, AnnotationError>
where
    T: TryFrom<i64>,
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
{
    match args.get(name) {
        None => Ok(None),
        Some(num) => {
            let value = match num {
                Numeric::Int8(v) => T::try_from(i64::from(*v)).ok(),
                Numeric::Int16(v) => T::try_from(i64::from(*v)).ok(),
                Numeric::Int32(v) => T::try_from(i64::from(*v)).ok(),
                Numeric::Int64(v) => T::try_from(*v).ok(),
                Numeric::Octet(v) => T::try_from(i64::from(*v)).ok(),
                Numeric::UInt16(v) => T::try_from(i64::from(*v)).ok(),
                Numeric::UInt32(v) => T::try_from(i64::from(*v)).ok(),
                Numeric::UInt64(v) => i64::try_from(*v).ok().and_then(|i| T::try_from(i).ok()),
                _ => None,
            };

            value
                .ok_or(AnnotationError::InvalidArgumentType {
                    name,
                    expected: "numeric",
                })
                .map(Some)
        }
    }
}

/// Helper to get a required string argument
#[allow(dead_code)]
fn get_string_arg<'a>(
    args: &'a HashMap<String, &'a Numeric>,
    name: &'static str,
) -> Result<&'a str, AnnotationError> {
    match args.get(name) {
        None => Err(AnnotationError::MissingArgument { name }),
        Some(Numeric::String(s)) => Ok(s),
        Some(_) => Err(AnnotationError::InvalidArgumentType {
            name,
            expected: "string",
        }),
    }
}

/// The @optional annotation
#[derive(Debug, Clone, PartialEq)]
pub struct Optional {
    /// Whether the member is optional (default: true)
    pub value: bool,
}

impl Annotation for Optional {
    const NAME: &'static str = "optional";

    fn from_hir(ann: &Ann) -> Result<Self, AnnotationError> {
        if ann.ident.name != Self::NAME {
            return Err(AnnotationError::WrongAnnotationType {
                expected: Self::NAME,
                actual: ann.ident.name.clone(),
            });
        }

        let args = extract_args(ann);

        // Check for unexpected arguments
        for key in args.keys() {
            if key != "value" && key != "0" {
                return Err(AnnotationError::UnexpectedArgument { name: key.clone() });
            }
        }

        // Get value argument (can be named "value" or positional at index 0)
        let value = if let Some(val) = args.get("value") {
            match val {
                Numeric::Bool(b) => *b,
                _ => {
                    return Err(AnnotationError::InvalidArgumentType {
                        name: "value",
                        expected: "boolean",
                    });
                }
            }
        } else if let Some(val) = args.get("0") {
            match val {
                Numeric::Bool(b) => *b,
                _ => {
                    return Err(AnnotationError::InvalidArgumentType {
                        name: "value",
                        expected: "boolean",
                    });
                }
            }
        } else {
            true // default value
        };

        Ok(Self { value })
    }
}

/// The @range annotation
#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    /// Minimum value (inclusive)
    pub min: Option<i64>,
    /// Maximum value (inclusive)
    pub max: Option<i64>,
}

impl Annotation for Range {
    const NAME: &'static str = "range";

    fn from_hir(ann: &Ann) -> Result<Self, AnnotationError> {
        if ann.ident.name != Self::NAME {
            return Err(AnnotationError::WrongAnnotationType {
                expected: Self::NAME,
                actual: ann.ident.name.clone(),
            });
        }

        let args = extract_args(ann);

        // Check for unexpected arguments
        for key in args.keys() {
            if key != "min" && key != "max" {
                return Err(AnnotationError::UnexpectedArgument { name: key.clone() });
            }
        }

        let min: Option<i64> = get_numeric_arg(&args, "min")?;
        let max: Option<i64> = get_numeric_arg(&args, "max")?;

        // Validate range
        if let (Some(min_val), Some(max_val)) = (min, max) {
            if min_val > max_val {
                return Err(AnnotationError::ValueOutOfRange {
                    name: "range",
                    value: format!("min {min_val} > max {max_val}"),
                });
            }
        }

        Ok(Self { min, max })
    }
}

/// The @default annotation
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultValue {
    /// The default value
    pub value: Numeric,
}

impl Annotation for DefaultValue {
    const NAME: &'static str = "default";

    fn from_hir(ann: &Ann) -> Result<Self, AnnotationError> {
        if ann.ident.name != Self::NAME {
            return Err(AnnotationError::WrongAnnotationType {
                expected: Self::NAME,
                actual: ann.ident.name.clone(),
            });
        }

        let args = extract_args(ann);

        // Check for unexpected arguments
        for key in args.keys() {
            if key != "value" && key != "0" {
                return Err(AnnotationError::UnexpectedArgument { name: key.clone() });
            }
        }

        // Get value argument (can be named "value" or positional at index 0)
        let value = args
            .get("value")
            .or_else(|| args.get("0"))
            .ok_or(AnnotationError::MissingArgument { name: "value" })?;

        Ok(Self {
            value: (*value).clone(),
        })
    }
}

/// Extension trait for Ann to provide downcasting
pub trait AnnExt {
    /// Try to downcast this annotation to a specific type
    ///
    /// # Errors
    ///
    /// Returns `AnnotationError` if the downcast fails
    fn downcast<T: Annotation>(&self) -> Result<T, AnnotationError>;

    /// Check if this annotation is of a specific type
    fn is<T: Annotation>(&self) -> bool;
}

impl AnnExt for Ann {
    fn downcast<T: Annotation>(&self) -> Result<T, AnnotationError> {
        T::from_hir(self)
    }

    fn is<T: Annotation>(&self) -> bool {
        self.ident.name == T::NAME
    }
}

/// Helper function to find and downcast an annotation from a list
#[must_use]
pub fn find_annotation<T: Annotation>(annotations: &[Ann]) -> Option<Result<T, AnnotationError>> {
    annotations
        .iter()
        .find(|ann| ann.is::<T>())
        .map(AnnExt::downcast::<T>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{AnnArg, DefId, Ident};

    fn make_ann(name: &str, args: Vec<AnnArg>) -> Ann {
        Ann {
            ident: Ident {
                name: name.to_string(),
                span: ic_syntax::Span::default(),
            },
            def_id: DefId::_do_not_use(),
            args,
        }
    }

    fn make_arg(name: Option<&str>, value: Numeric) -> AnnArg {
        AnnArg {
            ident: name.map(|n| Ident {
                name: n.to_string(),
                span: ic_syntax::Span::default(),
            }),
            value,
        }
    }

    #[test]
    fn test_optional_default() {
        let ann = make_ann("optional", vec![]);
        let optional = ann.downcast::<Optional>().unwrap();
        assert!(optional.value);
    }

    #[test]
    fn test_optional_explicit() {
        let ann = make_ann(
            "optional",
            vec![make_arg(Some("value"), Numeric::Bool(false))],
        );
        let optional = ann.downcast::<Optional>().unwrap();
        assert!(!optional.value);
    }

    #[test]
    fn test_optional_positional() {
        let ann = make_ann("optional", vec![make_arg(None, Numeric::Bool(false))]);
        let optional = ann.downcast::<Optional>().unwrap();
        assert!(!optional.value);
    }

    #[test]
    fn test_range() {
        let ann = make_ann(
            "range",
            vec![
                make_arg(Some("min"), Numeric::Int32(0)),
                make_arg(Some("max"), Numeric::Int32(100)),
            ],
        );
        let range = ann.downcast::<Range>().unwrap();
        assert_eq!(range.min, Some(0));
        assert_eq!(range.max, Some(100));
    }

    #[test]
    fn test_range_invalid() {
        let ann = make_ann(
            "range",
            vec![
                make_arg(Some("min"), Numeric::Int32(100)),
                make_arg(Some("max"), Numeric::Int32(0)),
            ],
        );
        let err = ann.downcast::<Range>().unwrap_err();
        assert!(matches!(err, AnnotationError::ValueOutOfRange { .. }));
    }

    #[test]
    fn test_default() {
        let ann = make_ann(
            "default",
            vec![make_arg(
                Some("value"),
                Numeric::String("hello".to_string()),
            )],
        );
        let default = ann.downcast::<DefaultValue>().unwrap();
        assert_eq!(default.value, Numeric::String("hello".to_string()));
    }

    #[test]
    fn test_wrong_annotation_type() {
        let ann = make_ann("optional", vec![]);
        let err = ann.downcast::<Range>().unwrap_err();
        assert!(matches!(err, AnnotationError::WrongAnnotationType { .. }));
    }

    #[test]
    fn test_find_annotation() {
        let annotations = vec![
            make_ann(
                "optional",
                vec![make_arg(Some("value"), Numeric::Bool(true))],
            ),
            make_ann(
                "range",
                vec![
                    make_arg(Some("min"), Numeric::Int32(0)),
                    make_arg(Some("max"), Numeric::Int32(10)),
                ],
            ),
        ];

        let range = find_annotation::<Range>(&annotations).unwrap().unwrap();
        assert_eq!(range.min, Some(0));
        assert_eq!(range.max, Some(10));

        assert!(find_annotation::<DefaultValue>(&annotations).is_none());
    }
}
