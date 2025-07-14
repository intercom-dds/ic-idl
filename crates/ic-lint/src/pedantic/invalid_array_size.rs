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

use ic_diagnostic::Label;
use ic_syntax::{Declarator, Expr, LiteralValue, OpKind};
use ic_syntax::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

pub struct InvalidArraySize<'a> {
    ctx: &'a LintCtx<'a>,
}

// Reasonable limit for array sizes - 1 million elements
const MAX_REASONABLE_ARRAY_SIZE: i64 = 1_000_000;

impl<'a> Lint<'a> for InvalidArraySize<'a> {
    fn name() -> &'static str {
        "InvalidArraySize"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[ic_syntax::Item]) {
        let mut visitor = InvalidArraySize { ctx };
        walk_tree(&mut visitor, ast);
    }
}

impl<'a> InvalidArraySize<'a> {
    fn check_array_bounds(&mut self, arr: &ic_syntax::ArrayDeclarator) {
        for bound_expr in &arr.bounds {
            if let Some(size) = self.get_literal_value(bound_expr) {
                if size > MAX_REASONABLE_ARRAY_SIZE {
                    if let Some(mut diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        &format!(
                            "array size {} exceeds reasonable limit of {}",
                            size, MAX_REASONABLE_ARRAY_SIZE
                        ),
                        Label::new(ic_syntax::util::expr_span(bound_expr))
                            .message("very large array size"),
                    ) {
                        diag = diag.help("consider using a sequence instead for large collections");
                        self.ctx.report(Self::name(), Self::category(), diag);
                    }
                } else if size < 0 {
                    // This would be caught by other lints, but report it here too
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        "negative array size",
                        Label::new(ic_syntax::util::expr_span(bound_expr))
                            .message("array size must be positive"),
                    ) {
                        self.ctx.report(Self::name(), Self::category(), diag);
                    }
                }
            }
        }
    }

    fn get_literal_value(&self, expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Literal(lit) => match &lit.value {
                LiteralValue::Int(v) => Some(*v as i64),
                _ => None,
            },
            Expr::Unary(unary) => {
                if let Some(val) = self.get_literal_value(&unary.expr) {
                    match &unary.op.kind {
                        OpKind::Sub => Some(-val),
                        OpKind::Add => Some(val),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> Visitor<'a> for InvalidArraySize<'a> {
    fn visit_declarator(&mut self, decl: &'a Declarator) {
        if let Declarator::Array(arr) = decl {
            self.check_array_bounds(arr);
        }
        // No walk function for declarator
    }
}