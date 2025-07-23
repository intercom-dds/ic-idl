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
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, Def};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// HIR-based duplicate annotations lint that properly handles annotation resolution
pub struct DuplicateAnnotationsHir<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for DuplicateAnnotationsHir<'a> {
    fn name() -> &'static str {
        "duplicate_annotations"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when annotations are duplicated on the same item"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateAnnotationsHir { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl DuplicateAnnotationsHir<'_> {
    fn check_annotation_list(&mut self, annotations: &[Ann]) {
        let mut seen = HashSet::new();

        for ann in annotations {
            // Use the resolved DefId for proper comparison
            let ann_id = ann.def_id;

            if !seen.insert(ann_id) {
                // Found duplicate - emit diagnostic
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!("duplicate annotation '@{}'", ann.ident.name),
                    Label::new(ann.ident.span).message("duplicate annotation"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
        }

        // Also check for semantically incompatible annotations
        self.check_incompatible_annotations(annotations);
    }

    fn check_incompatible_annotations(&mut self, annotations: &[Ann]) {
        // Check for mutually exclusive annotations
        let has_optional = annotations.iter().any(|a| a.ident.name == "optional");
        let has_key = annotations.iter().any(|a| a.ident.name == "key");

        if has_optional && has_key {
            // Find the optional annotation for the span
            if let Some(optional_ann) = annotations.iter().find(|a| a.ident.name == "optional") {
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    "@optional and @key are mutually exclusive",
                    Label::new(optional_ann.ident.span)
                        .message("@optional cannot be used with @key"),
                ) {
                    Self::report(self.ctx, diag.help("remove either @optional or @key"));
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for DuplicateAnnotationsHir<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        self.check_annotation_list(&def.annotations);
        ic_hir::visit::walk_def(self, def);
    }
}
