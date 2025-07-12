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

use crate::{Category, Lint, LintCtx, lint_impl};

/// Lint that warns about potentially ambiguous operator precedence.
///
/// This lint detects cases where operators with different precedence levels
/// are used together without parentheses, which may lead to confusion about
/// the order of evaluation.
pub struct AmbiguousPrecedence<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> AmbiguousPrecedence<'a> {
    /// Get a human-readable name for an operator
    fn op_name(op: OpKind) -> &'static str {
        match op {
            OpKind::Add => "+",
            OpKind::Sub => "-",
            OpKind::Multiply => "*",
            OpKind::Divide => "/",
            OpKind::Modulo => "%",
            OpKind::And => "&",
            OpKind::Or => "|",
            OpKind::Xor => "^",
            OpKind::Lshift => "<<",
            OpKind::Rshift => ">>",
            OpKind::Not => "~",
        }
    }

    fn precedence(op: OpKind) -> u8 {
        match op {
            OpKind::Or => 1,
            OpKind::Xor => 2,
            OpKind::And => 3,
            OpKind::Lshift | OpKind::Rshift => 4,
            OpKind::Add | OpKind::Sub => 5,
            OpKind::Multiply | OpKind::Divide | OpKind::Modulo => 6,
            OpKind::Not => 7,
        }
    }

    /// Check if we should warn about this specific precedence situation.
    /// We only warn when mixing operators from different "families" where
    /// the precedence might not be intuitive.
    fn should_warn_precedence(parent_op: OpKind, child_op: OpKind, is_left: bool) -> bool {
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

    /// Check a binary expression for ambiguous precedence
    fn check_binary_expr(&mut self, binary: &'a Binary) {
        match &binary.lhs {
            Expr::Binary(left_binary) => {
                if Self::should_warn_precedence(binary.op.kind, left_binary.op.kind, true) {
                    self.report_ambiguous_precedence(
                        &binary.op,
                        &left_binary.op,
                        binary,
                        left_binary,
                    );
                }
                self.visit_expr_binary(left_binary);
            }
            Expr::Group(group) => self.visit_expr(&group.expr),
            _ => self.visit_expr(&binary.lhs),
        }

        match &binary.rhs {
            Expr::Binary(right_binary) => {
                if Self::should_warn_precedence(binary.op.kind, right_binary.op.kind, false) {
                    self.report_ambiguous_precedence(
                        &binary.op,
                        &right_binary.op,
                        binary,
                        right_binary,
                    );
                }
                self.visit_expr_binary(right_binary);
            }
            Expr::Group(group) => self.visit_expr(&group.expr),
            _ => self.visit_expr(&binary.rhs),
        }
    }

    fn report_ambiguous_precedence(
        &self,
        parent_op: &ic_syntax::Op,
        child_op: &ic_syntax::Op,
        parent_expr: &Binary,
        child_expr: &Binary,
    ) {
        let parent_name = Self::op_name(parent_op.kind);
        let child_name = Self::op_name(child_op.kind);
        let child_span = util::expr_span(&Expr::Binary(Box::new(child_expr.clone())));

        if let Some(diag) = self.ctx.diag_span(
            Self::name(),
            Self::category(),
            format!("operator `{parent_name}` has lower precedence than `{child_name}`"),
            Label::new(parent_op.span).message("this operator has lower precedence"),
        ) {
            let diag = diag
                .label(Label::new(child_span).message("than this expression"))
                .help("add parentheses to clarify the intended order of operations");

            self.ctx.report(Self::name(), Self::category(), diag);
        }
    }

    fn format_expr_hint(expr: &Expr) -> &'static str {
        match expr {
            Expr::Literal(_) | Expr::Path(_) | Expr::Unary(_) => "...",
            Expr::Binary(_) | Expr::Group(_) => "(...)",
            Expr::InitList(_) => "{...}",
        }
    }
}

impl<'a> Visitor<'a> for AmbiguousPrecedence<'a> {
    fn visit_expr_binary(&mut self, binary: &'a Binary) {
        self.check_binary_expr(binary);
        // Traversal is handled in check_binary_expr
    }

    fn visit_const(&mut self, def: &'a ic_syntax::ConstDef) {
        // Visit the constant's value expression
        self.visit_expr(&def.value);
    }
}

impl<'a> Lint<'a> for AmbiguousPrecedence<'a> {
    lint_impl! {
        name: "ambiguous_precedence",
        category: Category::Pedantic,
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[ic_syntax::Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_ordering() {
        assert!(
            AmbiguousPrecedence::precedence(OpKind::Multiply)
                > AmbiguousPrecedence::precedence(OpKind::Add)
        );
        assert!(
            AmbiguousPrecedence::precedence(OpKind::Add)
                > AmbiguousPrecedence::precedence(OpKind::Lshift)
        );
        assert!(
            AmbiguousPrecedence::precedence(OpKind::Lshift)
                > AmbiguousPrecedence::precedence(OpKind::And)
        );
        assert!(
            AmbiguousPrecedence::precedence(OpKind::And)
                > AmbiguousPrecedence::precedence(OpKind::Xor)
        );
        assert!(
            AmbiguousPrecedence::precedence(OpKind::Xor)
                > AmbiguousPrecedence::precedence(OpKind::Or)
        );
    }

    #[test]
    fn test_should_warn_precedence() {
        // Should warn: shift with arithmetic on right side only
        assert!(!AmbiguousPrecedence::should_warn_precedence(
            OpKind::Lshift,
            OpKind::Add,
            true
        ));
        assert!(AmbiguousPrecedence::should_warn_precedence(
            OpKind::Lshift,
            OpKind::Add,
            false
        ));

        // Should warn: bitwise with arithmetic
        assert!(AmbiguousPrecedence::should_warn_precedence(
            OpKind::And,
            OpKind::Add,
            true
        ));
        assert!(AmbiguousPrecedence::should_warn_precedence(
            OpKind::Or,
            OpKind::Multiply,
            false
        ));

        // Should NOT warn: arithmetic precedence is well-known
        assert!(!AmbiguousPrecedence::should_warn_precedence(
            OpKind::Add,
            OpKind::Multiply,
            true
        ));
        assert!(!AmbiguousPrecedence::should_warn_precedence(
            OpKind::Sub,
            OpKind::Divide,
            false
        ));
    }
}
