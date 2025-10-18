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

mod de;
mod ser;

pub use de::{from_be_bytes, from_bytes, from_bytes_mut, from_le_bytes};
pub use ser::{to_be_bytes, to_bytes, to_le_bytes};

use crate::cdr::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Encoding {
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

#[derive(Copy, Clone, Debug)]
pub struct Encapsulation {
    pub scheme: Scheme,
    pub len: usize,
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

pub fn encapsulation(bytes: &[u8]) -> Result<Encapsulation, Error> {
    if bytes.len() >= 4 {
        let scheme = Scheme::try_from(bytes[1])?;
        let padding = usize::from(bytes[3] & 3);

        if bytes.len() >= 4 + padding {
            let len = bytes.len() - 4 - padding;
            return Ok(Encapsulation { scheme, len });
        }
    }

    Err(Error::InvalidLen)
}
