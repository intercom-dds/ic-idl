// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

use std::marker::PhantomData;

use super::Error;
use crate::buf::Cursor;
use crate::buf::endian::{Big, Endian, Little};
use crate::decode::{
    ArrayDeserializer, Deserializer, EnumDeserializer, EnumVisitor, MapDeserializer,
    OptionDeserializer, SeqDeserializer, StructDeserializer, UnionDeserializer, Unmarshal,
};
use crate::{MemberInfo, TypeInfo};

pub struct CdrReader<'de, E: Endian> {
    buf: Cursor<'de>,
    _endian: PhantomData<E>,
}

impl<'a, E: Endian> CdrReader<'a, E> {
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            buf: Cursor::new(input),
            _endian: PhantomData::<E>,
        }
    }

    #[inline]
    fn aligned_read<T>(&mut self, f: unsafe fn(*const u8) -> T) -> Result<T, Error> {
        self.buf.align_to::<T>();
        if self.buf.unread_bytes() >= size_of::<T>() {
            // SAFETY: bounds checked
            unsafe {
                let val = f(self.buf.read_ptr());
                self.buf.advance_unchecked(size_of::<T>());
                Ok(val)
            }
        } else {
            Err(Error::InvalidLen)
        }
    }
}

impl<'a, 'de, E: Endian> Deserializer<'a> for &'a mut CdrReader<'de, E> {
    type Struct = Self;
    type Union = Self;
    type Enum = Self;
    type Map = MemberSeq<'a, 'de, E>;
    type Sequence = MemberSeq<'a, 'de, E>;
    type Array = MemberSeq<'a, 'de, E>;
    type Option = Self;

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
        self.aligned_read(E::read_u8_raw)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.decode_u16().map(|v| v as i16)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.aligned_read(E::read_u16_raw)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.decode_u32().map(|v| v as i32)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.aligned_read(E::read_u32_raw)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.decode_u64().map(|v| v as i64)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.aligned_read(E::read_u64_raw)
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
                    self.buf.advance_unchecked(len);
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

    #[inline]
    fn decode_struct(self, _: &TypeInfo<'a>) -> Result<Self::Struct, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_union(self, _: &TypeInfo<'a>) -> Result<Self::Union, Self::Error> {
        Ok(self)
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
    fn begin_decode_option(self) -> Result<Self::Option, Self::Error> {
        Ok(self)
    }
}

impl<E: Endian> OptionDeserializer for &mut CdrReader<'_, E> {
    type Error = Error;

    fn is_some(&mut self) -> bool {
        self.decode_bool().unwrap_or(false)
    }

    fn decode_some<T>(self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        value.unmarshal_mut(self)
    }
}

impl<'a, E: Endian> StructDeserializer<'a> for &mut CdrReader<'_, E> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn decode_field<T>(&mut self, _: &MemberInfo<'a>, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        value.unmarshal_mut(&mut **self)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, E: Endian> UnionDeserializer<'a> for &mut CdrReader<'_, E> {
    type Ok = Self;
    type Error = Error;

    #[inline]
    fn decode_discriminant<T>(&mut self, value: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        value.unmarshal_mut(&mut **self)
    }

    #[inline]
    fn decode_variant<T>(self, _: &MemberInfo<'a>, value: &mut T) -> Result<Self::Ok, Self::Error>
    where
        T: Unmarshal,
    {
        value.unmarshal_mut(&mut *self)?;
        Ok(self)
    }
}

impl<E: Endian> EnumDeserializer for &mut CdrReader<'_, E> {
    type Error = Error;

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
