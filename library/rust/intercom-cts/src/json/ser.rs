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

use std::io::Write;
use std::ops::{Deref, DerefMut};

use super::error::Error;
use super::key::KeySerializer;
use crate::encode::{
    ArraySerializer, EnumSerializer, MapSerializer, Marshal, SeqSerializer, Serializer,
    StructSerializer, UnionSerializer,
};
use crate::error::Error as _;
use crate::{DISC_INFO, MemberInfo, TypeInfo};

struct JsonWriter<W: Write> {
    w: W,
    indent: usize,
    pretty: bool,
}

impl<W: Write> JsonWriter<W> {
    fn write<S: AsRef<[u8]>>(&mut self, value: S) -> Result<(), Error> {
        self.w.write_all(value.as_ref()).map_err(Error::custom)
    }

    fn null(&mut self) -> Result<(), Error> {
        self.write("null")
    }

    fn comma(&mut self) -> Result<(), Error> {
        self.write(",")
    }

    fn begin(&mut self, c: char) -> Result<(), Error> {
        self.write(format!("{c}"))?;
        self.indent += 1;
        Ok(())
    }

    fn end(&mut self, c: char, was_empty: bool) -> Result<(), Error> {
        self.indent -= 1;
        if !was_empty {
            self.newl()?;
        }
        self.write(format!("{c}"))?;
        Ok(())
    }

    fn newl(&mut self) -> Result<(), Error> {
        if self.pretty {
            self.write("\n")?;
            self.indent()?;
        }
        Ok(())
    }

    fn write_str(&mut self, value: &str) -> Result<(), Error> {
        self.write("\"")?;
        for c in value.chars() {
            match c {
                '"' => self.write("\\\"")?,
                '\\' => self.write("\\\\")?,
                '\x08' => self.write("\\b")?,
                '\x0C' => self.write("\\f")?,
                '\n' => self.write("\\n")?,
                '\r' => self.write("\\r")?,
                '\t' => self.write("\\t")?,
                c if c.is_control() => {
                    self.write(format!("\\u{:04x}", c as u32))?;
                }
                c => self.write(c.to_string())?,
            }
        }
        self.write("\"")
    }

    fn indent(&mut self) -> Result<(), Error> {
        self.write(" ".repeat(self.indent * 2))
    }
}

impl<'a, W: Write> Serializer for &'a mut JsonWriter<W> {
    type Ok = ();
    type Error = Error;

    type Struct = JsonObject<'a, W>;
    type Union = JsonObject<'a, W>;
    type Enum = Self;
    type Sequence = JsonArray<'a, W>;
    type Array = JsonArray<'a, W>;
    type Map = JsonObject<'a, W>;

    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        let str = if value { "true" } else { "false" };
        self.write(str)
    }

    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        let s = value.to_string();
        self.write_str(&s)
    }

    fn encode_wchar(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_char(value)
    }

    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.write(value.to_string())
    }

    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        if value.is_nan() || value.is_infinite() {
            self.null()
        } else {
            self.write(value.to_string())
        }
    }

    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if value.is_nan() || value.is_infinite() {
            self.null()
        } else {
            self.write(value.to_string())
        }
    }

    fn encode_option<T>(self, value: &Option<T>) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        if let Some(v) = value {
            v.marshal(self)
        } else {
            self.null()
        }
    }

    fn encode_string(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.write_str(value)
    }

    fn encode_wstring(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.encode_string(value)
    }

    fn encode_struct(self, _: &TypeInfo<'_>) -> Result<Self::Struct, Self::Error> {
        JsonObject::new(self)
    }

    fn encode_union(self, _: &TypeInfo<'_>) -> Result<Self::Union, Self::Error> {
        JsonObject::new(self)
    }

    fn encode_enum(self, _: &str) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        JsonArray::new(self)
    }

    fn encode_array(self, _: usize) -> Result<Self::Array, Self::Error> {
        JsonArray::new(self)
    }

    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        JsonObject::new(self)
    }
}

struct JsonObject<'a, W: Write> {
    writer: &'a mut JsonWriter<W>,
    first: bool,
}

impl<'a, W: Write> JsonObject<'a, W> {
    fn new(writer: &'a mut JsonWriter<W>) -> Result<Self, Error> {
        writer.begin('{')?;
        Ok(Self {
            writer,
            first: true,
        })
    }

    fn write_pair<T>(&mut self, key: &str, value: T) -> Result<(), Error>
    where
        T: Marshal,
    {
        if !self.first {
            self.comma()?;
        }
        self.first = false;
        self.newl()?;
        self.write_str(key)?;
        self.write(":")?;
        if self.pretty {
            self.write(" ")?;
        }
        value.marshal(&mut **self)
    }
}

impl<W: Write> Deref for JsonObject<'_, W> {
    type Target = JsonWriter<W>;

    fn deref(&self) -> &Self::Target {
        self.writer
    }
}

impl<W: Write> DerefMut for JsonObject<'_, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writer
    }
}

impl<W: Write> StructSerializer for JsonObject<'_, W> {
    type Ok = ();
    type Error = Error;

    fn encode_field<T>(&mut self, info: &MemberInfo<'_>, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        self.write_pair(info.name, value)
    }

    fn encode_optional<T>(
        &mut self,
        info: &MemberInfo<'_>,
        value: &Option<T>,
    ) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        if let Some(v) = value {
            self.encode_field(info, v)
        } else {
            Ok(())
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.end('}', self.first)?;
        Ok(())
    }
}

impl<W: Write> UnionSerializer for JsonObject<'_, W> {
    type Ok = ();
    type Error = Error;

    fn encode_discriminant<D>(&mut self, discriminant: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        self.write_pair(DISC_INFO.name, discriminant)
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
        StructSerializer::end(self)
    }
}

impl<W: Write> MapSerializer for JsonObject<'_, W> {
    type Ok = ();
    type Error = Error;

    fn encode_pair<K, S>(&mut self, key: &K, value: &S) -> Result<Self::Ok, Self::Error>
    where
        K: Marshal,
        S: Marshal,
    {
        let key = key.marshal(KeySerializer)?;
        self.write_pair(&key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        StructSerializer::end(self)
    }
}

impl<W: Write> EnumSerializer for &mut JsonWriter<W> {
    type Ok = ();
    type Error = Error;

    fn encode_variant<T>(self, name: &str, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        name.marshal(self)
    }
}

struct JsonArray<'a, W: Write> {
    w: &'a mut JsonWriter<W>,
    first: bool,
}

impl<'a, W: Write> JsonArray<'a, W> {
    fn new(w: &'a mut JsonWriter<W>) -> Result<Self, Error> {
        w.begin('[')?;
        Ok(Self { w, first: true })
    }
}

impl<W: Write> SeqSerializer for JsonArray<'_, W> {
    type Ok = ();
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        if !self.first {
            self.w.comma()?;
        }
        self.w.newl()?;
        self.first = false;
        value.marshal(&mut *self.w)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.w.end(']', self.first)?;
        Ok(())
    }
}

impl<W: Write> ArraySerializer for JsonArray<'_, W> {
    type Ok = ();
    type Error = Error;

    fn encode_next<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        SeqSerializer::encode_next(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SeqSerializer::end(self)
    }
}

/// Serialize the given data structore to a sequence of bytes of JSON data.
pub fn to_bytes<T>(value: &T, pretty: bool) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Marshal,
{
    let mut buf = Vec::with_capacity(128);
    let mut writer = JsonWriter {
        w: &mut buf,
        indent: 0,
        pretty,
    };
    value.marshal(&mut writer)?;
    Ok(buf)
}

/// Serialize the given data structore to string of JSON data.
pub fn to_string<T>(value: &T, pretty: bool) -> Result<String, Error>
where
    T: ?Sized + Marshal,
{
    let bytes = to_bytes(value, pretty)?;
    String::from_utf8(bytes).map_err(Error::custom)
}
