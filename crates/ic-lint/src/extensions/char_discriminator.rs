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
use ic_hir::hir::{Def, PrimitiveTy, TyKind, UnionTy};
use ic_syntax::{Item, UnionDef};

use crate::{Category, Lint, LintCtx};

/// Lint that warns about using char types as union discriminators or in case labels.
pub struct CharDiscriminator<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for CharDiscriminator<'a> {
    fn name() -> &'static str {
        "char-discriminator"
    }

    fn description() -> &'static str {
        "Char types as union discriminators or in case labels"
    }

    fn category() -> Category {
        Category::Extensions
    }

    fn check(ctx: &'a LintCtx<'a>, ast: &[Item]) {
        let mut syntax_visitor = CharCaseLabelChecker { ctx };
        ic_syntax::visit::walk_tree(&mut syntax_visitor, ast);
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ic_hir::ResolvedGraph) {
        let mut hir_visitor = Self { ctx, hir };
        ic_hir::visit::walk_tree(&mut hir_visitor, hir);
    }
}

/// Checks for char literals in union case labels (AST).
struct CharCaseLabelChecker<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> ic_syntax::visit::Visitor<'a> for CharCaseLabelChecker<'a> {
    fn visit_union(&mut self, union_def: &'a UnionDef) {
        for element in &union_def.fields {
            for label in &element.labels {
                if let ic_syntax::Label::Case(ic_syntax::Expr::Literal(lit)) = &label
                    && let ic_syntax::LiteralValue::Char(_) = lit.value
                    && let Some(diag) = self.ctx.diag_span(
                        CharDiscriminator::name(),
                        CharDiscriminator::category(),
                        "char literals should not be used in union case labels",
                        Label::new(lit.span).message("char literal"),
                    )
                {
                    CharDiscriminator::report(
                        self.ctx,
                        diag.help("consider using an integer or enum instead"),
                    );
                }
            }
        }
        ic_syntax::visit::walk_union(self, union_def);
    }
}

impl<'a> ic_hir::visit::Visitor<'a> for CharDiscriminator<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, _def: &'a Def, union_ty: &'a UnionTy) {
        if let TyKind::Primitive(PrimitiveTy::Char) = &union_ty.disc.ty.kind
            && let Some(diag) = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                "char types should not be used as union discriminators",
                Label::new(union_ty.disc.ty.span).message("char type"),
            )
        {
            Self::report(
                self.ctx,
                diag.help("consider using an integer or enum value instead"),
            );
        }
    }
}
