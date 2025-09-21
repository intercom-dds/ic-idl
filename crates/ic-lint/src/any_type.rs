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

//! Lint that warns when the 'any' type is used

use ic_diagnostic::{Label, warn_span};
use ic_hir::hir::{Ty, TyKind};
use ic_hir::visit::{self, Visitor, walk_ty};

use crate::{Category, Lint, LintCtx};

/// Lint that warns when the 'any' type is used
pub struct AnyType<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for AnyType<'a> {
    fn name() -> &'static str {
        "any-type"
    }

    fn description() -> &'static str {
        "Checks for uses of the 'any' type"
    }

    fn category() -> Category {
        Category::Unsupported
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ic_hir::ResolvedGraph) {
        let mut visitor = Self { ctx, hir };
        visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for AnyType<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        if matches!(ty.kind, TyKind::Any) {
            let diag = warn_span(
                "the 'any' type is not fully supported",
                Label::new(ty.span).message("'any' type used here"),
            )
            .help("consider using a concrete type");

            Self::report(self.ctx, diag);
        }

        // Continue traversing nested types
        walk_ty(self, ty);
    }
}
