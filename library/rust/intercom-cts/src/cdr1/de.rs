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

use std::marker::PhantomData;

use crate::buf::Cursor;
use crate::buf::endian::{Big, Endian, Little};
use crate::cdr::Error;
use crate::cdr1::{Encoding, MemberFlag};
use crate::decode::{
    ArrayDeserializer, Deserializer, EnumDeserializer, EnumVisitor, MapDeserializer,
    SeqDeserializer, StructDeserializer, UnionDeserializer, Unmarshal,
};
use crate::{DISC_INFO, MemberInfo, TypeFlag, TypeInfo};

const PID_EXTENDED: u16 = 0x3F01;
const PID_LIST_END: u32 = 0x3F02;
const PID_PID_MASK: u16 = 0x3FFF;
const _FLAG_MUST_UNDERSTAND: u16 = 0x4000;
const _FLAG_IMPL_EXTENSION: u16 = 0x8000;
const MEMBER_ID_MASK: u32 = 0x0FFF_FFFF;

struct CdrReader<'de, E: Endian> {
    buf: Cursor<'de>,
    enc: Encoding,
    align_base: usize,
    type_offset: usize,
    _endian: PhantomData<E>,
}

impl<'a, E: Endian> CdrReader<'a, E> {
    const fn new(input: &'a [u8]) -> Self {
        Self {
            buf: Cursor::new(input),
            enc: Encoding::Plain,
            align_base: 0,
            type_offset: 0,
            _endian: PhantomData::<E>,
        }
    }

    fn align(&mut self, align: usize) {
        let dt = (align - ((self.buf.pos() - self.align_base) & (align - 1))) & (align - 1);

        // SAFETY: bounds checked
        unsafe {
            if dt >= self.buf.unread_bytes() {
                self.buf.set_pos(self.buf.total_len());
            } else {
                self.buf.advance(dt);
            }
        }
    }

    #[inline]
    fn aligned_read<F, T>(&mut self, f: F) -> Result<T, Error>
    where
        F: Fn(&[u8]) -> T,
    {
        self.align(size_of::<T>());
        if self.buf.unread_bytes() >= size_of::<T>() {
            let val = f(self.buf.as_ref());
            // SAFETY: bounds checked
            unsafe {
                self.buf.advance(size_of::<T>());
            }
            Ok(val)
        } else {
            Err(Error::Eof)
        }
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
        // Update encoding to match the new type
        let enc = if flags.contains(TypeFlag::IS_FINAL) {
            Encoding::Plain
        } else if flags.contains(TypeFlag::IS_MUTABLE) {
            Encoding::PL
        } else {
            Encoding::Delimited
        };

        self.enc = enc;
        self.type_offset = self.buf.pos();
    }

    #[inline]
    fn decode_subtype<T: Unmarshal>(
        &mut self,
        align_base: usize,
        value: &mut T,
    ) -> Result<(), Error> {
        let mut reader = CdrReader {
            buf: self.buf.clone(),
            enc: Encoding::Plain,
            align_base,
            type_offset: self.buf.pos(),
            _endian: PhantomData::<E>,
        };
        value.unmarshal_mut(&mut reader)?;
        self.buf = reader.buf;
        Ok(())
    }

    fn decode_mutable_header(&mut self) -> Result<(u32, usize), Error> {
        self.align(4);
        let pid = self.decode_u16()? & PID_PID_MASK;
        let res = if pid == PID_EXTENDED {
            let shift = usize::from(self.decode_u16()?);
            if shift > 2 * size_of::<u32>() && self.buf.unread_bytes() > shift {
                // SAFETY: bounds checked
                unsafe {
                    self.buf.advance(shift - 2 * size_of::<u32>());
                }
            }

            // Long PL
            let member_id = self.decode_u32()? & MEMBER_ID_MASK;
            let len = self.decode_u32()? as usize;
            (member_id, len)
        } else {
            let member_id = u32::from(pid);
            let len = self.decode_u16()?;
            (member_id, usize::from(len))
        };
        Ok(res)
    }

    #[inline]
    fn end_type(&mut self) -> Result<(), Error> {
        if self.is_mutable() {
            loop {
                let (sentinel, len) = self.decode_mutable_header()?;
                if sentinel == PID_LIST_END {
                    break;
                }

                if self.buf.unread_bytes() >= len {
                    // SAFETY: bounds checked
                    unsafe {
                        self.buf.advance(len);
                    }
                } else {
                    return Err(Error::Eof);
                }
            }
        }
        Ok(())
    }
}

impl<'a, 'de, E: Endian> Deserializer for &'a mut CdrReader<'de, E> {
    type Struct = Self;
    type Union = Self;
    type Enum = Self;
    type Map = MemberSeq<'a, 'de, E>;
    type Sequence = MemberSeq<'a, 'de, E>;
    type Array = MemberSeq<'a, 'de, E>;

    type Error = Error;

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.decode_u8().map(|v| v != 0)
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        self.decode_u8().map(|v| v as char)
    }

    #[inline]
    fn decode_wchar(self) -> Result<char, Self::Error> {
        let v: u16 = self.decode_u16()?;
        char::from_u32(u32::from(v)).ok_or(Error::InvalidChar)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.decode_u8().map(|v| v as i8)
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.aligned_read(E::read_u8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.decode_u16().map(|v| v as i16)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.aligned_read(E::read_u16)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.decode_u32().map(|v| v as i32)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.aligned_read(E::read_u32)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.decode_u64().map(|v| {
            // Safe reinterpretation of u64 bits as i64 for decoding
            i64::from_ne_bytes(v.to_ne_bytes())
        })
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.aligned_read(E::read_u64)
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.decode_u32().map(f32::from_bits)
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.decode_u64().map(f64::from_bits)
    }

    #[inline]
    fn decode_string(self) -> Result<String, Self::Error> {
        let len = self.decode_u32()? as usize;
        match len {
            0 => Ok(String::new()),
            _ if len <= self.buf.unread_bytes() => {
                // SAFETY: bounds checked
                let slice = unsafe {
                    // skip null terminator
                    let bytes = self.buf.slice(len - 1);
                    self.buf.advance(len);
                    bytes
                };

                let str = std::str::from_utf8(slice).map_err(|_| Error::InvalidUtf8)?;
                Ok(str.to_string())
            }
            _ => Err(Error::InvalidLen),
        }
    }

    #[inline]
    fn decode_wstring(self) -> Result<String, Self::Error> {
        let len = self.decode_u32()? as usize;
        match len {
            0 => Ok(String::new()),
            _ if len <= self.buf.unread_bytes() => {
                // SAFETY: bounds checked
                let slice = unsafe {
                    let bytes = self.buf.slice(len);
                    self.buf.advance(len);
                    bytes
                };

                char::decode_utf16(slice.chunks_exact(2).map(E::read_u16))
                    .collect::<Result<_, _>>()
                    .map_err(|_| Error::InvalidUtf8)
            }
            _ => Err(Error::InvalidLen),
        }
    }

    #[inline]
    fn decode_option<T>(self) -> Result<Option<T>, Self::Error>
    where
        T: Unmarshal + Default,
    {
        Ok(Some(T::unmarshal(&mut *self)?))
    }

    #[inline]
    fn decode_struct(self, info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        self.set_encoding(info.flags);
        Ok(self)
    }

    #[inline]
    fn decode_union(self, info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        self.decode_struct(info)
    }

    #[inline]
    fn decode_enum(self, _: &str) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        let len = self.decode_u32()? as usize;
        self.decode_array(len)
    }

    #[inline]
    fn decode_array(self, len: usize) -> Result<Self::Array, Self::Error> {
        Ok(MemberSeq { reader: self, len })
    }

    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        self.decode_sequence()
    }

    #[inline]
    fn decode_option_mut<T>(self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        let res = if self.decode_bool()? {
            value.unmarshal_mut(&mut *self)?;
            true
        } else {
            false
        };
        Ok(res)
    }
}

impl<E: Endian> StructDeserializer for &mut CdrReader<'_, E> {
    type Ok = ();
    type Error = <Self as Deserializer>::Error;

    #[inline]
    fn decode_field<T>(&mut self, info: &MemberInfo<'_>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        if self.is_mutable() || info.flags.contains(MemberFlag::IS_OPTIONAL) {
            let start = self.buf.pos();

            // Mutable fields may not appear in order, and although optional
            // fields are treated as mutable, they must appear in-order for
            // final and appendable types.
            if self.is_mutable() {
                // SAFETY: `type_offset` is the index that was recorded when
                // the type started, so it is guaranteed to be within bounds.
                unsafe {
                    self.buf.set_pos(self.type_offset);
                }
            }

            while self.buf.unread_bytes() > 0 {
                self.align(4);
                let (member_id, len) = self.decode_mutable_header()?;
                let end = self.buf.pos() + len;

                if member_id == info.member_id {
                    if len > 0 {
                        self.decode_subtype(self.buf.pos(), value)?;
                    }

                    // Skip to the end of the type. There may be additional
                    // fields we don't know about and shouldn't waste time
                    // parsing.
                    if self.buf.total_len() >= end {
                        unsafe { self.buf.set_pos(end) };
                    } else {
                        return Err(Error::Eof);
                    }
                    return Ok(());
                } else if member_id == PID_LIST_END {
                    break;
                }

                // Wrong member, skip to the next
                if self.buf.total_len() >= end {
                    unsafe { self.buf.set_pos(end) };
                } else {
                    return Err(Error::Eof);
                }
            }

            // Member was not found, move cursor back to its original position.
            //
            // SAFETY: `start` was the initial position of the cursor, so it is
            // guaranteed to be within bounds.
            unsafe {
                self.buf.set_pos(start);
            }
        } else if !self.buf.is_empty() {
            self.decode_subtype(self.align_base, value)?;
        }

        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_type()
    }
}

impl<E: Endian> UnionDeserializer for &mut CdrReader<'_, E> {
    type Ok = Self;
    type Error = <Self as Deserializer>::Error;

    #[inline]
    fn decode_discriminant<T>(&mut self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        self.decode_field(&DISC_INFO, value)
    }

    #[inline]
    fn decode_variant<T>(
        mut self,
        info: &MemberInfo<'_>,
        value: &mut T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Unmarshal,
    {
        self.decode_field(info, value)?;
        self.end_type()?;
        Ok(self)
    }
}

impl<E: Endian> EnumDeserializer for &mut CdrReader<'_, E> {
    type Error = <Self as Deserializer>::Error;

    #[inline]
    fn decode_enumerator<V>(self, visitor: V) -> Result<V, Self::Error>
    where
        V: EnumVisitor + Unmarshal,
    {
        visitor.member_id(&mut *self)
    }
}

pub struct MemberSeq<'a, 'de, E: Endian> {
    reader: &'a mut CdrReader<'de, E>,
    len: usize,
}

impl<E: Endian> SeqDeserializer for MemberSeq<'_, '_, E> {
    type Error = Error;

    #[inline]
    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        if self.len > 0 {
            self.len -= 1;
            value.unmarshal_mut(&mut *self.reader)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<E: Endian> ArrayDeserializer for MemberSeq<'_, '_, E> {
    type Error = Error;

    #[inline]
    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        if self.len > 0 {
            self.len -= 1;
            value.unmarshal_mut(&mut *self.reader)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<E: Endian> MapDeserializer for MemberSeq<'_, '_, E> {
    type Error = Error;

    #[inline]
    fn decode_pair<K, V>(&mut self, key: &mut K, value: &mut V) -> Result<bool, Self::Error>
    where
        K: Unmarshal,
        V: Unmarshal,
    {
        if self.len > 0 {
            self.len -= 1;
            key.unmarshal_mut(&mut *self.reader)?;
            value.unmarshal_mut(&mut *self.reader)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

/// Deserialize an instance of type T from plain, little-endian CDR.
pub fn from_le_bytes<T: Unmarshal + Default>(input: &[u8]) -> Result<T, Error> {
    from_bytes::<_, Little>(input)
}

/// Deserialize an instance of type T from plain, big-endian CDR.
pub fn from_be_bytes<T: Unmarshal + Default>(input: &[u8]) -> Result<T, Error> {
    from_bytes::<_, Big>(input)
}

/// Deserialize an instance of type T with the specified endianness.
pub fn from_bytes<T: Unmarshal + Default, E: Endian>(input: &[u8]) -> Result<T, Error> {
    let mut reader = CdrReader::<E>::new(input);
    T::unmarshal(&mut reader)
}

/// In-place, stateful deserialization of type T with the specified endianness.
pub fn from_bytes_mut<T: Unmarshal, E: Endian>(input: &[u8], value: &mut T) -> Result<(), Error> {
    let mut reader = CdrReader::<E>::new(input);
    value.unmarshal_mut(&mut reader)
}
