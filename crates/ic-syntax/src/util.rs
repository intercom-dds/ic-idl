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

use crate::{Expr, Path, Span, Type};

#[must_use]
pub fn path_name(path: &Path) -> String {
    let mut segments = vec![];
    if path.leading_colons.is_some() {
        segments.push("::");
    }
    segments.extend(path.segments.iter().map(|v| v.name.as_str()));
    segments.join("::")
}

#[must_use]
pub fn type_name(path: &Type) -> String {
    match path {
        Type::Any(..) => "any".to_string(),
        Type::String_(..) => "string".to_string(),
        Type::Map(..) => "map".to_string(),
        Type::Fixed(..) => "fixed".to_string(),
        Type::Sequence(seq) => format!("sequence<{}>", type_name(seq.ty.as_ref())),
        Type::Path(ty) => path_name(ty),
    }
}

#[must_use]
pub fn path_span(path: &Path) -> Span {
    let start = path.leading_colons.map_or_else(
        || path.segments.first().map_or(0, |v| v.span.start),
        |v| v.start,
    );

    let end = path.segments.last().map_or(0, |v| v.span.end);
    Span { start, end }
}

#[must_use]
pub fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Literal(v) => v.span,
        Expr::Path(v) => path_span(v),
        Expr::Unary(v) => {
            let start = v.op.span.start;
            let end = expr_span(&v.expr).end;
            Span { start, end }
        }
        Expr::Binary(v) => {
            let start = expr_span(&v.lhs).start;
            let end = expr_span(&v.rhs).end;
            Span { start, end }
        }
        Expr::InitList(v) => {
            let start = v.first().map(expr_span).unwrap_or_default().start;
            let end = v.last().map(expr_span).unwrap_or_default().end;
            Span { start, end }
        }
    }
}

#[must_use]
pub fn ty_span(ty: &Type) -> Span {
    match ty {
        Type::Any(v) => v.span,
        Type::Sequence(v) => v.span,
        Type::String_(v) => v.span,
        Type::Map(v) => v.span,
        Type::Fixed(v) => v.span,
        Type::Path(v) => path_span(v),
    }
}

impl Expr {
    #[must_use]
    pub fn span(&self) -> Span {
        expr_span(self)
    }
}
