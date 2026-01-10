// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

use std::{error, fmt};

use crate::buf;

/// This type represents all possible errors that can occur when serializing or
/// deseriailzing CDR data.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Error {
    Eof,
    InvalidUtf8,
    InvalidChar,
    UnsupportedType,
    UnsupportedEnc,
    InvalidLen,
    Unknown(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eof => write!(f, "Unexpected EOF"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 codepoint"),
            Self::InvalidChar => write!(f, "Invalid value for char"),
            Self::UnsupportedType => write!(f, "Unsupported type"),
            Self::UnsupportedEnc => write!(f, "Unsupported encoding"),
            Self::InvalidLen => write!(f, "Invalid length of container"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

impl self::error::Error for Error {}

impl crate::error::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

impl From<buf::Error> for Error {
    fn from(_: buf::Error) -> Self {
        Self::Eof
    }
}
