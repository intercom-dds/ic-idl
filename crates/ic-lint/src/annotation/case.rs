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

use ic_diagnostic::Label;
use ic_hir::hir::Ann;
use ic_hir::visit::{self, Visitor};

use crate::{Category, Lint, LintCtx};

pub struct AnnotationCase<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for AnnotationCase<'a> {
    fn name() -> &'static str {
        "ann-case"
    }

    fn description() -> &'static str {
        "Annotations with different capitalization than their definition"
    }

    fn category() -> Category {
        Category::Annotation
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ic_hir::ResolvedGraph) {
        let mut visitor = Self { ctx, hir };
        visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for AnnotationCase<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_annotation(&mut self, ann: &'a Ann) {
        let Some(def_id) = ann.def_id else {
            return;
        };

        let used = ann
            .ident
            .name
            .rsplit("::")
            .next()
            .unwrap_or(&ann.ident.name);

        let canonical = &self.hir.context.type_of(def_id).ident.name;
        if used == canonical {
            return;
        }

        let diag = self.ctx.diag_span(
            Self::name(),
            Self::category(),
            format!("inconsistent capitalization: `{used}` should be `{canonical}`"),
            Label::new(ann.ident.span).message("annotation used here"),
        );

        Self::report(
            self.ctx,
            diag.note(format!("the canonical name is `{canonical}`")),
        );
    }
}
