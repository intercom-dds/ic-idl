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
use crate::buf::endian::{Endian, Little};
use crate::cdr::Error;
use crate::decode::{
    ArrayDeserializer, BitmaskDeserializer, Deserializer, EnumDeserializer, EnumVisitor,
    MapDeserializer, OptionDeserializer, SeqDeserializer, StructDeserializer, UnionDeserializer,
    Unmarshal,
};
use crate::{MemberInfo, TypeInfo};

const MEMBER_ID_MASK: u32 = 0x0FFF_FFFF;
const LC_MASK: u32 = 0x7;
const LC_SHIFT: u32 = 28;
const EMHEADER_SIZE: usize = 4;

const LC_1BYTE: u8 = 0;
const LC_2BYTE: u8 = 1;
const LC_4BYTE: u8 = 2;
const LC_8BYTE: u8 = 3;
const LC_NEXTINT_1BYTE: u8 = 4;
const LC_NEXTINT_DHEADER: u8 = 5;
const LC_NEXTINT_4BYTE_ELEMENTS: u8 = 6;
const LC_NEXTINT_8BYTE_ELEMENTS: u8 = 7;

pub struct Xcdr2Reader<'de, E: Endian> {
    buf: Cursor<'de>,
    align_base: usize,
    skip_option_flag: bool,
    seq_len: Option<usize>,
    type_stack: Vec<TypeInfo<'de>>,
    _endian: PhantomData<E>,
}

impl<'a, E: Endian> Xcdr2Reader<'a, E> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            buf: Cursor::new(input),
            align_base: 0,
            skip_option_flag: false,
            seq_len: None,
            type_stack: Vec::new(),
            _endian: PhantomData::<E>,
        }
    }

    fn push_type_info(&mut self, info: &TypeInfo<'a>) {
        self.type_stack.push(*info);
    }

    fn pop_type_info(&mut self) {
        self.type_stack.pop();
    }

    #[inline]
    fn aligned_read<T, const N: usize>(
        &mut self,
        f: unsafe fn(*const u8) -> T,
    ) -> Result<T, Error> {
        let dt = (N - ((self.buf.pos() - self.align_base) & (N - 1))) & (N - 1);
        let total_needed = dt + size_of::<T>();

        if self.buf.unread_bytes() >= total_needed {
            // SAFETY: Bounds checked
            unsafe {
                self.buf.advance_unchecked(dt);
                let val = f(self.buf.read_ptr());
                self.buf.advance_unchecked(size_of::<T>());
                Ok(val)
            }
        } else {
            Err(Error::Eof)
        }
    }
}

impl<'r, 'de, E: Endian> Deserializer<'de> for &'r mut Xcdr2Reader<'de, E> {
    type Error = Error;
    type Struct = Xcdr2StructDeserializer<'r, 'de, E>;
    type Union = Xcdr2UnionDeserializer<'r, 'de, E>;
    type Enum = Xcdr2EnumDeserializer<'r, 'de, E>;
    type Bitmask = Self;
    type Sequence = Xcdr2CollectionDeserializer<'r, 'de, E>;
    type Array = Xcdr2CollectionDeserializer<'r, 'de, E>;
    type Map = Xcdr2MapDeserializer<'r, 'de, E>;
    type Option = Self;

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
        let v = self.decode_u16()?;
        char::from_u32(u32::from(v)).ok_or(Error::InvalidChar)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.decode_u8().map(u8::cast_signed)
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.aligned_read::<_, 1>(E::read_u8_raw)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.decode_u16().map(u16::cast_signed)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.aligned_read::<_, 2>(E::read_u16_raw)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.decode_u32().map(u32::cast_signed)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.aligned_read::<_, 4>(E::read_u32_raw)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.decode_u64().map(u64::cast_signed)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.aligned_read::<_, 4>(E::read_u64_raw)
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
                // SAFETY: bounds checked, skip null terminator
                let slice = unsafe {
                    let bytes = self.buf.slice(len - 1);
                    self.buf.advance_unchecked(len);
                    bytes
                };

                let str = std::str::from_utf8(slice).map_err(|_| Error::InvalidUtf8)?;
                Ok(str.to_owned())
            }
            _ => Err(Error::InvalidLen),
        }
    }

    #[inline]
    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn decode_wstring(self) -> Result<String, Self::Error> {
        let len = self.decode_u32()? as usize;
        match len {
            0 => Ok(String::new()),
            _ if len <= self.buf.unread_bytes() => {
                // SAFETY: bounds checked
                let slice = unsafe {
                    let bytes = self.buf.slice(len);
                    self.buf.advance_unchecked(len);
                    bytes
                };

                char::decode_utf16(slice.chunks_exact(2).map(E::read_u16))
                    .collect::<Result<_, _>>()
                    .map_err(|_| Error::InvalidUtf8)
            }
            _ => Err(Error::InvalidLen),
        }
    }

    fn decode_struct(self, info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        let type_start = self.buf.pos();
        let is_mutable = info.is_mutable();
        let is_appendable = info.is_appendable();

        // APPENDABLE structs have a DHEADER at the start
        let dheader_end = if is_appendable {
            self.buf.align_to::<u32>();
            let dheader = self.decode_u32()?;
            let pos = self.buf.pos();
            let end = pos.checked_add(dheader as usize).ok_or(Error::InvalidLen)?;
            if end > self.buf.total_len() {
                return Err(Error::InvalidLen);
            }
            Some(end)
        } else {
            None
        };

        Ok(Xcdr2StructDeserializer {
            reader: self,
            is_mutable,
            is_appendable,
            dheader_end,
            next_member_pos: type_start,
            type_start,
        })
    }

    fn decode_union(self, info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        let is_mutable = info.is_mutable();
        let is_appendable = info.is_appendable();
        let _type_start = self.buf.pos();

        // APPENDABLE/MUTABLE unions have a DHEADER at the start
        let dheader_end = if is_appendable || is_mutable {
            self.buf.align_to::<u32>();
            let dheader = self.decode_u32()?;
            let pos = self.buf.pos();
            let end = pos.checked_add(dheader as usize).ok_or(Error::InvalidLen)?;
            if end > self.buf.total_len() {
                return Err(Error::InvalidLen);
            }
            Some(end)
        } else {
            None
        };

        Ok(Xcdr2UnionDeserializer {
            reader: self,
            is_mutable,
            dheader_end,
        })
    }

    fn decode_enum(self, _: &TypeInfo<'_>) -> Result<Self::Enum, Self::Error> {
        Ok(Xcdr2EnumDeserializer { reader: self })
    }

    fn decode_bitmask(self, _: &TypeInfo<'_>) -> Result<Self::Bitmask, Self::Error> {
        Ok(self)
    }

    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        let len = if let Some(known_len) = self.seq_len.take() {
            known_len
        } else {
            self.decode_u32()? as usize
        };
        let element_info = self.type_stack.last().and_then(|info| info.element_info);
        Ok(Xcdr2CollectionDeserializer {
            reader: self,
            len,
            index: 0,
            element_info,
        })
    }

    fn decode_array(self, len: usize) -> Result<Self::Array, Self::Error> {
        let element_info = self.type_stack.last().and_then(|info| info.element_info);
        Ok(Xcdr2CollectionDeserializer {
            reader: self,
            len,
            index: 0,
            element_info,
        })
    }

    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        let len = self.decode_u32()? as usize;
        Ok(Xcdr2MapDeserializer {
            reader: self,
            len,
            index: 0,
        })
    }

    #[inline]
    fn begin_decode_option(self) -> Result<Self::Option, Self::Error> {
        Ok(self)
    }
}

impl<E: Endian> OptionDeserializer for &mut Xcdr2Reader<'_, E> {
    type Error = Error;

    fn is_some(&mut self) -> bool {
        if self.skip_option_flag {
            true
        } else {
            self.decode_u8().unwrap_or(0) != 0
        }
    }

    fn decode_some<T>(self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        value.unmarshal_mut(self)
    }
}

pub struct Xcdr2StructDeserializer<'a, 'de, E: Endian> {
    reader: &'a mut Xcdr2Reader<'de, E>,
    is_mutable: bool,
    is_appendable: bool,
    dheader_end: Option<usize>,
    type_start: usize,
    next_member_pos: usize,
}

impl<'de, E: Endian> Xcdr2StructDeserializer<'_, 'de, E> {
    fn decode_mutable_field<T>(
        &mut self,
        info: &MemberInfo<'de>,
        value: &mut T,
    ) -> Result<(), Error>
    where
        T: Unmarshal,
    {
        let members_start = if self.dheader_end.is_none() {
            // SAFETY: `type_start` was saved from a valid buffer position when
            // this deserializer was created, so it's within bounds.
            unsafe {
                self.reader.buf.set_pos_unchecked(self.type_start);
            }

            self.reader.buf.align_to::<u32>();
            let dheader = self.reader.decode_u32()?;
            let start = self.reader.buf.pos();
            let dheader_end = start
                .checked_add(dheader as usize)
                .ok_or(Error::InvalidLen)?;
            if dheader_end > self.reader.buf.total_len() {
                return Err(Error::InvalidLen);
            }
            self.dheader_end = Some(dheader_end);
            self.next_member_pos = start;
            start
        } else {
            let mut pos = self.type_start;
            pos = (pos + 3) & !3;
            pos + 4
        };

        let dheader_end = self.dheader_end.unwrap();

        // SAFETY: `next_member_pos` tracks valid positions within the buffer
        // as we decode members, always staying within the dheader bounds.
        unsafe {
            self.reader.buf.set_pos_unchecked(self.next_member_pos);
        }

        while self.reader.buf.pos() < dheader_end {
            if let Some(end_pos) = self.try_read_member(info, value, dheader_end)? {
                self.next_member_pos = end_pos;
                return Ok(());
            }
        }

        if self.next_member_pos > members_start {
            // SAFETY: members_start was saved from the buffer position just after the
            // DHEADER, which is within bounds.
            unsafe {
                self.reader.buf.set_pos_unchecked(members_start);
            }

            while self.reader.buf.pos() < self.next_member_pos {
                if let Some(_end_pos) = self.try_read_member(info, value, self.next_member_pos)? {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn try_read_member<T>(
        &mut self,
        info: &MemberInfo<'_>,
        value: &mut T,
        limit: usize,
    ) -> Result<Option<usize>, Error>
    where
        T: Unmarshal,
    {
        self.reader.buf.align_to::<u32>();

        if self.reader.buf.pos() >= limit {
            return Ok(None);
        }

        if self.reader.buf.unread_bytes() < EMHEADER_SIZE {
            return Err(Error::Eof);
        }

        // Read EMHEADER1
        let emheader = self.reader.decode_u32()?;
        let lc = ((emheader >> LC_SHIFT) & LC_MASK) as u8;
        let member_id = emheader & MEMBER_ID_MASK;

        let (length, nextint_pos, nextint_value) = if lc >= LC_NEXTINT_1BYTE {
            let pos = self.reader.buf.pos();
            let nextint = self.reader.decode_u32()?;
            let len = match lc {
                LC_NEXTINT_4BYTE_ELEMENTS => (nextint as usize).wrapping_mul(4),
                LC_NEXTINT_8BYTE_ELEMENTS => (nextint as usize).wrapping_mul(8),
                _ => nextint as usize,
            };
            (len, Some(pos), Some(nextint))
        } else {
            let len = match lc {
                LC_1BYTE => 1,
                LC_2BYTE => 2,
                LC_4BYTE => 4,
                LC_8BYTE => 8,
                _ => 0,
            };
            (len, None, None)
        };

        if member_id == info.member_id {
            let is_optional_in_mutable = info.flags.contains(crate::MemberFlag::IS_OPTIONAL);

            if lc == LC_NEXTINT_DHEADER {
                let needs_adjustment = !matches!(
                    info.type_info.kind,
                    crate::TypeKind::Array | crate::TypeKind::Sequence | crate::TypeKind::Map
                );
                if needs_adjustment && let Some(pos) = nextint_pos {
                    self.reader.buf.set_pos(pos)?;
                }
            } else if lc == LC_NEXTINT_4BYTE_ELEMENTS || lc == LC_NEXTINT_8BYTE_ELEMENTS {
                let element_count = nextint_value.ok_or(Error::UnsupportedType)?;
                self.reader.seq_len = Some(element_count as usize);
                value.unmarshal_mut(&mut *self.reader)?;

                let end_pos = self.reader.buf.pos();
                return Ok(Some(end_pos));
            }

            if is_optional_in_mutable {
                self.reader.skip_option_flag = true;
            }

            value.unmarshal_mut(&mut *self.reader)?;

            if is_optional_in_mutable {
                self.reader.skip_option_flag = false;
            }

            let end_pos = self.reader.buf.pos();
            return Ok(Some(end_pos));
        }

        let data_end = self.reader.buf.pos().wrapping_add(length);
        self.reader.buf.set_pos(data_end)?;
        Ok(None)
    }

    fn decode_field_with_dheader<T>(
        &mut self,
        info: &MemberInfo<'de>,
        value: &mut T,
    ) -> Result<(), Error>
    where
        T: Unmarshal,
    {
        let needs_dheader = super::ser::needs_dheader_as_field(info.type_info);
        let dheader_end = if needs_dheader {
            self.reader.buf.align_to::<u32>();
            let dheader = self.reader.decode_u32()?;
            Some(self.reader.buf.pos() + dheader as usize)
        } else {
            None
        };

        self.reader.push_type_info(info.type_info);
        value.unmarshal_mut(&mut *self.reader)?;
        self.reader.pop_type_info();

        if let Some(end_pos) = dheader_end
            && self.reader.buf.pos() < end_pos
        {
            self.reader.buf.set_pos(end_pos)?;
        }

        Ok(())
    }
}

impl<'de, E: Endian> StructDeserializer<'de> for Xcdr2StructDeserializer<'_, 'de, E> {
    type Ok = ();
    type Error = Error;

    fn decode_field<T>(&mut self, info: &MemberInfo<'de>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        if self.is_mutable {
            self.decode_mutable_field(info, value)
        } else if self.is_appendable {
            if !self.reader.buf.is_empty() {
                if let Some(end) = self.dheader_end
                    && self.reader.buf.pos() >= end
                {
                    return Ok(());
                }

                let is_optional = info.flags.contains(crate::MemberFlag::IS_OPTIONAL);
                if is_optional {
                    let present = self.reader.decode_bool()?;
                    if !present {
                        return Ok(());
                    }
                    self.reader.skip_option_flag = true;
                }

                self.decode_field_with_dheader(info, value)?;

                if is_optional {
                    self.reader.skip_option_flag = false;
                }
            }
            Ok(())
        } else {
            if !self.reader.buf.is_empty() {
                let is_optional = info.flags.contains(crate::MemberFlag::IS_OPTIONAL);
                if is_optional {
                    let present = self.reader.decode_bool()?;
                    if !present {
                        return Ok(());
                    }
                    self.reader.skip_option_flag = true;
                }

                self.decode_field_with_dheader(info, value)?;

                if is_optional {
                    self.reader.skip_option_flag = false;
                }
            }
            Ok(())
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct Xcdr2UnionDeserializer<'a, 'de, E: Endian> {
    reader: &'a mut Xcdr2Reader<'de, E>,
    is_mutable: bool,
    dheader_end: Option<usize>,
}

impl<'de, E: Endian> UnionDeserializer<'de> for Xcdr2UnionDeserializer<'_, 'de, E> {
    type Ok = ();
    type Error = Error;

    fn decode_discriminant<T>(&mut self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        if self.is_mutable {
            // MUTABLE unions: Read EMHEADER for discriminant (member_id=0)
            // DHEADER was already read in decode_union
            self.reader.buf.align_to::<u32>();
            let _emheader = self.reader.decode_u32()?;
            // Should be: member_id=0, LC=discriminant's LC
            // For now, just read the discriminant value
            value.unmarshal_mut(&mut *self.reader)?;
            Ok(())
        } else {
            // FINAL/APPENDABLE unions: discriminant directly
            value.unmarshal_mut(&mut *self.reader)?;
            Ok(())
        }
    }

    fn decode_variant<T>(
        self,
        info: &MemberInfo<'de>,
        value: &mut T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Unmarshal,
    {
        if self.is_mutable {
            // MUTABLE: Find the EMHEADER for the variant's member_id
            // We're positioned after the discriminant, search for the variant
            let dheader_end = self.dheader_end.ok_or(Error::UnsupportedType)?;

            while self.reader.buf.pos() < dheader_end {
                self.reader.buf.align_to::<u32>();

                if self.reader.buf.unread_bytes() < EMHEADER_SIZE {
                    return Err(Error::Eof);
                }

                // Read EMHEADER
                let emheader = self.reader.decode_u32()?;
                let lc = ((emheader >> LC_SHIFT) & LC_MASK) as u8;
                let member_id = emheader & MEMBER_ID_MASK;

                // Read NEXTINT if LC >= 4
                let (nextint_pos, nextint_value) = if lc >= LC_NEXTINT_1BYTE {
                    let pos = self.reader.buf.pos();
                    let nextint = self.reader.decode_u32()?;
                    (Some(pos), Some(nextint))
                } else {
                    (None, None)
                };

                if member_id == info.member_id {
                    // Found the variant!
                    if lc == LC_NEXTINT_DHEADER {
                        // LC=5: NEXTINT is reused as DHEADER for strings/structs
                        let needs_adjustment = !matches!(
                            info.type_info.kind,
                            crate::TypeKind::Array
                                | crate::TypeKind::Sequence
                                | crate::TypeKind::Map
                        );
                        if needs_adjustment && let Some(pos) = nextint_pos {
                            self.reader.buf.set_pos(pos)?;
                        }
                    }
                    value.unmarshal_mut(&mut *self.reader)?;
                    return Ok(());
                }

                // Skip this member - we need to advance past it to continue searching
                let bytes_to_skip = if let Some(nextint) = nextint_value {
                    // LC >= 4: NEXTINT contains length, but encoding varies by LC
                    match lc {
                        LC_NEXTINT_1BYTE | LC_NEXTINT_DHEADER => nextint as usize,
                        LC_NEXTINT_4BYTE_ELEMENTS => nextint.wrapping_mul(4) as usize,
                        LC_NEXTINT_8BYTE_ELEMENTS => nextint.wrapping_mul(8) as usize,
                        _ => return Err(Error::UnsupportedType),
                    }
                } else {
                    // LC 0-3: fixed size based on LC value
                    match lc {
                        LC_1BYTE => 1,
                        LC_2BYTE => 2,
                        LC_4BYTE => 4,
                        LC_8BYTE => 8,
                        _ => return Err(Error::UnsupportedType),
                    }
                };
                self.reader.buf.advance(bytes_to_skip)?;
            }

            Err(Error::Eof)
        } else {
            // FINAL/APPENDABLE: variant data follows discriminant
            value.unmarshal_mut(&mut *self.reader)?;
            Ok(())
        }
    }
}

pub struct Xcdr2EnumDeserializer<'a, 'de, E: Endian> {
    reader: &'a mut Xcdr2Reader<'de, E>,
}

impl<E: Endian> EnumDeserializer for Xcdr2EnumDeserializer<'_, '_, E> {
    type Error = Error;

    fn decode_enumerator<T>(self, visitor: T) -> Result<T, Self::Error>
    where
        T: EnumVisitor + Unmarshal,
    {
        visitor.member_id(&mut *self.reader)
    }
}

impl<'a, E: Endian> BitmaskDeserializer<'a> for &mut Xcdr2Reader<'_, E> {
    type Error = Error;

    fn decode_flags<T>(self, _: &[MemberInfo<'a>]) -> Result<T, Self::Error>
    where
        T: Unmarshal + Default,
    {
        T::unmarshal(self)
    }
}

pub struct Xcdr2CollectionDeserializer<'a, 'de, E: Endian> {
    reader: &'a mut Xcdr2Reader<'de, E>,
    len: usize,
    index: usize,
    element_info: Option<&'de TypeInfo<'de>>,
}

impl<E: Endian> Xcdr2CollectionDeserializer<'_, '_, E> {
    fn decode_element<T>(&mut self, value: &mut T) -> Result<bool, Error>
    where
        T: Unmarshal,
    {
        if self.index < self.len {
            if let Some(info) = self.element_info {
                self.reader.push_type_info(info);
            }

            // Check if element needs DHEADER
            // In XCDR2, arrays/sequences/maps with non-primitive elements get DHEADER
            let needs_dheader = self
                .element_info
                .is_some_and(super::ser::needs_dheader_as_field);
            if needs_dheader {
                self.reader.buf.align_to::<u32>();
                let _dheader = self.reader.decode_u32()?;
            }

            value.unmarshal_mut(&mut *self.reader)?;

            if self.element_info.is_some() {
                self.reader.pop_type_info();
            }

            self.index += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<E: Endian> SeqDeserializer for Xcdr2CollectionDeserializer<'_, '_, E> {
    type Error = Error;

    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        self.decode_element(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<E: Endian> ArrayDeserializer for Xcdr2CollectionDeserializer<'_, '_, E> {
    type Error = Error;

    fn decode_next<T>(&mut self, value: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        self.decode_element(value)
    }
}

pub struct Xcdr2MapDeserializer<'a, 'de, E: Endian> {
    reader: &'a mut Xcdr2Reader<'de, E>,
    len: usize,
    index: usize,
}

impl<E: Endian> MapDeserializer for Xcdr2MapDeserializer<'_, '_, E> {
    type Error = Error;

    fn decode_pair<K, V>(&mut self, key: &mut K, value: &mut V) -> Result<bool, Self::Error>
    where
        K: Unmarshal,
        V: Unmarshal,
    {
        if self.index < self.len {
            key.unmarshal_mut(&mut *self.reader)?;
            value.unmarshal_mut(&mut *self.reader)?;
            self.index += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

pub fn from_le_bytes<T: Unmarshal + Default>(input: &[u8]) -> Result<T, Error> {
    from_bytes::<_, Little>(input)
}

pub fn from_be_bytes<T: Unmarshal + Default>(input: &[u8]) -> Result<T, Error> {
    from_bytes::<_, crate::buf::endian::Big>(input)
}

pub fn from_bytes<T: Unmarshal + Default, E: Endian>(input: &[u8]) -> Result<T, Error> {
    let mut reader = Xcdr2Reader::<E>::new(input);
    T::unmarshal(&mut reader)
}

pub fn from_bytes_mut<T: Unmarshal, E: Endian>(input: &[u8], value: &mut T) -> Result<(), Error> {
    let mut reader = Xcdr2Reader::<E>::new(input);
    value.unmarshal_mut(&mut reader)
}
