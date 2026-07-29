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

#![allow(clippy::cast_possible_truncation)]

use crate::DISC_INFO;
use crate::buf::{Big, Buffer, Endian, Little};
use crate::cdr::Error;
use crate::cdr1::MemberFlag;
use crate::encode::{
    ArraySerializer, BitmaskSerializer, EnumSerializer, MapSerializer, Marshal, SeqSerializer,
    Serializer, StructSerializer, UnionSerializer,
};
use crate::type_info::{MemberInfo, TypeInfo, TypeKind};

const MEMBER_ID_MASK: u32 = 0x0FFF_FFFF;
const M_FLAG_SHIFT: u32 = 31;
const LC_SHIFT: u32 = 28;

const LC_NEXTINT_1BYTE: u8 = 4;
const LC_NEXTINT_DHEADER: u8 = 5;
const LC_NEXTINT_4BYTE_ELEMENTS: u8 = 6;
const LC_NEXTINT_8BYTE_ELEMENTS: u8 = 7;

/// Determine if type needs DHEADER in XCDR2
fn needs_dheader(info: &TypeInfo) -> bool {
    info.is_appendable() || info.is_mutable()
}

fn has_xcdr2_dheader(info: &TypeInfo) -> bool {
    use TypeKind::{Array, Map, Sequence, Struct, Union};

    match info.kind {
        Struct | Union => info.is_appendable() || info.is_mutable(),
        Sequence | Array => info.element_info.is_some_and(|i| !i.is_primitive()),
        Map => {
            info.key_info.is_some_and(|i| !i.is_primitive())
                || info.element_info.is_some_and(|i| !i.is_primitive())
        }
        _ => false,
    }
}

pub(crate) fn needs_dheader_as_field(info: &TypeInfo) -> bool {
    match info.kind {
        // Collections need DHEADER if they have non-primitive elements
        TypeKind::Array | TypeKind::Sequence | TypeKind::Map => info
            .element_info
            .is_some_and(|elem_info| !elem_info.is_primitive()),

        // Other types default to false
        _ => false,
    }
}

/// Calculate the Length Code (LC) for XCDR2 `PL_CDR2` encoding (MUTABLE types).
///
/// LC is a 3-bit value (0-7) that encodes how the member's serialized size is represented:
/// - LC 0-3: Direct length (1, 2, 4, 8 bytes), no NEXTINT field
/// - LC 4: NEXTINT field contains byte length
/// - LC 5: NEXTINT reused as member's DHEADER (for strings/structs) or separate DHEADER (collections)
/// - LC 6: NEXTINT × 4 = byte length (for sequences of 4-byte primitives)
/// - LC 7: NEXTINT × 8 = byte length (for sequences of 8-byte primitives)
fn lcode_for_mutable(info: &TypeInfo) -> u8 {
    use TypeKind::{
        Bitmask, Bool, Char8, Char16, Enum, F32, F64, I8, I16, I32, I64, Sequence, String8, U8,
        U16, U32, U64,
    };
    match info.kind {
        // LC 0-3: direct length encoding (1, 2, 4, 8 bytes)
        // 1 byte
        Bool | I8 | U8 => 0,

        // 2 bytes
        I16 | U16 | Char16 => 1,

        // 4 bytes - includes bitmasks and enums which are typically i32/u32
        I32 | U32 | F32 | Char8 | Bitmask | Enum => 2,

        // 8 bytes
        I64 | U64 | F64 => 3,

        Sequence => {
            if let Some(element_info) = info.element_info {
                match element_info.kind {
                    Char8 | I8 | U8 => 5,
                    I32 | U32 | F32 => 6,
                    I64 | U64 | F64 => 7,
                    _ => {
                        if has_xcdr2_dheader(info) {
                            5
                        } else {
                            4
                        }
                    }
                }
            } else {
                4
            }
        }

        String8 => 5,

        _ => {
            if has_xcdr2_dheader(info) {
                5
            } else {
                4
            }
        }
    }
}

pub struct Cdr2Writer<'a, E: Endian> {
    buf: &'a mut Buffer<E>,
    skip_collection_length: bool,
    align_base: usize,
    type_stack: Vec<TypeInfo<'a>>,
}

impl<'a, E: Endian> Cdr2Writer<'a, E> {
    pub fn new(buf: &'a mut Buffer<E>) -> Self {
        Self {
            buf,
            skip_collection_length: false,
            align_base: 0,
            type_stack: Vec::new(),
        }
    }

    fn push_type_info(&mut self, info: &TypeInfo<'a>) {
        self.type_stack.push(*info);
    }

    fn pop_type_info(&mut self) {
        self.type_stack.pop();
    }

    fn with_dheader<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        self.align(size_of::<u32>());
        let dheader_pos = self.buf.pos();
        self.buf.write_u32(0);

        let saved_align_base = self.align_base;
        self.align_base = self.buf.pos();
        let data_start = self.buf.pos();

        f(self)?;

        let data_end = self.buf.pos();
        self.align_base = saved_align_base;

        let length = (data_end - data_start) as u32;
        let saved_pos = self.buf.pos();
        self.buf.set_pos(dheader_pos);
        self.buf.write_u32(length);
        self.buf.set_pos(saved_pos);

        Ok(())
    }

    fn with_simple_dheader<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        self.align(size_of::<u32>());
        let dheader_pos = self.buf.pos();
        self.buf.write_u32(0);

        f(self)?;

        let end_pos = self.buf.pos();
        let dheader_value = (end_pos - dheader_pos - 4) as u32;
        let saved_pos = self.buf.pos();
        self.buf.set_pos(dheader_pos);
        self.buf.write_u32(dheader_value);
        self.buf.set_pos(saved_pos);

        Ok(())
    }

    fn align(&mut self, align: usize) {
        const PADDING: [u8; 3] = [0, 0, 0];
        let dt = (align - ((self.buf.pos() - self.align_base) & (align - 1))) & (align - 1);
        if dt > 0 {
            self.buf.extend(&PADDING[..dt]);
        }
    }

    #[inline]
    fn write_len(&mut self, len: usize) -> Result<(), Error> {
        let len = u32::try_from(len).map_err(|_| Error::InvalidLen)?;
        self.align(size_of::<u32>());
        self.buf.write_u32(len);
        Ok(())
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.buf.write_u8(value);
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.align(size_of::<u16>());
        self.buf.write_u16(value);
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.align(size_of::<u32>());
        self.buf.write_u32(value);
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.align(size_of::<u32>());
        self.buf.write_u64(value);
    }
}

impl<'a, 'b, E: Endian> Serializer<'a> for &'b mut Cdr2Writer<'a, E> {
    type Struct = Cdr2CompositeWriter<'a, 'b, E>;
    type Union = Cdr2CompositeWriter<'a, 'b, E>;
    type Enum = Self;
    type Bitmask = Self;
    type Sequence = Cdr2CollectionWriter<'a, 'b, E>;
    type Array = Cdr2CollectionWriter<'a, 'b, E>;
    type Map = Cdr2MapWriter<'a, 'b, E>;

    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.write_u8(u8::from(value));
        Ok(())
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.write_u8(value.try_into().map_err(|_| Error::InvalidChar)?);
        Ok(())
    }

    #[inline]
    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error> {
        let v = u16::try_from(value as u32).map_err(|_| Error::InvalidChar)?;
        self.write_u16(v);
        Ok(())
    }

    #[inline]
    fn encode_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.write_u8(v.cast_unsigned());
        Ok(())
    }

    #[inline]
    fn encode_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.write_u8(v);
        Ok(())
    }

    #[inline]
    fn encode_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.write_u16(v.cast_unsigned());
        Ok(())
    }

    #[inline]
    fn encode_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.write_u16(v);
        Ok(())
    }

    #[inline]
    fn encode_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.write_u32(v.cast_unsigned());
        Ok(())
    }

    #[inline]
    fn encode_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.write_u32(v);
        Ok(())
    }

    #[inline]
    fn encode_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(v.cast_unsigned());
        Ok(())
    }

    #[inline]
    fn encode_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(v);
        Ok(())
    }

    #[inline]
    fn encode_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.write_u32(v.to_bits());
        Ok(())
    }

    #[inline]
    fn encode_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(v.to_bits());
        Ok(())
    }

    #[inline]
    fn encode_string(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let len = v.len().checked_add(1).ok_or(Error::InvalidLen)?;
        self.write_len(len)?;
        self.buf.extend(v.as_bytes());
        self.write_u8(0);
        Ok(())
    }

    #[inline]
    fn encode_wstring(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let utf16: Vec<u16> = v.encode_utf16().collect();
        let len = utf16.len().checked_mul(2).ok_or(Error::InvalidLen)?;

        self.write_len(len)?;
        for unit in utf16 {
            self.write_u16(unit);
        }
        Ok(())
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
    fn encode_struct(self, info: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        Ok(Cdr2CompositeWriter::new(self, info))
    }

    #[inline]
    fn encode_union(self, info: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        Ok(Cdr2CompositeWriter::new(self, info))
    }

    #[inline]
    fn encode_enum(self, _: &TypeInfo<'_>) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_bitmask(self, _: &TypeInfo<'_>) -> Result<Self::Bitmask, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error> {
        if !self.skip_collection_length {
            self.write_len(len)?;
        }
        self.skip_collection_length = false;

        let element_info = self
            .type_stack
            .last()
            .and_then(|info| info.element_info.copied());

        Ok(Cdr2CollectionWriter::new(self, element_info))
    }

    #[inline]
    fn encode_array(self, _len: usize) -> Result<Self::Array, Self::Error> {
        let element_info = self
            .type_stack
            .last()
            .and_then(|info| info.element_info.copied());

        Ok(Cdr2CollectionWriter::new(self, element_info))
    }

    #[inline]
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error> {
        if !self.skip_collection_length {
            self.write_len(len)?;
        }
        self.skip_collection_length = false;

        let element_info = self
            .type_stack
            .last()
            .and_then(|info| info.element_info.copied());
        let key_info = self
            .type_stack
            .last()
            .and_then(|info| info.key_info.copied());

        Ok(Cdr2MapWriter::new(self, element_info, key_info))
    }
}

pub struct Cdr2CompositeWriter<'a, 'b, E: Endian> {
    writer: &'b mut Cdr2Writer<'a, E>,
    is_mutable: bool,
    dheader_pos: Option<usize>,
}

impl<'a, 'b, E: Endian> Cdr2CompositeWriter<'a, 'b, E> {
    fn new(writer: &'b mut Cdr2Writer<'a, E>, info: &TypeInfo<'_>) -> Self {
        let _is_appendable = info.is_appendable();
        let is_mutable = info.is_mutable();

        let dheader_pos = if needs_dheader(info) {
            writer.align(size_of::<u32>());
            let pos = writer.buf.pos();
            writer.buf.write_u32(0);
            writer.align_base = writer.buf.pos();
            Some(pos)
        } else {
            None
        };

        Self {
            writer,
            is_mutable,
            dheader_pos,
        }
    }

    fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    fn encode_mutable_member<T>(&mut self, info: &MemberInfo<'a>, value: &T) -> Result<(), Error>
    where
        T: Marshal,
    {
        encode_mutable_member_impl(self.writer, info, value)
    }
}

impl<'a, E: Endian> StructSerializer<'a> for Cdr2CompositeWriter<'a, '_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_field<T>(&mut self, info: &MemberInfo<'a>, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if self.is_mutable() {
            // Mutable encoding uses EMHEADER + NEXTINT
            self.encode_mutable_field(info, value)
        } else {
            // Final/Appendable encoding
            // Wrap fields that need DHEADER (collections with complex elements, strings, etc.)
            let needs_dheader_flag = needs_dheader_as_field(info.type_info);

            if needs_dheader_flag {
                self.writer.with_dheader(|writer| {
                    writer.push_type_info(info.type_info);
                    value.marshal(&mut *writer)?;
                    writer.pop_type_info();
                    Ok(())
                })?;
            } else {
                // No DHEADER needed
                self.writer.push_type_info(info.type_info);
                value.marshal(&mut *self.writer)?;
                self.writer.pop_type_info();
            }
            Ok(())
        }
    }

    fn encode_optional<T>(
        &mut self,
        info: &MemberInfo<'a>,
        value: &Option<T>,
    ) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if self.is_mutable() {
            // For XCDR2 MUTABLE, optional members use the same EMHEADER1 encoding
            // as non-optional members (per spec rule 22). Just skip if None.
            if let Some(v) = value {
                self.encode_field(info, v)?;
            }
        } else {
            // For appendable, encode presence flag
            self.writer.write_u8(u8::from(value.is_some()));

            if let Some(v) = value {
                self.encode_field(info, v)?;
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(dheader_pos) = self.dheader_pos {
            let end_pos = self.writer.buf.pos();
            let length = end_pos - dheader_pos - 4;

            let saved_pos = self.writer.buf.pos();
            self.writer.buf.set_pos(dheader_pos);
            self.writer.buf.write_u32(length as u32);
            self.writer.buf.set_pos(saved_pos);
        }
        Ok(())
    }
}

fn encode_mutable_member_impl<'a, T, E: Endian>(
    writer: &mut Cdr2Writer<'a, E>,
    info: &MemberInfo<'a>,
    value: &T,
) -> Result<(), Error>
where
    T: Marshal,
{
    // EMHEADER format for mutable
    writer.align(size_of::<u32>());

    // XCDR2 uses EMHEADER1 encoding for ALL members, regardless of member_id size
    // (unlike XCDR1 which uses PID_EXTENDED for large member_ids)
    let lcode = lcode_for_mutable(info.type_info);
    let m_flag = u32::from(info.flags.contains(MemberFlag::IS_MUST_UNDERSTAND));
    let member_id = info.member_id & MEMBER_ID_MASK;

    // EMHEADER1 = (M_FLAG << 31) + (LC << 28) + MemberId
    let emheader1 = (m_flag << M_FLAG_SHIFT) | (u32::from(lcode) << LC_SHIFT) | member_id;

    writer.write_u32(emheader1);

    if lcode >= LC_NEXTINT_1BYTE {
        // LC >= 4 means we need NEXTINT
        let nextint_pos = writer.buf.pos();
        writer.write_u32(0); // NEXTINT placeholder

        // LC=5 has different semantics depending on type:
        // - Strings/Structs/Unions: NEXTINT is REUSED (position adjusted back)
        // - Collections: NEXTINT is separate DHEADER (no position adjustment)
        let lc5_adjust_position = lcode == LC_NEXTINT_DHEADER
            && !matches!(
                info.type_info.kind,
                TypeKind::Array | TypeKind::Sequence | TypeKind::Map
            );

        if lc5_adjust_position {
            // LC=5 with position adjustment: NEXTINT is reused as member's first 4 bytes
            writer.buf.set_pos(nextint_pos);
            writer.push_type_info(info.type_info);
            value.marshal(&mut *writer)?;
            writer.pop_type_info();
            // Member writes the DHEADER value, no backpatch needed
        } else {
            let data_start = writer.buf.pos();
            // For LC=6/7, skip writing collection length prefix (NEXTINT encodes it)
            if lcode == LC_NEXTINT_4BYTE_ELEMENTS || lcode == LC_NEXTINT_8BYTE_ELEMENTS {
                writer.skip_collection_length = true;
            }
            writer.push_type_info(info.type_info);
            value.marshal(&mut *writer)?;
            writer.pop_type_info();
            writer.skip_collection_length = false;
            let data_end = writer.buf.pos();

            let length = data_end - data_start;

            // Optimize LC=4: if data fits in 1, 2, 4, or 8 bytes, use direct encoding
            if lcode == LC_NEXTINT_1BYTE && matches!(length, 1 | 2 | 4 | 8) {
                // Shift data back 4 bytes (remove NEXTINT)
                let emheader_pos = nextint_pos - size_of::<u32>();
                writer.buf.mem_move(data_start..data_end, nextint_pos);
                writer.buf.set_pos(data_end - size_of::<u32>());

                // Update EMHEADER with optimized LC
                let new_lcode = match length {
                    1 => 0,
                    2 => 1,
                    4 => 2,
                    8 => 3,
                    _ => unreachable!(),
                };
                let new_emheader = (m_flag << M_FLAG_SHIFT) | (new_lcode << LC_SHIFT) | member_id;
                let saved_pos = writer.buf.pos();
                writer.buf.set_pos(emheader_pos);
                writer.buf.write_u32(new_emheader);
                writer.buf.set_pos(saved_pos);
            } else {
                // Keep LC=4/5/6/7, backpatch NEXTINT
                let nextint_value = match lcode {
                    LC_NEXTINT_4BYTE_ELEMENTS => length / 4,
                    LC_NEXTINT_8BYTE_ELEMENTS => length / 8,
                    _ => length,
                };
                let saved_pos = writer.buf.pos();
                writer.buf.set_pos(nextint_pos);
                writer.buf.write_u32(nextint_value as u32);
                writer.buf.set_pos(saved_pos);
            }
        }
    } else {
        // LC 0-3: no NEXTINT, just serialize the data
        writer.push_type_info(info.type_info);
        value.marshal(&mut *writer)?;
        writer.pop_type_info();
    }

    Ok(())
}

impl<'a, E: Endian> Cdr2CompositeWriter<'a, '_, E> {
    fn encode_mutable_field<T>(&mut self, info: &MemberInfo<'a>, value: &T) -> Result<(), Error>
    where
        T: Marshal,
    {
        // Add struct DHEADER if this is the first non-optional member
        if self.dheader_pos.is_none() {
            self.writer.align(size_of::<u32>());
            let pos = self.writer.buf.pos();
            self.writer.buf.write_u32(0); // Placeholder
            self.dheader_pos = Some(pos);
        }

        encode_mutable_member_impl(self.writer, info, value)
    }
}

pub struct Cdr2CollectionWriter<'a, 'b, E: Endian> {
    writer: &'b mut Cdr2Writer<'a, E>,
    element_info: Option<TypeInfo<'a>>,
}

impl<'a, 'b, E: Endian> Cdr2CollectionWriter<'a, 'b, E> {
    fn new(writer: &'b mut Cdr2Writer<'a, E>, element_info: Option<TypeInfo<'a>>) -> Self {
        Self {
            writer,
            element_info,
        }
    }

    fn encode_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Marshal,
    {
        if let Some(info) = self.element_info {
            self.writer.push_type_info(&info);
        }

        // Check if element needs DHEADER
        // In XCDR2, arrays/sequences/maps with non-primitive elements get DHEADER
        let needs_dheader = self
            .element_info
            .is_some_and(|info| needs_dheader_as_field(&info));
        if needs_dheader {
            self.writer
                .with_simple_dheader(|writer| value.marshal(&mut *writer))?;
        } else {
            value.marshal(&mut *self.writer)?;
        }

        if self.element_info.is_some() {
            self.writer.pop_type_info();
        }
        Ok(())
    }
}

impl<E: Endian> SeqSerializer for Cdr2CollectionWriter<'_, '_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        self.encode_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<E: Endian> ArraySerializer for Cdr2CollectionWriter<'_, '_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        self.encode_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, E: Endian> UnionSerializer<'a> for Cdr2CompositeWriter<'a, '_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_discriminant<D>(&mut self, discriminant: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        if self.is_mutable {
            // For MUTABLE unions, encode discriminant as MMEMBER with EMHEADER1
            self.encode_mutable_member(&DISC_INFO, discriminant)?;
        } else {
            // For FINAL/APPENDABLE unions, push discriminant type info and marshal
            self.writer.push_type_info(DISC_INFO.type_info);
            discriminant.marshal(&mut *self.writer)?;
            self.writer.pop_type_info();
        }
        Ok(())
    }

    fn encode_variant<V>(
        mut self,
        info: &MemberInfo<'a>,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal,
    {
        if self.is_mutable {
            // For MUTABLE unions, encode variant as MMEMBER with EMHEADER1
            self.encode_mutable_member(info, value)?;
        } else {
            // For FINAL/APPENDABLE unions, just marshal variant
            self.writer.push_type_info(info.type_info);
            value.marshal(&mut *self.writer)?;
            self.writer.pop_type_info();
        }

        // Backpatch DHEADER if needed
        if let Some(dheader_pos) = self.dheader_pos {
            let end_pos = self.writer.buf.pos();
            let length = end_pos - dheader_pos - 4;

            let saved_pos = self.writer.buf.pos();
            self.writer.buf.set_pos(dheader_pos);
            self.writer.buf.write_u32(length as u32);
            self.writer.buf.set_pos(saved_pos);
        }

        Ok(())
    }

    fn encode_null(self) -> Result<Self::Ok, Self::Error> {
        // Backpatch DHEADER for null case
        if let Some(dheader_pos) = self.dheader_pos {
            let end_pos = self.writer.buf.pos();
            let length = end_pos - dheader_pos - 4;

            let saved_pos = self.writer.buf.pos();
            self.writer.buf.set_pos(dheader_pos);
            self.writer.buf.write_u32(length as u32);
            self.writer.buf.set_pos(saved_pos);
        }
        Ok(())
    }
}

impl<'a, E: Endian> EnumSerializer<'a> for &mut Cdr2Writer<'_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_variant<T>(self, _: &MemberInfo<'a>, value: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        value.marshal(self)
    }
}

impl<'a, E: Endian> BitmaskSerializer<'a> for &mut Cdr2Writer<'_, E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn encode_flag<T>(self, value: T, _: &[MemberInfo<'a>]) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal + Into<u64>,
    {
        value.marshal(self)
    }
}

pub struct Cdr2MapWriter<'a, 'b, E: Endian> {
    writer: &'b mut Cdr2Writer<'a, E>,
    key_info: Option<TypeInfo<'a>>,
    value_info: Option<TypeInfo<'a>>,
}

impl<'a, 'b, E: Endian> Cdr2MapWriter<'a, 'b, E> {
    fn new(
        writer: &'b mut Cdr2Writer<'a, E>,
        value_info: Option<TypeInfo<'a>>,
        key_info: Option<TypeInfo<'a>>,
    ) -> Self {
        Self {
            writer,
            key_info,
            value_info,
        }
    }
}

impl<E: Endian> MapSerializer for Cdr2MapWriter<'_, '_, E> {
    type Ok = ();
    type Error = Error;

    fn encode_pair<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Marshal,
        V: Marshal,
    {
        if let Some(info) = self.key_info {
            self.writer.push_type_info(&info);
        }
        key.marshal(&mut *self.writer)?;
        if self.key_info.is_some() {
            self.writer.pop_type_info();
        }

        if let Some(info) = self.value_info {
            self.writer.push_type_info(&info);
        }
        value.marshal(&mut *self.writer)?;
        if self.value_info.is_some() {
            self.writer.pop_type_info();
        }
        Ok(())
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
    let mut buf = Buffer::<E>::with_capacity(64);
    to_buffer(&mut buf, value)?;
    Ok(buf.to_vec())
}

pub fn to_buffer<T, E>(buffer: &mut Buffer<E>, value: &T) -> Result<(), Error>
where
    E: Endian,
    T: Marshal + ?Sized,
{
    let mut writer = Cdr2Writer::<E>::new(buffer);
    value.marshal(&mut writer)?;
    Ok(())
}

/// Serialize to XCDR2 with little-endian byte order.
pub fn to_le_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Marshal,
{
    to_bytes::<_, Little>(value)
}

/// Serialize to XCDR2 with big-endian byte order.
pub fn to_be_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Marshal,
{
    to_bytes::<_, Big>(value)
}
