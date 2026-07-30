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
use ic_hir::annotation::{AnnCtsExt, Max, Min, Range};
use ic_hir::hir::{Ann, Def, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir::visit::Visitor;
use ic_hir::{ResolvedGraph, visit};

use crate::{Category, Lint, LintCtx};

/// Checks that `@min`, `@max`, and `@range` annotations are valid
pub struct RangeBound<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for RangeBound<'a> {
    fn name() -> &'static str {
        "range-bound"
    }

    fn category() -> Category {
        Category::Annotation
    }

    fn description() -> &'static str {
        "Validates @range, @min, and @max annotation values"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = RangeBound { ctx, hir };
        visit::walk_tree(&mut visitor, hir);
    }
}

impl RangeBound<'_> {
    /// Get the valid range for a type
    fn get_type_bounds(ty: &Ty) -> Option<(i64, i64)> {
        match &ty.kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Int8 => Some((i64::from(i8::MIN), i64::from(i8::MAX))),
                PrimitiveTy::Int16 => Some((i64::from(i16::MIN), i64::from(i16::MAX))),
                PrimitiveTy::Int32 => Some((i64::from(i32::MIN), i64::from(i32::MAX))),
                PrimitiveTy::Int64 => Some((i64::MIN, i64::MAX)),
                PrimitiveTy::UInt8 => Some((0, i64::from(u8::MAX))),
                PrimitiveTy::UInt16 => Some((0, i64::from(u16::MAX))),
                PrimitiveTy::UInt32 => Some((0, i64::from(u32::MAX))),
                PrimitiveTy::UInt64 => Some((0, 9_223_372_036_854_775_807)),
                PrimitiveTy::Char => Some((0, 127)),
                PrimitiveTy::WChar => Some((0, 65535)),
                _ => None,
            },
            TyKind::Adt(_) => {
                // TODO: Follow typedef/alias to get underlying type
                None
            }
            _ => None,
        }
    }

    fn check_value_in_bounds(&mut self, value: i64, ty: &Ty, ann: &Ann, annotation_type: &str) {
        if let Some((min_bound, max_bound)) = Self::get_type_bounds(ty) {
            if value < min_bound {
                let diag = self
                    .ctx
                    .diag_span(
                        Self::name(),
                        Self::category(),
                        format!(
                            "@{annotation_type} value {value} is less than type minimum \
                             {min_bound}"
                        ),
                        Label::new(ann.ident.span).message("value out of bounds"),
                    )
                    .help(format!("valid range is {min_bound}..{max_bound}"));
                Self::report(self.ctx, diag);
            } else if value > max_bound {
                let diag = self
                    .ctx
                    .diag_span(
                        Self::name(),
                        Self::category(),
                        format!(
                            "@{annotation_type} value {value} exceeds type maximum {max_bound}"
                        ),
                        Label::new(ann.ident.span).message("value out of bounds"),
                    )
                    .help(format!("valid range is {min_bound}..{max_bound}"));
                Self::report(self.ctx, diag);
            }
        }
    }

    fn check_range_annotation(&mut self, ann: &Ann, ty: &Ty) {
        if ann.ident.name != "range" {
            return;
        }

        // Use the CTS annotation system to deserialize the range
        match ann.unmarshal::<Range>("range") {
            Ok(range) => {
                // Check bounds for min value
                if let Some(min) = range.min {
                    self.check_value_in_bounds(min, ty, ann, "range min");
                }

                // Check bounds for max value
                if let Some(max) = range.max {
                    self.check_value_in_bounds(max, ty, ann, "range max");
                }

                // Validate the range values
                if let (Some(min), Some(max)) = (range.min, range.max) {
                    if min > max {
                        let diag = self
                            .ctx
                            .diag_span(
                                Self::name(),
                                Self::category(),
                                format!(
                                    "@range min value ({min}) is greater than max value ({max})"
                                ),
                                Label::new(ann.ident.span).message("invalid range"),
                            )
                            .help("swap min and max values");
                        Self::report(self.ctx, diag);
                    }
                } else if range.min.is_none() && range.max.is_none() {
                    let diag = self
                        .ctx
                        .diag_span(
                            Self::name(),
                            Self::category(),
                            "@range annotation requires at least one of min or max",
                            Label::new(ann.ident.span).message("empty range"),
                        )
                        .help("specify either min=value, max=value, or both");
                    Self::report(self.ctx, diag);
                }
            }
            Err(err) => {
                // Report deserialization errors
                let diag = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!("invalid @range annotation: {err}"),
                    Label::new(ann.ident.span).message("malformed annotation"),
                );
                Self::report(self.ctx, diag);
            }
        }
    }

    fn check_min_max_annotations(&mut self, annotations: &[Ann], ty: &Ty) {
        let mut min_value: Option<i64> = None;
        let mut max_value: Option<i64> = None;
        let mut min_span = None;
        let mut max_span = None;
        let mut min_ann = None;
        let mut max_ann = None;

        for ann in annotations {
            match ann.ident.name.as_str() {
                "min" => match ann.unmarshal::<Min>("min") {
                    Ok(min) => {
                        min_value = Some(min.value);
                        min_span = Some(ann.ident.span);
                        min_ann = Some(ann);
                    }
                    Err(err) => {
                        let diag = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!("invalid @min annotation: {err}"),
                            Label::new(ann.ident.span).message("malformed annotation"),
                        );
                        Self::report(self.ctx, diag);
                    }
                },
                "max" => match ann.unmarshal::<Max>("max") {
                    Ok(max) => {
                        max_value = Some(max.value);
                        max_span = Some(ann.ident.span);
                        max_ann = Some(ann);
                    }
                    Err(err) => {
                        let diag = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!("invalid @max annotation: {err}"),
                            Label::new(ann.ident.span).message("malformed annotation"),
                        );
                        Self::report(self.ctx, diag);
                    }
                },
                _ => {}
            }
        }

        // Check type bounds for min value
        if let (Some(value), Some(ann)) = (min_value, min_ann) {
            self.check_value_in_bounds(value, ty, ann, "min");
        }

        // Check type bounds for max value
        if let (Some(value), Some(ann)) = (max_value, max_ann) {
            self.check_value_in_bounds(value, ty, ann, "max");
        }

        // Check if min > max when both are present
        if let (Some(min), Some(max), Some(min_sp), Some(max_sp)) =
            (min_value, max_value, min_span, max_span)
            && min > max
        {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    format!("@min value ({min}) is greater than @max value ({max})"),
                    Label::new(min_sp).message("min value here"),
                )
                .label(Label::new(max_sp).message("max value here"))
                .help("ensure @min is less than or equal to @max");

            Self::report(self.ctx, diag);
        }
    }

    fn check_annotations(&mut self, annotations: &[Ann], ty: &Ty) {
        for ann in annotations {
            self.check_range_annotation(ann, ty);
        }
        self.check_min_max_annotations(annotations, ty);
    }
}

impl<'a> Visitor<'a> for RangeBound<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        match &def.kind {
            DefKind::Const(const_ty) => {
                self.check_annotations(&def.annotations, &const_ty.ty);
            }
            DefKind::Alias(alias_ty) => {
                self.check_annotations(&def.annotations, &alias_ty.ty);
            }
            _ => {}
        }
        visit::walk_def(self, def);
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a ic_hir::hir::StructTy) {
        for member in &data.members {
            self.check_annotations(&member.annotations, &member.ty);
        }
        visit::walk_struct(self, data);
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a ic_hir::hir::UnionTy) {
        for variant in &data.variants {
            self.check_annotations(&variant.annotations, &variant.ty);
        }
        visit::walk_union(self, data);
    }

    fn visit_except(&mut self, _def: &'a Def, data: &'a ic_hir::hir::ExceptTy) {
        for member in &data.members {
            self.check_annotations(&member.annotations, &member.ty);
        }
        visit::walk_except(self, data);
    }
}
