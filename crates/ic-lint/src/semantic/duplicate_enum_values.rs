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

use std::collections::HashMap;

use ic_alloc::insensitive::CaseSet;
use ic_cli::color::Colorize as _;
use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, EnumTy, Numeric};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that checks for duplicate values in enum definitions.
/// This is an error because duplicate enum values are not allowed.
pub struct DuplicateEnumValues<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for DuplicateEnumValues<'a> {
    fn name() -> &'static str {
        "duplicate-enum-values"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when enum values are duplicated"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateEnumValues { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for DuplicateEnumValues<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_enum(&mut self, def: &'a Def, enum_ty: &'a EnumTy) {
        // Skip built-in types
        if def.ident.name.starts_with("intercom::") {
            return;
        }

        let mut field_names = CaseSet::default();
        let mut field_values: HashMap<isize, Vec<&str>> = HashMap::new();

        for &field_id in &enum_ty.fields {
            let field_def = self.context().definitions.get(field_id);

            // Check for duplicate names (case-insensitive)
            if !field_names.insert(field_def.ident.name.as_str())
                && let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!(
                        "duplicate field `{}` in enum `{}`",
                        field_def.ident.name.yellow(),
                        def.ident.name
                    ),
                    Label::new(field_def.ident.span).message("redefined here"),
                )
            {
                Self::report(self.ctx, diag.note("field names are case-insensitive"));
            }

            // Track values for duplicate checking
            if let DefKind::Const(const_ty) = &field_def.kind {
                let value = match const_ty.value {
                    Numeric::Int32(v) => isize::try_from(i64::from(v)).unwrap_or(isize::MAX),
                    Numeric::Int64(v) => isize::try_from(v).unwrap_or(isize::MAX),
                    _ => continue,
                };

                field_values
                    .entry(value)
                    .or_default()
                    .push(&field_def.ident.name);
            }
        }

        // Check for duplicate values
        for (value, names) in field_values {
            if names.len() > 1 {
                // Find the span of the first occurrence
                let first_field_id = enum_ty
                    .fields
                    .iter()
                    .find(|&&id| {
                        let def = self.context().definitions.get(id);
                        def.ident.name == names[0]
                    })
                    .unwrap();
                let first_field = self.context().definitions.get(*first_field_id);

                let mut diag = ic_diagnostic::error_span(
                    format!("duplicate value {} in enum `{}`", value, def.ident.name),
                    Label::new(first_field.ident.span)
                        .message(format!("first use of value {value}")),
                );

                // Add labels for other occurrences
                for name in &names[1..] {
                    if let Some(&field_id) = enum_ty.fields.iter().find(|&&id| {
                        let def = self.context().definitions.get(id);
                        def.ident.name == *name
                    }) {
                        let field_def = self.context().definitions.get(field_id);
                        diag = diag
                            .label(Label::new(field_def.ident.span).message("value already used"));
                    }
                }

                Self::report(self.ctx, diag);
            }
        }

        // Continue visiting
        ic_hir::visit::walk_enum(self, enum_ty);
    }
}
