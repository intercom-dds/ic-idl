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

#![allow(dead_code, clippy::cast_possible_truncation)]

use super::TypeFlag;
use crate::buf::endian::{Big, Endian, Little};
use crate::buf::Buffer;
use crate::cdr::Error;
use crate::cdr1::{Encoding, MemberFlag};
use crate::encode::{
    ArraySerializer, EnumSerializer, MapSerializer, Marshal, MemberInfo, SeqSerializer, Serializer,
    StructSerializer, UnionSerializer,
};
use crate::{TypeInfo, DISC_INFO};

const PID_EXTENDED: u16 = 0x3F01;
const PID_LIST_END: u16 = 0x3F02;
const FLAG_MUST_UNDERSTAND: u16 = 0x4000;
const FLAG_IMPL_EXTENSION: u16 = 0x8000;

struct CdrWriter<'a, E: Endian> {
    buf: &'a mut Buffer<E>,
    enc: Encoding,
    align_base: usize,
}

impl<'a, E: Endian> CdrWriter<'a, E> {
    const fn new(buf: &'a mut Buffer<E>) -> Self {
        Self {
            buf,
            enc: Encoding::Delimited,
            align_base: 0,
        }
    }

    fn align(&mut self, align: usize) {
        let dt = (align - ((self.buf.pos() - self.align_base) & (align - 1))) & (align - 1);
        self.buf.advance(dt);
    }

    #[inline]
    fn aligned_write<F, T>(&mut self, f: F, val: T)
    where
        F: for<'c> Fn(&'c mut Buffer<E>, T),
    {
        self.align(size_of::<T>());
        f(self.buf, val);
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

    #[inline]
    fn is_final(&self) -> bool {
        self.encoding() == Encoding::Plain
    }

    #[inline]
    fn is_appendable(&self) -> bool {
        self.encoding() == Encoding::Delimited
    }

    #[inline]
    fn is_mutable(&self) -> bool {
        self.encoding() == Encoding::PL
    }

    #[inline]
    const fn encoding(&self) -> Encoding {
        self.enc
    }

    #[inline]
    fn set_encoding(&mut self, flags: TypeFlag) {
        let enc = if flags.contains(TypeFlag::IS_FINAL) {
            Encoding::Plain
        } else if flags.contains(TypeFlag::IS_MUTABLE) {
            Encoding::PL
        } else {
            Encoding::Delimited
        };

        // Update encoding to match the new type
        self.enc = enc;
    }

    #[inline]
    fn mutable_member<F>(&mut self, info: &MemberInfo<'_>, value: F) -> Result<(), Error>
    where
        F: for<'c> FnOnce(&mut CdrWriter<'c, E>) -> Result<(), Error>,
    {
        self.align(4);

        // Reserve space for the short PL header
        let header_pos = self.buf.pos();
        self.buf.advance(2 * size_of::<u16>());

        // Serialize the member
        let align_base = self.buf.pos();
        let mut writer = CdrWriter {
            buf: self.buf,
            enc: self.enc,
            align_base,
        };
        value(&mut writer)?;

        let end = self.buf.pos();
        let written = end - header_pos - 2 * size_of::<u16>();

        // Fill in the appropriate header, shifting the already-written
        // contents if necessary.
        self.buf.set_pos(header_pos);

        let end_index = if (info.member_id != u32::from(PID_LIST_END)
            && info.member_id >= u32::from(PID_EXTENDED))
            || written > 0xFFFF
        {
            let shift = 2 * size_of::<u32>();
            self.buf.set_pos(header_pos);

            // Shift the contents to make place for the extended header
            // TODO: get rid of really_reserve_n. in this case we want to make
            // space for two additional ints, regardless of the current
            // position of the buffer.
            self.buf.really_reserve_n(2 * size_of::<u32>());
            self.buf.mem_move(header_pos..end, header_pos + shift);

            // Fill in the long PL header
            let pid = PID_EXTENDED | (info.member_id as u16 & FLAG_IMPL_EXTENSION);
            let pid = if info.flags.contains(MemberFlag::IS_MUST_UNDERSTAND) {
                pid | FLAG_MUST_UNDERSTAND
            } else {
                pid
            };

            self.write_u16(pid);
            self.write_u16(shift as u16);
            self.write_u32(info.member_id);
            self.write_u32(written as u32);
            end + shift
        } else {
            // Flags + member ID
            let pid = info.member_id as u16;
            let pid = if info.flags.contains(MemberFlag::IS_MUST_UNDERSTAND) {
                pid | FLAG_MUST_UNDERSTAND
            } else {
                pid
            };

            self.write_u16(pid);
            self.write_u16(written as u16);
            end
        };

        // Move the cursor back to the end of the butter
        self.buf.set_pos(end_index);

        Ok(())
    }
}

impl<E: Endian> Serializer for &mut CdrWriter<'_, E> {
    type Ok = ();
    type Error = Error;
    type Struct = Self;
    type Union = Self;
    type Enum = Self;
    type Sequence = Self;

    type Array = Self;
    type Map = Self;

    #[inline]
    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(u8::from(value))
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value.try_into().map_err(|_| Error::InvalidChar)?)
    }

    #[inline]
    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error> {
        let v = u16::try_from(value as u32).map_err(|_| Error::InvalidChar)?;
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
    fn encode_option<T>(self, _: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn encode_struct(self, info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        self.set_encoding(info.flags);
        Ok(self)
    }

    #[inline]
    fn encode_union(self, info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        self.encode_struct(info)
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

impl<E: Endian> StructSerializer for &mut CdrWriter<'_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_field<T>(&mut self, info: &MemberInfo<'_>, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        if self.is_mutable() {
            self.mutable_member(info, |ar| value.marshal(ar))?;
        } else {
            let mut writer = CdrWriter {
                buf: self.buf,
                enc: self.enc,
                align_base: self.align_base,
            };
            value.marshal(&mut writer)?;
        }
        Ok(())
    }

    // TODO: is this necessary now that we have TypeFlags?
    fn encode_optional<T>(
        &mut self,
        info: &MemberInfo<'_>,
        value: &Option<T>,
    ) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if self.is_mutable() {
            if let Some(v) = value {
                self.encode_field(info, v)?;
            }
        } else {
            self.mutable_member(info, |ar| {
                if let Some(v) = value {
                    v.marshal(ar)
                } else {
                    Ok(())
                }
            })?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.is_mutable() {
            self.align(4);
            self.write_u16(PID_LIST_END);
            self.write_u16(0);
        }
        Ok(())
    }
}

impl<E: Endian> EnumSerializer for &mut CdrWriter<'_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_variant<T>(self, _: &str, value: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        value.marshal(self)
    }
}

impl<E: Endian> UnionSerializer for &mut CdrWriter<'_, E> {
    type Ok = ();
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
        StructSerializer::end(self)
    }

    fn encode_null(self) -> Result<Self::Ok, Self::Error> {
        // TODO: is this correct?
        StructSerializer::end(self)
    }
}

impl<E: Endian> ArraySerializer for &mut CdrWriter<'_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        value.marshal(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> SeqSerializer for &mut CdrWriter<'_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        value.marshal(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> MapSerializer for &mut CdrWriter<'_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_pair<K, S>(&mut self, key: &K, value: &S) -> Result<(), Self::Error>
    where
        K: Marshal,
        S: Marshal,
    {
        key.marshal(&mut **self)?;
        value.marshal(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub fn to_bytes<T, E>(value: &T) -> Result<Vec<u8>, Error>
where
    E: Endian,
    T: Marshal + ?Sized,
{
    let mut buf = Buffer::<E>::new();
    let mut writer = CdrWriter::new(&mut buf);
    value.marshal(&mut writer)?;
    Ok(buf.bytes())
}

/// Serialize the given data structore using the specified CDR encoding with
/// little-endian byte order.
pub fn to_le_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Marshal + ?Sized,
{
    to_bytes::<_, Little>(value)
}

/// Serialize the given data structore using the specified CDR encoding with
/// big-endian byte order.
pub fn to_be_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Marshal + ?Sized,
{
    to_bytes::<_, Big>(value)
}
