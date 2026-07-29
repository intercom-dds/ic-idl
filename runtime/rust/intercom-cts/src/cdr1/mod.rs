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

mod de;
mod ser;

pub use de::{CdrReader, from_be_bytes, from_bytes, from_bytes_mut, from_le_bytes};
pub use ser::{CdrWriter, to_be_bytes, to_buffer, to_bytes, to_le_bytes};

use crate::buf::endian::Endian;
use crate::buf::{Big, Buffer, Little};
pub use crate::cdr::Error;
use crate::decode::Unmarshal;
use crate::{cdr, cdr2};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Encoding {
    /// Plain CDR encoding, typically used for final types.
    Plain,

    /// Delimited CDR encoding, typically used for appendable types.
    Delimited,

    /// Parameter list CDR encoding, typically used for mutable types.
    PL,
}

crate::bitmask! {
    #[derive(Copy, Clone)]
    pub MemberFlag: u16 {
        TRY_CONSTRUCT1 = 0x01,
        TRY_CONSTRUCT2 = 0x02,
        IS_EXTERNAL = 0x04,
        IS_OPTIONAL = 0x08,
        IS_MUST_UNDERSTAND = 0x10,
        IS_KEY = 0x20,
        IS_DEFAULT = 0x40,
    }
}

crate::bitmask! {
    #[derive(Copy, Clone)]
    pub TypeFlag: u16 {
        IS_FINAL = 0x01,
        IS_APPENDABLE = 0x02,
        IS_MUTABLE = 0x04,
        IS_NESTED = 0x08,
        IS_AUTOID_HASH = 0x10,
        IS_KEYED = 0x20,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scheme {
    CdrBe = 0,
    CdrLe = 1,
    PlCdrBe = 2,
    PlCdrLe = 3,
    Xml = 4,
    Cdr2Be = 6,
    Cdr2Le = 7,
    DelimitedCdr2Be = 8,
    DelimitedCdr2Le = 9,
    PlCdr2Be = 10,
    PlCdr2Le = 11,
    PlainCdrBe = 128,
    PlainCdrLe = 129,
}

impl Scheme {
    #[must_use]
    pub const fn is_le(self) -> bool {
        matches!(
            self,
            Scheme::CdrLe
                | Scheme::PlCdrLe
                | Scheme::Cdr2Le
                | Scheme::DelimitedCdr2Le
                | Scheme::PlCdr2Le
                | Scheme::PlainCdrLe
        )
    }

    #[must_use]
    pub const fn is_be(self) -> bool {
        !self.is_le()
    }
}

impl TryFrom<u8> for Scheme {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::CdrBe),
            1 => Ok(Self::CdrLe),
            2 => Ok(Self::PlCdrBe),
            3 => Ok(Self::PlCdrLe),
            4 => Ok(Self::Xml),
            6 => Ok(Self::Cdr2Be),
            7 => Ok(Self::Cdr2Le),
            8 => Ok(Self::DelimitedCdr2Be),
            9 => Ok(Self::DelimitedCdr2Le),
            10 => Ok(Self::PlCdr2Be),
            11 => Ok(Self::PlCdr2Le),
            128 => Ok(Self::PlainCdrBe),
            129 => Ok(Self::PlainCdrLe),
            _ => Err(Error::UnsupportedEnc),
        }
    }
}

pub fn encapsulation(bytes: &[u8]) -> Result<(Scheme, &[u8]), Error> {
    if bytes.len() >= 4 {
        let scheme = Scheme::try_from(bytes[1])?;
        let padding = usize::from(bytes[3] & 3);

        if bytes.len() >= 4 + padding {
            let len = bytes.len() - 4 - padding;
            return Ok((scheme, &bytes[4..4 + len]));
        }
    }

    Err(Error::InvalidLen)
}

pub fn write_encapsulation<E: Endian>(buffer: &mut Buffer<E>, scheme: Scheme) {
    buffer.write_u8(0);
    buffer.write_u8(scheme as u8);
    buffer.write_u8(0);
    buffer.write_u8(0);
}

pub fn decode_into<T: Unmarshal>(bytes: &[u8], value: &mut T) -> cdr::Result<()> {
    let (scheme, bytes) = encapsulation(bytes)?;
    match scheme {
        Scheme::CdrLe | Scheme::PlCdrLe => from_bytes_mut::<_, Little>(bytes, value),
        Scheme::CdrBe | Scheme::PlCdrBe => from_bytes_mut::<_, Big>(bytes, value),
        Scheme::Cdr2Le | Scheme::DelimitedCdr2Le | Scheme::PlCdr2Le => {
            cdr2::from_bytes_mut::<_, Little>(bytes, value)
        }
        Scheme::Cdr2Be | Scheme::DelimitedCdr2Be | Scheme::PlCdr2Be => {
            cdr2::from_bytes_mut::<_, Big>(bytes, value)
        }
        Scheme::PlainCdrLe => cdr::from_bytes_mut::<_, Little>(bytes, value),
        Scheme::PlainCdrBe => cdr::from_bytes_mut::<_, Big>(bytes, value),
        Scheme::Xml => Err(Error::UnsupportedEnc),
    }
}

pub fn decode<T: Unmarshal + Default>(bytes: &[u8]) -> cdr::Result<T> {
    let mut value = T::default();
    decode_into(bytes, &mut value)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encapsulate(scheme: Scheme, payload: Vec<u8>) -> Vec<u8> {
        let mut bytes = vec![0, scheme as u8, 0, 0];
        bytes.extend(payload);
        bytes
    }

    #[test]
    fn decodes_cdr_le() {
        let payload = to_bytes::<_, Little>(&"hello".to_string()).unwrap();
        let bytes = encapsulate(Scheme::CdrLe, payload);

        let decoded: String = decode(&bytes).unwrap();
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn decodes_cdr_be() {
        let payload = to_bytes::<_, Big>(&0x1234_5678u32).unwrap();
        let bytes = encapsulate(Scheme::CdrBe, payload);

        let decoded: u32 = decode(&bytes).unwrap();
        assert_eq!(decoded, 0x1234_5678);
    }

    #[test]
    fn decodes_cdr2_le() {
        let payload = cdr2::to_bytes::<_, Little>(&"hello".to_string()).unwrap();
        let bytes = encapsulate(Scheme::Cdr2Le, payload);

        let decoded: String = decode(&bytes).unwrap();
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn decodes_pl_cdr_le() {
        let payload = to_bytes::<_, Little>(&42u32).unwrap();
        let bytes = encapsulate(Scheme::PlCdrLe, payload);

        let mut decoded = 0u32;
        decode_into(&bytes, &mut decoded).unwrap();
        assert_eq!(decoded, 42);
    }

    #[test]
    fn errors_on_truncated_input() {
        assert!(decode::<u32>(&[0, 1, 0]).is_err());

        let bytes = encapsulate(Scheme::CdrLe, vec![1, 2]);
        assert!(decode::<u32>(&bytes).is_err());
    }

    #[test]
    fn errors_on_xml_scheme() {
        let bytes = encapsulate(Scheme::Xml, vec![]);
        assert!(matches!(decode::<u32>(&bytes), Err(Error::UnsupportedEnc)));
    }
}
