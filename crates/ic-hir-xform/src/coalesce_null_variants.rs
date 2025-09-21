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

//! Coalesce multiple null variants in unions into a single variant
//!
//! This transformation merges multiple union variants that have `null` type
//! into a single variant, combining their case labels.

use ic_hir::ResolvedGraph;
use ic_hir::fold::Fold;
use ic_hir::hir::{DefKind, Numeric, TyKind, UnionTy, Variant};

struct CoalesceNullVariants;

impl Fold for CoalesceNullVariants {
    fn fold_union_ty(&mut self, mut u: UnionTy) -> UnionTy {
        // First, fold the inner types
        u = ic_hir::fold::fold_union_ty(self, u);

        // Group variants by whether they're null and whether they're default
        let mut null_variant: Option<Variant> = None;
        let mut default_variant: Option<Variant> = None;
        let mut other_variants = Vec::new();

        for variant in u.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                // This is a null variant
                match &mut null_variant {
                    Some(existing) => {
                        // Merge labels from this variant into the existing null variant
                        existing.labels.extend(variant.labels);
                        // Keep annotations from the first null variant
                    }
                    None => {
                        // This is the first null variant we've seen
                        null_variant = Some(variant);
                    }
                }
            } else if variant.is_default {
                // This is the default variant (non-null)
                default_variant = Some(variant);
            } else {
                // Regular non-null, non-default variant
                other_variants.push(variant);
            }
        }

        // Rebuild the variants list with proper ordering:
        // 1. Regular variants
        // 2. Null variant (if any)
        // 3. Default variant last (if any)
        let mut new_variants = other_variants;

        if let Some(null_var) = null_variant {
            new_variants.push(null_var);
        }

        if let Some(default_var) = default_variant {
            new_variants.push(default_var);
        }

        u.variants = new_variants;
        u
    }
}

/// Transform HIR to coalesce multiple null variants in unions
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    let mut folder = CoalesceNullVariants;

    // Collect all definition IDs first to avoid borrowing issues
    let def_ids: Vec<_> = hir.context.definitions.iter().map(|(id, _)| id).collect();

    // Transform all definitions
    for id in def_ids {
        hir.context.definitions.fold(id, |def| folder.fold_def(def));
    }

    hir
}
