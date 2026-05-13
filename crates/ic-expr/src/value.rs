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

use std::fmt;

use crate::rank::{FloatRank, IntRank};

/// A value with type information, generic over external references.
///
/// The type parameter `R` represents external references (e.g., `DefId` for
/// HIR, or `Infallible` when references aren't needed).
#[derive(Clone, Debug)]
pub enum Value<R> {
    /// Signed integer with rank.
    Int(i128, IntRank),

    /// Unsigned integer with rank.
    UInt(u128, IntRank),

    /// Floating-point with rank.
    Float(f64, FloatRank),

    /// Boolean.
    Bool(bool),

    /// Narrow character.
    Char(char),

    /// Wide character.
    WChar(char),

    /// Narrow string.
    String(String),

    /// Wide string.
    WString(String),

    /// Null value.
    Null,

    /// External reference (e.g., constant `DefId`).
    Ref(R),
}

impl<R> Value<R> {
    /// Create a signed integer value.
    #[must_use]
    pub fn int(v: i128, rank: IntRank) -> Self {
        Self::Int(v, rank)
    }

    /// Create an unsigned integer value.
    #[must_use]
    pub fn uint(v: u128, rank: IntRank) -> Self {
        Self::UInt(v, rank)
    }

    /// Create a float value.
    #[must_use]
    pub fn float(v: f64, rank: FloatRank) -> Self {
        Self::Float(v, rank)
    }

    /// Check if this is an integer (signed or unsigned).
    #[must_use]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..) | Self::UInt(..))
    }

    /// Check if this is a float.
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(..))
    }

    /// Check if this is a reference.
    #[must_use]
    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    /// Get the integer rank if this is an integer.
    #[must_use]
    pub fn int_rank(&self) -> Option<IntRank> {
        match self {
            Self::Int(_, r) | Self::UInt(_, r) => Some(*r),
            _ => None,
        }
    }

    /// Get the float rank if this is a float.
    #[must_use]
    pub fn float_rank(&self) -> Option<FloatRank> {
        match self {
            Self::Float(_, r) => Some(*r),
            _ => None,
        }
    }

    /// Convert to i128 if possible.
    #[must_use]
    pub fn to_i128(&self) -> Option<i128> {
        match self {
            Self::Int(v, _) => Some(*v),
            Self::UInt(v, _) => i128::try_from(*v).ok(),
            Self::Bool(b) => Some(i128::from(*b)),
            Self::Char(c) => Some(i128::from(*c as u32)),
            _ => None,
        }
    }

    /// Convert to u128 if possible.
    #[must_use]
    pub fn to_u128(&self) -> Option<u128> {
        match self {
            Self::Int(v, _) if *v >= 0 => Some(*v as u128),
            Self::UInt(v, _) => Some(*v),
            Self::Bool(b) => Some(u128::from(*b)),
            Self::Char(c) => Some(u128::from(*c as u32)),
            _ => None,
        }
    }

    /// Convert to f64 if possible.
    #[must_use]
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Int(v, _) => Some(*v as f64),
            Self::UInt(v, _) => Some(*v as f64),
            Self::Float(v, _) => Some(*v),
            _ => None,
        }
    }

    /// Convert to bool.
    #[must_use]
    pub fn to_bool(&self) -> bool {
        match self {
            Self::Int(v, _) => *v != 0,
            Self::UInt(v, _) => *v != 0,
            Self::Float(v, _) => *v != 0.0,
            Self::Bool(b) => *b,
            Self::Char(c) | Self::WChar(c) => *c != '\0',
            Self::String(s) | Self::WString(s) => !s.is_empty(),
            Self::Null => false,
            Self::Ref(_) => true,
        }
    }

    /// Human-readable description of the value kind for error messages.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Int(..) => "integer value",
            Self::UInt(..) => "unsigned integer value",
            Self::Float(..) => "floating-point value",
            Self::Bool(_) => "boolean value",
            Self::Char(_) => "character value",
            Self::WChar(_) => "wide character value",
            Self::String(_) => "string value",
            Self::WString(_) => "wide string value",
            Self::Null => "null value",
            Self::Ref(_) => "constant reference",
        }
    }
}

impl<R: fmt::Display> fmt::Display for Value<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v, _) => write!(f, "{v}"),
            Self::UInt(v, _) => write!(f, "{v}"),
            Self::Float(v, _) => write!(f, "{v}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Char(c) => write!(f, "'{c}'"),
            Self::WChar(c) => write!(f, "L'{c}'"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::WString(s) => write!(f, "L\"{s}\""),
            Self::Null => write!(f, "null"),
            Self::Ref(r) => write!(f, "{r}"),
        }
    }
}
