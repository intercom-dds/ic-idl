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
use ic_syntax::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct AnnTemplate<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for AnnTemplate<'a> {
    fn name() -> &'static str {
        "ann-template"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Invalid use of annotations on template types"
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[ic_syntax::Item]) {
        let mut visitor = AnnTemplate { ctx };
        ic_syntax::visit::walk_tree(&mut visitor, ast);
    }
}

impl AnnTemplate<'_> {
    fn report_invalid_annotation(&self, ann: &ic_syntax::AnnotationAppl, kind: &str) {
        let span = ic_syntax::util::path_span(&ann.ident);
        let diag = self
            .ctx
            .diag_span(
                Self::name(),
                Self::category(),
                format!("{kind} types cannot be annotated"),
                Label::new(span).message("invalid use of annotation"),
            )
            .help(format!(
                "create a typedef for the {kind} type and apply the annotation there"
            ));
        Self::report(self.ctx, diag);
    }
}

impl<'a> Visitor<'a> for AnnTemplate<'a> {
    fn visit_type(&mut self, ty: &'a ic_syntax::Type) {
        match ty {
            ic_syntax::Type::Sequence(seq) => {
                for ann in &seq.annotations {
                    self.report_invalid_annotation(ann, "element");
                }
            }
            ic_syntax::Type::Map(map) => {
                for ann in &map.key_annotations {
                    self.report_invalid_annotation(ann, "key");
                }
                for ann in &map.value_annotations {
                    self.report_invalid_annotation(ann, "element");
                }
            }
            _ => (),
        }

        ic_syntax::visit::walk_type(self, ty);
    }
}
