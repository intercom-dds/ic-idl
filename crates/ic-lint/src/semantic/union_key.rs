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

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, Def, UnionTy, Variant};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Enforces union annotation rules:
/// 1. @key is only allowed on the discriminator, not on union members
/// 2. @optional is not allowed on the discriminator
pub struct UnionKey<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for UnionKey<'a> {
    fn name() -> &'static str {
        "union-key"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "enforces @key and @optional rules for unions"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = UnionKey { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> UnionKey<'a> {
    fn check_discriminator_annotations(&self, union_def: &'a Def, discriminator_anns: &[Ann]) {
        for ann in discriminator_anns {
            if ann.ident.name == "optional" {
                Self::report(
                    self.ctx,
                    error_span(
                        format!(
                            "union discriminator for '{}' cannot be @optional",
                            union_def.ident.name
                        ),
                        Label::new(ann.ident.span)
                            .message("@optional not allowed on discriminator"),
                    )
                    .note("union discriminators must always be present")
                    .help("remove @optional from the discriminator"),
                );
            }
        }
    }
}

impl<'a> Visitor<'a> for UnionKey<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        // Check discriminator annotations
        self.check_discriminator_annotations(def, &union_ty.disc.annotations);

        // Continue visiting to check variants
        ic_hir::visit::walk_union(self, union_ty);
    }

    fn visit_variant(&mut self, variant: &'a Variant) {
        // Check if variant has @key annotation
        for ann in &variant.annotations {
            if ann.ident.name == "key" {
                Self::report(
                    self.ctx,
                    error_span(
                        format!("union variant '{}' cannot use @key", variant.ident.name),
                        Label::new(ann.ident.span).message("@key not allowed on union variant"),
                    )
                    .note("only the union discriminator can be marked as @key")
                    .help("remove @key from the variant or move it to the discriminator"),
                );
            }
        }

        ic_hir::visit::walk_variant(self, variant);
    }
}
