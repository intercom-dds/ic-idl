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
use ic_syntax::{BinaryExpr, ExprKind, Op, Spanned, util};
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
        parent_op: &Spanned<Op>,
        child_op: &Spanned<Op>,
        _parent_expr: &BinaryExpr,
        child_expr: &BinaryExpr,
    ) {
        let parent_name = util::op_name(parent_op.value);
        let child_name = util::op_name(child_op.value);
        let child_span = Span {
            start: child_expr.lhs.span.start,
            end: child_expr.rhs.span.end,
        };

        let diag = self
            .ctx
            .diag_span(
                Self::name(),
                Self::category(),
                format!("operator `{parent_name}` has lower precedence than `{child_name}`"),
                Label::new(parent_op.span).message("this operator has lower precedence"),
            )
            .label(Label::new(child_span).message("than this expression"))
            .help("add parentheses to clarify the intended order of operations");
        Self::report(self.ctx, diag);
    }
}

impl<'a> Visitor<'a> for AmbiguousPrecedence<'a> {
    fn visit_expr_binary(&mut self, binary: &'a BinaryExpr) {
        match &binary.lhs.value {
            ExprKind::Binary(left_binary) => {
                if is_ambiguous(binary.op.value, left_binary.op.value, true) {
                    self.report_ambiguous(&binary.op, &left_binary.op, binary, left_binary);
                }
                self.visit_expr_binary(left_binary);
            }
            ExprKind::Group(group) => self.visit_expr(group),
            _ => self.visit_expr(&binary.lhs),
        }

        match &binary.rhs.value {
            ExprKind::Binary(right_binary) => {
                if is_ambiguous(binary.op.value, right_binary.op.value, false) {
                    self.report_ambiguous(&binary.op, &right_binary.op, binary, right_binary);
                }
                self.visit_expr_binary(right_binary);
            }
            ExprKind::Group(group) => self.visit_expr(group),
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
fn is_ambiguous(parent_op: Op, child_op: Op, is_left: bool) -> bool {
    match (parent_op, child_op) {
        // Warn when mixing bitwise and arithmetic, bitwise precedence
        // between different bitwise ops, and when bitiwse ops are mixed
        // with bitshifts.
        (
            Op::And | Op::Or | Op::Xor,
            Op::Add | Op::Sub | Op::Multiply | Op::Divide | Op::Modulo | Op::LShift | Op::RShift,
        )
        | (Op::Or, Op::And | Op::Xor)
        | (Op::Xor, Op::And) => true,

        // Special case: for shift operators, only warn on the right side
        // because "a << b + c" is confusing but "a + b << c" is less so
        (Op::LShift | Op::RShift, Op::Add | Op::Sub) => !is_left,

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
        assert!(!is_ambiguous(Op::LShift, Op::Add, true));
        assert!(is_ambiguous(Op::LShift, Op::Add, false));

        // Should warn: bitwise with arithmetic
        assert!(is_ambiguous(Op::And, Op::Add, true));
        assert!(is_ambiguous(Op::Or, Op::Multiply, false));

        // Should NOT warn: arithmetic precedence is well-known
        assert!(!is_ambiguous(Op::Add, Op::Multiply, true));
        assert!(!is_ambiguous(Op::Sub, Op::Divide, false));
    }

    #[test]
    fn test_bitwise_precedence_warnings() {
        // Should warn: different bitwise operators
        assert!(is_ambiguous(Op::Or, Op::And, true));
        assert!(is_ambiguous(Op::Or, Op::Xor, false));
        assert!(is_ambiguous(Op::Xor, Op::And, true));

        // Should NOT warn: same bitwise operator
        assert!(!is_ambiguous(Op::And, Op::And, true));
        assert!(!is_ambiguous(Op::Or, Op::Or, false));
    }
}
