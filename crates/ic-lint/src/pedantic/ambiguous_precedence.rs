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
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{Binary, Expr, OpKind, util};
use ic_vfs::Span;

use crate::{Category, Lint, LintCtx};

/// Lint that warns about potentially ambiguous operator precedence.
///
/// This lint detects cases where operators with different precedence levels
/// are used together without parentheses, which may lead to confusion about
/// the order of evaluation.
pub struct AmbiguousPrecedence<'a> {
    ctx: &'a LintCtx<'a>,
}

impl AmbiguousPrecedence<'_> {
    fn report_ambiguous(
        &self,
        parent_op: &ic_syntax::Op,
        child_op: &ic_syntax::Op,
        _parent_expr: &Binary,
        child_expr: &Binary,
    ) {
        let parent_name = util::op_name(parent_op.kind);
        let child_name = util::op_name(child_op.kind);
        let child_span = Span {
            start: util::expr_span(&child_expr.lhs).start,
            end: util::expr_span(&child_expr.rhs).end,
        };

        if let Some(diag) = self.ctx.diag_span(
            Self::name(),
            Self::category(),
            format!("operator `{parent_name}` has lower precedence than `{child_name}`"),
            Label::new(parent_op.span).message("this operator has lower precedence"),
        ) {
            let diag = diag
                .label(Label::new(child_span).message("than this expression"))
                .help("add parentheses to clarify the intended order of operations");

            Self::report(self.ctx, diag);
        }
    }
}

impl<'a> Visitor<'a> for AmbiguousPrecedence<'a> {
    fn visit_expr_binary(&mut self, binary: &'a Binary) {
        match &binary.lhs {
            Expr::Binary(left_binary) => {
                if is_ambiguous(binary.op.kind, left_binary.op.kind, true) {
                    self.report_ambiguous(&binary.op, &left_binary.op, binary, left_binary);
                }
                self.visit_expr_binary(left_binary);
            }
            Expr::Group(group) => self.visit_expr(&group.expr),
            _ => self.visit_expr(&binary.lhs),
        }

        match &binary.rhs {
            Expr::Binary(right_binary) => {
                if is_ambiguous(binary.op.kind, right_binary.op.kind, false) {
                    self.report_ambiguous(&binary.op, &right_binary.op, binary, right_binary);
                }
                self.visit_expr_binary(right_binary);
            }
            Expr::Group(group) => self.visit_expr(&group.expr),
            _ => self.visit_expr(&binary.rhs),
        }
    }
}

impl<'a> Lint<'a> for AmbiguousPrecedence<'a> {
    fn name() -> &'static str {
        "ambiguous-precedence"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Potentially confusing operator precedence"
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[ic_syntax::Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, tree);
    }
}

/// Check if we should warn about this specific precedence situation.
/// We only warn when mixing operators from different "families" where
/// the precedence might not be intuitive.
fn is_ambiguous(parent_op: OpKind, child_op: OpKind, is_left: bool) -> bool {
    match (parent_op, child_op) {
        // Warn when mixing bitwise and arithmetic, bitwise precedence
        // between different bitwise ops, and when bitiwse ops are mixed
        // with bitshifts.
        (
            OpKind::And | OpKind::Or | OpKind::Xor,
            OpKind::Add
            | OpKind::Sub
            | OpKind::Multiply
            | OpKind::Divide
            | OpKind::Modulo
            | OpKind::Lshift
            | OpKind::Rshift,
        )
        | (OpKind::Or, OpKind::And | OpKind::Xor)
        | (OpKind::Xor, OpKind::And) => true,

        // Special case: for shift operators, only warn on the right side
        // because "a << b + c" is confusing but "a + b << c" is less so
        (OpKind::Lshift | OpKind::Rshift, OpKind::Add | OpKind::Sub) => !is_left,

        // Don't warn about standard arithmetic precedence or any other combinations
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_warn_precedence() {
        // Should warn: shift with arithmetic on right side only
        assert!(!is_ambiguous(OpKind::Lshift, OpKind::Add, true));
        assert!(is_ambiguous(OpKind::Lshift, OpKind::Add, false));

        // Should warn: bitwise with arithmetic
        assert!(is_ambiguous(OpKind::And, OpKind::Add, true));
        assert!(is_ambiguous(OpKind::Or, OpKind::Multiply, false));

        // Should NOT warn: arithmetic precedence is well-known
        assert!(!is_ambiguous(OpKind::Add, OpKind::Multiply, true));
        assert!(!is_ambiguous(OpKind::Sub, OpKind::Divide, false));
    }

    #[test]
    fn test_bitwise_precedence_warnings() {
        // Should warn: different bitwise operators
        assert!(is_ambiguous(OpKind::Or, OpKind::And, true));
        assert!(is_ambiguous(OpKind::Or, OpKind::Xor, false));
        assert!(is_ambiguous(OpKind::Xor, OpKind::And, true));

        // Should NOT warn: same bitwise operator
        assert!(!is_ambiguous(OpKind::And, OpKind::And, true));
        assert!(!is_ambiguous(OpKind::Or, OpKind::Or, false));
    }
}
