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

use std::collections::HashSet;

use ic_diagnostic::Label;
use ic_hir::hir::{Def, UnionTy};
use ic_hir::visit::Visitor;
use ic_hir::ResolvedGraph;

use crate::{Category, Lint, LintCtx};

/// Lint that checks for duplicate variant names in union definitions.
/// This is an error because duplicate variant names are not allowed.
pub struct DuplicateUnionVariants<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for DuplicateUnionVariants<'a> {
    fn name() -> &'static str {
        "duplicate_union_variants"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateUnionVariants { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> Visitor<'a> for DuplicateUnionVariants<'a> {
    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        let mut variant_names = HashSet::new();

        for variant in &union_ty.variants {
            if !variant_names.insert(variant.ident.name.as_str()) {
                Self::report(
                    self.ctx,
                    ic_diagnostic::error_span(
                        format!(
                            "duplicate variant `{}` in union `{}`",
                            variant.ident.name, def.ident.name
                        ),
                        Label::new(variant.ident.span).message("duplicate variant"),
                    ),
                );
            }
        }

        // Continue visiting
        ic_hir::visit::walk_union(self, union_ty);
    }
}