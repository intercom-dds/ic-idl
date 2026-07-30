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
use ic_syntax::visit::{self, Visitor};

use crate::{Category, Lint, LintCtx, SyntaxInput};

pub struct AnnPlacement;

struct PlacementVisitor<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for PlacementVisitor<'a> {
    fn visit_prototype_param(&mut self, param: &'a ic_syntax::Param) {
        for annotation in &param.meta.annotations {
            if annotation
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.name == "doc")
            {
                continue;
            }

            let diag = self.ctx.diag_span(
                AnnPlacement::name(),
                AnnPlacement::category(),
                "annotations on prototype parameters are not allowed",
                Label::new(annotation.span).message("invalid annotation placement"),
            );
            AnnPlacement::report(self.ctx, diag);
        }

        visit::walk_param(self, param);
    }
}

impl<'a> Lint<'a> for AnnPlacement {
    fn name() -> &'static str {
        "ann-placement"
    }

    fn category() -> Category {
        Category::Annotation
    }

    fn description() -> &'static str {
        "Annotations not attached to any declaration"
    }

    fn check_syntax(ctx: &'a LintCtx<'_>, input: &SyntaxInput<'_>) {
        for ann in input.orphaned_annotations {
            let diag = ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    "annotation has no effect in this context",
                    Label::new(ann.span).message("misplaced annotation"),
                )
                .note("annotation is not attached to any declaration");
            Self::report(ctx, diag);
        }

        let mut visitor = PlacementVisitor { ctx };
        visit::walk_tree(&mut visitor, input.tree);
    }
}
