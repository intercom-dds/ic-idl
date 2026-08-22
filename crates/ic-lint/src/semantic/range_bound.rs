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

use std::fmt;

use ic_diagnostic::Label;
use ic_hir::hir::{Ann, AnnArg, Def, DefFlags, DefKind, Numeric, PrimitiveTy, Ty, TyKind};
use ic_hir::visit::Visitor;
use ic_hir::{ResolvedGraph, visit};

use crate::{Category, Lint, LintCtx};

/// Checks that `@min`, `@max`, and `@range` annotations are valid
pub struct RangeBound<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

#[derive(Clone, Copy)]
enum BoundValue {
    Integer(i64),
    Float(f64),
}

impl BoundValue {
    fn is_greater_than(self, other: Self) -> bool {
        match (self, other) {
            (Self::Integer(lhs), Self::Integer(rhs)) => lhs > rhs,
            (Self::Float(lhs), Self::Float(rhs)) => lhs > rhs,
            _ => false,
        }
    }
}

impl fmt::Display for BoundValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => value.fmt(formatter),
            Self::Float(value) => value.fmt(formatter),
        }
    }
}

impl<'a> Lint<'a> for RangeBound<'a> {
    fn name() -> &'static str {
        "range-bound"
    }

    fn category() -> Category {
        Category::Semantic
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
    fn annotation_name<'a>(&'a self, annotation: &Ann) -> Option<&'a str> {
        let def = self.hir.context.base_def_of(annotation.def_id?);
        def.flags
            .contains(DefFlags::IS_BUILTIN)
            .then_some(def.ident.name.as_str())
    }

    fn annotation_argument<'a>(
        annotation: &'a Ann,
        name: &str,
        position: usize,
    ) -> Option<&'a AnnArg> {
        annotation
            .args
            .iter()
            .find(|argument| argument.ident.name == name)
            .or_else(|| {
                annotation
                    .args
                    .iter()
                    .filter(|argument| argument.ident.name.is_empty())
                    .nth(position)
            })
    }

    fn annotation_value(
        &mut self,
        annotation: &Ann,
        argument: &str,
        position: usize,
        annotation_type: &str,
        floating: bool,
    ) -> Result<Option<BoundValue>, ()> {
        let Some(argument_value) = Self::annotation_argument(annotation, argument, position) else {
            return Ok(None);
        };
        let value = if floating {
            self.numeric_float(&argument_value.value)
                .map(BoundValue::Float)
        } else {
            self.numeric_integer(&argument_value.value)
                .map(BoundValue::Integer)
        };
        let Some(value) = value else {
            let expected = if floating { "a number" } else { "an integer" };
            let message = if argument == "value" {
                format!("@{annotation_type} value must be {expected}")
            } else {
                format!("@{annotation_type} argument '{argument}' must be {expected}")
            };
            let diag = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                message,
                Label::new(argument_value.ident.span).message(format!("must be {expected}")),
            );
            Self::report(self.ctx, diag);

            return Err(());
        };

        Ok(Some(value))
    }

    fn numeric_integer(&self, value: &Numeric) -> Option<i64> {
        match value {
            Numeric::Char(value) | Numeric::WChar(value) => Some(i64::from(u32::from(*value))),
            Numeric::Int8(value) => Some(i64::from(*value)),
            Numeric::UInt8(value) => Some(i64::from(*value)),
            Numeric::Int16(value) => Some(i64::from(*value)),
            Numeric::UInt16(value) => Some(i64::from(*value)),
            Numeric::Int32(value) => Some(i64::from(*value)),
            Numeric::UInt32(value) => Some(i64::from(*value)),
            Numeric::Int64(value) => Some(*value),
            Numeric::UInt64(value) => i64::try_from(*value).ok(),
            Numeric::Const(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                let DefKind::Const(constant) = &def.kind else {
                    return None;
                };

                self.numeric_integer(&constant.value)
            }
            _ => None,
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "floating range comparison requires f64 values"
    )]
    fn numeric_float(&self, value: &Numeric) -> Option<f64> {
        match value {
            Numeric::Char(value) | Numeric::WChar(value) => Some(f64::from(u32::from(*value))),
            Numeric::Int8(value) => Some(f64::from(*value)),
            Numeric::UInt8(value) => Some(f64::from(*value)),
            Numeric::Int16(value) => Some(f64::from(*value)),
            Numeric::UInt16(value) => Some(f64::from(*value)),
            Numeric::Int32(value) => Some(f64::from(*value)),
            Numeric::UInt32(value) => Some(f64::from(*value)),
            Numeric::Int64(value) => Some(*value as f64),
            Numeric::UInt64(value) => Some(*value as f64),
            Numeric::Float(value) => Some(f64::from(*value)),
            Numeric::Double(value) => Some(*value),
            Numeric::Const(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                let DefKind::Const(constant) = &def.kind else {
                    return None;
                };

                self.numeric_float(&constant.value)
            }
            _ => None,
        }
    }

    /// Get the valid range for a type, resolving through aliases
    fn get_type_bounds(&self, ty: &Ty) -> Option<(i64, i64)> {
        match &self.hir.context.resolve_ty(ty).kind {
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
            _ => None,
        }
    }

    fn check_value_in_bounds(&mut self, value: i64, ty: &Ty, ann: &Ann, annotation_type: &str) {
        if let Some((min_bound, max_bound)) = self.get_type_bounds(ty) {
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

    fn check_range_annotation(&mut self, annotation: &Ann, ty: &Ty, floating: bool) {
        if self.annotation_name(annotation) != Some("range") {
            return;
        }

        let Ok(min) = self.annotation_value(annotation, "min", 0, "range", floating) else {
            return;
        };
        let Ok(max) = self.annotation_value(annotation, "max", 1, "range", floating) else {
            return;
        };

        if let Some(BoundValue::Integer(min)) = min {
            self.check_value_in_bounds(min, ty, annotation, "range min");
        }
        if let Some(BoundValue::Integer(max)) = max {
            self.check_value_in_bounds(max, ty, annotation, "range max");
        }

        if let (Some(min), Some(max)) = (min, max)
            && min.is_greater_than(max)
        {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    format!("@range min value ({min}) is greater than max value ({max})"),
                    Label::new(annotation.ident.span).message("invalid range"),
                )
                .help("swap min and max values");
            Self::report(self.ctx, diag);
        } else if min.is_none() && max.is_none() {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    "@range annotation requires at least one of min or max",
                    Label::new(annotation.ident.span).message("empty range"),
                )
                .help("specify either min=value, max=value, or both");
            Self::report(self.ctx, diag);
        }
    }

    fn check_min_max_annotations(&mut self, annotations: &[Ann], ty: &Ty, floating: bool) {
        let mut min_value = None;
        let mut max_value = None;
        let mut min_span = None;
        let mut max_span = None;
        let mut min_annotation = None;
        let mut max_annotation = None;

        for annotation in annotations {
            match self.annotation_name(annotation) {
                Some("min") => {
                    if let Ok(value) =
                        self.annotation_value(annotation, "value", 0, "min", floating)
                    {
                        min_value = value;
                        min_span = Some(annotation.ident.span);
                        min_annotation = Some(annotation);
                    }
                }
                Some("max") => {
                    if let Ok(value) =
                        self.annotation_value(annotation, "value", 0, "max", floating)
                    {
                        max_value = value;
                        max_span = Some(annotation.ident.span);
                        max_annotation = Some(annotation);
                    }
                }
                _ => {}
            }
        }

        if let (Some(BoundValue::Integer(value)), Some(annotation)) = (min_value, min_annotation) {
            self.check_value_in_bounds(value, ty, annotation, "min");
        }
        if let (Some(BoundValue::Integer(value)), Some(annotation)) = (max_value, max_annotation) {
            self.check_value_in_bounds(value, ty, annotation, "max");
        }

        if let (Some(min), Some(max), Some(min_span), Some(max_span)) =
            (min_value, max_value, min_span, max_span)
            && min.is_greater_than(max)
        {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    format!("@min value ({min}) is greater than @max value ({max})"),
                    Label::new(min_span).message("min value here"),
                )
                .label(Label::new(max_span).message("max value here"))
                .help("ensure @min is less than or equal to @max");

            Self::report(self.ctx, diag);
        }
    }

    fn check_annotations(&mut self, annotations: &[Ann], ty: &Ty) {
        let floating = matches!(
            self.hir.context.resolve_ty(ty).kind,
            TyKind::Primitive(PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128)
        );

        for ann in annotations {
            self.check_range_annotation(ann, ty, floating);
        }
        self.check_min_max_annotations(annotations, ty, floating);
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
