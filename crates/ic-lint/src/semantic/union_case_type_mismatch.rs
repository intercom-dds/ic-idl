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
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, Numeric, TyKind, UnionTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct UnionCaseTypeMismatch<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for UnionCaseTypeMismatch<'a> {
    fn name() -> &'static str {
        "disc_mismatch"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Detects when union case labels are from the wrong enum type"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = UnionCaseTypeMismatch { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl UnionCaseTypeMismatch<'_> {
    fn check_union(&mut self, union_ty: &UnionTy) {
        // Get the discriminator type
        let disc_enum_id = match &union_ty.disc.kind {
            TyKind::Adt(def_id) => {
                let def = self.context().definitions.get(*def_id);
                match &def.kind {
                    DefKind::Enum(_) => Some(*def_id),
                    _ => None,
                }
            }
            _ => None,
        };

        // If discriminator is not an enum, nothing to check
        let Some(expected_enum_id) = disc_enum_id else {
            return;
        };

        let expected_enum = self.context().definitions.get(expected_enum_id);

        // Check each variant's labels
        for variant in &union_ty.variants {
            for label in &variant.labels {
                if let Numeric::Const(const_id) = label {
                    let const_def = self.context().definitions.get(*const_id);

                    // Find which enum this constant belongs to
                    if let Some(parent_id) = const_def.parent {
                        let parent_def = self.context().definitions.get(parent_id);

                        // Check if the parent is an enum and if it matches the discriminator
                        if matches!(parent_def.kind, DefKind::Enum(_))
                            && parent_id != expected_enum_id
                        {
                            if let Some(diag) = self.ctx.diag_span(
                                Self::name(),
                                Self::category(),
                                format!(
                                    "case label `{}` is from enum `{}`, but discriminator is of \
                                     type `{}`",
                                    const_def.ident.name,
                                    parent_def.ident.name,
                                    expected_enum.ident.name,
                                ),
                                Label::new(variant.ident.span).message("mismatched case label"),
                            ) {
                                Self::report(self.ctx, diag);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for UnionCaseTypeMismatch<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a UnionTy) {
        self.check_union(data);
        ic_hir::visit::walk_union(self, data);
    }
}
