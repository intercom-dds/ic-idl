// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

use super::Error;
use crate::decode::Deserializer;
use crate::encode::{EnumSerializer, Serializer};
use crate::error::Error as Err;
use crate::infallible::Never;
use crate::{Marshal, TypeInfo};

pub struct KeySerializer;

fn invalid<T, E: Err>() -> Result<T, E> {
    Err(Err::custom("map keys must be strings, integers or enums"))
}

impl Serializer<'_> for KeySerializer {
    type Ok = String;
    type Error = Error;

    type Struct = Never<Self::Ok, Self::Error>;
    type Union = Never<Self::Ok, Self::Error>;
    type Enum = Self;
    type Sequence = Never<Self::Ok, Self::Error>;
    type Array = Never<Self::Ok, Self::Error>;
    type Map = Never<Self::Ok, Self::Error>;

    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_string(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_wstring(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn encode_enum(self, _: &str) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    fn encode_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        invalid()
    }

    fn encode_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        invalid()
    }

    fn encode_option<T>(self, _: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        invalid()
    }

    fn encode_struct(self, _: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        invalid()
    }

    fn encode_union(self, _: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        invalid()
    }

    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        invalid()
    }

    fn encode_array(self, _: usize) -> Result<Self::Array, Self::Error> {
        invalid()
    }

    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        invalid()
    }
}

impl EnumSerializer for KeySerializer {
    type Ok = String;
    type Error = Error;

    fn encode_variant<T>(self, name: &str, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        Ok(name.to_string())
    }
}

pub struct KeyDeserializer<D>(pub D);

impl<'a, D: Deserializer<'a>> Deserializer<'a> for KeyDeserializer<D> {
    type Error = D::Error;

    type Struct = Never<(), Self::Error>;
    type Union = Never<(), Self::Error>;
    type Enum = D::Enum;
    type Map = Never<(), Self::Error>;
    type Sequence = Never<(), Self::Error>;
    type Array = Never<(), Self::Error>;
    type Option = Never<(), Self::Error>;

    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.0.decode_bool()
    }

    fn decode_char(self) -> Result<char, Self::Error> {
        self.0.decode_char()
    }

    fn decode_wchar(self) -> Result<char, Self::Error> {
        self.0.decode_wchar()
    }

    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.0.decode_i8()
    }

    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.0.decode_u8()
    }

    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.0.decode_i16()
    }

    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.0.decode_u16()
    }

    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.0.decode_i32()
    }

    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.0.decode_u32()
    }

    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.0.decode_i64()
    }

    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.0.decode_u64()
    }

    fn decode_string(self) -> Result<String, Self::Error> {
        self.0.decode_string()
    }

    fn decode_wstring(self) -> Result<String, Self::Error> {
        self.0.decode_wstring()
    }

    fn decode_f32(self) -> Result<f32, Self::Error> {
        invalid()
    }

    fn decode_f64(self) -> Result<f64, Self::Error> {
        invalid()
    }

    fn decode_struct(self, _: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        invalid()
    }

    fn decode_union(self, _: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        invalid()
    }

    fn decode_enum(self, name: &str) -> Result<Self::Enum, Self::Error> {
        self.0.decode_enum(name)
    }

    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        invalid()
    }

    fn decode_array(self, _: usize) -> Result<Self::Array, Self::Error> {
        invalid()
    }

    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        invalid()
    }

    fn begin_decode_option(self) -> Result<Self::Option, Self::Error> {
        invalid()
    }
}
