// Copyright 2023 KONGSBERG
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

use std::io::Write;
use std::ops::{Deref, DerefMut};

use super::error::Error;
use super::key::KeySerializer;
use crate::encode::{
    ArraySerializer, BitmaskSerializer, EnumSerializer, MapSerializer, Marshal, SeqSerializer,
    Serializer, StructSerializer, UnionSerializer,
};
use crate::error::Error as _;
use crate::type_info::{DISC_INFO, MemberInfo, TypeInfo};

/// Options controlling how a value is serialized to JSON.
#[derive(Copy, Clone, Debug)]
pub struct Options {
    /// Pretty-print the output with indentation and newlines.
    pub pretty: bool,

    /// Serialize integers that fall outside the range of integers that can be
    /// represented exactly by an IEEE-754 double (`[-(2^53 - 1), 2^53 - 1]`) as
    /// JSON strings instead of numbers.
    ///
    /// This is a common convention in JavaScript, where numbers are backed by
    /// doubles and integers beyond `Number.MAX_SAFE_INTEGER` lose precision.
    /// When disabled (the default), all integers are serialized as JSON
    /// numbers, retaining the historical behavior.
    pub large_integers_as_strings: bool,
}

impl Options {
    /// Create a new set of options with all features disabled, matching the
    /// historical serialization behavior.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pretty: false,
            large_integers_as_strings: false,
        }
    }

    /// Enable or disable pretty-printing.
    #[must_use]
    pub const fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Enable or disable serializing large integers as strings.
    #[must_use]
    pub const fn large_integers_as_strings(mut self, enabled: bool) -> Self {
        self.large_integers_as_strings = enabled;
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

struct JsonWriter<W: Write> {
    w: W,
    indent: usize,
    options: Options,
}

impl<W: Write> JsonWriter<W> {
    /// The largest magnitude an integer can have while still being exactly
    /// representable as an IEEE-754 double, matching JavaScript's
    /// `Number.MAX_SAFE_INTEGER` (`2^53 - 1`).
    const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

    fn write<S: AsRef<[u8]>>(&mut self, value: S) -> Result<(), Error> {
        self.w.write_all(value.as_ref()).map_err(Error::custom)
    }

    /// Write a signed integer, encoding it as a JSON string when it falls
    /// outside the safe range and the corresponding option is enabled.
    fn write_i64(&mut self, value: i64) -> Result<(), Error> {
        if self.options.large_integers_as_strings
            && !(-Self::MAX_SAFE_INTEGER..=Self::MAX_SAFE_INTEGER).contains(&value)
        {
            self.write_str(&value.to_string())
        } else {
            self.write(value.to_string())
        }
    }

    /// Write an unsigned integer, encoding it as a JSON string when it falls
    /// outside the safe range and the corresponding option is enabled.
    fn write_u64(&mut self, value: u64) -> Result<(), Error> {
        #[allow(clippy::cast_sign_loss)]
        if self.options.large_integers_as_strings && value > Self::MAX_SAFE_INTEGER as u64 {
            self.write_str(&value.to_string())
        } else {
            self.write(value.to_string())
        }
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
        if self.options.pretty {
            self.write("\n")?;
            self.indent()?;
        }
        Ok(())
    }

    fn write_str(&mut self, value: &str) -> Result<(), Error> {
        self.write("\"")?;
        self.write(value.escape_default().to_string())?;
        self.write("\"")
    }

    fn indent(&mut self) -> Result<(), Error> {
        self.write(" ".repeat(self.indent * 2))
    }
}

impl<'a, W: Write> Serializer<'a> for &'a mut JsonWriter<W> {
    type Ok = ();
    type Error = Error;

    type Struct = JsonObject<'a, W>;
    type Union = JsonObject<'a, W>;
    type Enum = Self;
    type Bitmask = Self;
    type Sequence = JsonArray<'a, W>;
    type Array = JsonArray<'a, W>;
    type Map = JsonObject<'a, W>;

    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        let str = if value { "true" } else { "false" };
        self.write(str)
    }

    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.write_str(&value.to_string())
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
        self.write_i64(value)
    }

    fn encode_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(value)
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

    fn encode_enum(self, _: &TypeInfo<'_>) -> Result<Self::Enum, Self::Error> {
        Ok(self)
    }

    fn encode_bitmask(self, _: &TypeInfo<'_>) -> Result<Self::Bitmask, Self::Error> {
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
        if self.options.pretty {
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

impl<W: Write> StructSerializer<'_> for JsonObject<'_, W> {
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

impl<W: Write> UnionSerializer<'_> for JsonObject<'_, W> {
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

impl<'a, W: Write> EnumSerializer<'a> for &mut JsonWriter<W> {
    type Ok = ();
    type Error = Error;

    fn encode_variant<T>(self, info: &MemberInfo<'a>, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        info.name.marshal(self)
    }
}

impl<'a, W: Write> BitmaskSerializer<'a> for &mut JsonWriter<W> {
    type Ok = ();
    type Error = Error;

    fn encode_flag<T>(self, value: T, members: &[MemberInfo<'a>]) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal + Into<u64>,
    {
        let bits = value.into();

        let mut flags = Vec::new();
        for member in members {
            let bit = 1 << u64::from(member.member_id);
            if (bits & bit) != 0 {
                flags.push(member.name);
            }
        }

        let flags_str = flags.join("|");
        self.write_str(&flags_str)
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
///
/// Integers are always serialized as JSON numbers. Use [`to_bytes_with`] to
/// enable additional behavior such as serializing large integers as strings.
pub fn to_bytes<T>(value: &T, pretty: bool) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Marshal,
{
    to_bytes_with(value, Options::new().pretty(pretty))
}

/// Serialize the given data structore to string of JSON data.
///
/// Integers are always serialized as JSON numbers. Use [`to_string_with`] to
/// enable additional behavior such as serializing large integers as strings.
pub fn to_string<T>(value: &T, pretty: bool) -> Result<String, Error>
where
    T: ?Sized + Marshal,
{
    to_string_with(value, Options::new().pretty(pretty))
}

/// Serialize the given data structure to a sequence of bytes of JSON data,
/// using the provided [`Options`].
pub fn to_bytes_with<T>(value: &T, options: Options) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Marshal,
{
    let mut buf = Vec::with_capacity(128);
    let mut writer = JsonWriter {
        w: &mut buf,
        indent: 0,
        options,
    };
    value.marshal(&mut writer)?;
    Ok(buf)
}

/// Serialize the given data structure to a string of JSON data, using the
/// provided [`Options`].
pub fn to_string_with<T>(value: &T, options: Options) -> Result<String, Error>
where
    T: ?Sized + Marshal,
{
    let bytes = to_bytes_with(value, options)?;
    String::from_utf8(bytes).map_err(Error::custom)
}

#[cfg(test)]
mod tests {
    use super::{Options, to_string, to_string_with};
    use crate::json::from_str;

    const MAX_SAFE: i64 = (1 << 53) - 1;

    fn with_strings() -> Options {
        Options::new().large_integers_as_strings(true)
    }

    #[test]
    fn default_behavior_keeps_integers_as_numbers() {
        // Large integers stay numbers when the option is disabled.
        assert_eq!(
            to_string(&(MAX_SAFE + 1), false).unwrap(),
            "9007199254740992"
        );
        assert_eq!(to_string(&i64::MAX, false).unwrap(), "9223372036854775807");
        assert_eq!(to_string(&u64::MAX, false).unwrap(), "18446744073709551615");
        assert_eq!(to_string(&i64::MIN, false).unwrap(), "-9223372036854775808");
    }

    #[test]
    fn small_integers_stay_numbers_even_when_enabled() {
        // Values inside the safe range are never stringified.
        assert_eq!(to_string_with(&0_i64, with_strings()).unwrap(), "0");
        assert_eq!(
            to_string_with(&MAX_SAFE, with_strings()).unwrap(),
            "9007199254740991"
        );
        assert_eq!(
            to_string_with(&(-MAX_SAFE), with_strings()).unwrap(),
            "-9007199254740991"
        );
        assert_eq!(
            to_string_with(&u64::from(u32::MAX), with_strings()).unwrap(),
            "4294967295"
        );
        // Smaller width integer types can never exceed the safe range.
        assert_eq!(
            to_string_with(&i32::MAX, with_strings()).unwrap(),
            "2147483647"
        );
        // Values between 2^52 and 2^53 - 1 are still safe JS integers and must
        // remain JSON numbers.
        assert_eq!(
            to_string_with(&(1_i64 << 52), with_strings()).unwrap(),
            "4503599627370496"
        );
    }

    #[test]
    fn large_integers_become_strings_when_enabled() {
        let opts = with_strings();
        assert_eq!(
            to_string_with(&(MAX_SAFE + 1), opts).unwrap(),
            "\"9007199254740992\""
        );
        assert_eq!(
            to_string_with(&(-(MAX_SAFE + 1)), opts).unwrap(),
            "\"-9007199254740992\""
        );
        assert_eq!(
            to_string_with(&i64::MAX, opts).unwrap(),
            "\"9223372036854775807\""
        );
        assert_eq!(
            to_string_with(&i64::MIN, opts).unwrap(),
            "\"-9223372036854775808\""
        );
        assert_eq!(
            to_string_with(&u64::MAX, opts).unwrap(),
            "\"18446744073709551615\""
        );
    }

    #[test]
    fn stringified_large_integers_round_trip() {
        let opts = with_strings();

        let json = to_string_with(&i64::MAX, opts).unwrap();
        assert_eq!(from_str::<i64>(&json).unwrap(), i64::MAX);

        let json = to_string_with(&u64::MAX, opts).unwrap();
        assert_eq!(from_str::<u64>(&json).unwrap(), u64::MAX);

        let json = to_string_with(&i64::MIN, opts).unwrap();
        assert_eq!(from_str::<i64>(&json).unwrap(), i64::MIN);
    }

    #[test]
    fn option_can_combine_with_pretty() {
        let opts = Options::new().pretty(true).large_integers_as_strings(true);
        let value = vec![i64::MAX, 1];
        let json = to_string_with(&value, opts).unwrap();
        assert_eq!(json, "[\n  \"9223372036854775807\",\n  1\n]");
    }
}
