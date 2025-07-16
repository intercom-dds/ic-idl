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
use ic_hir::hir::{Def, UnionTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that checks for multiple default cases in union types.
/// Having multiple default cases is non-deterministic and should be an error.
pub struct MultipleDefaultCases<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for MultipleDefaultCases<'a> {
    fn name() -> &'static str {
        "multiple_default_cases"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = MultipleDefaultCases { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> Visitor<'a> for MultipleDefaultCases<'a> {
    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        let mut default_count = 0;
        let mut first_default_span = None;
        let mut second_default_span = None;

        // Count default cases
        for variant in &union_ty.variants {
            if variant.is_default {
                default_count += 1;
                if default_count == 1 {
                    first_default_span = Some(variant.ident.span);
                } else if default_count == 2 {
                    second_default_span = Some(variant.ident.span);
                }
            }
        }

        // Report error if multiple defaults found
        if default_count > 1 {
            let diag = ic_diagnostic::error_span(
                format!(
                    "union `{}` has {} default cases, but only one is allowed",
                    def.ident.name, default_count
                ),
                Label::new(second_default_span.unwrap()).message("additional default case here"),
            )
            .label(Label::new(first_default_span.unwrap()).message("first default case here"));

            self.ctx.report_error(diag);
        }

        // Continue visiting
        ic_hir::visit::walk_union(self, union_ty);
    }
}
