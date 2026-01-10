// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2024 KONGSBERG - All rights reserved

mod de;
mod ser;

pub use de::{CdrReader, from_be_bytes, from_bytes, from_bytes_mut, from_le_bytes};
pub use ser::{CdrWriter, to_be_bytes, to_buffer, to_bytes, to_le_bytes};

use crate::buf::Buffer;
use crate::buf::endian::Endian;
pub use crate::cdr::Error;

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
    pub fn is_le(&self) -> bool {
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
    pub fn is_be(&self) -> bool {
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
