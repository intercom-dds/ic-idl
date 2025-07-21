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
use ic_hir::annotation::{AnnCtsExt, Max, Min, Range};
use ic_hir::hir::{Ann, Def};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Checks that `@min`, `@max`, and `@range` annotations are valid
pub struct RangeBound<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for RangeBound<'a> {
    fn name() -> &'static str {
        "range_bound"
    }

    fn category() -> Category {
        Category::Annotation
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = RangeBound { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl RangeBound<'_> {
    fn check_range_annotation(&mut self, ann: &Ann) {
        if ann.ident.name != "range" {
            return;
        }

        // Use the CTS annotation system to deserialize the range
        match ann.unmarshal::<Range>("range") {
            Ok(range) => {
                // Validate the range values
                if let (Some(min), Some(max)) = (range.min, range.max) {
                    if min > max {
                        if let Some(diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!("@range min value ({min}) is greater than max value ({max})"),
                            Label::new(ann.ident.span).message("invalid range"),
                        ) {
                            Self::report(self.ctx, diag.help("swap min and max values"));
                        }
                    }
                } else if range.min.is_none() && range.max.is_none() {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        "@range annotation requires at least one of min or max",
                        Label::new(ann.ident.span).message("empty range"),
                    ) {
                        Self::report(
                            self.ctx,
                            diag.help("specify either min=value, max=value, or both"),
                        );
                    }
                }
            }
            Err(err) => {
                // Report deserialization errors
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!("invalid @range annotation: {err}"),
                    Label::new(ann.ident.span).message("malformed annotation"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
        }
    }

    fn check_min_max_annotations(&mut self, annotations: &[Ann]) {
        let mut min_value: Option<i64> = None;
        let mut max_value: Option<i64> = None;
        let mut min_span = None;
        let mut max_span = None;

        for ann in annotations {
            match ann.ident.name.as_str() {
                "min" => match ann.unmarshal::<Min>("min") {
                    Ok(min) => {
                        min_value = Some(min.value);
                        min_span = Some(ann.ident.span);
                    }
                    Err(err) => {
                        if let Some(diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!("invalid @min annotation: {err}"),
                            Label::new(ann.ident.span).message("malformed annotation"),
                        ) {
                            Self::report(self.ctx, diag);
                        }
                    }
                },
                "max" => match ann.unmarshal::<Max>("max") {
                    Ok(max) => {
                        max_value = Some(max.value);
                        max_span = Some(ann.ident.span);
                    }
                    Err(err) => {
                        if let Some(diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!("invalid @max annotation: {err}"),
                            Label::new(ann.ident.span).message("malformed annotation"),
                        ) {
                            Self::report(self.ctx, diag);
                        }
                    }
                },
                _ => {}
            }
        }

        // Check if min > max when both are present
        if let (Some(min), Some(max), Some(min_sp), Some(max_sp)) =
            (min_value, max_value, min_span, max_span)
        {
            if min > max {
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!("@min value ({min}) is greater than @max value ({max})"),
                    Label::new(min_sp).message("min value here"),
                ) {
                    Self::report(
                        self.ctx,
                        diag.label(Label::new(max_sp).message("max value here"))
                            .help("ensure @min is less than or equal to @max"),
                    );
                }
            }
        }
    }

    fn check_annotations(&mut self, annotations: &[Ann]) {
        // Check individual @range annotations
        for ann in annotations {
            self.check_range_annotation(ann);
        }

        // Check @min/@max combinations
        self.check_min_max_annotations(annotations);
    }
}

impl<'a> Visitor<'a> for RangeBound<'a> {
    fn visit_def(&mut self, def: &'a Def) {
        self.check_annotations(&def.annotations);
        ic_hir::visit::walk_def(self, def);
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a ic_hir::hir::StructTy) {
        for member in &data.members {
            self.check_annotations(&member.annotations);
        }
        ic_hir::visit::walk_struct(self, data);
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a ic_hir::hir::UnionTy) {
        for variant in &data.variants {
            self.check_annotations(&variant.annotations);
        }
        ic_hir::visit::walk_union(self, data);
    }
}
