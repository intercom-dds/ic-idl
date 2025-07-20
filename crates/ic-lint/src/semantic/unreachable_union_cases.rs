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

impl UnreachableUnionCases<'_> {
    fn check_union(&mut self, union_ty: &UnionTy, _union_name: &str) {
        // Check for out-of-range case labels
        if let Some((min, max)) = Self::get_discriminator_range(&union_ty.disc.kind) {
            for variant in &union_ty.variants {
                for label in &variant.labels {
                    if let Some(value) = Self::numeric_to_i64(label) {
                        // For unsigned discriminators, negative values wrap around
                        let effective_value =
                            if Self::is_unsigned_discriminator(&union_ty.disc.kind) && value < 0 {
                                // Apply wrapping conversion for negative values on unsigned types
                                Self::wrap_to_unsigned(value, &union_ty.disc.kind)
                            } else {
                                value
                            };

                        if effective_value < min || effective_value > max {
                            if let Some(diag) = self.ctx.diag_span(
                                Self::name(),
                                Self::category(),
                                format!(
                                    "case label {value} is outside the range [{min}, {max}] of \
                                     the discriminator type"
                                ),
                                Label::new(variant.ident.span).message("case label out of range"),
                            ) {
                                Self::report(self.ctx, diag);
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_discriminator_range(ty_kind: &TyKind) -> Option<(i64, i64)> {
        match ty_kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => Some((0, 1)),
                PrimitiveTy::Char => Some((0, 127)), // ASCII range
                PrimitiveTy::WChar => Some((0, 65535)), // Unicode BMP
                PrimitiveTy::Int8 => Some((i64::from(i8::MIN), i64::from(i8::MAX))),
                PrimitiveTy::Int16 => Some((i64::from(i16::MIN), i64::from(i16::MAX))),
                PrimitiveTy::Int32 => Some((i64::from(i32::MIN), i64::from(i32::MAX))),
                PrimitiveTy::Int64 => Some((i64::MIN, i64::MAX)),
                PrimitiveTy::UInt8 => Some((0, i64::from(u8::MAX))),
                PrimitiveTy::UInt16 => Some((0, i64::from(u16::MAX))),
                PrimitiveTy::UInt32 => Some((0, i64::from(u32::MAX))),
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

    fn numeric_to_i64(num: &Numeric) -> Option<i64> {
        match num {
            Numeric::Bool(v) => Some(i64::from(*v)),
            Numeric::Char(v) => Some(i64::from(*v as u32)),
            Numeric::Int8(v) => Some(i64::from(*v)),
            Numeric::Int16(v) => Some(i64::from(*v)),
            Numeric::Int32(v) => Some(i64::from(*v)),
            Numeric::Int64(v) => Some(*v),
            Numeric::Octet(v) => Some(i64::from(*v)),
            Numeric::UInt16(v) => Some(i64::from(*v)),
            Numeric::UInt32(v) => Some(i64::from(*v)),
            Numeric::UInt64(v) => i64::try_from(*v).ok(),
            _ => None,
        }
    }

    fn is_unsigned_discriminator(ty_kind: &TyKind) -> bool {
        matches!(
            ty_kind,
            TyKind::Primitive(
                PrimitiveTy::Bool
                    | PrimitiveTy::Char
                    | PrimitiveTy::WChar
                    | PrimitiveTy::UInt8
                    | PrimitiveTy::UInt16
                    | PrimitiveTy::UInt32
                    | PrimitiveTy::UInt64
            )
        )
    }

    fn wrap_to_unsigned(value: i64, ty_kind: &TyKind) -> i64 {
        match ty_kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => ((value as u8) & 1) as i64,
                PrimitiveTy::Char => (value as u8) as i64,
                PrimitiveTy::WChar => (value as u16) as i64,
                PrimitiveTy::UInt8 => (value as u8) as i64,
                PrimitiveTy::UInt16 => (value as u16) as i64,
                PrimitiveTy::UInt32 => (value as u32) as i64,
                PrimitiveTy::UInt64 => value as u64 as i64,
                _ => value, // For signed types, no wrapping
            },
            _ => value,
        }
    }
}

impl<'a> Visitor<'a> for UnreachableUnionCases<'a> {
    fn visit_union(&mut self, def: &'a Def, data: &'a UnionTy) {
        self.check_union(data, &def.ident.name);
        ic_hir::visit::walk_union(self, data);
    }
}
