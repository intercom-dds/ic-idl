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

pub struct PreferEnumName<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for PreferEnumName<'a> {
    fn name() -> &'static str {
        "prefer-enum-name"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Prefer using enum member names instead of numeric literals"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = PreferEnumName { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl PreferEnumName<'_> {
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

        // Find the enum member with this value
        let enum_def = self.context().definitions.get(enum_id);
        if let DefKind::Enum(enum_ty) = &enum_def.kind {
            for &field_id in &enum_ty.fields {
                let field_def = self.context().definitions.get(field_id);
                if let DefKind::Const(field_const) = &field_def.kind {
                    let field_value = match field_const.value {
                        Numeric::Int32(v) => i64::from(v),
                        Numeric::Int64(v) => v,
                        _ => continue,
                    };

                    if field_value == int_value {
                        // Found the matching enum member
                        if let Some(diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!(
                                "prefer using enum member name '{}' instead of numeric literal",
                                field_def.ident.name
                            ),
                            Label::new(const_def.span).message(format!(
                                "consider using '{}' instead of '{}'",
                                field_def.ident.name,
                                format_numeric_value(&const_ty.value)
                            )),
                        ) {
                            Self::report(self.ctx, diag);
                        }
                        return;
                    }
                }
            }
        }
    }
}

/// Format a numeric value for display in diagnostics
fn format_numeric_value(value: &Numeric) -> String {
    match value {
        Numeric::Int8(v) => v.to_string(),
        Numeric::Int16(v) => v.to_string(),
        Numeric::Int32(v) => v.to_string(),
        Numeric::Int64(v) => v.to_string(),
        Numeric::Octet(v) => v.to_string(),
        Numeric::UInt16(v) => v.to_string(),
        Numeric::UInt32(v) => v.to_string(),
        Numeric::UInt64(v) => v.to_string(),
        _ => "?".to_string(),
    }
}

impl<'a> Visitor<'a> for PreferEnumName<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_enum(&mut self, _: &'a Def, _: &'a ic_hir::hir::EnumTy) {
        // Don't traverse enum members since they're also constants
    }

    fn visit_const(&mut self, def: &'a Def, data: &'a ConstTy) {
        // Skip enum members themselves
        if def.parent.is_some() {
            if let Some(parent_def) = def.parent.map(|p| self.context().definitions.get(p)) {
                if matches!(parent_def.kind, DefKind::Enum(_)) {
                    // This is an enum member, skip checking
                    return;
                }
            }
        }

        self.check_const(def, data);
        walk_const(self, data);
    }
}
