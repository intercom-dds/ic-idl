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

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{BuildHasher, Hash};
use std::mem;

use super::error::Error;
use crate::{WChar, WString};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    String,
    Sequence,
    Map,
}

pub trait Deserializer {
    /// Error produced by the deserializer.
    type Error: Error;

    /// Deserializer used to deserialize `struct`s.
    type Struct: StructDeserializer<Error = Self::Error>;

    /// Deserializer used to deserialize complex `enum`s.
    type Union: UnionDeserializer<Error = Self::Error>;

    /// Deserializer used to deserialize plain, C-like `enums`s.
    type Enum: EnumDeserializer<Error = Self::Error>;

    /// Deserializer used to deserialize sequences.
    type Sequence: SeqDeserializer<Error = Self::Error>;

    /// Deserializer used to deserialize fixed-size arrays.
    type Array: ArrayDeserializer<Error = Self::Error>;

    /// Deserializer used to deserialize maps.
    type Map: MapDeserializer<Error = Self::Error>;

    /// Deserializers for self-described formats like `JSON`, `TOML`, etc. can
    /// use this function as a way to tell the unmarshaller what comes next.
    /// This can be used to drive the serializer for types that do not have
    /// static, compile-time definitions.
    ///
    /// Formats that are not self-described should not implement this function.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use intercom_cts::decode::{Deserializer, Type}, Unmarshal};
    /// # use std::collections::HashMap;
    /// #
    /// enum Value {
    ///     Bool(bool),
    ///     String(String),
    ///     Object(HashMap<String, Value>),
    /// }
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         if let Some(next) = archive.peek()? {
    ///             match next {
    ///                 Type::Bool => *self = Value::Bool(archive.decode_bool()?),
    ///                 Type::String => *self = Value::String(archive.decode_string()?),
    ///                 Type::Map => *self = Value::Object(HashMap::unmarshal(archive)?),
    ///                 _ => todo!(),
    ///             }
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn peek(&self) -> Result<Option<Type>, Self::Error> {
        Err(Self::Error::custom(
            "`peek` is not supported by this deserializer",
        ))
    }

    /// Deserialize a `bool` value.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(bool);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_bool()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_bool(self) -> Result<bool, Self::Error>;

    /// Deserialize a character.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(char);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_char()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_char(self) -> Result<char, Self::Error>;

    /// Deserialize a wide character. The returned character is represented as
    /// typical UTF-8 `char`, but it was deserialized as an UTF-16 character.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(char);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_wchar()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_wchar(self) -> Result<char, Self::Error>;

    /// Deserialize a signed 8-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(i8);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_i8()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_i8(self) -> Result<i8, Self::Error>;

    /// Deserialize an unsigned 8-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(u8);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_u8()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_u8(self) -> Result<u8, Self::Error>;

    /// Deserialize a signed 16-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(i16);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_i16()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_i16(self) -> Result<i16, Self::Error>;

    /// Deserialize an unsigned 16-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(u16);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_u16()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_u16(self) -> Result<u16, Self::Error>;

    /// Deserialize a signed 32-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(i32);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_i32()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_i32(self) -> Result<i32, Self::Error>;

    /// Deserialize an unsigned 32-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(u32);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_u32()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_u32(self) -> Result<u32, Self::Error>;

    /// Deserialize a signed 64-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(i64);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_i64()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_i64(self) -> Result<i64, Self::Error>;

    /// Deserialize an unsigned 64-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(u64);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_u64()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_u64(self) -> Result<u64, Self::Error>;

    /// Deserialize a 32-bit floating point.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(f32);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_f32()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_f32(self) -> Result<f32, Self::Error>;

    /// Deserialize a 64-bit floating point.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(f64);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_f64()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_f64(self) -> Result<f64, Self::Error>;

    /// Deserialize a string.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(String);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_string()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_string(self) -> Result<String, Self::Error>;

    /// Deserialize a wide-character string. Similar to [`decode_wchar`], the
    /// returned string is represented as a typical UTF-8 [`String`], but it
    /// was deserialized as an UTF-16 string.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value(String);
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_wstring()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_wstring(self) -> Result<String, Self::Error>;

    /// Deserialize an optional value.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{decode::Deserializer, Unmarshal};
    /// #
    /// struct Value<T>(Option<T>);
    ///
    /// impl<T> Unmarshal for Value<T>
    /// where
    ///     T: Unmarshal + Default,
    /// {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         self.0 = archive.decode_option()?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_option<T>(self) -> Result<Option<T>, Self::Error>
    where
        T: Unmarshal + Default;

    /// Deserialize a struct.
    ///
    /// # Example
    /// ```
    /// use intercom_cts::Unmarshal;
    /// use intercom_cts::decode::{Deserializer, StructDeserializer};
    ///
    /// struct Value {
    ///     key: u32,
    ///     value: String,
    /// }
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         let mut state = archive.decode_struct("Value")?;
    ///         state.decode_field(0, "key", &mut self.key)?;
    ///         state.decode_field(1, "value", &mut self.value)?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_struct(self, name: &str) -> Result<Self::Struct, Self::Error>;

    /// Deserialize a complex enum.
    ///
    /// # Example
    /// ```
    /// use intercom_cts::Unmarshal;
    /// use intercom_cts::error::Error;
    /// use intercom_cts::decode::{Deserializer, UnionDeserializer};
    ///
    /// enum Value {
    ///     Int(usize),
    ///     String(String),
    /// }
    ///
    /// impl Unmarshal for Value {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         let mut state = archive.decode_union("Value")?;
    ///         let mut disc = 0_i32;
    ///         state.decode_discriminant(&mut disc)?;
    ///
    ///         *self = match disc {
    ///             123 => {
    ///                 let mut value = 0;
    ///                 state.decode_variant(1, "Int", &mut value)?;
    ///                 Self::Int(value)
    ///             }
    ///             456 => {
    ///                 let mut value = String::default();
    ///                 state.decode_variant(2, "String", &mut value)?;
    ///                 Self::String(value)
    ///             },
    ///             _ => return Err(D::Error::custom("Unknown discriminant")),
    ///         };
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn decode_union(self, name: &str) -> Result<Self::Union, Self::Error>;

    fn decode_enum(self, name: &str) -> Result<Self::Enum, Self::Error>;

    /// Deserialize a sequence.
    ///
    /// # Example
    ///
    /// ```
    /// use intercom_cts::Unmarshal;
    /// use intercom_cts::decode::{Deserializer, SeqDeserializer};
    ///
    /// struct Value<T>(Vec<T>);
    ///
    /// impl<T> Unmarshal for Value<T>
    /// where
    ///     T: Unmarshal + Default,
    /// {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         let mut state = archive.decode_sequence()?;
    ///         let mut value = T::default();
    ///         while state.decode_next(&mut value)? {
    ///             self.0.push(std::mem::take(&mut value));
    ///         }
    ///         Ok(())
    ///     }
    /// }
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error>;

    /// Deserialize a fixed-size array.
    ///
    /// # Example
    ///
    /// ```
    /// use intercom_cts::Unmarshal;
    /// use intercom_cts::decode::{Deserializer, ArrayDeserializer};
    /// use intercom_cts::error::Error;
    ///
    /// struct Value<T>([T; 128]);
    ///
    /// impl<T: Unmarshal> Unmarshal for Value<T> {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         let mut state = archive.decode_array(128)?;
    ///         for elem in &mut self.0 {
    ///             if !state.decode_next(elem)? {
    ///                 return Err(D::Error::custom("Missing value in array"));
    ///             }
    ///         }
    ///         Ok(())
    ///     }
    /// }
    fn decode_array(self, len: usize) -> Result<Self::Array, Self::Error>;

    /// Deserialize a map.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use intercom_cts::Unmarshal;
    /// use intercom_cts::decode::{Deserializer, MapDeserializer};
    ///
    /// struct Value<T>(HashMap<String, T>);
    ///
    /// impl<T> Unmarshal for Value<T>
    /// where
    ///     T: Unmarshal + Default,
    /// {
    ///     fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    ///     where
    ///         D: Deserializer,
    ///     {
    ///         let mut state = archive.decode_map()?;
    ///         let mut key = String::default();
    ///         let mut value = T::default();
    ///         while state.decode_pair(&mut key, &mut value)? {
    ///             self.0.insert(std::mem::take(&mut key), std::mem::take(&mut value));
    ///         }
    ///         Ok(())
    ///     }
    /// }
    fn decode_map(self) -> Result<Self::Map, Self::Error>;
}

pub trait StructDeserializer {
    type Error: Error;

    fn decode_field<T>(&mut self, id: usize, key: &str, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal;
}

pub trait UnionDeserializer {
    type Ok;
    type Error: Error;

    fn decode_discriminant<T>(&mut self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal;

    fn decode_variant<T>(
        self,
        id: usize,
        name: &str,
        value: &mut T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Unmarshal;
}

pub trait EnumVisitor {
    fn member_id<D>(self, de: D) -> Result<Self, D::Error>
    where
        Self: Sized,
        D: Deserializer;

    fn member_field<D>(self, name: &str) -> Result<Self, D::Error>
    where
        Self: Sized,
        D: Deserializer;
}

pub trait EnumDeserializer {
    type Error: Error;

    fn decode_enumerator<T>(self, visitor: T) -> Result<T, Self::Error>
    where
        T: Unmarshal + EnumVisitor;
}

pub trait SeqDeserializer {
    type Error: Error;

    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal;

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait ArrayDeserializer {
    type Error: Error;

    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal;
}

pub trait MapDeserializer {
    type Error: Error;

    fn decode_pair<K, V>(&mut self, key: &mut K, value: &mut V) -> Result<bool, Self::Error>
    where
        K: Unmarshal,
        V: Unmarshal;

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait Unmarshal {
    /// Stateless unmarshaling that produces a new value of type T. Values that
    /// were not specified in the input will be default constructed.
    fn unmarshal<D>(archive: D) -> Result<Self, D::Error>
    where
        D: Deserializer,
        Self: Default,
    {
        let mut value = Self::default();
        value.unmarshal_mut(archive)?;
        Ok(value)
    }

    /// Stateful, mutating unmarshaling. Instead of producing a new value of
    /// type T, it will mutate an existing value in-place. In most cases
    /// stateless unmarshaling is preferable; unless the type requires an
    /// external descriptor to serialize properly, it is preferable to use the
    /// stateless counterpart [`Unmarshal::unmarshal`].
    ///
    /// Stateful marshaling is necessary in cases where it is not possible to
    /// unmarshal a type solely based on its definition. One such example is
    /// `DynamicData`, whose description is defined at run-time by an
    /// associated `DynamicType`.
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer;
}

impl Unmarshal for bool {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_bool()?;
        Ok(())
    }
}

impl Unmarshal for char {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_char()?;
        Ok(())
    }
}

impl Unmarshal for i8 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_i8()?;
        Ok(())
    }
}

impl Unmarshal for u8 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_u8()?;
        Ok(())
    }
}

impl Unmarshal for i16 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_i16()?;
        Ok(())
    }
}

impl Unmarshal for u16 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_u16()?;
        Ok(())
    }
}

impl Unmarshal for i32 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_i32()?;
        Ok(())
    }
}

impl Unmarshal for u32 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_u32()?;
        Ok(())
    }
}

impl Unmarshal for i64 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_i64()?;
        Ok(())
    }
}

impl Unmarshal for u64 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive.decode_u64().map(|v| *self = v)
    }
}

impl Unmarshal for isize {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive
            .decode_i64()?
            .try_into()
            .map_err(D::Error::custom)
            .map(|v| *self = v)
    }
}

impl Unmarshal for usize {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive
            .decode_u64()?
            .try_into()
            .map_err(D::Error::custom)
            .map(|v| *self = v)
    }
}

impl Unmarshal for f32 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive.decode_f32().map(|v| *self = v)
    }
}

impl Unmarshal for f64 {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive.decode_f64().map(|v| *self = v)
    }
}

impl Unmarshal for String {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive.decode_string().map(|v| *self = v)
    }
}

impl Unmarshal for WChar<&mut char> {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive.decode_wchar().map(|v| *self.0 = v)
    }
}

impl Unmarshal for WString<&mut String> {
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        archive.decode_wstring().map(|v| *self.0 = v)
    }
}

fn reserve_hint(len: Option<usize>) -> usize {
    const MAX: usize = 128;
    std::cmp::min(len.unwrap_or(0), MAX)
}

impl<T> Unmarshal for Vec<T>
where
    T: Unmarshal + Default,
{
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.clear();
        let mut state = archive.decode_sequence()?;
        self.reserve(reserve_hint(state.size_hint()));

        let mut value = T::default();
        while state.decode_next(&mut value)? {
            self.push(mem::take(&mut value));
        }
        Ok(())
    }
}

impl<T> Unmarshal for Option<T>
where
    T: Unmarshal + Default,
{
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        *self = archive.decode_option()?;
        Ok(())
    }
}

impl<T> Unmarshal for Box<T>
where
    T: Unmarshal,
{
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        (**self).unmarshal_mut(archive)
    }
}

impl<T> Unmarshal for &mut T
where
    T: ?Sized + Unmarshal,
{
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        (**self).unmarshal_mut(archive)
    }
}

impl<T, const N: usize> Unmarshal for [T; N]
where
    T: Unmarshal,
{
    fn unmarshal<D>(archive: D) -> Result<Self, D::Error>
    where
        D: Deserializer,
        Self: Default,
    {
        let mut value = Self::default();
        value.unmarshal_mut(archive)?;
        Ok(value)
    }

    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        let mut state = archive.decode_array(N)?;
        for elem in self.iter_mut() {
            if !state.decode_next(elem)? {
                return Err(D::Error::custom("too few elements in array"));
            }
        }
        Ok(())
    }
}

impl<T, S> Unmarshal for HashSet<T, S>
where
    T: Unmarshal + Hash + Eq + Default,
    S: BuildHasher + Default,
{
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.clear();
        let mut state = archive.decode_sequence()?;
        self.reserve(reserve_hint(state.size_hint()));

        let mut value = T::default();
        while state.decode_next(&mut value)? {
            self.insert(mem::take(&mut value));
        }
        Ok(())
    }
}

impl<K, V, S> Unmarshal for HashMap<K, V, S>
where
    K: Unmarshal + Hash + Eq + Default,
    V: Unmarshal + Default,
    S: BuildHasher + Default,
{
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.clear();
        let mut state = archive.decode_map()?;
        self.reserve(reserve_hint(state.size_hint()));

        let mut key = K::default();
        let mut value = V::default();
        while state.decode_pair(&mut key, &mut value)? {
            self.insert(mem::take(&mut key), mem::take(&mut value));
        }
        Ok(())
    }
}

impl<K, V> Unmarshal for BTreeMap<K, V>
where
    K: Unmarshal + Ord + Default,
    V: Unmarshal + Default,
{
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.clear();
        let mut state = archive.decode_map()?;
        loop {
            let mut key = K::default();
            let mut value = V::default();
            if state.decode_pair(&mut key, &mut value)? {
                self.insert(key, value);
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl<T> Unmarshal for BTreeSet<T>
where
    T: Unmarshal + Ord + Default,
{
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.clear();
        let mut value = T::default();
        let mut state = archive.decode_sequence()?;
        while state.decode_next(&mut value)? {
            self.insert(value);
            value = T::default();
        }
        Ok(())
    }
}

impl<T> Unmarshal for std::ops::Range<T>
where
    T: Unmarshal,
{
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        let mut state = archive.decode_struct("Range<T>")?;
        state.decode_field(0, "start", &mut self.start)?;
        state.decode_field(1, "end", &mut self.end)
    }
}
