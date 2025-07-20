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

//! Annotation system using intercom-cts serialization framework.
//!
//! This module provides an alternative implementation of the annotation system
//! that leverages the intercom-cts Marshal/Unmarshal traits for serialization.

use std::collections::HashMap;

use intercom_cts::Unmarshal;
use intercom_cts::decode::Deserializer;
use intercom_cts::error::Error as CtsError;

use crate::hir::{Ann, Numeric};

/// Error type for CTS-based annotation operations
#[derive(Debug, Clone)]
pub enum CtsAnnotationError {
    /// Annotation name mismatch
    WrongAnnotationType {
        expected: &'static str,
        actual: String,
    },
    /// Deserialization error  
    DeserializationError(String),
    /// Field not found
    FieldNotFound(String),
    /// Type conversion error
    TypeConversionError {
        field: String,
        expected: &'static str,
    },
}

impl std::fmt::Display for CtsAnnotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongAnnotationType { expected, actual } => {
                write!(f, "Expected annotation @{expected} but found @{actual}")
            }
            Self::DeserializationError(msg) => write!(f, "Deserialization error: {msg}"),
            Self::FieldNotFound(name) => write!(f, "Field '{name}' not found"),
            Self::TypeConversionError { field, expected } => {
                write!(f, "Field '{field}' has wrong type, expected {expected}")
            }
        }
    }
}

impl std::error::Error for CtsAnnotationError {}

impl CtsError for CtsAnnotationError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::DeserializationError(msg.to_string())
    }
}

/// Trait for annotations that can be deserialized using intercom-cts
pub trait CtsAnnotation: Sized {
    /// The annotation name (without @ prefix)
    const NAME: &'static str;

    /// Convert from HIR annotation
    ///
    /// # Errors
    ///
    /// Returns error if conversion fails
    fn from_hir(ann: &Ann) -> Result<Self, CtsAnnotationError>;
}

/// The @optional annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Optional {
    pub value: bool,
}

impl CtsAnnotation for Optional {
    const NAME: &'static str = "optional";

    fn from_hir(ann: &Ann) -> Result<Self, CtsAnnotationError> {
        if ann.ident.name != Self::NAME {
            return Err(CtsAnnotationError::WrongAnnotationType {
                expected: Self::NAME,
                actual: ann.ident.name.clone(),
            });
        }

        let mut result = Self::default();

        // Build args map
        let mut args = HashMap::new();
        for arg in &ann.args {
            let key = arg.ident.as_ref().map_or("value", |id| id.name.as_str());
            args.insert(key, &arg.value);
        }

        // Get value field
        if let Some(numeric) = args.get("value") {
            match numeric {
                Numeric::Bool(b) => result.value = *b,
                _ => {
                    return Err(CtsAnnotationError::TypeConversionError {
                        field: "value".to_string(),
                        expected: "bool",
                    });
                }
            }
        } else {
            // Default is true for optional annotation
            result.value = true;
        }

        Ok(result)
    }
}

impl Unmarshal for Optional {
    fn unmarshal_mut<D>(&mut self, _archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        // Since we're using from_hir for the actual conversion,
        // this is just a placeholder implementation
        Ok(())
    }
}

/// The @range annotation  
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Range {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl CtsAnnotation for Range {
    const NAME: &'static str = "range";

    fn from_hir(ann: &Ann) -> Result<Self, CtsAnnotationError> {
        if ann.ident.name != Self::NAME {
            return Err(CtsAnnotationError::WrongAnnotationType {
                expected: Self::NAME,
                actual: ann.ident.name.clone(),
            });
        }

        let mut result = Self::default();

        // Build args map
        let mut args = HashMap::new();
        for arg in &ann.args {
            if let Some(name) = &arg.ident {
                args.insert(name.name.as_str(), &arg.value);
            }
        }

        // Get min field
        if let Some(numeric) = args.get("min") {
            result.min = Some(convert_to_i64(numeric, "min")?);
        }

        // Get max field
        if let Some(numeric) = args.get("max") {
            result.max = Some(convert_to_i64(numeric, "max")?);
        }

        Ok(result)
    }
}

impl Unmarshal for Range {
    fn unmarshal_mut<D>(&mut self, _archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        // Since we're using from_hir for the actual conversion,
        // this is just a placeholder implementation
        Ok(())
    }
}

/// The @default annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DefaultValue {
    pub value: String, // Store as string representation
}

impl CtsAnnotation for DefaultValue {
    const NAME: &'static str = "default";

    fn from_hir(ann: &Ann) -> Result<Self, CtsAnnotationError> {
        if ann.ident.name != Self::NAME {
            return Err(CtsAnnotationError::WrongAnnotationType {
                expected: Self::NAME,
                actual: ann.ident.name.clone(),
            });
        }

        let mut result = Self::default();

        // Get the first argument (positional or named "value")
        if let Some(arg) = ann.args.first() {
            result.value = numeric_to_string(&arg.value);
        } else {
            return Err(CtsAnnotationError::FieldNotFound("value".to_string()));
        }

        Ok(result)
    }
}

impl Unmarshal for DefaultValue {
    fn unmarshal_mut<D>(&mut self, _archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        // Since we're using from_hir for the actual conversion,
        // this is just a placeholder implementation
        Ok(())
    }
}

/// Helper to convert Numeric to i64
fn convert_to_i64(numeric: &Numeric, field: &str) -> Result<i64, CtsAnnotationError> {
    match numeric {
        Numeric::Int8(v) => Ok(i64::from(*v)),
        Numeric::Int16(v) => Ok(i64::from(*v)),
        Numeric::Int32(v) => Ok(i64::from(*v)),
        Numeric::Int64(v) => Ok(*v),
        Numeric::Octet(v) => Ok(i64::from(*v)),
        Numeric::UInt16(v) => Ok(i64::from(*v)),
        Numeric::UInt32(v) => Ok(i64::from(*v)),
        Numeric::UInt64(v) => {
            i64::try_from(*v).map_err(|_| CtsAnnotationError::TypeConversionError {
                field: field.to_string(),
                expected: "i64 (value too large)",
            })
        }
        _ => Err(CtsAnnotationError::TypeConversionError {
            field: field.to_string(),
            expected: "integer",
        }),
    }
}

/// Helper to convert Numeric to string representation
fn numeric_to_string(numeric: &Numeric) -> String {
    match numeric {
        Numeric::Null => "null".to_string(),
        Numeric::Bool(b) => b.to_string(),
        Numeric::Char(c) => c.to_string(),
        Numeric::Int8(v) => v.to_string(),
        Numeric::Int16(v) => v.to_string(),
        Numeric::Int32(v) => v.to_string(),
        Numeric::Int64(v) => v.to_string(),
        Numeric::Octet(v) => v.to_string(),
        Numeric::UInt16(v) => v.to_string(),
        Numeric::UInt32(v) => v.to_string(),
        Numeric::UInt64(v) => v.to_string(),
        Numeric::Float(v) => v.to_string(),
        Numeric::Double(v) => v.to_string(),
        Numeric::String(s) => s.clone(),
        _ => "<complex>".to_string(),
    }
}

/// Extension trait for Ann to provide CTS-based downcasting
pub trait AnnCtsExt {
    /// Try to downcast this annotation using CTS
    ///
    /// # Errors
    ///
    /// Returns error if downcasting fails
    fn downcast_cts<T: CtsAnnotation>(&self) -> Result<T, CtsAnnotationError>;

    /// Check if this annotation matches a CTS annotation type
    fn is_cts<T: CtsAnnotation>(&self) -> bool;
}

impl AnnCtsExt for Ann {
    fn downcast_cts<T: CtsAnnotation>(&self) -> Result<T, CtsAnnotationError> {
        T::from_hir(self)
    }

    fn is_cts<T: CtsAnnotation>(&self) -> bool {
        self.ident.name == T::NAME
    }
}

/// Find and downcast an annotation using CTS
#[must_use]
pub fn find_annotation_cts<T: CtsAnnotation>(
    annotations: &[Ann],
) -> Option<Result<T, CtsAnnotationError>> {
    annotations
        .iter()
        .find(|ann| ann.is_cts::<T>())
        .map(AnnCtsExt::downcast_cts::<T>)
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
    fn test_optional_cts() {
        let ann = make_ann(
            "optional",
            vec![make_arg(Some("value"), Numeric::Bool(false))],
        );

        let optional = ann.downcast_cts::<Optional>().unwrap();
        assert!(!optional.value);
    }

    #[test]
    fn test_optional_default() {
        let ann = make_ann("optional", vec![]);

        let optional = ann.downcast_cts::<Optional>().unwrap();
        assert!(optional.value); // Default for optional is true
    }

    #[test]
    fn test_range_cts() {
        let ann = make_ann(
            "range",
            vec![
                make_arg(Some("min"), Numeric::Int32(0)),
                make_arg(Some("max"), Numeric::Int32(100)),
            ],
        );

        let range = ann.downcast_cts::<Range>().unwrap();
        assert_eq!(range.min, Some(0));
        assert_eq!(range.max, Some(100));
    }

    #[test]
    fn test_range_partial() {
        let ann = make_ann("range", vec![make_arg(Some("min"), Numeric::Int32(5))]);

        let range = ann.downcast_cts::<Range>().unwrap();
        assert_eq!(range.min, Some(5));
        assert_eq!(range.max, None);
    }

    #[test]
    fn test_default_cts() {
        let ann = make_ann(
            "default",
            vec![make_arg(None, Numeric::String("hello".to_string()))],
        );

        let default = ann.downcast_cts::<DefaultValue>().unwrap();
        assert_eq!(default.value, "hello");
    }

    #[test]
    fn test_default_numeric() {
        let ann = make_ann("default", vec![make_arg(None, Numeric::Int32(42))]);

        let default = ann.downcast_cts::<DefaultValue>().unwrap();
        assert_eq!(default.value, "42");
    }

    #[test]
    fn test_wrong_type() {
        let ann = make_ann("optional", vec![]);
        let err = ann.downcast_cts::<Range>().unwrap_err();
        assert!(matches!(
            err,
            CtsAnnotationError::WrongAnnotationType { .. }
        ));
    }

    #[test]
    fn test_find_annotation_cts() {
        let annotations = vec![
            make_ann(
                "optional",
                vec![make_arg(Some("value"), Numeric::Bool(true))],
            ),
            make_ann(
                "range",
                vec![
                    make_arg(Some("min"), Numeric::Int32(5)),
                    make_arg(Some("max"), Numeric::Int32(15)),
                ],
            ),
        ];

        let range = find_annotation_cts::<Range>(&annotations).unwrap().unwrap();
        assert_eq!(range.min, Some(5));
        assert_eq!(range.max, Some(15));
    }
}
