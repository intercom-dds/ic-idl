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
use ic_hir::hir::{Def, Numeric, PrimitiveTy, TyKind, UnionTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct UnreachableUnionCases<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for UnreachableUnionCases<'a> {
    fn name() -> &'static str {
        "UnreachableUnionCases"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = UnreachableUnionCases { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> UnreachableUnionCases<'a> {
    fn check_union(&mut self, union_ty: &UnionTy, _union_name: &str) {
        let mut default_index = None;

        // First pass: find if there's a default case
        for (idx, variant) in union_ty.variants.iter().enumerate() {
            if variant.is_default {
                default_index = Some(idx);
                break;
            }
        }

        // Check for cases after default
        if let Some(default_idx) = default_index {
            for (idx, variant) in union_ty.variants.iter().enumerate() {
                if idx > default_idx && !variant.is_default {
                    if let Some(mut diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        &format!(
                            "case '{}' is unreachable because it appears after default",
                            variant.ident.name
                        ),
                        Label::new(variant.ident.span).message("unreachable case"),
                    ) {
                        diag = diag.help("move this case before the default case");
                        self.ctx.report(Self::name(), Self::category(), diag);
                    }
                }
            }
        }

        // Check for out-of-range case labels
        if let Some((min, max)) = self.get_discriminator_range(&union_ty.disc.kind) {
            for variant in &union_ty.variants {
                for label in &variant.labels {
                    if let Some(value) = self.numeric_to_i64(label) {
                        if value < min || value > max {
                            if let Some(diag) = self.ctx.diag_span(
                                Self::name(),
                                Self::category(),
                                &format!(
                                    "case label {} is outside the range [{}, {}] of the discriminator type",
                                    value, min, max
                                ),
                                Label::new(variant.ident.span)
                                    .message("case label out of range"),
                            ) {
                                self.ctx.report(Self::name(), Self::category(), diag);
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_discriminator_range(&self, ty_kind: &TyKind) -> Option<(i64, i64)> {
        match ty_kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => Some((0, 1)),
                PrimitiveTy::Char => Some((0, 127)), // ASCII range
                PrimitiveTy::WChar => Some((0, 65535)), // Unicode BMP
                PrimitiveTy::Int8 => Some((i8::MIN as i64, i8::MAX as i64)),
                PrimitiveTy::Int16 => Some((i16::MIN as i64, i16::MAX as i64)),
                PrimitiveTy::Int32 => Some((i32::MIN as i64, i32::MAX as i64)),
                PrimitiveTy::Int64 => Some((i64::MIN, i64::MAX)),
                PrimitiveTy::UInt8 => Some((0, u8::MAX as i64)),
                PrimitiveTy::UInt16 => Some((0, u16::MAX as i64)),
                PrimitiveTy::UInt32 => Some((0, u32::MAX as i64)),
                PrimitiveTy::UInt64 => Some((0, i64::MAX)), // Limited by i64
                _ => None,
            },
            TyKind::Adt(_) => {
                // TODO: Handle enum types - need to get enum value range
                None
            }
            _ => None,
        }
    }

    fn numeric_to_i64(&self, num: &Numeric) -> Option<i64> {
        match num {
            Numeric::Bool(v) => Some(if *v { 1 } else { 0 }),
            Numeric::Char(v) => Some(*v as i64),
            Numeric::Int8(v) => Some(*v as i64),
            Numeric::Int16(v) => Some(*v as i64),
            Numeric::Int32(v) => Some(*v as i64),
            Numeric::Int64(v) => Some(*v),
            Numeric::Octet(v) => Some(*v as i64),
            Numeric::UInt16(v) => Some(*v as i64),
            Numeric::UInt32(v) => Some(*v as i64),
            Numeric::UInt64(v) => {
                if *v <= i64::MAX as u64 {
                    Some(*v as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> Visitor<'a> for UnreachableUnionCases<'a> {
    fn visit_union(&mut self, def: &'a Def, data: &'a UnionTy) {
        self.check_union(data, &def.ident.name);
        ic_hir::visit::walk_union(self, data);
    }
}
