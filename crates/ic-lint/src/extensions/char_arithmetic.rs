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

use ic_diagnostic::Label;
use ic_syntax::{Expr, Item, LiteralValue, OpKind};

use crate::{Category, Lint, LintCtx};

/// Lint that warns about using char literals in arithmetic expressions.
pub struct CharArithmetic<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for CharArithmetic<'a> {
    fn name() -> &'static str {
        "char-arithmetic"
    }

    fn description() -> &'static str {
        "Char literals in arithmetic expressions"
    }

    fn category() -> Category {
        Category::Extensions
    }

    fn check(ctx: &'a LintCtx<'a>, ast: &[Item]) {
        let mut visitor = Self { ctx };
        ic_syntax::visit::walk_tree(&mut visitor, ast);
    }
}

impl<'a> ic_syntax::visit::Visitor<'a> for CharArithmetic<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Binary(binary) => {
                if matches!(
                    binary.op.kind,
                    OpKind::Add
                        | OpKind::Sub
                        | OpKind::Multiply
                        | OpKind::Divide
                        | OpKind::Modulo
                        | OpKind::And
                        | OpKind::Or
                        | OpKind::Xor
                        | OpKind::Lshift
                        | OpKind::Rshift
                ) {
                    if let Expr::Literal(lit) = &binary.lhs
                        && let LiteralValue::Char(_) = lit.value
                    {
                        let diag = self
                            .ctx
                            .diag_span(
                                Self::name(),
                                Self::category(),
                                "char literal used in arithmetic expression",
                                Label::new(lit.span).message("char literal"),
                            )
                            .help("consider converting to an integer value");
                        Self::report(self.ctx, diag);
                    }

                    if let Expr::Literal(lit) = &binary.rhs
                        && let LiteralValue::Char(_) = lit.value
                    {
                        let diag = self
                            .ctx
                            .diag_span(
                                Self::name(),
                                Self::category(),
                                "char literal used in arithmetic expression",
                                Label::new(lit.span).message("char literal"),
                            )
                            .help("consider converting to an integer value");
                        Self::report(self.ctx, diag);
                    }
                }
            }
            Expr::Unary(unary) => {
                if matches!(unary.op.kind, OpKind::Not | OpKind::Sub | OpKind::Add)
                    && let Expr::Literal(lit) = &unary.expr
                    && let LiteralValue::Char(_) = lit.value
                {
                    let diag = self
                        .ctx
                        .diag_span(
                            Self::name(),
                            Self::category(),
                            "char literal used in arithmetic expression",
                            Label::new(lit.span).message("char literal"),
                        )
                        .help("consider converting to an integer value");
                    Self::report(self.ctx, diag);
                }
            }
            _ => {}
        }

        ic_syntax::visit::walk_expr(self, expr);
    }
}
