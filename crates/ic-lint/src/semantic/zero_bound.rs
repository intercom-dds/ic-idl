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
use ic_hir::hir::{Ty, TyKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that checks for zero-sized arrays and zero bounds on sequences, strings, and maps.
/// These are treated as errors in standard IDL.
pub struct ZeroBound<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for ZeroBound<'a> {
    fn name() -> &'static str {
        "zero_bound"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when arrays or sequences have zero size/bounds"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = ZeroBound { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> Visitor<'a> for ZeroBound<'a> {
    fn visit_ty(&mut self, ty: &'a Ty) {
        match &ty.kind {
            TyKind::Array { len, len_span, .. } => {
                if *len == 0 {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        "array size must be greater than zero",
                        Label::new(*len_span).message("invalid array size"),
                    ) {
                        Self::report(self.ctx, diag);
                    }
                }
            }
            TyKind::Sequence {
                bound: Some(b),
                bound_span,
                ..
            } if *b == 0 => {
                // Use bound_span if available, otherwise fall back to ty.span
                let span = bound_span.unwrap_or(ty.span);
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    "sequence bound must be greater than zero",
                    Label::new(span).message("invalid sequence bound"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
            TyKind::String {
                bound: Some(b),
                bound_span,
                ..
            } if *b == 0 => {
                // Use bound_span if available, otherwise fall back to ty.span
                let span = bound_span.unwrap_or(ty.span);
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    "string bound must be greater than zero",
                    Label::new(span).message("invalid string bound"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
            TyKind::Map {
                bound: Some(b),
                bound_span,
                ..
            } if *b == 0 => {
                // Use bound_span if available, otherwise fall back to ty.span
                let span = bound_span.unwrap_or(ty.span);
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    "map bound must be greater than zero",
                    Label::new(span).message("invalid map bound"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
            _ => {}
        }

        // Continue visiting nested types
        ic_hir::visit::walk_ty(self, ty);
    }
}
