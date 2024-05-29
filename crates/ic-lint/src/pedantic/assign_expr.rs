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

use ic_parse::syntax;
use ic_parse::visit::Visitor;

/// Lint that checks for enumerators and bitmask flags where a field was
/// assigned a value using an assignment expression instead of an annotation.
pub struct AssignExpr;

impl<'a> Visitor<'a> for AssignExpr {
    fn visit_bitmask_bit(&mut self, flag: &'a syntax::Bit) {
        if flag.value.is_some() {
            // TODO: we should use the span of the expression
            let span = flag.name.span;
            eprintln!(
                "{}:{}: assignment operator on bitmask flags is an InterCOM extension",
                span.index,
                span.index + span.len,
            );
            eprintln!(" = help: use the `@position` annotation instead");
            eprintln!(" = note: warning produced by -Wpedantic");
        }
    }

    fn visit_enum_variant(&mut self, variant: &'a syntax::Enumerator) {
        if variant.value.is_some() {
            let span = variant.name.span;
            eprintln!(
                "{}:{}: assignment operator on enumerators is an InterCOM extension",
                span.index,
                span.index + span.len,
            );
            eprintln!(" = help: use the `@value` annotation instead");
            eprintln!(" = note: warning produced by -Wpedantic");
        }
    }
}

#[cfg(test)]
mod tests {
    use ic_parse::syntax::*;

    use super::*;

    #[test]
    fn omitted_value() {
        let variant = Enumerator {
            annotations: vec![],
            name: Ident {
                name: Symbol,
                span: Span::default(),
            },
            value: None,
        };
        AssignExpr.visit_enum_variant(&variant);
    }

    #[test]
    fn value_expr() {
        let variant = Enumerator {
            annotations: vec![],
            name: Ident {
                name: Symbol,
                span: Span::default(),
            },
            value: Some(Expr::Numeric(Numeric {
                kind: NumericKind::Int,
                span: Span::default(),
            })),
        };
        AssignExpr.visit_enum_variant(&variant);
    }
}
