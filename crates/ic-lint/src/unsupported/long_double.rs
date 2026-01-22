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
use ic_hir::ResolvedGraph;
use ic_hir::hir::{self, PrimitiveTy, TyKind};
use ic_hir::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Warns when `long double` is used.
pub struct LongDouble<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for LongDouble<'a> {
    fn name() -> &'static str {
        "long-double"
    }

    fn category() -> Category {
        Category::Unsupported
    }

    fn description() -> &'static str {
        "Checks for use of `long double`"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut lint = Self { ctx, hir };
        walk_tree(&mut lint, hir);
    }
}

impl<'a> Visitor<'a> for LongDouble<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_ty(&mut self, ty: &'a hir::Ty) {
        if let TyKind::Primitive(PrimitiveTy::Float128) = ty.kind {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    "long double is not supported",
                    Label::new(ty.span).message("used here"),
                )
                .note("long double will be treated as a normal double during codegen");
            Self::report(self.ctx, diag);
        }

        ic_hir::visit::walk_ty(self, ty);
    }
}
