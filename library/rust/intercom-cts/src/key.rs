// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2025 KONGSBERG - All rights reserved

//! Machinery for key-only serialization.

use crate::decode::{Deserializer, StructDeserializer};
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

/// Adapter for key-only serialization. This will skip non-key members.
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
    fn encode_enum(self, name: &str) -> Result<Self::Enum, Self::Error> {
        self.inner.encode_enum(name)
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
        todo!()
    }
}

impl<'a, D: Deserializer<'a>> Deserializer<'a> for KeyAdapter<D> {
    type Error = D::Error;
    type Struct = KeyAdapter<D::Struct>;
    type Union = D::Union;
    type Enum = D::Enum;
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
        self.inner.decode_union(info)
    }

    #[inline]
    fn decode_enum(self, name: &str) -> Result<Self::Enum, Self::Error> {
        self.inner.decode_enum(name)
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
        if info.flags.contains(MemberFlag::IS_KEY) || self.implicit {
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
