// @generated
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

pub mod xtypes;

pub type ParameterId = u16;

pub const PID_SENTINEL: crate::types::ParameterId = 0x1;

pub const PID_EXTENDED: crate::types::ParameterId = 0x3F01;

pub const PID_LIST_END: crate::types::ParameterId = 0x3F02;

pub const PID_IGNORE: crate::types::ParameterId = 0x3F03;

pub const PID_FLAG_MUST_UNDERSTAND: crate::types::ParameterId = 0x4000;

pub const PID_FLAG_IMPL_EXTENSION: crate::types::ParameterId = 0x8000;

pub const PID_PID_MASK: crate::types::ParameterId = 0x3FFF;

::intercom_cts::bitmask! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub SerializerFlags: u32 {
        SERIALIZER_KEY_ONLY = 1 << 2,
        SERIALIZER_SKIP_MISSING = 1 << 3,
        SERIALIZER_PRETTY = 1 << 4,
        SERIALIZER_STRICT = 1 << 5,
        CDR_LITTLE_ENDIAN = 1 << 6,
        CDR_BIG_ENDIAN = 1 << 7,
        CDR_XCDR1 = 1 << 8,
        CDR_XCDR2 = 1 << 9,
        CDR_XCDR_PLAIN = 1 << 10,
        CDR_XCDR_BUILTIN = 1 << 11,
        SERIALIZER_MINIMUM_PROFILE = 1 << 12,
    }
}

impl SerializerFlags {
    #[must_use]
    pub fn new() -> Self {
        crate::types::SerializerFlags(0)
    }
}

impl ::std::default::Default for SerializerFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum EncapsulationSchemeIdentifier {
    CdrBe,
    CdrLe,
    PlCdrBe,
    PlCdrLe,
    Xml,
    Cdr2Be = 6,
    Cdr2Le,
    DelimitedCdr2Be,
    DelimitedCdr2Le,
    PlCdr2Be,
    PlCdr2Le,
    Cdr2BeOld = 16,
    Cdr2LeOld,
    PlCdr2BeOld,
    PlCdr2LeOld,
    DelimitedCdr2BeOld,
    DelimitedCdr2LeOld,
    PlainCdrBe = 128,
    PlainCdrLe,
}

impl EncapsulationSchemeIdentifier {
    #[must_use]
    pub const fn new() -> Self {
        crate::types::EncapsulationSchemeIdentifier::CdrBe
    }
}

impl ::std::str::FromStr for EncapsulationSchemeIdentifier {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "ENC_CDR_BE" => Ok(Self::CdrBe),
            "ENC_CDR_LE" => Ok(Self::CdrLe),
            "ENC_PL_CDR_BE" => Ok(Self::PlCdrBe),
            "ENC_PL_CDR_LE" => Ok(Self::PlCdrLe),
            "ENC_XML" => Ok(Self::Xml),
            "ENC_CDR2_BE" => Ok(Self::Cdr2Be),
            "ENC_CDR2_LE" => Ok(Self::Cdr2Le),
            "ENC_DELIMITED_CDR2_BE" => Ok(Self::DelimitedCdr2Be),
            "ENC_DELIMITED_CDR2_LE" => Ok(Self::DelimitedCdr2Le),
            "ENC_PL_CDR2_BE" => Ok(Self::PlCdr2Be),
            "ENC_PL_CDR2_LE" => Ok(Self::PlCdr2Le),
            "ENC_CDR2_BE_OLD" => Ok(Self::Cdr2BeOld),
            "ENC_CDR2_LE_OLD" => Ok(Self::Cdr2LeOld),
            "ENC_PL_CDR2_BE_OLD" => Ok(Self::PlCdr2BeOld),
            "ENC_PL_CDR2_LE_OLD" => Ok(Self::PlCdr2LeOld),
            "ENC_DELIMITED_CDR2_BE_OLD" => Ok(Self::DelimitedCdr2BeOld),
            "ENC_DELIMITED_CDR2_LE_OLD" => Ok(Self::DelimitedCdr2LeOld),
            "ENC_PLAIN_CDR_BE" => Ok(Self::PlainCdrBe),
            "ENC_PLAIN_CDR_LE" => Ok(Self::PlainCdrLe),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for EncapsulationSchemeIdentifier {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::CdrBe => f.write_str("ENC_CDR_BE"),
            Self::CdrLe => f.write_str("ENC_CDR_LE"),
            Self::PlCdrBe => f.write_str("ENC_PL_CDR_BE"),
            Self::PlCdrLe => f.write_str("ENC_PL_CDR_LE"),
            Self::Xml => f.write_str("ENC_XML"),
            Self::Cdr2Be => f.write_str("ENC_CDR2_BE"),
            Self::Cdr2Le => f.write_str("ENC_CDR2_LE"),
            Self::DelimitedCdr2Be => f.write_str("ENC_DELIMITED_CDR2_BE"),
            Self::DelimitedCdr2Le => f.write_str("ENC_DELIMITED_CDR2_LE"),
            Self::PlCdr2Be => f.write_str("ENC_PL_CDR2_BE"),
            Self::PlCdr2Le => f.write_str("ENC_PL_CDR2_LE"),
            Self::Cdr2BeOld => f.write_str("ENC_CDR2_BE_OLD"),
            Self::Cdr2LeOld => f.write_str("ENC_CDR2_LE_OLD"),
            Self::PlCdr2BeOld => f.write_str("ENC_PL_CDR2_BE_OLD"),
            Self::PlCdr2LeOld => f.write_str("ENC_PL_CDR2_LE_OLD"),
            Self::DelimitedCdr2BeOld => f.write_str("ENC_DELIMITED_CDR2_BE_OLD"),
            Self::DelimitedCdr2LeOld => f.write_str("ENC_DELIMITED_CDR2_LE_OLD"),
            Self::PlainCdrBe => f.write_str("ENC_PLAIN_CDR_BE"),
            Self::PlainCdrLe => f.write_str("ENC_PLAIN_CDR_LE"),
        }
    }
}

impl ::std::default::Default for EncapsulationSchemeIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

pub const ENCAPSULATION_SIZE: u16 = 4;

