// Copyright 2025 KONGSBERG
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

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use std::collections::HashSet;

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefId, DefKind, Numeric, PrimitiveTy, TyKind, UnionTy, Variant};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that checks for default cases in unions when all possible
/// discriminator values are covered.
pub struct ExhaustiveUnionDefaultLint<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for ExhaustiveUnionDefaultLint<'a> {
    fn name() -> &'static str {
        "exhaustive_union_default"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "errors when default cases are used with exhaustive case coverage"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = ExhaustiveUnionDefaultLint { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for ExhaustiveUnionDefaultLint<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        if !union_ty.variants.iter().any(|v| v.is_default) {
            return;
        }

        let non_default_variants: Vec<_> =
            union_ty.variants.iter().filter(|v| !v.is_default).collect();

        if non_default_variants.is_empty() {
            return;
        }

        match &union_ty.disc.kind {
            TyKind::Adt(def_id) => {
                let adt_def = self.hir.context.definitions.get(*def_id);
                if let DefKind::Enum(_) = &adt_def.kind {
                    self.check_enum_exhaustiveness(*def_id, &non_default_variants, def);
                }
            }
            TyKind::Primitive(PrimitiveTy::Bool) => {
                self.check_bool_exhaustiveness(&non_default_variants, def);
            }
            TyKind::Primitive(prim) => {
                self.check_integer_exhaustiveness(*prim, &non_default_variants, def);
            }
            _ => {}
        }
    }
}

impl ExhaustiveUnionDefaultLint<'_> {
    fn report_exhaustive_default(&self, union_def: &Def, count: &str, help: String) {
        let diag = error_span(
            format!(
                "default case is not allowed when all {count} values are covered in union `{}`",
                union_def.ident.name
            ),
            Label::new(union_def.span).message("exhausted all possible values"),
        )
        .help(help)
        .note(
            "default cases are only allowed when some discriminator values are not explicitly \
             handled",
        );

        Self::report(self.ctx, diag);
    }

    fn check_enum_exhaustiveness(
        &mut self,
        enum_id: DefId,
        non_default_variants: &[&Variant],
        union_def: &Def,
    ) {
        let enum_def = self.hir.context.definitions.get(enum_id);
        let enum_name = enum_def.ident.name.clone();

        let mut enumerator_count = 0;

        for (_, const_def) in &self.hir.context.definitions {
            if let DefKind::Const(const_ty) = &const_def.kind {
                match &const_ty.ty.kind {
                    TyKind::Adt(const_enum_id) if *const_enum_id == enum_id => {
                        enumerator_count += 1;
                    }
                    _ => {}
                }
            }
        }

        let mut referenced_enumerators = HashSet::new();
        for variant in non_default_variants {
            for label in &variant.labels {
                if let Numeric::Const(const_id) = &label.value {
                    let const_def = self.hir.context.definitions.get(*const_id);
                    if let DefKind::Const(const_ty) = &const_def.kind {
                        if let TyKind::Adt(const_enum_id) = &const_ty.ty.kind {
                            if *const_enum_id == enum_id {
                                referenced_enumerators.insert(*const_id);
                            }
                        }
                    }
                }
            }
        }

        if referenced_enumerators.len() == enumerator_count && enumerator_count > 0 {
            self.report_exhaustive_default(
                union_def,
                "enum",
                format!(
                    "all {enumerator_count} enumerators of '{enum_name}' are already handled by \
                     explicit cases"
                ),
            );
        }
    }

    fn check_bool_exhaustiveness(&mut self, non_default_variants: &[&Variant], union_def: &Def) {
        let mut has_true = false;
        let mut has_false = false;

        for variant in non_default_variants {
            for label in &variant.labels {
                match &label.value {
                    Numeric::Bool(true) => has_true = true,
                    Numeric::Bool(false) => has_false = true,
                    _ => {}
                }
            }
        }

        if has_true && has_false {
            self.report_exhaustive_default(
                union_def,
                "boolean",
                "both 'true' and 'false' are already handled by explicit cases".to_string(),
            );
        }
    }

    fn check_integer_exhaustiveness(
        &mut self,
        prim: PrimitiveTy,
        non_default_variants: &[&Variant],
        union_def: &Def,
    ) {
        let (min, max, type_name) = match prim {
            PrimitiveTy::Int8 => (i64::from(i8::MIN), i64::from(i8::MAX), "int8"),
            PrimitiveTy::UInt8 => (i64::from(u8::MIN), i64::from(u8::MAX), "uint8"),
            PrimitiveTy::Int16 => (i64::from(i16::MIN), i64::from(i16::MAX), "int16"),
            PrimitiveTy::UInt16 => (i64::from(u16::MIN), i64::from(u16::MAX), "uint16"),
            _ => return,
        };

        let total_possible_values = (max - min + 1) as usize;
        let total_case_labels = non_default_variants
            .iter()
            .map(|v| v.labels.len())
            .sum::<usize>();

        if total_case_labels < total_possible_values {
            return;
        }

        let mut covered_values = HashSet::new();
        for variant in non_default_variants {
            for label in &variant.labels {
                match &label.value {
                    Numeric::Int8(v) => {
                        covered_values.insert(*v as i64);
                    }
                    Numeric::Octet(v) => {
                        covered_values.insert(*v as i64);
                    }
                    Numeric::Int16(v) => {
                        covered_values.insert(*v as i64);
                    }
                    Numeric::UInt16(v) => {
                        covered_values.insert(*v as i64);
                    }
                    Numeric::Int32(v) => {
                        covered_values.insert(*v as i64);
                    }
                    Numeric::UInt32(v) => {
                        covered_values.insert(*v as i64);
                    }
                    Numeric::Int64(v) => {
                        covered_values.insert(*v);
                    }
                    Numeric::UInt64(v) => {
                        if i64::try_from(*v).is_ok() {
                            covered_values.insert(*v as i64);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check if all values in range are covered
        if covered_values.len() == total_possible_values {
            // Verify all values are actually in range and exhaustive
            let mut all_covered = true;
            for value in min..=max {
                if !covered_values.contains(&value) {
                    all_covered = false;
                    break;
                }
            }

            if all_covered {
                self.report_exhaustive_default(
                    union_def,
                    type_name,
                    format!(
                        "all {total_possible_values} possible values of '{type_name}' ({min} to \
                         {max}) are already handled by explicit cases"
                    ),
                );
            }
        }
    }
}
