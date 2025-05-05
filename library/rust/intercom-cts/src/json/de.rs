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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::btree_map;
use std::{mem, vec};

use super::key::KeyDeserializer;
use super::parse::parse;
use super::{Error, Number, Value};
use crate::decode::{
    ArrayDeserializer, Deserializer, EnumDeserializer, EnumVisitor, MapDeserializer,
    SeqDeserializer, StructDeserializer, Type, UnionDeserializer, Unmarshal,
};
use crate::error::Error as _;
use crate::{MemberInfo, TypeInfo, DISC_INFO};

pub struct JsonReader {
    value: Value,
}

impl Deserializer for &mut JsonReader {
    type Struct = Self;
    type Union = Self;
    type Enum = Self;
    type Map = Indexed<btree_map::IntoIter<String, Value>>;
    type Sequence = Indexed<vec::IntoIter<Value>>;
    type Array = Self::Sequence;
    type Error = Error;

    fn peek(&self) -> Result<Option<Type>, Self::Error> {
        let result = match &self.value {
            Value::Null => None,
            Value::Bool(_) => Some(Type::Bool),
            Value::Number(v) => match v {
                Number::Signed(_) => Some(Type::I64),
                Number::Unsigned(_) => Some(Type::U64),
                Number::Float(_) => Some(Type::F64),
            },
            Value::String(_) => Some(Type::String),
            Value::Array(_) => Some(Type::Sequence),
            Value::Object(_) => Some(Type::Map),
        };
        Ok(result)
    }

    fn decode_bool(self) -> Result<bool, Self::Error> {
        match self.value {
            Value::Bool(v) => Ok(v),
            Value::String(ref v) => Ok(v.parse()?),
            _ => Err(Error::custom("expected bool")),
        }
    }

    fn decode_char(self) -> Result<char, Self::Error> {
        if let Value::String(v) = &self.value {
            Ok(v.chars().next().unwrap_or_default())
        } else {
            Err(Error::custom("expected char"))
        }
    }

    fn decode_wchar(self) -> Result<char, Self::Error> {
        self.decode_char()
    }

    fn decode_i8(self) -> Result<i8, Self::Error> {
        Ok(self.decode_i64()?.try_into()?)
    }

    fn decode_u8(self) -> Result<u8, Self::Error> {
        Ok(self.decode_u64()?.try_into()?)
    }

    fn decode_i16(self) -> Result<i16, Self::Error> {
        Ok(self.decode_i64()?.try_into()?)
    }

    fn decode_u16(self) -> Result<u16, Self::Error> {
        Ok(self.decode_u64()?.try_into()?)
    }

    fn decode_i32(self) -> Result<i32, Self::Error> {
        Ok(self.decode_i64()?.try_into()?)
    }

    fn decode_u32(self) -> Result<u32, Self::Error> {
        Ok(self.decode_u64()?.try_into()?)
    }

    fn decode_i64(self) -> Result<i64, Self::Error> {
        match self.value {
            Value::Number(Number::Signed(v)) => Ok(v),
            Value::Number(Number::Unsigned(v)) => Ok((v).try_into()?),
            Value::String(ref v) => Ok(v.parse()?),
            _ => Err(Error::custom("expected number")),
        }
    }

    fn decode_u64(self) -> Result<u64, Self::Error> {
        match self.value {
            Value::Number(Number::Unsigned(v)) => Ok(v),
            Value::String(ref v) => Ok(v.parse()?),
            _ => Err(Error::custom("expected unsigned number")),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.decode_f64().map(|v| v as f32)
    }

    #[allow(clippy::cast_precision_loss)]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        match self.value {
            Value::Number(num) => Ok(match num {
                Number::Signed(v) => v as f64,
                Number::Unsigned(v) => v as f64,
                Number::Float(v) => v,
            }),
            _ => Err(Error::custom("expected float")),
        }
    }

    fn decode_string(self) -> Result<String, Self::Error> {
        if let Value::String(str) = &mut self.value {
            Ok(mem::take(str))
        } else {
            Err(Error::custom("expected string"))
        }
    }

    fn decode_wstring(self) -> Result<String, Self::Error> {
        self.decode_string()
    }

    fn decode_option<T>(self) -> Result<Option<T>, Self::Error>
    where
        T: Unmarshal + Default,
    {
        let value = if self.value.is_null() {
            None
        } else {
            Some(T::unmarshal(&mut *self)?)
        };
        Ok(value)
    }

    fn decode_option_mut<T>(self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        let res = if self.value.is_null() {
            false
        } else {
            value.unmarshal_mut(self)?;
            true
        };
        Ok(res)
    }

    fn decode_struct(self, _: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Ok(self)
    }

    fn decode_union(self, _: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        Ok(self)
    }

    fn decode_enum(self, _name: &str) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    fn decode_array(self, _len: usize) -> Result<Self::Array, Self::Error> {
        self.decode_sequence()
    }

    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        if let Value::Array(obj) = mem::take(&mut self.value) {
            Ok(Indexed(obj.into_iter()))
        } else {
            Err(Error::custom("expected sequence"))
        }
    }

    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        if let Value::Object(obj) = mem::take(&mut self.value) {
            Ok(Indexed(obj.into_iter()))
        } else {
            Err(Error::custom("expected object"))
        }
    }
}

impl StructDeserializer for &mut JsonReader {
    type Ok = ();
    type Error = Error;

    fn decode_field<T>(&mut self, info: &MemberInfo<'_>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        let Value::Object(ref mut obj) = self.value else {
            return Err(Error::custom("expected object"));
        };

        // Replace the value instead of removing the entry to prevent the tree
        // from being balanced.
        if let Some(val) = obj.get_mut(info.name).map(mem::take) {
            let mut reader = JsonReader { value: val };
            value.unmarshal_mut(&mut reader)?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl UnionDeserializer for &mut JsonReader {
    type Ok = ();
    type Error = Error;

    fn decode_discriminant<T>(&mut self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        if !self.value.is_null() {
            self.decode_field(&DISC_INFO, value)?;
        }
        Ok(())
    }

    fn decode_variant<T>(mut self, info: &MemberInfo<'_>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        self.decode_field(info, value)
    }
}

impl EnumDeserializer for &mut JsonReader {
    type Error = Error;

    fn decode_enumerator<T>(self, visitor: T) -> Result<T, Self::Error>
    where
        T: EnumVisitor + Unmarshal,
    {
        let value = match self.value {
            Value::Number(_) => visitor.member_id(&mut *self)?,
            Value::String(ref name) => visitor.member_field::<Self>(name)?,
            _ => return Err(Error::custom("expected enum")),
        };
        Ok(value)
    }
}

pub struct Indexed<T: Iterator>(T);

impl<I: Iterator<Item = Value>> SeqDeserializer for Indexed<I> {
    type Error = Error;

    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        if let Some(elem) = self.0.next() {
            let mut reader = JsonReader { value: elem };
            value.unmarshal_mut(&mut reader)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint().1
    }
}

impl<I: Iterator<Item = Value>> ArrayDeserializer for Indexed<I> {
    type Error = Error;

    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        SeqDeserializer::decode_next(self, value)
    }
}

impl<I: Iterator<Item = (String, Value)>> MapDeserializer for Indexed<I> {
    type Error = Error;

    fn decode_pair<K, V>(&mut self, key: &mut K, value: &mut V) -> Result<bool, Self::Error>
    where
        K: Unmarshal,
        V: Unmarshal,
    {
        if let Some(entry) = self.0.next() {
            let mut reader = JsonReader {
                value: Value::String(entry.0),
            };
            key.unmarshal_mut(KeyDeserializer(&mut reader))?;

            let mut reader = JsonReader { value: entry.1 };
            value.unmarshal_mut(&mut reader)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint().1
    }
}

pub fn from_str<T>(input: &str) -> Result<T, Error>
where
    T: Unmarshal + Default,
{
    let mut value = T::default();
    from_string_mut(input, &mut value)?;
    Ok(value)
}

pub fn from_string_mut<T>(input: &str, out: &mut T) -> Result<(), Error>
where
    T: Unmarshal,
{
    let value = parse(input)?;
    let mut reader = JsonReader { value };
    out.unmarshal_mut(&mut reader)
}

pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: Unmarshal + Default,
{
    let mut reader = JsonReader { value };
    T::unmarshal(&mut reader)
}

pub fn from_value_mut<T>(value: Value, out: &mut T) -> Result<(), Error>
where
    T: Unmarshal,
{
    let mut reader = JsonReader { value };
    out.unmarshal_mut(&mut reader)
}
