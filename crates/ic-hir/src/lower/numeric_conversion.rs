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

//! Numeric type conversion utilities for expression evaluation.

use ic_expr::GenericNumeric;

use crate::hir::{DefId, Ident, Numeric};

/// Converts a generic numeric value from ic-expr to HIR numeric.
pub fn to_hir_numeric(val: GenericNumeric) -> Numeric {
    match val {
        GenericNumeric::Null => Numeric::Null,
        GenericNumeric::Bool(v) => Numeric::Bool(v),
        GenericNumeric::Int(v) => Numeric::Int64(v),
        GenericNumeric::Uint(v) => Numeric::UInt64(v),
        GenericNumeric::Float(v) => Numeric::Double(v),
        GenericNumeric::Char(v) => Numeric::Char(v),
        GenericNumeric::String(v) => Numeric::String(v),
        GenericNumeric::Array(vec) => Numeric::Array {
            values: vec.into_iter().map(to_hir_numeric).collect(),
        },
        GenericNumeric::Map(vec) => Numeric::Map {
            values: vec
                .into_iter()
                .map(|(k, v)| (to_hir_numeric(k), to_hir_numeric(v)))
                .collect(),
        },
        GenericNumeric::Sequence(vec) => Numeric::Sequence {
            values: vec.into_iter().map(to_hir_numeric).collect(),
        },
    }
}

/// Converts a HIR numeric value to generic numeric for expression evaluation.
pub fn from_hir_numeric(n: &Numeric) -> Option<GenericNumeric> {
    Some(match n {
        Numeric::Null => GenericNumeric::Null,
        Numeric::String(s) => GenericNumeric::String(s.clone()),
        Numeric::Bool(v) => GenericNumeric::Bool(*v),
        Numeric::Char(v) => GenericNumeric::Char(*v),
        Numeric::Int8(v) => GenericNumeric::Int(i64::from(*v)),
        Numeric::Octet(v) => GenericNumeric::Uint(u64::from(*v)),
        Numeric::Int16(v) => GenericNumeric::Int(i64::from(*v)),
        Numeric::UInt16(v) => GenericNumeric::Uint(u64::from(*v)),
        Numeric::Int32(v) => GenericNumeric::Int(i64::from(*v)),
        Numeric::UInt32(v) => GenericNumeric::Uint(u64::from(*v)),
        Numeric::Int64(v) => GenericNumeric::Int(*v),
        Numeric::UInt64(v) => GenericNumeric::Uint(*v),
        Numeric::Float(v) => GenericNumeric::Float(f64::from(*v)),
        Numeric::Double(v) => GenericNumeric::Float(*v),
        Numeric::Const(_) | Numeric::Struct { .. } | Numeric::Union { .. } => return None,
        Numeric::Array { values } => {
            let converted: Option<Vec<_>> = values.iter().map(from_hir_numeric).collect();
            GenericNumeric::Array(converted?)
        }
        Numeric::Sequence { values } => {
            let converted: Option<Vec<_>> = values.iter().map(from_hir_numeric).collect();
            GenericNumeric::Sequence(converted?)
        }
        Numeric::Map { values } => {
            let converted: Option<Vec<_>> = values
                .iter()
                .map(|(k, v)| {
                    Some((from_hir_numeric(k)?, from_hir_numeric(v)?))
                })
                .collect();
            GenericNumeric::Map(converted?)
        }
    })
}

/// Literal type for ic-expr evaluation.
#[derive(Debug, Clone)]
pub struct IdlLiteral {
    pub const_id: Option<DefId>,
    pub enum_id: Option<DefId>,
    pub field: Option<Ident>,
    pub numeric: GenericNumeric,
}