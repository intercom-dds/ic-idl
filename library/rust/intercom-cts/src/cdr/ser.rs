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

use super::Error;
use crate::buf::Buffer;
use crate::buf::endian::{Big, Endian, Little};
use crate::encode::{
    ArraySerializer, EnumSerializer, MapSerializer, Marshal, SeqSerializer, Serializer,
    StructSerializer, UnionSerializer,
};
use crate::{MemberInfo, TypeInfo};

struct CdrWriter<E: Endian> {
    buf: Buffer<E>,
}

impl<E: Endian> CdrWriter<E> {
    fn new() -> Self {
        Self {
            buf: Buffer::with_capacity(64),
        }
    }

    fn bytes(self) -> Vec<u8> {
        self.buf.bytes()
    }

    #[inline]
    fn aligned_write<F, T>(&mut self, f: F, val: T)
    where
        F: for<'a, 'b> Fn(&'a mut Buffer<E>, T),
    {
        self.buf.align_to::<T>();
        f(&mut self.buf, val);
    }

    #[inline]
    fn write_len(&mut self, len: usize) -> Result<(), Error> {
        let len = u32::try_from(len).map_err(|_| Error::InvalidLen)?;
        self.write_u32(len);
        Ok(())
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.buf.write_u8(value);
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.aligned_write(Buffer::write_u16, value);
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.aligned_write(Buffer::write_u32, value);
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.aligned_write(Buffer::write_u64, value);
    }
}

impl<E: Endian> Serializer for &mut CdrWriter<E> {
    type Struct = Self;
    type Union = Self;
    type Enum = Self;
    type Sequence = Self;
    type Array = Self;
    type Map = Self;

    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(u8::from(v))
    }

    #[inline]
    fn encode_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(v.try_into().map_err(|_| Error::InvalidChar)?)
    }

    #[inline]
    fn encode_wchar(self, v: char) -> Result<Self::Ok, Self::Error> {
        let v = u16::try_from(v as u32).map_err(|_| Error::InvalidChar)?;
        self.encode_u16(v)
    }

    #[inline]
    fn encode_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(v as u8)
    }

    #[inline]
    fn encode_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.write_u8(v);
        Ok(())
    }

    #[inline]
    fn encode_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.encode_u16(v as u16)
    }

    #[inline]
    fn encode_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.write_u16(v);
        Ok(())
    }

    #[inline]
    fn encode_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(v as u32)
    }

    #[inline]
    fn encode_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.write_u32(v);
        Ok(())
    }

    #[inline]
    fn encode_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(v as u64)
    }

    #[inline]
    fn encode_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(v);
        Ok(())
    }

    #[inline]
    fn encode_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(v.to_bits())
    }

    #[inline]
    fn encode_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(v.to_bits())
    }

    #[inline]
    fn encode_option<T>(self, value: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        self.encode_bool(value.is_some())?;
        if let Some(v) = value {
            v.marshal(self)?;
        }
        Ok(())
    }

    #[inline]
    fn encode_string(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let len = if v.is_empty() {
            0
        } else {
            v.len().checked_add(1).ok_or(Error::InvalidLen)?
        };
        self.write_len(len)?;

        if !v.is_empty() {
            self.buf.extend(v.as_bytes());
            self.write_u8(0);
        }
        Ok(())
    }

    #[inline]
    fn encode_wstring(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let len = v
            .encode_utf16()
            .count()
            .checked_mul(2)
            .ok_or(Error::InvalidLen)?;

        self.write_len(len)?;
        v.encode_utf16().try_for_each(|v| self.encode_u16(v))
    }

    #[inline]
    fn encode_struct(self, _: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_union(self, _: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_enum(self, _: &str) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error> {
        self.write_len(len)?;
        Ok(self)
    }

    #[inline]
    fn encode_array(self, _: usize) -> Result<Self::Array, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error> {
        self.write_len(len)?;
        Ok(self)
    }
}

impl<E: Endian> StructSerializer for &mut CdrWriter<E> {
    type Ok = ();
    type Error = Error;

    fn encode_field<T>(&mut self, _: &MemberInfo<'_>, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        value.marshal(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> ArraySerializer for &mut CdrWriter<E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_next<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        value.marshal(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> SeqSerializer for &mut CdrWriter<E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_next<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        value.marshal(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> EnumSerializer for &mut CdrWriter<E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_variant<T>(self, _: &str, value: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        value.marshal(self)
    }
}

impl<E: Endian> UnionSerializer for &mut CdrWriter<E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_discriminant<D>(&mut self, discriminant: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        discriminant.marshal(&mut **self)
    }

    #[inline]
    fn encode_variant<V>(self, _: &MemberInfo<'_>, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal,
    {
        value.marshal(self)
    }

    #[inline]
    fn encode_null(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> MapSerializer for &mut CdrWriter<E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_pair<K, V>(&mut self, key: &K, value: &V) -> Result<Self::Ok, Self::Error>
    where
        K: Marshal,
        V: Marshal,
    {
        key.marshal(&mut **self)?;
        value.marshal(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Serialize the given data structore to plain, little-endian CDR.
pub fn to_le_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Marshal,
{
    let mut writer = CdrWriter::<Little>::new();
    value.marshal(&mut writer)?;
    Ok(writer.bytes())
}

/// Serialize the given data structore to plain, big-endian CDR.
pub fn to_be_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Marshal,
{
    let mut writer = CdrWriter::<Big>::new();
    value.marshal(&mut writer)?;
    Ok(writer.bytes())
}
