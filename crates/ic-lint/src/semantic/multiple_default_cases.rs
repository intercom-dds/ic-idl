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
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for MultipleDefaultCases<'a> {
    fn name() -> &'static str {
        "multiple_default_cases"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when unions have multiple default cases"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = MultipleDefaultCases { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for MultipleDefaultCases<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        let defaults: Vec<_> = union_ty
            .variants
            .iter()
            .filter(|v| v.is_default)
            .map(|v| v.ident.span)
            .collect();

        match defaults.as_slice() {
            [] | [_] => {}
            [first, rest @ ..] => {
                let diag = ic_diagnostic::error_span(
                    format!(
                        "union `{}` has {} default cases, but only one is allowed",
                        def.ident.name,
                        defaults.len(),
                    ),
                    Label::new(*first).message("first default case here"),
                )
                .labels(
                    rest.iter()
                        .map(|&case| Label::new(case).message("additional default case here")),
                );
                Self::report(self.ctx, diag);
            }
        }
    }
}
