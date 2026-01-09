// Copyright 2025 KONGSBERG
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

use std::collections::BTreeMap;
use std::fmt;

use super::error::Error;
use super::key::KeySerializer;
use crate::decode::{Deserializer, Type};
use crate::encode::{
    ArraySerializer, EnumSerializer, MapSerializer, SeqSerializer, Serializer, StructSerializer,
    UnionSerializer,
};
use crate::error::Error as _;
use crate::json::to_string;
use crate::{DISC_INFO, Marshal, MemberInfo, TypeInfo, Unmarshal};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Number {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

macro_rules! impl_from {
    ($var:tt, $int:ty: $($ty:ty)+) => {
        $(impl From<$ty> for Number {
            #[allow(clippy::cast_lossless)]
            fn from(value: $ty) -> Self {
                Self::$var(value as $int)
            }
        })*
    };
}

impl_from!(Float, f64: f32 f64);
impl_from!(Signed, i64: i8 i16 i32 i64 isize);
impl_from!(Unsigned, u64: u8 u16 u32 u64 usize);

impl Default for Number {
    fn default() -> Self {
        Self::Signed(0)
    }
}

/// An enum that represents all possible JSON data types.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Number(Number::Signed(_) | Number::Unsigned(_)))
    }

    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Number(Number::Float(_)))
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    #[must_use]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        if let Self::Number(Number::Unsigned(v)) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        if let Self::Number(Number::Signed(v)) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        if let Self::Number(Number::Float(v)) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Self>> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Self>> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Value {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self::Null
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = to_string(self, true).map_err(|_| fmt::Error)?;
        f.write_str(&str)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(Number::Float(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        value.map_or_else(|| Self::Null, Into::into)
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl Marshal for Value {
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => archive.encode_option::<Value>(&None),
            Value::Bool(v) => v.marshal(archive),
            Value::Number(v) => v.marshal(archive),
            Value::String(v) => v.marshal(archive),
            Value::Array(v) => v.marshal(archive),
            Value::Object(v) => v.marshal(archive),
        }
    }
}

impl Unmarshal for Value {
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        if let Some(next) = archive.peek()? {
            *self = match next {
                Type::Bool => Value::Bool(archive.decode_bool()?),
                Type::String => Value::String(archive.decode_string()?),
                Type::Map => Value::Object(BTreeMap::unmarshal(archive)?),
                Type::Sequence => Value::Array(Vec::unmarshal(archive)?),
                _ => Value::Number(Number::unmarshal(archive)?),
            }
        }
        Ok(())
    }
}

impl Marshal for Number {
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Number::Signed(v) => v.marshal(archive),
            Number::Unsigned(v) => v.marshal(archive),
            Number::Float(v) => v.marshal(archive),
        }
    }
}

impl Unmarshal for Number {
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        if let Some(next) = archive.peek()? {
            *self = match next {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
                    Number::Signed(archive.decode_i64()?)
                }
                Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                    Number::Unsigned(archive.decode_u64()?)
                }
                Type::F32 | Type::F64 => Number::Float(archive.decode_f64()?),
                _ => Err(D::Error::custom("expected number"))?,
            }
        }
        Ok(())
    }
}

/// The `value!` macro provides a convenient way to construct a `Value` from a
/// JSON literal.
///
/// ## Usage
///
/// ```rust
/// # use intercom_cts::json::value;
/// let value = value!({
///     "id": 43,
///     "method": "$/foobar",
///     "params": {
///         "key": "value",
///         "error": null
///     }
/// });
/// assert!(value.is_object());
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! value {
    (null) => {{
        $crate::json::Value::Null
    }};

    (true) => {{
        $crate::json::Value::Bool(true)
    }};

    (false) => {{
        $crate::json::Value::Bool(false)
    }};

    ([ $($val:tt),* $(,)? ]) => {{
        $crate::json::Value::Array(vec![
            $($crate::json::value!($val),)*
        ])
    }};

    ({ $($key:tt: $val:tt),* $(,)? }) => {{
        $crate::json::Value::Object(std::collections::BTreeMap::from([
            $(($key.to_string(), $crate::json::value!($val)),)*
        ]))
    }};

    ($val:expr) => {{
        $crate::json::to_value(&$val).unwrap()
    }};
}
pub use value;

struct S(Value);

impl Serializer for S {
    type Ok = Value;
    type Error = Error;

    type Struct = Self;
    type Union = Self;
    type Enum = Self;
    type Sequence = Self;
    type Array = Self;
    type Map = Self;

    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(value))
    }

    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_char(value)
    }

    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_i64(i64::from(value))
    }

    fn encode_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(u64::from(value))
    }

    fn encode_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.encode_i64(i64::from(value))
    }

    fn encode_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(u64::from(value))
    }

    fn encode_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.encode_i64(i64::from(value))
    }

    fn encode_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(u64::from(value))
    }

    fn encode_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::Signed(value)))
    }

    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::Unsigned(value)))
    }

    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.encode_f64(f64::from(value))
    }

    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if value.is_nan() || value.is_infinite() {
            Ok(Value::Null)
        } else {
            Ok(Value::Number(Number::Float(value)))
        }
    }

    fn encode_option<T>(self, value: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        if let Some(v) = value {
            v.marshal(self)
        } else {
            Ok(Value::Null)
        }
    }

    fn encode_string(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn encode_wstring(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.encode_string(value)
    }

    fn encode_struct(self, _: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Ok(Self(Value::Object(BTreeMap::new())))
    }

    fn encode_union(self, info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        self.encode_struct(info)
    }

    fn encode_enum(self, _: &str) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        Ok(Self(Value::Array(Vec::new())))
    }

    fn encode_array(self, len: usize) -> Result<<Self as Serializer>::Array, Self::Error> {
        self.encode_sequence(len)
    }

    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        Ok(Self(Value::Object(BTreeMap::new())))
    }
}

impl StructSerializer for S {
    type Ok = Value;
    type Error = Error;

    fn encode_field<T>(&mut self, info: &MemberInfo<'_>, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if let Value::Object(v) = &mut self.0 {
            v.insert(info.name.to_string(), to_value(value)?);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0)
    }
}

impl UnionSerializer for S {
    type Ok = Value;
    type Error = Error;

    fn encode_discriminant<D>(&mut self, discriminant: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        self.encode_field(&DISC_INFO, discriminant)
    }

    fn encode_variant<V>(
        mut self,
        info: &MemberInfo<'_>,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal,
    {
        self.encode_field(info, value)?;
        Ok(self.0)
    }

    fn encode_null(mut self) -> Result<Self::Ok, Self::Error> {
        self.0 = Value::Null;
        Ok(self.0)
    }
}

impl MapSerializer for S {
    type Ok = Value;
    type Error = Error;

    fn encode_pair<K, S>(&mut self, key: &K, value: &S) -> Result<(), Self::Error>
    where
        K: Marshal,
        S: Marshal,
    {
        if let Value::Object(v) = &mut self.0 {
            let key = key.marshal(KeySerializer)?;
            v.insert(key, to_value(value)?);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        StructSerializer::end(self)
    }
}

impl EnumSerializer for S {
    type Ok = Value;
    type Error = Error;

    fn encode_variant<T>(self, name: &str, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        name.marshal(self)
    }
}

impl SeqSerializer for S {
    type Ok = Value;
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if let Value::Array(v) = &mut self.0 {
            v.push(to_value(value)?);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0)
    }
}

impl ArraySerializer for S {
    type Ok = Value;
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        SeqSerializer::encode_next(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0)
    }
}

/// Serialize the given data structore into a `Value` instance.
pub fn to_value<T>(value: &T) -> Result<Value, Error>
where
    T: ?Sized + Marshal,
{
    value.marshal(S(Value::Null))
}
