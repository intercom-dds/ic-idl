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

#![allow(unused_variables)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::BuildHasher;

use super::error::Error;
use crate::cdr1::MemberFlag;
use crate::{TypeFlag, WChar, WString};

pub trait Serializer: Sized {
    /// The type returned by the serializer once serialization has finished.
    type Ok;

    /// Error produced by the serializer.
    type Error: Error;

    /// Serializer used to serialize `struct`s.
    type Struct: StructSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Serializer used to serialize complex `enum`s.
    type Union: UnionSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Serializer used to serialize plain, C-like `enum`s.
    type Enum: EnumSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Serializer used to serialize sequences.
    type Sequence: SeqSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Serializer used to serialize fixed-size arrays.
    type Array: ArraySerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Serializer used to serialize maps.
    type Map: MapSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Serialize a `bool` value.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(bool);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_bool(self.0)
    ///     }
    /// }
    /// ```
    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error>;

    /// Serialize a character.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(char);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_char(self.0)
    ///     }
    /// }
    /// ```
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error>;

    /// Serialize a wide character. The passed value is represented as a
    /// typical UTF-8 `char`, but the intended target representation is a
    /// UTF-16 character.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(char);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_wchar(self.0)
    ///     }
    /// }
    /// ```
    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error>;

    /// Serialize a signed 8-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(i8);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_i8(self.0)
    ///     }
    /// }
    /// ```
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error>;

    /// Serialize an unsigned 8-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(u8);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_u8(self.0)
    ///     }
    /// }
    /// ```
    fn encode_u8(self, value: u8) -> Result<Self::Ok, Self::Error>;

    /// Serialize a signed 16-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(i16);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_i16(self.0)
    ///     }
    /// }
    /// ```
    fn encode_i16(self, value: i16) -> Result<Self::Ok, Self::Error>;

    /// Serialize an unsigned 16-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(u16);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_u16(self.0)
    ///     }
    /// }
    /// ```
    fn encode_u16(self, value: u16) -> Result<Self::Ok, Self::Error>;

    /// Serialize a signed 32-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(i32);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_i32(self.0)
    ///     }
    /// }
    /// ```
    fn encode_i32(self, value: i32) -> Result<Self::Ok, Self::Error>;

    /// Serialize an unsigned 32-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(u32);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_u32(self.0)
    ///     }
    /// }
    /// ```
    fn encode_u32(self, value: u32) -> Result<Self::Ok, Self::Error>;

    /// Serialize a signed 64-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(i64);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_i64(self.0)
    ///     }
    /// }
    /// ```
    fn encode_i64(self, value: i64) -> Result<Self::Ok, Self::Error>;

    /// Serialize an unsigned 64-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(u64);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_u64(self.0)
    ///     }
    /// }
    /// ```
    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error>;

    /// Serialize a 32-bit floating point.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(f32);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_f32(self.0)
    ///     }
    /// }
    /// ```
    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error>;

    /// Serialize a 64-bit floating point value.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(f64);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_f64(self.0)
    ///     }
    /// }
    /// ```
    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `str` value.
    ///
    /// # Example
    ///
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(String);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_string(self.0.as_str())
    ///     }
    /// }
    /// ```
    fn encode_string(self, value: &str) -> Result<Self::Ok, Self::Error>;

    /// Serialize a wide-character `str` value. Similar to [`encode_wchar`],
    /// the passed string is represented as a typical UTF-8 [`str`], but the
    /// intended target representation is a UTF-16 string.
    ///
    /// # Example
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value(String);
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_wstring(self.0.as_str())
    ///     }
    /// }
    /// ```
    ///
    /// [`str`]: std::str::str
    fn encode_wstring(self, value: &str) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `Option` value.
    ///
    /// # Example
    /// ```
    /// # use intercom_cts::{encode::Serializer, Marshal};
    /// #
    /// struct Value<T>(Option<T>);
    ///
    /// impl<T: Marshal> Marshal for Value<T> {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         archive.encode_option(&self.0)
    ///     }
    /// }
    /// ```
    fn encode_option<T>(self, value: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal;

    /// Serialize a struct.
    fn encode_struct(self, info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error>;

    /// Serialize a complex enum.
    fn encode_union(self, info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error>;

    /// Serialize a plain, C-like enum.
    ///
    /// # Example
    /// ```
    /// use intercom_cts::Marshal;
    /// use intercom_cts::encode::{EnumSerializer, Serializer};
    ///
    /// enum Value {
    ///     Red,
    ///     Green,
    ///     Blue = 9
    /// }
    ///
    /// impl Marshal for Value {
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         let state = archive.encode_enum("Value")?;
    ///         match self {
    ///             Self::Red => state.encode_variant("Red", 0),
    ///             Self::Green => state.encode_variant("Green", 1),
    ///             Self::Blue => state.encode_variant("Blue", 9),
    ///         }
    ///     }
    /// }
    /// ```
    fn encode_enum(self, name: &str) -> Result<Self::Enum, Self::Error>;

    /// Serialize a sequence,
    ///
    /// # Example
    ///
    /// ```
    /// use intercom_cts::Marshal;
    /// use intercom_cts::encode::{Serializer, SeqSerializer};
    ///
    /// struct Value<T>(Vec<T>);
    ///
    /// impl<T> Marshal for Value<T>
    /// where
    ///     T: Marshal,
    /// {
    ///     #[inline]
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         let mut state = archive.encode_sequence(self.0.len())?;
    ///         for val in &self.0 {
    ///             state.encode_next(val)?;
    ///         }
    ///         state.end()
    ///     }
    /// }
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error>;

    /// Serialize a fixed-size array.
    ///
    /// # Example
    ///
    /// ```
    /// use intercom_cts::Marshal;
    /// use intercom_cts::encode::{ArraySerializer, Serializer};
    ///
    /// struct Value<T>([T; 128]);
    ///
    /// impl<T> Marshal for Value<T>
    /// where
    ///     T: Marshal,
    /// {
    ///     #[inline]
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         let mut state = archive.encode_array(self.0.len())?;
    ///         for val in &self.0 {
    ///             state.encode_next(val)?;
    ///         }
    ///         state.end()
    ///     }
    /// }
    fn encode_array(self, len: usize) -> Result<Self::Array, Self::Error>;

    /// Serialize a map as key-value pairs.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// use intercom_cts::Marshal;
    /// use intercom_cts::encode::{MapSerializer, Serializer};
    ///
    /// struct Value<T>(HashMap<String, T>);
    ///
    /// impl<T> Marshal for Value<T>
    /// where
    ///     T: Marshal,
    /// {
    ///     #[inline]
    ///     fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         let mut state = archive.encode_map(self.0.len())?;
    ///         for (key, value) in &self.0 {
    ///             state.encode_pair(key, value)?;
    ///         }
    ///         state.end()
    ///     }
    /// }
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error>;
}

#[derive(Debug)]
pub struct MemberInfo<'a> {
    pub name: &'a str,
    pub member_id: u32,
    pub flags: MemberFlag,
}

#[derive(Debug)]
pub struct TypeInfo<'a> {
    pub name: &'a str,
    pub flags: TypeFlag,
    pub kind: TypeKind,
    pub key_kind: TypeKind,
    pub element_kind: TypeKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKind {
    None,
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
    Char8,
    Char16,
    Alias,
    Struct,
    Union,
    Bitmask,
    Enum,
    String8,
    String16,
    Annotation,
    Array,
    Map,
    Sequence,
}

#[doc(hidden)]
pub const DISC_INFO: MemberInfo<'static> = MemberInfo {
    name: "$discriminator",
    member_id: 0,
    flags: MemberFlag::IS_MUST_UNDERSTAND,
};

pub trait StructSerializer {
    type Ok;
    type Error: Error;

    fn encode_field<T>(&mut self, info: &MemberInfo<'_>, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal;

    fn encode_optional<T>(
        &mut self,
        info: &MemberInfo<'_>,
        value: &Option<T>,
    ) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        self.encode_field(info, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait UnionSerializer {
    type Ok;
    type Error: Error;

    fn encode_discriminant<D>(&mut self, discriminant: &D) -> Result<(), Self::Error>
    where
        D: Marshal;

    fn encode_variant<V>(self, info: &MemberInfo<'_>, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal;

    fn encode_null(self) -> Result<Self::Ok, Self::Error>;
}

pub trait ArraySerializer {
    type Ok;
    type Error: Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait SeqSerializer {
    type Ok;
    type Error: Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait EnumSerializer {
    type Ok;
    type Error: Error;

    fn encode_variant<T>(self, name: &str, value: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal;
}

pub trait MapSerializer {
    type Ok;
    type Error: Error;

    fn encode_pair<K, S>(&mut self, key: &K, value: &S) -> Result<(), Self::Error>
    where
        K: Marshal,
        S: Marshal;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait Marshal {
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl Marshal for bool {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_bool(*self)
    }
}

impl Marshal for char {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_char(*self)
    }
}

impl Marshal for i8 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_i8(*self)
    }
}

impl Marshal for u8 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_u8(*self)
    }
}

impl Marshal for i16 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_i16(*self)
    }
}

impl Marshal for u16 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_u16(*self)
    }
}

impl Marshal for i32 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_i32(*self)
    }
}

impl Marshal for u32 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_u32(*self)
    }
}

impl Marshal for i64 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_i64(*self)
    }
}

impl Marshal for u64 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_u64(*self)
    }
}

impl Marshal for f32 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_f32(*self)
    }
}

impl Marshal for f64 {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_f64(*self)
    }
}

impl Marshal for isize {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_i64((*self).try_into().map_err(S::Error::custom)?)
    }
}

impl Marshal for usize {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_u64((*self).try_into().map_err(S::Error::custom)?)
    }
}

impl Marshal for str {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_string(self)
    }
}

impl Marshal for String {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_string(self.as_str())
    }
}

impl Marshal for WChar<&char> {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_wchar(*self.0)
    }
}

impl Marshal for WString<&str> {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_wstring(self.0)
    }
}

impl Marshal for WString<&String> {
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_wstring(self.0)
    }
}

impl<T> Marshal for Option<T>
where
    T: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        archive.encode_option(self)
    }
}

impl<T> Marshal for Box<T>
where
    T: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().marshal(archive)
    }
}

impl<T> Marshal for &T
where
    T: ?Sized + Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**self).marshal(archive)
    }
}

impl<T> Marshal for Vec<T>
where
    T: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_sequence(self.len())?;
        for val in self {
            state.encode_next(val)?;
        }
        state.end()
    }
}

impl<T> Marshal for [T]
where
    T: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_sequence(self.len())?;
        self.iter().try_for_each(|v| state.encode_next(v))?;
        state.end()
    }
}

impl<T, const N: usize> Marshal for [T; N]
where
    T: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_array(N)?;
        self.iter().try_for_each(|v| state.encode_next(v))?;
        state.end()
    }
}

impl<T> Marshal for BTreeSet<T>
where
    T: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_sequence(self.len())?;
        for elem in self {
            state.encode_next(elem)?;
        }
        state.end()
    }
}

impl<K, V> Marshal for BTreeMap<K, V>
where
    K: Marshal,
    V: Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_map(self.len())?;
        for (key, value) in self {
            state.encode_pair(key, value)?;
        }
        state.end()
    }
}

impl<T, H> Marshal for HashSet<T, H>
where
    T: Marshal,
    H: BuildHasher,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_sequence(self.len())?;
        for elem in self {
            state.encode_next(elem)?;
        }
        state.end()
    }
}

impl<K, V, H> Marshal for HashMap<K, V, H>
where
    K: Marshal,
    V: Marshal,
    H: BuildHasher,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_map(self.len())?;
        for (key, value) in self {
            state.encode_pair(key, value)?;
        }
        state.end()
    }
}
