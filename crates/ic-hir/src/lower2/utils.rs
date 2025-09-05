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

//! Common utilities for the lowering process.

use ic_syntax::Path;

use crate::hir::Numeric;

/// Convert a path to a string for error messages.
pub fn path_to_string(path: &Path) -> String {
    let mut result = String::new();

    if path.leading_colons.is_some() {
        result.push_str("::");
    }

    for (i, segment) in path.segments.iter().enumerate() {
        if i > 0 {
            result.push_str("::");
        }
        result.push_str(&segment.name);
    }

    result
}

/// Convert AST literal to HIR numeric value.
pub fn literal_to_numeric(lit: &ic_syntax::LiteralValue) -> Numeric {
    match lit {
        ic_syntax::LiteralValue::Bool(b) => Numeric::Bool(*b),
        ic_syntax::LiteralValue::Char(c) => Numeric::Char(*c),
        ic_syntax::LiteralValue::Int(i) => {
            // Default to Int32 for untyped integers
            #[allow(clippy::cast_possible_truncation)]
            Numeric::Int32(*i as i32)
        }
        ic_syntax::LiteralValue::Float(f) => Numeric::Double(*f),
        ic_syntax::LiteralValue::String(s) => Numeric::String(s.clone()),
        ic_syntax::LiteralValue::Null => Numeric::Null,
    }
}

/// Get the span of a path.
pub fn path_span(path: &Path) -> ic_syntax::Span {
    ic_syntax::util::path_span(path)
}

/// Extension trait for Ty to get ADT `DefId`.
pub trait TyExt {
    fn as_adt(&self) -> Option<crate::hir::DefId>;
}

impl TyExt for crate::hir::Ty {
    fn as_adt(&self) -> Option<crate::hir::DefId> {
        match &self.kind {
            crate::hir::TyKind::Adt(id) => Some(*id),
            _ => None,
        }
    }
}
