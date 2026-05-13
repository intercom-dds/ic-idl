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

use intercom_cts::decode::{Deserializer, EnumDeserializer as _, StructDeserializer};
use intercom_cts::error::Error as CtsError;
use intercom_cts::type_info::type_info;
use intercom_cts::{MemberFlag, MemberInfo, TypeFlag, TypeInfo, TypeKind, Unmarshal};

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

/// Unified deserializer that can handle both annotations and numeric values
enum AnnDeserializer<'a> {
    /// Deserializing a full annotation (struct-like)
    Annotation(&'a Ann),

    /// Deserializing a numeric value directly
    Numeric(&'a Numeric),
}

impl<'a> AnnDeserializer<'a> {
    fn from_annotation(ann: &'a Ann) -> Self {
        Self::Annotation(ann)
    }

    fn from_numeric(value: &'a Numeric) -> Self {
        Self::Numeric(value)
    }
}

/// Struct deserializer for annotation arguments
struct AnnStructDeserializer<'a> {
    ann: &'a Ann,
    field_index: usize,
}

impl<'a> StructDeserializer<'a> for AnnStructDeserializer<'a> {
    type Ok = ();
    type Error = CtsAnnotationError;

    fn decode_field<T>(&mut self, info: &MemberInfo<'a>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        // Find the argument by name or by position
        let arg = self
            .ann
            .args
            .iter()
            .find(|arg| arg.ident.name == info.name)
            .or_else(|| {
                // If not found by name and this is the first field, try positional
                if info.name == "value" && self.field_index == 0 {
                    self.ann.args.first()
                } else {
                    None
                }
            });

        if let Some(arg) = arg {
            let deserializer = AnnDeserializer::from_numeric(&arg.value);
            value.unmarshal_mut(deserializer)?;
        }

        self.field_index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Enum deserializer for string-based enum values
struct StringEnumDeserializer {
    value: String,
}

impl StringEnumDeserializer {
    fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl intercom_cts::decode::EnumDeserializer for StringEnumDeserializer {
    type Error = CtsAnnotationError;

    fn decode_enumerator<T>(self, visitor: T) -> Result<T, Self::Error>
    where
        T: Unmarshal + intercom_cts::decode::EnumVisitor,
    {
        visitor.member_field::<AnnDeserializer>(&self.value)
    }
}

struct Never;

impl<'a> StructDeserializer<'a> for Never {
    type Ok = ();
    type Error = CtsAnnotationError;

    fn decode_field<T>(&mut self, _info: &MemberInfo<'a>, _value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<'a> intercom_cts::decode::UnionDeserializer<'a> for Never {
    type Ok = ();
    type Error = CtsAnnotationError;

    fn decode_discriminant<T>(&mut self, _value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }

    fn decode_variant<T>(
        self,
        _info: &MemberInfo<'a>,
        _value: &mut T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }
}

impl intercom_cts::decode::EnumDeserializer for Never {
    type Error = CtsAnnotationError;

    fn decode_enumerator<T>(self, _visitor: T) -> Result<T, Self::Error>
    where
        T: Unmarshal + intercom_cts::decode::EnumVisitor,
    {
        unreachable!()
    }
}

impl intercom_cts::decode::SeqDeserializer for Never {
    type Error = CtsAnnotationError;

    fn decode_next<T>(&mut self, _value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }

    fn size_hint(&self) -> Option<usize> {
        unreachable!()
    }
}

impl intercom_cts::decode::ArrayDeserializer for Never {
    type Error = CtsAnnotationError;

    fn decode_next<T>(&mut self, _value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }
}

impl intercom_cts::decode::MapDeserializer for Never {
    type Error = CtsAnnotationError;

    fn decode_pair<K, V>(&mut self, _key: &mut K, _value: &mut V) -> Result<bool, Self::Error>
    where
        K: Unmarshal,
        V: Unmarshal,
    {
        unreachable!()
    }

    fn size_hint(&self) -> Option<usize> {
        unreachable!()
    }
}

impl intercom_cts::decode::OptionDeserializer for Never {
    type Error = CtsAnnotationError;

    fn is_some(&mut self) -> bool {
        unreachable!()
    }

    fn decode_some<T>(self, _value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }
}

/// Option deserializer for annotation values
struct AnnOptionDeserializer<'a> {
    numeric: Option<&'a Numeric>,
}

impl intercom_cts::decode::OptionDeserializer for AnnOptionDeserializer<'_> {
    type Error = CtsAnnotationError;

    fn is_some(&mut self) -> bool {
        self.numeric.is_some()
    }

    fn decode_some<T>(self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        if let Some(numeric) = self.numeric {
            value.unmarshal_mut(AnnDeserializer::from_numeric(numeric))?;
        }
        Ok(())
    }
}

impl<'a> Deserializer<'a> for AnnDeserializer<'a> {
    type Error = CtsAnnotationError;
    type Struct = AnnStructDeserializer<'a>;
    type Enum = StringEnumDeserializer;
    type Union = Never;
    type Array = Never;
    type Sequence = Never;
    type Map = Never;
    type Option = AnnOptionDeserializer<'a>;

    fn decode_bool(self) -> Result<bool, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "bool",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Bool(b) => Ok(*b),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "bool",
                }),
            },
        }
    }

    fn decode_i8(self) -> Result<i8, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "i8",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Int8(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "i8",
                }),
            },
        }
    }

    fn decode_i16(self) -> Result<i16, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "i16",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Int8(v) => Ok(i16::from(*v)),
                Numeric::Int16(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "i16",
                }),
            },
        }
    }

    fn decode_i32(self) -> Result<i32, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "i32",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Int8(v) => Ok(i32::from(*v)),
                Numeric::Int16(v) => Ok(i32::from(*v)),
                Numeric::Int32(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "i32",
                }),
            },
        }
    }

    fn decode_i64(self) -> Result<i64, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "i64",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Int8(v) => Ok(i64::from(*v)),
                Numeric::Int16(v) => Ok(i64::from(*v)),
                Numeric::Int32(v) => Ok(i64::from(*v)),
                Numeric::Int64(v) => Ok(*v),
                Numeric::UInt8(v) => Ok(i64::from(*v)),
                Numeric::UInt16(v) => Ok(i64::from(*v)),
                Numeric::UInt32(v) => Ok(i64::from(*v)),
                Numeric::UInt64(v) => {
                    i64::try_from(*v).map_err(|_| CtsAnnotationError::TypeConversionError {
                        field: "value".to_string(),
                        expected: "i64 (value too large)",
                    })
                }
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "i64",
                }),
            },
        }
    }

    fn decode_u8(self) -> Result<u8, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "u8",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::UInt8(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "u8",
                }),
            },
        }
    }

    fn decode_u16(self) -> Result<u16, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "u16",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::UInt8(v) => Ok(u16::from(*v)),
                Numeric::UInt16(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "u16",
                }),
            },
        }
    }

    fn decode_u32(self) -> Result<u32, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "u32",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::UInt8(v) => Ok(u32::from(*v)),
                Numeric::UInt16(v) => Ok(u32::from(*v)),
                Numeric::UInt32(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "u32",
                }),
            },
        }
    }

    fn decode_u64(self) -> Result<u64, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "u64",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::UInt8(v) => Ok(u64::from(*v)),
                Numeric::UInt16(v) => Ok(u64::from(*v)),
                Numeric::UInt32(v) => Ok(u64::from(*v)),
                Numeric::UInt64(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "u64",
                }),
            },
        }
    }

    fn decode_f32(self) -> Result<f32, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "f32",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Float(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "f32",
                }),
            },
        }
    }

    fn decode_f64(self) -> Result<f64, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "f64",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Float(v) => Ok(f64::from(*v)),
                Numeric::Double(v) => Ok(*v),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "f64",
                }),
            },
        }
    }

    fn decode_char(self) -> Result<char, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "char",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::Char(c) | Numeric::WChar(c) => Ok(*c),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "char",
                }),
            },
        }
    }

    fn decode_wchar(self) -> Result<char, Self::Error> {
        self.decode_char()
    }

    fn decode_string(self) -> Result<String, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "string",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::String(s) | Numeric::WString(s) => Ok(s.clone()),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "string",
                }),
            },
        }
    }

    fn decode_wstring(self) -> Result<String, Self::Error> {
        self.decode_string()
    }

    fn begin_decode_option(self) -> Result<Self::Option, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "option",
            }),
            Self::Numeric(numeric) => Ok(AnnOptionDeserializer {
                numeric: Some(numeric),
            }),
        }
    }

    fn decode_struct(self, _info: &TypeInfo<'a>) -> Result<Self::Struct, Self::Error> {
        match self {
            Self::Annotation(ann) => Ok(AnnStructDeserializer {
                ann,
                field_index: 0,
            }),
            Self::Numeric(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "struct",
            }),
        }
    }

    fn decode_enum(self, _name: &str) -> Result<Self::Enum, Self::Error> {
        match self {
            Self::Annotation(_) => Err(CtsAnnotationError::TypeConversionError {
                field: "annotation".to_string(),
                expected: "enum",
            }),
            Self::Numeric(numeric) => match numeric {
                Numeric::String(s) | Numeric::WString(s) => Ok(StringEnumDeserializer::new(s)),
                _ => Err(CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "string (for enum)",
                }),
            },
        }
    }

    fn decode_union(self, _info: &TypeInfo<'a>) -> Result<Self::Union, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: match self {
                Self::Annotation(_) => "annotation",
                Self::Numeric(_) => "value",
            }
            .to_string(),
            expected: "union",
        })
    }

    fn decode_array(self, _len: usize) -> Result<Self::Array, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: match self {
                Self::Annotation(_) => "annotation",
                Self::Numeric(_) => "value",
            }
            .to_string(),
            expected: "array",
        })
    }

    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: match self {
                Self::Annotation(_) => "annotation",
                Self::Numeric(_) => "value",
            }
            .to_string(),
            expected: "sequence",
        })
    }

    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: match self {
                Self::Annotation(_) => "annotation",
                Self::Numeric(_) => "value",
            }
            .to_string(),
            expected: "map",
        })
    }
}

// Now we can define our annotation types with proper Unmarshal implementations

/// The @optional annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Optional {
    pub value: bool,
}

impl Unmarshal for Optional {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        // Set default
        self.value = true;

        let mut state = archive.decode_struct(&TypeInfo {
            name: "Optional",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_info: None,
            element_info: None,
        })?;

        state.decode_field(
            &MemberInfo {
                name: "value",
                member_id: 0,
                flags: MemberFlag::nil(),
                type_info: type_info::<bool>(),
            },
            &mut self.value,
        )?;

        state.end()?;
        Ok(())
    }
}

/// The @range annotation  
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Range {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl Unmarshal for Range {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "Range",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_info: None,
            element_info: None,
        })?;

        state.decode_field(
            &MemberInfo {
                name: "min",
                member_id: 0,
                flags: MemberFlag::nil(),
                type_info: type_info::<Option<i64>>(),
            },
            &mut self.min,
        )?;

        state.decode_field(
            &MemberInfo {
                name: "max",
                member_id: 1,
                flags: MemberFlag::nil(),
                type_info: type_info::<Option<i64>>(),
            },
            &mut self.max,
        )?;

        state.end()?;
        Ok(())
    }
}

/// The @min annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Min {
    pub value: i64,
}

impl Unmarshal for Min {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "Min",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_info: None,
            element_info: None,
        })?;

        state.decode_field(
            &MemberInfo {
                name: "value",
                member_id: 0,
                flags: MemberFlag::nil(),
                type_info: type_info::<i64>(),
            },
            &mut self.value,
        )?;

        state.end()?;
        Ok(())
    }
}

/// The @max annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Max {
    pub value: i64,
}

impl Unmarshal for Max {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "Max",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_info: None,
            element_info: None,
        })?;

        state.decode_field(
            &MemberInfo {
                name: "value",
                member_id: 0,
                flags: MemberFlag::nil(),
                type_info: type_info::<i64>(),
            },
            &mut self.value,
        )?;

        state.end()?;
        Ok(())
    }
}

/// The @default annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DefaultValue {
    pub value: String,
}

impl Unmarshal for DefaultValue {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "DefaultValue",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_info: None,
            element_info: None,
        })?;

        state.decode_field(
            &MemberInfo {
                name: "value",
                member_id: 0,
                flags: MemberFlag::nil(),
                type_info: type_info::<String>(),
            },
            &mut self.value,
        )?;

        state.end()?;
        Ok(())
    }
}

/// Example enum for the @mode annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Mode {
    #[default]
    ReadWrite,
    ReadOnly,
    WriteOnly,
}

impl intercom_cts::decode::EnumVisitor for Mode {
    fn member_id<'a, D>(self, _de: D) -> Result<Self, D::Error>
    where
        Self: Sized,
        D: Deserializer<'a>,
    {
        Err(D::Error::custom("Mode enum does not support member IDs"))
    }

    fn member_field<'a, D>(self, name: &str) -> Result<Self, D::Error>
    where
        Self: Sized,
        D: Deserializer<'a>,
    {
        match name {
            "read_write" => Ok(Mode::ReadWrite),
            "read_only" => Ok(Mode::ReadOnly),
            "write_only" => Ok(Mode::WriteOnly),
            _ => Err(D::Error::custom(format!("Unknown mode: {name}"))),
        }
    }
}

impl Unmarshal for Mode {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let state = archive.decode_enum("Mode")?;
        *self = state.decode_enumerator(Mode::default())?;
        Ok(())
    }
}

/// The @mode annotation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModeAnnotation {
    pub value: Mode,
}

/// Type info for Mode enum
static MODE_TYPE_INFO: TypeInfo<'static> = TypeInfo {
    name: "Mode",
    flags: TypeFlag::IS_FINAL,
    kind: TypeKind::Enum,
    key_info: None,
    element_info: None,
};

impl Unmarshal for ModeAnnotation {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "ModeAnnotation",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_info: None,
            element_info: None,
        })?;

        state.decode_field(
            &MemberInfo {
                name: "value",
                member_id: 0,
                flags: MemberFlag::nil(),
                type_info: &MODE_TYPE_INFO,
            },
            &mut self.value,
        )?;

        state.end()?;
        Ok(())
    }
}

/// Helper to deserialize an annotation with a specific expected name
///
/// # Errors
///
/// Returns error if annotation name doesn't match or deserialization fails
pub fn unmarshal_annotation<T>(
    ann: &Ann,
    expected_name: &'static str,
) -> Result<T, CtsAnnotationError>
where
    T: Unmarshal + Default,
{
    if ann.ident.name != expected_name {
        return Err(CtsAnnotationError::WrongAnnotationType {
            expected: expected_name,
            actual: ann.ident.name.clone(),
        });
    }

    let deserializer = AnnDeserializer::from_annotation(ann);
    let mut value = T::default();
    value.unmarshal_mut(deserializer)?;
    Ok(value)
}

/// Extension trait for Ann to provide CTS-based deserialization
pub trait AnnCtsExt {
    /// Try to deserialize this annotation to a specific type
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails
    fn unmarshal<T: Unmarshal + Default>(
        &self,
        expected_name: &'static str,
    ) -> Result<T, CtsAnnotationError>;
}

impl AnnCtsExt for Ann {
    fn unmarshal<T: Unmarshal + Default>(
        &self,
        expected_name: &'static str,
    ) -> Result<T, CtsAnnotationError> {
        unmarshal_annotation(self, expected_name)
    }
}

/// Find and unmarshal an annotation from a list
#[must_use]
pub fn find_annotation<T>(
    annotations: &[Ann],
    name: &'static str,
) -> Option<Result<T, CtsAnnotationError>>
where
    T: Unmarshal + Default,
{
    annotations
        .iter()
        .find(|ann| ann.ident.name == name)
        .map(|ann| ann.unmarshal(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{AnnArg, Ident};

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
    fn test_optional_unmarshal() {
        let ann = make_ann(
            "optional",
            vec![make_arg(Some("value"), Numeric::Bool(false))],
        );

        let optional: Optional = ann.unmarshal("optional").unwrap();
        assert!(!optional.value);
    }

    #[test]
    fn test_optional_default() {
        let ann = make_ann("optional", vec![]);

        let optional: Optional = ann.unmarshal("optional").unwrap();
        assert!(optional.value); // Default for optional is true
    }

    #[test]
    fn test_range_unmarshal() {
        let ann = make_ann(
            "range",
            vec![
                make_arg(Some("min"), Numeric::Int32(0)),
                make_arg(Some("max"), Numeric::Int32(100)),
            ],
        );

        let range: Range = ann.unmarshal("range").unwrap();
        assert_eq!(range.min, Some(0));
        assert_eq!(range.max, Some(100));
    }

    #[test]
    fn test_range_partial() {
        let ann = make_ann("range", vec![make_arg(Some("min"), Numeric::Int32(5))]);

        let range: Range = ann.unmarshal("range").unwrap();
        assert_eq!(range.min, Some(5));
        assert_eq!(range.max, None);
    }

    #[test]
    fn test_default_unmarshal() {
        let ann = make_ann(
            "default",
            vec![make_arg(None, Numeric::String("hello".to_string()))],
        );

        let default: DefaultValue = ann.unmarshal("default").unwrap();
        assert_eq!(default.value, "hello");
    }

    #[test]
    fn test_wrong_type() {
        let ann = make_ann("optional", vec![]);
        let err = ann.unmarshal::<Range>("range").unwrap_err();
        assert!(matches!(
            err,
            CtsAnnotationError::WrongAnnotationType { .. }
        ));
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
                    make_arg(Some("min"), Numeric::Int32(5)),
                    make_arg(Some("max"), Numeric::Int32(15)),
                ],
            ),
        ];

        let range: Range = find_annotation(&annotations, "range").unwrap().unwrap();
        assert_eq!(range.min, Some(5));
        assert_eq!(range.max, Some(15));
    }

    #[test]
    fn test_mode_enum_unmarshal() {
        let ann = make_ann(
            "mode",
            vec![make_arg(None, Numeric::String("read_only".to_string()))],
        );

        let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
        assert_eq!(mode.value, Mode::ReadOnly);
    }

    #[test]
    fn test_mode_enum_with_name() {
        let ann = make_ann(
            "mode",
            vec![make_arg(
                Some("value"),
                Numeric::String("write_only".to_string()),
            )],
        );

        let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
        assert_eq!(mode.value, Mode::WriteOnly);
    }

    #[test]
    fn test_mode_enum_default() {
        let ann = make_ann(
            "mode",
            vec![make_arg(None, Numeric::String("read_write".to_string()))],
        );

        let mode: ModeAnnotation = ann.unmarshal("mode").unwrap();
        assert_eq!(mode.value, Mode::ReadWrite);
    }

    #[test]
    fn test_mode_enum_invalid() {
        let ann = make_ann(
            "mode",
            vec![make_arg(None, Numeric::String("invalid_mode".to_string()))],
        );

        let result: Result<ModeAnnotation, _> = ann.unmarshal("mode");
        assert!(result.is_err());
    }

    #[test]
    fn test_min_unmarshal() {
        let ann = make_ann("min", vec![make_arg(None, Numeric::Int32(-100))]);

        let min: Min = ann.unmarshal("min").unwrap();
        assert_eq!(min.value, -100);
    }

    #[test]
    fn test_min_with_name() {
        let ann = make_ann("min", vec![make_arg(Some("value"), Numeric::Int64(-1000))]);

        let min: Min = ann.unmarshal("min").unwrap();
        assert_eq!(min.value, -1000);
    }

    #[test]
    fn test_max_unmarshal() {
        let ann = make_ann("max", vec![make_arg(None, Numeric::Int32(100))]);

        let max: Max = ann.unmarshal("max").unwrap();
        assert_eq!(max.value, 100);
    }

    #[test]
    fn test_max_with_name() {
        let ann = make_ann("max", vec![make_arg(Some("value"), Numeric::Int64(1000))]);

        let max: Max = ann.unmarshal("max").unwrap();
        assert_eq!(max.value, 1000);
    }

    #[test]
    fn test_min_max_conversion() {
        // Test various numeric types convert properly to i64
        let ann = make_ann("min", vec![make_arg(None, Numeric::Int8(-128))]);
        let min: Min = ann.unmarshal("min").unwrap();
        assert_eq!(min.value, -128);

        let ann = make_ann("max", vec![make_arg(None, Numeric::UInt32(4_294_967_295))]);
        let max: Max = ann.unmarshal("max").unwrap();
        assert_eq!(max.value, 4_294_967_295);
    }
}
