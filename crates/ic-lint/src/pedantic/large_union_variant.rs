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
use ic_hir::type_size::type_size;
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

// Threshold ratio: warn if one variant is more than 3x larger than the average of others
const SIZE_RATIO_THRESHOLD: usize = 3;
// Minimum size difference in bytes to trigger the warning
const MIN_SIZE_DIFFERENCE: usize = 256;

/// Lint that checks for union variants that are significantly larger than others.
/// This can lead to inefficient memory usage since unions allocate space for the largest variant.
pub struct LargeUnionVariant<'a> {
    ctx: &'a LintCtx<'a>,
    hir_ctx: &'a ic_hir::Context,
}

impl<'a> Lint<'a> for LargeUnionVariant<'a> {
    fn name() -> &'static str {
        "large_union_variant"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Union variants varying significantly in size"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = LargeUnionVariant {
            ctx,
            hir_ctx: &hir.context,
        };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> Visitor<'a> for LargeUnionVariant<'a> {
    fn visit_union(&mut self, _def: &'a Def, union_ty: &'a UnionTy) {
        // Calculate sizes for all variants
        let mut variant_sizes = Vec::new();
        let mut largest_variant = None;
        let mut largest_size = 0;

        for variant in &union_ty.variants {
            if let Some(size) = type_size(&variant.ty, self.hir_ctx) {
                variant_sizes.push((variant, size));
                if size > largest_size {
                    largest_size = size;
                    largest_variant = Some(variant);
                }
            }
        }

        // Skip if we couldn't calculate sizes for all variants
        if variant_sizes.len() != union_ty.variants.len() {
            return;
        }

        // Skip if there are fewer than 2 variants
        if variant_sizes.len() < 2 {
            return;
        }

        // Calculate average size of all variants except the largest
        let sum_except_largest: usize = variant_sizes
            .iter()
            .filter(|(_, size)| *size != largest_size)
            .map(|(_, size)| *size)
            .sum();
        let avg_except_largest = sum_except_largest / (variant_sizes.len() - 1);

        // Check if the largest variant is significantly larger
        if avg_except_largest > 0
            && largest_size > avg_except_largest * SIZE_RATIO_THRESHOLD
            && largest_size - avg_except_largest >= MIN_SIZE_DIFFERENCE
        {
            if let Some(largest) = largest_variant {
                let diag = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!(
                        "union variant `{}` is {} bytes, which is significantly larger than the \
                         average of other variants ({} bytes)",
                        largest.ident.name, largest_size, avg_except_largest
                    ),
                    Label::new(largest.ident.span)
                        .message(format!("large variant ({largest_size} bytes)")),
                );

                if let Some(mut diag) = diag {
                    // Add notes about other variants for context
                    for (variant, size) in &variant_sizes {
                        if variant.ident.name != largest.ident.name {
                            diag = diag.label(
                                Label::new(variant.ident.span).message(format!("{size} bytes")),
                            );
                        }
                    }

                    diag = diag.note(
                        "consider annotating large variants with `@shared` to heap allocate them",
                    );

                    Self::report(self.ctx, diag);
                }
            }
        }

        // Continue visiting
        ic_hir::visit::walk_union(self, union_ty);
    }
}
