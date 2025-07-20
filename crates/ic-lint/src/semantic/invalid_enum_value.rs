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
use ic_hir::hir::{Def, EnumTy, PrimitiveTy, TyKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct InvalidEnumValue<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for InvalidEnumValue<'a> {
    fn name() -> &'static str {
        "invalid_enum_value"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = InvalidEnumValue { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl InvalidEnumValue<'_> {
    fn check_enum(&mut self, enum_ty: &EnumTy, _enum_name: &str) {
        // Get the underlying type's range
        let (min, max) = match &enum_ty.ty.kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Int8 => (i64::from(i8::MIN), i64::from(i8::MAX)),
                PrimitiveTy::Int16 => (i64::from(i16::MIN), i64::from(i16::MAX)),
                PrimitiveTy::Int64 => (i64::MIN, i64::MAX),
                PrimitiveTy::UInt8 => (0, i64::from(u8::MAX)),
                PrimitiveTy::UInt16 => (0, i64::from(u16::MAX)),
                PrimitiveTy::UInt32 => (0, i64::from(u32::MAX)),
                PrimitiveTy::UInt64 => (0, i64::MAX), // Limited by i64
                _ => (i64::from(i32::MIN), i64::from(i32::MAX)), // Default to int32
            },
            _ => (i64::from(i32::MIN), i64::from(i32::MAX)), // Default to int32
        };

        // Check each enumerator
        for field in &enum_ty.fields {
            let value = i64::try_from(field.value).unwrap();

            // Check if value is in range
            if value < min || value > max {
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!(
                        "enum value {value} is outside the range [{min}, {max}] for the \
                         underlying type"
                    ),
                    Label::new(field.ident.span).message("value out of range"),
                ) {
                    Self::report(self.ctx, diag);
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for InvalidEnumValue<'a> {
    fn visit_enum(&mut self, def: &'a Def, data: &'a EnumTy) {
        self.check_enum(data, &def.ident.name);
        ic_hir::visit::walk_enum(self, data);
    }
}
