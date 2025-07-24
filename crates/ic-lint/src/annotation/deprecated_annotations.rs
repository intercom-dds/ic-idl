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
use ic_hir::hir::{Ann, DefFlags};
use ic_hir::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

pub struct DeprecatedAnnotations<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for DeprecatedAnnotations<'a> {
    fn name() -> &'static str {
        "deprecated_ann"
    }

    fn category() -> Category {
        Category::Annotation
    }

    fn description() -> &'static str {
        "Warns when deprecated annotations are used"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DeprecatedAnnotations { ctx, hir };
        walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for DeprecatedAnnotations<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_annotation(&mut self, ann: &'a Ann) {
        let ann_def = self.hir.context.type_of(ann.def_id);
        if ann_def.flags.contains(DefFlags::IS_BUILTIN)
            && ann_def.ident.name == "shared"
            && let Some(diag) = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                "use of deprecated annotation `@shared`",
                Label::new(ann.ident.span).message("deprecated annotation used here"),
            )
        {
            let diag = diag.help("use `@external` instead");
            Self::report(self.ctx, diag);
        }
    }
}
