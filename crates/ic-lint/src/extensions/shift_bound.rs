// Copyright 2026 KONGSBERG
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
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{Expr, OpKind, Type};

use crate::{Category, Lint, LintCtx};

/// Lint that warns about bitshift expressions in template type bounds.
///
/// This is something that we support but other IDL compilers likely do not.
/// For example, `sequence<string, 1 >> N>` uses `>>` which could be parsed as
/// two closing angle brackets by other compilers.
pub struct ShiftBound<'a> {
    ctx: &'a LintCtx<'a>,
}

impl ShiftBound<'_> {
    fn check_expr(&self, expr: &Expr) {
        if let Some(shift_op) = find_shift_op(expr) {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    "bitshift expressions in template bounds are not portable",
                    Label::new(shift_op.span).message("this shift operator"),
                )
                .note("other IDL compilers may not support bitshift expressions in template bounds")
                .help("consider using a constant instead");
            Self::report(self.ctx, diag);
        }
    }
}

impl<'a> Visitor<'a> for ShiftBound<'a> {
    fn visit_type(&mut self, ty: &'a Type) {
        match ty {
            Type::Sequence(seq) => {
                if let Some(bound) = &seq.bound {
                    self.check_expr(bound);
                }
                self.visit_type(&seq.ty);
            }
            Type::String(s) => {
                if let Some(bound) = &s.bound {
                    self.check_expr(bound);
                }
            }
            Type::Map(m) => {
                if let Some(bound) = &m.bound {
                    self.check_expr(bound);
                }
                self.visit_type(&m.key);
                self.visit_type(&m.value);
            }
            Type::Fixed(f) => {
                if let Some(bounds) = &f.bounds {
                    self.check_expr(&bounds.total);
                    self.check_expr(&bounds.fractional);
                }
            }
            Type::Path(_) => ic_syntax::visit::walk_type(self, ty),
        }
    }
}

impl<'a> Lint<'a> for ShiftBound<'a> {
    fn name() -> &'static str {
        "shift-bound"
    }

    fn category() -> Category {
        Category::Extensions
    }

    fn description() -> &'static str {
        "Bitshift expressions in template bounds"
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[ic_syntax::Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, tree);
    }
}

fn find_shift_op(expr: &Expr) -> Option<&ic_syntax::Op> {
    match expr {
        Expr::Binary(binary) => {
            if matches!(binary.op.kind, OpKind::Lshift | OpKind::Rshift) {
                return Some(&binary.op);
            }
            find_shift_op(&binary.lhs).or_else(|| find_shift_op(&binary.rhs))
        }
        Expr::Unary(unary) => find_shift_op(&unary.expr),
        Expr::Group(group) => find_shift_op(&group.expr),
        _ => None,
    }
}
