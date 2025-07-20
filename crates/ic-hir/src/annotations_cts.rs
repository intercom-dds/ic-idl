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
//! This module provides an implementation of the annotation system
//! that leverages the intercom-cts Marshal/Unmarshal traits for serialization.

use intercom_cts::{Unmarshal, TypeInfo, TypeKind, TypeFlag, MemberInfo, MemberFlag};
use intercom_cts::decode::{Deserializer, StructDeserializer};
use intercom_cts::error::Error as CtsError;

use crate::hir::{Ann, Numeric};

/// Error type for CTS-based annotation operations
#[derive(Debug, Clone)]
pub enum CtsAnnotationError {
    /// Annotation name mismatch
    WrongAnnotationType { expected: &'static str, actual: String },
    /// Deserialization error  
    DeserializationError(String),
    /// Field not found
    FieldNotFound(String),
    /// Type conversion error
    TypeConversionError { field: String, expected: &'static str },
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

/// Custom deserializer for HIR annotations
struct AnnDeserializer<'a> {
    ann: &'a Ann,
}

impl<'a> AnnDeserializer<'a> {
    fn new(ann: &'a Ann) -> Self {
        Self { ann }
    }
}

/// Struct deserializer for annotation arguments
struct AnnStructDeserializer<'a> {
    ann: &'a Ann,
    field_index: usize,
}

impl StructDeserializer for AnnStructDeserializer<'_> {
    type Ok = ();
    type Error = CtsAnnotationError;
    
    fn decode_field<T>(&mut self, info: &MemberInfo<'_>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        // Find the argument by name or by position
        let arg = self.ann.args.iter().find(|arg| {
            arg.ident.as_ref().is_some_and(|id| id.name == info.name)
        }).or_else(|| {
            // If not found by name and this is the first field, try positional
            if info.name == "value" && self.field_index == 0 {
                self.ann.args.first()
            } else {
                None
            }
        });
        
        if let Some(arg) = arg {
            let deserializer = OptionDeserializer::new(&arg.value);
            value.unmarshal_mut(deserializer)?;
        }
        // If field not found, leave default value
        
        self.field_index += 1;
        Ok(())
    }
    
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Deserializer for individual Numeric values
struct NumericDeserializer<'a> {
    value: &'a Numeric,
}

impl<'a> NumericDeserializer<'a> {
    fn new(value: &'a Numeric) -> Self {
        Self { value }
    }
}

/// Deserializer that wraps a Numeric value and provides Option support
struct OptionDeserializer<'a> {
    value: &'a Numeric,
}

impl<'a> OptionDeserializer<'a> {
    fn new(value: &'a Numeric) -> Self {
        Self { value }
    }
}

// Never types for unsupported deserializer types
struct Never;

impl StructDeserializer for Never {
    type Ok = ();
    type Error = CtsAnnotationError;
    
    fn decode_field<T>(&mut self, _info: &MemberInfo<'_>, _value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }
    
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl intercom_cts::decode::UnionDeserializer for Never {
    type Ok = ();
    type Error = CtsAnnotationError;
    
    fn decode_discriminant<T>(&mut self, _value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        unreachable!()
    }
    
    fn decode_variant<T>(self, _info: &MemberInfo<'_>, _value: &mut T) -> Result<Self::Ok, Self::Error>
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

impl Deserializer for NumericDeserializer<'_> {
    type Error = CtsAnnotationError;
    type Struct = Never;
    type Enum = Never;
    type Union = Never;
    type Array = Never;
    type Sequence = Never;
    type Map = Never;
    
    fn decode_bool(self) -> Result<bool, Self::Error> {
        match self.value {
            Numeric::Bool(b) => Ok(*b),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "bool",
            }),
        }
    }
    
    fn decode_i8(self) -> Result<i8, Self::Error> {
        match self.value {
            Numeric::Int8(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "i8",
            }),
        }
    }
    
    fn decode_i16(self) -> Result<i16, Self::Error> {
        match self.value {
            Numeric::Int8(v) => Ok(i16::from(*v)),
            Numeric::Int16(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "i16",
            }),
        }
    }
    
    fn decode_i32(self) -> Result<i32, Self::Error> {
        match self.value {
            Numeric::Int8(v) => Ok(i32::from(*v)),
            Numeric::Int16(v) => Ok(i32::from(*v)),
            Numeric::Int32(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "i32",
            }),
        }
    }
    
    fn decode_i64(self) -> Result<i64, Self::Error> {
        match self.value {
            Numeric::Int8(v) => Ok(i64::from(*v)),
            Numeric::Int16(v) => Ok(i64::from(*v)),
            Numeric::Int32(v) => Ok(i64::from(*v)),
            Numeric::Int64(v) => Ok(*v),
            Numeric::Octet(v) => Ok(i64::from(*v)),
            Numeric::UInt16(v) => Ok(i64::from(*v)),
            Numeric::UInt32(v) => Ok(i64::from(*v)),
            Numeric::UInt64(v) => i64::try_from(*v)
                .map_err(|_| CtsAnnotationError::TypeConversionError {
                    field: "value".to_string(),
                    expected: "i64 (value too large)",
                }),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "i64",
            }),
        }
    }
    
    fn decode_u8(self) -> Result<u8, Self::Error> {
        match self.value {
            Numeric::Octet(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "u8",
            }),
        }
    }
    
    fn decode_u16(self) -> Result<u16, Self::Error> {
        match self.value {
            Numeric::Octet(v) => Ok(u16::from(*v)),
            Numeric::UInt16(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "u16",
            }),
        }
    }
    
    fn decode_u32(self) -> Result<u32, Self::Error> {
        match self.value {
            Numeric::Octet(v) => Ok(u32::from(*v)),
            Numeric::UInt16(v) => Ok(u32::from(*v)),
            Numeric::UInt32(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "u32",
            }),
        }
    }
    
    fn decode_u64(self) -> Result<u64, Self::Error> {
        match self.value {
            Numeric::Octet(v) => Ok(u64::from(*v)),
            Numeric::UInt16(v) => Ok(u64::from(*v)),
            Numeric::UInt32(v) => Ok(u64::from(*v)),
            Numeric::UInt64(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "u64",
            }),
        }
    }
    
    fn decode_f32(self) -> Result<f32, Self::Error> {
        match self.value {
            Numeric::Float(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "f32",
            }),
        }
    }
    
    fn decode_f64(self) -> Result<f64, Self::Error> {
        match self.value {
            Numeric::Float(v) => Ok(f64::from(*v)),
            Numeric::Double(v) => Ok(*v),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "f64",
            }),
        }
    }
    
    fn decode_char(self) -> Result<char, Self::Error> {
        match self.value {
            Numeric::Char(c) => Ok(*c),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "char",
            }),
        }
    }
    
    fn decode_wchar(self) -> Result<char, Self::Error> {
        self.decode_char()
    }
    
    fn decode_string(self) -> Result<String, Self::Error> {
        match self.value {
            Numeric::String(s) => Ok(s.clone()),
            _ => Err(CtsAnnotationError::TypeConversionError {
                field: "value".to_string(),
                expected: "string",
            }),
        }
    }
    
    fn decode_wstring(self) -> Result<String, Self::Error> {
        self.decode_string()
    }
    
    fn decode_option<T>(self) -> Result<Option<T>, Self::Error>
    where
        T: Unmarshal + Default,
    {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_option_mut<T>(self, _value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_struct(self, _info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_enum(self, _name: &str) -> Result<Self::Enum, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_union(self, _info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_array(self, _len: usize) -> Result<Self::Array, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
}

impl Deserializer for OptionDeserializer<'_> {
    type Error = CtsAnnotationError;
    type Struct = Never;
    type Enum = Never;
    type Union = Never;
    type Array = Never;
    type Sequence = Never;
    type Map = Never;
    
    // Delegate all primitive type decoding to NumericDeserializer
    fn decode_bool(self) -> Result<bool, Self::Error> {
        NumericDeserializer::new(self.value).decode_bool()
    }
    
    fn decode_i8(self) -> Result<i8, Self::Error> {
        NumericDeserializer::new(self.value).decode_i8()
    }
    
    fn decode_i16(self) -> Result<i16, Self::Error> {
        NumericDeserializer::new(self.value).decode_i16()
    }
    
    fn decode_i32(self) -> Result<i32, Self::Error> {
        NumericDeserializer::new(self.value).decode_i32()
    }
    
    fn decode_i64(self) -> Result<i64, Self::Error> {
        NumericDeserializer::new(self.value).decode_i64()
    }
    
    fn decode_u8(self) -> Result<u8, Self::Error> {
        NumericDeserializer::new(self.value).decode_u8()
    }
    
    fn decode_u16(self) -> Result<u16, Self::Error> {
        NumericDeserializer::new(self.value).decode_u16()
    }
    
    fn decode_u32(self) -> Result<u32, Self::Error> {
        NumericDeserializer::new(self.value).decode_u32()
    }
    
    fn decode_u64(self) -> Result<u64, Self::Error> {
        NumericDeserializer::new(self.value).decode_u64()
    }
    
    fn decode_f32(self) -> Result<f32, Self::Error> {
        NumericDeserializer::new(self.value).decode_f32()
    }
    
    fn decode_f64(self) -> Result<f64, Self::Error> {
        NumericDeserializer::new(self.value).decode_f64()
    }
    
    fn decode_char(self) -> Result<char, Self::Error> {
        NumericDeserializer::new(self.value).decode_char()
    }
    
    fn decode_wchar(self) -> Result<char, Self::Error> {
        NumericDeserializer::new(self.value).decode_wchar()
    }
    
    fn decode_string(self) -> Result<String, Self::Error> {
        NumericDeserializer::new(self.value).decode_string()
    }
    
    fn decode_wstring(self) -> Result<String, Self::Error> {
        NumericDeserializer::new(self.value).decode_wstring()
    }
    
    // Handle Option<T> by always returning Some(value) since we have a value
    fn decode_option<T>(self) -> Result<Option<T>, Self::Error>
    where
        T: Unmarshal + Default,
    {
        let mut value = T::default();
        value.unmarshal_mut(NumericDeserializer::new(self.value))?;
        Ok(Some(value))
    }
    
    fn decode_option_mut<T>(self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        value.unmarshal_mut(NumericDeserializer::new(self.value))?;
        Ok(true)
    }
    
    fn decode_struct(self, _info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_enum(self, _name: &str) -> Result<Self::Enum, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_union(self, _info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_array(self, _len: usize) -> Result<Self::Array, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
    
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "value".to_string(),
            expected: "primitive type",
        })
    }
}

impl<'a> Deserializer for AnnDeserializer<'a> {
    type Error = CtsAnnotationError;
    type Struct = AnnStructDeserializer<'a>;
    type Enum = Never;
    type Union = Never;
    type Array = Never;
    type Sequence = Never;
    type Map = Never;
    
    fn decode_bool(self) -> Result<bool, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_i8(self) -> Result<i8, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_i16(self) -> Result<i16, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_i32(self) -> Result<i32, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_i64(self) -> Result<i64, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_u8(self) -> Result<u8, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_u16(self) -> Result<u16, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_u32(self) -> Result<u32, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_u64(self) -> Result<u64, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_f32(self) -> Result<f32, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_f64(self) -> Result<f64, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_char(self) -> Result<char, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_wchar(self) -> Result<char, Self::Error> {
        self.decode_char()
    }
    
    fn decode_string(self) -> Result<String, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_wstring(self) -> Result<String, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_option<T>(self) -> Result<Option<T>, Self::Error>
    where
        T: Unmarshal + Default,
    {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_option_mut<T>(self, _value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_struct(self, _info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Ok(AnnStructDeserializer {
            ann: self.ann,
            field_index: 0,
        })
    }
    
    fn decode_enum(self, _name: &str) -> Result<Self::Enum, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_union(self, _info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_array(self, _len: usize) -> Result<Self::Array, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
        })
    }
    
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(CtsAnnotationError::TypeConversionError {
            field: "annotation".to_string(),
            expected: "struct",
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
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        // Set default
        self.value = true;
        
        let mut state = archive.decode_struct(&TypeInfo {
            name: "Optional",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_kind: TypeKind::None,
            element_kind: TypeKind::None,
        })?;
        
        state.decode_field(&MemberInfo {
            name: "value",
            member_id: 0,
            flags: MemberFlag::nil(),
        }, &mut self.value)?;
        
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
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "Range",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_kind: TypeKind::None,
            element_kind: TypeKind::None,
        })?;
        
        state.decode_field(&MemberInfo {
            name: "min",
            member_id: 0,
            flags: MemberFlag::nil(),
        }, &mut self.min)?;
        
        state.decode_field(&MemberInfo {
            name: "max",
            member_id: 1,
            flags: MemberFlag::nil(),
        }, &mut self.max)?;
        
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
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        let mut state = archive.decode_struct(&TypeInfo {
            name: "DefaultValue",
            flags: TypeFlag::nil(),
            kind: TypeKind::Struct,
            key_kind: TypeKind::None,
            element_kind: TypeKind::None,
        })?;
        
        state.decode_field(&MemberInfo {
            name: "value",
            member_id: 0,
            flags: MemberFlag::nil(),
        }, &mut self.value)?;
        
        state.end()?;
        Ok(())
    }
}

/// Helper to deserialize an annotation with a specific expected name
/// 
/// # Errors
/// 
/// Returns error if annotation name doesn't match or deserialization fails
pub fn unmarshal_annotation<T>(ann: &Ann, expected_name: &'static str) -> Result<T, CtsAnnotationError>
where
    T: Unmarshal + Default,
{
    if ann.ident.name != expected_name {
        return Err(CtsAnnotationError::WrongAnnotationType {
            expected: expected_name,
            actual: ann.ident.name.clone(),
        });
    }
    
    let deserializer = AnnDeserializer::new(ann);
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
    fn unmarshal<T: Unmarshal + Default>(&self, expected_name: &'static str) -> Result<T, CtsAnnotationError>;
}

impl AnnCtsExt for Ann {
    fn unmarshal<T: Unmarshal + Default>(&self, expected_name: &'static str) -> Result<T, CtsAnnotationError> {
        unmarshal_annotation(self, expected_name)
    }
}

/// Find and unmarshal an annotation from a list
#[must_use]
pub fn find_annotation<T>(annotations: &[Ann], name: &'static str) -> Option<Result<T, CtsAnnotationError>>
where
    T: Unmarshal + Default,
{
    annotations.iter()
        .find(|ann| ann.ident.name == name)
        .map(|ann| ann.unmarshal(name))
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
    fn test_optional_unmarshal() {
        let ann = make_ann("optional", vec![
            make_arg(Some("value"), Numeric::Bool(false)),
        ]);
        
        let optional: Optional = ann.unmarshal("optional").unwrap();
        assert!(!optional.value);
    }
    
    #[test]
    fn test_optional_default() {
        let ann = make_ann("optional", vec![]);
        
        let optional: Optional = ann.unmarshal("optional").unwrap();
        assert!(optional.value);  // Default for optional is true
    }
    
    #[test]
    fn test_range_unmarshal() {
        let ann = make_ann("range", vec![
            make_arg(Some("min"), Numeric::Int32(0)),
            make_arg(Some("max"), Numeric::Int32(100)),
        ]);
        
        let range: Range = ann.unmarshal("range").unwrap();
        assert_eq!(range.min, Some(0));
        assert_eq!(range.max, Some(100));
    }
    
    #[test]
    fn test_range_partial() {
        let ann = make_ann("range", vec![
            make_arg(Some("min"), Numeric::Int32(5)),
        ]);
        
        let range: Range = ann.unmarshal("range").unwrap();
        assert_eq!(range.min, Some(5));
        assert_eq!(range.max, None);
    }
    
    #[test]
    fn test_default_unmarshal() {
        let ann = make_ann("default", vec![
            make_arg(None, Numeric::String("hello".to_string())),
        ]);
        
        let default: DefaultValue = ann.unmarshal("default").unwrap();
        assert_eq!(default.value, "hello");
    }
    
    #[test]
    fn test_wrong_type() {
        let ann = make_ann("optional", vec![]);
        let err = ann.unmarshal::<Range>("range").unwrap_err();
        assert!(matches!(err, CtsAnnotationError::WrongAnnotationType { .. }));
    }
    
    #[test]
    fn test_find_annotation() {
        let annotations = vec![
            make_ann("optional", vec![make_arg(Some("value"), Numeric::Bool(true))]),
            make_ann("range", vec![
                make_arg(Some("min"), Numeric::Int32(5)),
                make_arg(Some("max"), Numeric::Int32(15)),
            ]),
        ];
        
        let range: Range = find_annotation(&annotations, "range").unwrap().unwrap();
        assert_eq!(range.min, Some(5));
        assert_eq!(range.max, Some(15));
    }
}