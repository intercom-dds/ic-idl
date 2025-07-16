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
use ic_hir::hir::{Ann, Def, DefKind, Member, Numeric, PrimitiveTy, Ty, TyKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct RangeBound<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for RangeBound<'a> {
    fn name() -> &'static str {
        "RangeBound"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = RangeBound { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl RangeBound<'_> {
    fn check_annotation(&mut self, ann: &Ann, ty: &Ty) {
        let name = ann.path.segments.last().map_or("", |s| s.name.as_str());
        match name {
            "min" => {
                if let Some(value) = Self::get_numeric_arg(ann, 0) {
                    self.check_min_bound(value, ty, ann);
                }
            }
            "max" => {
                if let Some(value) = Self::get_numeric_arg(ann, 0) {
                    self.check_max_bound(value, ty, ann);
                }
            }
            "range" => {
                if let (Some(min), Some(max)) =
                    (Self::get_numeric_arg(ann, 0), Self::get_numeric_arg(ann, 1))
                {
                    self.check_min_bound(min, ty, ann);
                    self.check_max_bound(max, ty, ann);
                    self.check_range_order(min, max, ann);
                }
            }
            _ => {}
        }
    }

    fn get_numeric_arg(ann: &Ann, index: usize) -> Option<&Numeric> {
        ann.args.get(index).map(|arg| &arg.value)
    }

    fn check_min_bound(&mut self, value: &Numeric, ty: &Ty, ann: &Ann) {
        if let Some((min, _)) = Self::get_type_bounds(ty) {
            if let Some(val) = Self::numeric_to_i64(value) {
                if val < min {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        format!("@min value {val} is less than type minimum {min}"),
                        Label::new(ic_syntax::util::path_span(&ann.path))
                            .message("invalid minimum bound"),
                    ) {
                        Self::report(self.ctx, diag);
                    }
                }
            }
        }
    }

    fn check_max_bound(&mut self, value: &Numeric, ty: &Ty, ann: &Ann) {
        if let Some((_, max)) = Self::get_type_bounds(ty) {
            if let Some(val) = Self::numeric_to_i64(value) {
                if val > max {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        format!("@max value {val} is greater than type maximum {max}"),
                        Label::new(ic_syntax::util::path_span(&ann.path))
                            .message("invalid maximum bound"),
                    ) {
                        Self::report(self.ctx, diag);
                    }
                }
            }
        }
    }

    fn check_range_order(&mut self, min: &Numeric, max: &Numeric, ann: &Ann) {
        if let (Some(min_val), Some(max_val)) =
            (Self::numeric_to_i64(min), Self::numeric_to_i64(max))
        {
            if min_val > max_val {
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!("@range minimum {min_val} is greater than maximum {max_val}"),
                    Label::new(ic_syntax::util::path_span(&ann.path)).message("invalid range"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
        }
    }

    fn numeric_to_i64(num: &Numeric) -> Option<i64> {
        match num {
            Numeric::Int8(v) => Some(i64::from(*v)),
            Numeric::Int16(v) => Some(i64::from(*v)),
            Numeric::Int32(v) => Some(i64::from(*v)),
            Numeric::Int64(v) => Some(*v),
            Numeric::Octet(v) => Some(i64::from(*v)),
            Numeric::UInt16(v) => Some(i64::from(*v)),
            Numeric::UInt32(v) => Some(i64::from(*v)),
            Numeric::UInt64(v) => {
                // Be careful with large unsigned values
                i64::try_from(*v).ok()
            }
            _ => None,
        }
    }

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
                PrimitiveTy::UInt64 => Some((0, i64::MAX)), // Limited by i64 representation
                _ => None,
            },
            TyKind::Adt(_) => {
                // TODO: Follow typedef/alias to get underlying type
                None
            }
            _ => None,
        }
    }
}

impl<'a> Visitor<'a> for RangeBound<'a> {
    fn visit_def(&mut self, def: &'a Def) {
        // Check annotations on the definition itself
        match &def.kind {
            DefKind::Const(const_ty) => {
                for ann in &def.annotations {
                    self.check_annotation(ann, &const_ty.ty);
                }
            }
            DefKind::Alias(alias_ty) => {
                for ann in &def.annotations {
                    self.check_annotation(ann, &alias_ty.ty);
                }
            }
            _ => {}
        }

        // Continue visiting
        ic_hir::visit::walk_def(self, def);
    }

    fn visit_member(&mut self, member: &'a Member) {
        // Check annotations on struct/exception members
        for ann in &member.annotations {
            self.check_annotation(ann, &member.ty);
        }

        // Continue visiting
        ic_hir::visit::walk_member(self, member);
    }
}
