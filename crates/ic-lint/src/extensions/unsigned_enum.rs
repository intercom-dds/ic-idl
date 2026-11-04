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
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, EnumTy, PrimitiveTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that warns about enums whose values exceed the signed range of their
/// underlying type, which makes the enum lower to an unsigned type.
pub struct UnsignedEnum<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for UnsignedEnum<'a> {
    fn name() -> &'static str {
        "unsigned-enum"
    }

    fn category() -> Category {
        Category::Extensions
    }

    fn description() -> &'static str {
        "Enum values outside the signed range"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = UnsignedEnum { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

fn signed_counterpart(ty: PrimitiveTy) -> Option<(PrimitiveTy, i64)> {
    match ty {
        PrimitiveTy::UInt8 => Some((PrimitiveTy::Int8, i64::from(i8::MAX))),
        PrimitiveTy::UInt16 => Some((PrimitiveTy::Int16, i64::from(i16::MAX))),
        PrimitiveTy::UInt32 => Some((PrimitiveTy::Int32, i64::from(i32::MAX))),
        _ => None,
    }
}

impl<'a> Visitor<'a> for UnsignedEnum<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_enum(&mut self, def: &'a Def, enum_ty: &'a EnumTy) {
        let Some((signed_ty, signed_max)) = signed_counterpart(enum_ty.ty) else {
            return;
        };

        let first_unsigned = enum_ty.fields.iter().find_map(|&field_id| {
            let field_def = self.context().definitions.get(field_id);
            let DefKind::Const(const_ty) = &field_def.kind else {
                return None;
            };

            (self.context().integer_value(&const_ty.value) > signed_max).then_some(field_def)
        });

        let Some(field_def) = first_unsigned else {
            return;
        };

        let diag = self
            .ctx
            .diag_span(
                Self::name(),
                Self::category(),
                format!(
                    "enum `{}` has values outside the signed range",
                    def.ident.name
                ),
                Label::new(field_def.ident.span)
                    .message(format!("exceeds the `{}` range", signed_ty.name())),
            )
            .note("enums are supposed to be signed");

        Self::report(self.ctx, diag);
    }
}
