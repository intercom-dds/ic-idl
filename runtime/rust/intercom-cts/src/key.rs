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

//! Machinery for key-only serialization.

use crate::decode::{Deserializer, StructDeserializer, UnionDeserializer};
use crate::encode::{Serializer, StructSerializer, UnionSerializer};
use crate::{Marshal, MemberFlag, MemberInfo, TypeFlag, TypeInfo, Unmarshal};

struct Key<T> {
    value: T,
    implicit: bool,
}

impl<T: Marshal> Marshal for Key<T> {
    fn marshal<'a, S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer<'a>,
    {
        let adapter = KeyAdapter {
            inner: archive,
            implicit: self.implicit,
            depth: 1,
        };
        self.value.marshal(adapter)
    }
}

impl<T: Unmarshal> Unmarshal for Key<T> {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        let adapter = KeyAdapter {
            inner: archive,
            implicit: self.implicit,
            depth: 1,
        };
        self.value.unmarshal_mut(adapter)
    }
}

/// Adapter for key-only serialization. Wraps any [`Serializer`] or
/// [`Deserializer`] and filters out non-key members during traversal.
pub struct KeyAdapter<S> {
    inner: S,
    implicit: bool,
    depth: usize,
}

impl<S> KeyAdapter<S> {
    #[inline]
    pub const fn new(inner: S) -> Self {
        Self {
            inner,
            implicit: false,
            depth: 0,
        }
    }
}

impl<'a, S: Serializer<'a>> Serializer<'a> for KeyAdapter<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type Struct = KeyAdapter<S::Struct>;
    type Union = KeyAdapter<S::Union>;
    type Enum = S::Enum;
    type Bitmask = S::Bitmask;
    type Sequence = S::Sequence;
    type Array = S::Array;
    type Map = S::Map;

    #[inline]
    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_bool(value)
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_char(value)
    }

    #[inline]
    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_wchar(value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_i8(value)
    }

    #[inline]
    fn encode_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_u8(value)
    }

    #[inline]
    fn encode_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_i16(value)
    }

    #[inline]
    fn encode_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_u16(value)
    }

    #[inline]
    fn encode_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_i32(value)
    }

    #[inline]
    fn encode_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_u32(value)
    }

    #[inline]
    fn encode_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_i64(value)
    }

    #[inline]
    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_u64(value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_f32(value)
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_f64(value)
    }

    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_unit()
    }

    #[inline]
    fn encode_string(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_string(value)
    }

    #[inline]
    fn encode_wstring(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_wstring(value)
    }

    #[inline]
    fn encode_option<T>(self, value: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        self.inner.encode_option(value)
    }

    #[inline]
    fn encode_struct(self, info: &TypeInfo<'a>) -> Result<Self::Struct, Self::Error> {
        let implicit =
            self.implicit || (self.depth > 0 && !info.flags.contains(TypeFlag::IS_KEYED));
        self.inner.encode_struct(info).map(|inner| KeyAdapter {
            inner,
            implicit,
            depth: self.depth + 1,
        })
    }

    #[inline]
    fn encode_union(self, info: &TypeInfo<'a>) -> Result<Self::Union, Self::Error> {
        let implicit = self.implicit || info.flags.contains(TypeFlag::IS_KEYED);
        self.inner.encode_union(info).map(|inner| KeyAdapter {
            inner,
            implicit,
            depth: self.depth + 1,
        })
    }

    #[inline]
    fn encode_enum(self, info: &TypeInfo<'a>) -> Result<Self::Enum, Self::Error> {
        self.inner.encode_enum(info)
    }

    #[inline]
    fn encode_bitmask(self, info: &TypeInfo<'a>) -> Result<Self::Bitmask, Self::Error> {
        self.inner.encode_bitmask(info)
    }

    #[inline]
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error> {
        self.inner.encode_sequence(len)
    }

    #[inline]
    fn encode_array(self, len: usize) -> Result<Self::Array, Self::Error> {
        self.inner.encode_array(len)
    }

    #[inline]
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error> {
        self.inner.encode_map(len)
    }
}

impl<'a, S: StructSerializer<'a>> StructSerializer<'a> for KeyAdapter<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    #[inline]
    fn encode_field<T>(&mut self, info: &MemberInfo<'a>, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if info.flags.contains(MemberFlag::IS_KEY) || self.implicit {
            self.inner.encode_field(
                info,
                &Key {
                    value,
                    implicit: self.implicit,
                },
            )?;
        }
        Ok(())
    }

    #[inline]
    fn encode_optional<T>(&mut self, _: &MemberInfo<'a>, _: &Option<T>) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<'a, S: UnionSerializer<'a>> UnionSerializer<'a> for KeyAdapter<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    #[inline]
    fn encode_discriminant<D>(&mut self, discriminant: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        self.inner.encode_discriminant(discriminant)
    }

    #[inline]
    fn encode_variant<V>(self, _: &MemberInfo<'a>, _: &V) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal,
    {
        self.inner.encode_null()
    }

    #[inline]
    fn encode_null(self) -> Result<Self::Ok, Self::Error> {
        self.inner.encode_null()
    }
}

impl<'a, D: Deserializer<'a>> Deserializer<'a> for KeyAdapter<D> {
    type Error = D::Error;
    type Struct = KeyAdapter<D::Struct>;
    type Union = KeyAdapter<D::Union>;
    type Enum = D::Enum;
    type Bitmask = D::Bitmask;
    type Sequence = D::Sequence;
    type Array = D::Array;
    type Map = D::Map;
    type Option = D::Option;

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.inner.decode_bool()
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        self.inner.decode_char()
    }

    #[inline]
    fn decode_wchar(self) -> Result<char, Self::Error> {
        self.inner.decode_wchar()
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.inner.decode_i8()
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.inner.decode_u8()
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.inner.decode_i16()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.inner.decode_u16()
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.inner.decode_i32()
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.inner.decode_u32()
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.inner.decode_i64()
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.inner.decode_u64()
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.inner.decode_f32()
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.inner.decode_f64()
    }

    #[inline]
    fn decode_string(self) -> Result<String, Self::Error> {
        self.inner.decode_string()
    }

    #[inline]
    fn decode_wstring(self) -> Result<String, Self::Error> {
        self.inner.decode_wstring()
    }

    #[inline]
    fn decode_struct(self, info: &TypeInfo<'a>) -> Result<Self::Struct, Self::Error> {
        let implicit =
            self.implicit || (self.depth > 0 && !info.flags.contains(TypeFlag::IS_KEYED));
        self.inner.decode_struct(info).map(|inner| KeyAdapter {
            inner,
            implicit,
            depth: self.depth + 1,
        })
    }

    #[inline]
    fn decode_union(self, info: &TypeInfo<'a>) -> Result<Self::Union, Self::Error> {
        let implicit = self.implicit || info.flags.contains(TypeFlag::IS_KEYED);
        self.inner.decode_union(info).map(|inner| KeyAdapter {
            inner,
            implicit,
            depth: self.depth + 1,
        })
    }

    #[inline]
    fn decode_enum(self, info: &TypeInfo<'a>) -> Result<Self::Enum, Self::Error> {
        self.inner.decode_enum(info)
    }

    fn decode_bitmask(self, info: &TypeInfo<'a>) -> Result<Self::Bitmask, Self::Error> {
        self.inner.decode_bitmask(info)
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        self.inner.decode_sequence()
    }

    #[inline]
    fn decode_array(self, len: usize) -> Result<Self::Array, Self::Error> {
        self.inner.decode_array(len)
    }

    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        self.inner.decode_map()
    }

    #[inline]
    fn begin_decode_option(self) -> Result<Self::Option, Self::Error> {
        self.inner.begin_decode_option()
    }
}

impl<'a, D: StructDeserializer<'a>> StructDeserializer<'a> for KeyAdapter<D> {
    type Ok = D::Ok;
    type Error = D::Error;

    #[inline]
    fn decode_field<T>(&mut self, info: &MemberInfo<'a>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        let dominated = info.flags.contains(MemberFlag::IS_KEY) || self.implicit;
        let optional = info.flags.contains(MemberFlag::IS_OPTIONAL);
        if dominated && !optional {
            self.inner.decode_field(
                info,
                &mut Key {
                    value,
                    implicit: self.implicit,
                },
            )?;
        }
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<'a, U: UnionDeserializer<'a>> UnionDeserializer<'a> for KeyAdapter<U> {
    type Ok = ();
    type Error = U::Error;

    #[inline]
    fn decode_discriminant<T>(&mut self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        self.inner.decode_discriminant(value)
    }

    #[inline]
    fn decode_variant<T>(self, _: &MemberInfo<'a>, _: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        Ok(())
    }
}

/// Wrapper that makes [`Marshal`] and [`Unmarshal`] process only key members.
pub struct KeyOnly<T>(pub T);

impl<T: Marshal> Marshal for KeyOnly<T> {
    fn marshal<'a, S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer<'a>,
    {
        marshal_key(&self.0, archive)
    }
}

impl<T: Unmarshal> Unmarshal for KeyOnly<T> {
    fn unmarshal_mut<'a, D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer<'a>,
    {
        unmarshal_key(&mut self.0, archive)
    }
}

/// Serialize only the key members of `value`.
pub fn marshal_key<'a, T, S>(value: &T, archive: S) -> Result<S::Ok, S::Error>
where
    T: Marshal,
    S: Serializer<'a>,
{
    value.marshal(KeyAdapter::new(archive))
}

/// Deserialize only the key members into `value`.
pub fn unmarshal_key<'a, T, D>(value: &mut T, archive: D) -> Result<(), D::Error>
where
    T: Unmarshal,
    D: Deserializer<'a>,
{
    value.unmarshal_mut(KeyAdapter::new(archive))
}
