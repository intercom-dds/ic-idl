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
use ic_hir::hir::{ConstTy, Def, DefKind, Numeric, TyKind};
use ic_hir::visit::{Visitor, walk_const};

use crate::{Category, Lint, LintCtx};

pub struct InvalidEnumLiteral<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for InvalidEnumLiteral<'a> {
    fn name() -> &'static str {
        "invalid_enum_literal"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Enum constant outside of legal range"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = InvalidEnumLiteral { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl InvalidEnumLiteral<'_> {
    #[allow(clippy::cast_possible_wrap)]
    fn check_const(&mut self, const_def: &Def, const_ty: &ConstTy) {
        // Check if this constant has an enum type
        let enum_id = match &const_ty.ty.kind {
            TyKind::Adt(id) => {
                let def = self.context().definitions.get(*id);
                if matches!(def.kind, DefKind::Enum(_)) {
                    *id
                } else {
                    return;
                }
            }
            _ => return,
        };

        // Get the integer value if it's a direct integer literal
        let int_value = match &const_ty.value {
            Numeric::Int8(v) => i64::from(*v),
            Numeric::Int16(v) => i64::from(*v),
            Numeric::Int32(v) => i64::from(*v),
            Numeric::Int64(v) => *v,
            Numeric::Octet(v) => i64::from(*v),
            Numeric::UInt16(v) => i64::from(*v),
            Numeric::UInt32(v) => i64::from(*v),
            Numeric::UInt64(v) => *v as i64,
            _ => return,
        };

        // Check if this value matches any enum member
        let enum_def = self.context().definitions.get(enum_id);
        if let DefKind::Enum(enum_ty) = &enum_def.kind {
            let mut found = false;
            for &field_id in &enum_ty.fields {
                let field_def = self.context().definitions.get(field_id);
                if let DefKind::Const(field_const) = &field_def.kind {
                    let field_value = match field_const.value {
                        Numeric::Int32(v) => i64::from(v),
                        Numeric::Int64(v) => v,
                        _ => continue,
                    };

                    if field_value == int_value {
                        found = true;
                        break;
                    }
                }
            }

            if !found
                && let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!(
                        "`{int_value}` is not a valid `{}` value",
                        enum_def.ident.name
                    ),
                    Label::new(const_def.ident.span)
                        .message("this constant evaluates to an invalid value"),
                )
            {
                Self::report(self.ctx, diag);
            }
        }
    }
}

impl<'a> Visitor<'a> for InvalidEnumLiteral<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_enum(&mut self, _: &'a Def, _: &'a ic_hir::hir::EnumTy) {
        // Don't traverse enum members since they're also constants
    }

    fn visit_const(&mut self, def: &'a Def, data: &'a ConstTy) {
        self.check_const(def, data);
        walk_const(self, data);
    }
}
