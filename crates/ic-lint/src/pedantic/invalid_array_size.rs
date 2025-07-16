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
use ic_hir::type_size::type_size;
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

// Maximum reasonable size in bytes - 16KB
const MAX_REASONABLE_SIZE_BYTES: usize = 16384;

/// Lint that checks for unreasonably large array sizes based on total byte size.
pub struct InvalidArraySize<'a> {
    ctx: &'a LintCtx<'a>,
    hir_ctx: &'a ic_hir::Context,
}

impl<'a> Lint<'a> for InvalidArraySize<'a> {
    fn name() -> &'static str {
        "invalid_array_size"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = InvalidArraySize {
            ctx,
            hir_ctx: &hir.context,
        };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> Visitor<'a> for InvalidArraySize<'a> {
    fn visit_ty(&mut self, ty: &'a Ty) {
        match &ty.kind {
            TyKind::Array {
                ty: elem_ty,
                len,
                len_span,
            } => {
                // Calculate the total size of the array
                if let Some(elem_size) = type_size(elem_ty, self.hir_ctx) {
                    let total_size = elem_size * len;
                    if total_size > MAX_REASONABLE_SIZE_BYTES {
                        if let Some(diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!(
                                "array size {total_size} bytes exceeds reasonable limit of {MAX_REASONABLE_SIZE_BYTES} bytes ({len} elements × {elem_size} bytes each)"
                            ),
                            Label::new(*len_span).message("very large array"),
                        ) {
                            Self::report(self.ctx, diag);
                        }
                    }
                }
            }
            TyKind::Sequence {
                ty: elem_ty,
                bound: Some(b),
                bound_span,
            } => {
                // For bounded sequences, check the maximum possible size
                if let Some(elem_size) = type_size(elem_ty, self.hir_ctx) {
                    let max_size = elem_size * b;
                    if max_size > MAX_REASONABLE_SIZE_BYTES {
                        if let Some(diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!(
                                "sequence maximum size {max_size} bytes exceeds reasonable limit of {MAX_REASONABLE_SIZE_BYTES} bytes ({b} elements × {elem_size} bytes each)"
                            ),
                            Label::new(bound_span.unwrap_or(ty.span)).message("very large sequence bound"),
                        ) {
                            Self::report(self.ctx, diag);
                        }
                    }
                }
            }
            TyKind::String {
                bound: Some(b),
                wide,
                bound_span,
            } => {
                // For bounded strings, check based on character size
                let char_size = if *wide { 4 } else { 1 }; // wchar is 4 bytes, char is 1 byte
                let max_size = char_size * b;
                if max_size > MAX_REASONABLE_SIZE_BYTES {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        format!(
                            "string maximum size {max_size} bytes exceeds reasonable limit of {MAX_REASONABLE_SIZE_BYTES} bytes ({b} characters × {char_size} bytes each)"
                        ),
                        Label::new(bound_span.unwrap_or(ty.span)).message("very large string bound"),
                    ) {
                        Self::report(self.ctx, diag);
                    }
                }
            }
            _ => {}
        }

        // Continue visiting nested types
        ic_hir::visit::walk_ty(self, ty);
    }
}
