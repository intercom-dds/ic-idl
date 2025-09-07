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

use std::collections::HashSet;

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, Numeric, UnionTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct DuplicateCaseLabels<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for DuplicateCaseLabels<'a> {
    fn name() -> &'static str {
        "duplicate-case"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when union case labels are duplicated"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateCaseLabels { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl DuplicateCaseLabels<'_> {
    fn check_union(&mut self, union_ty: &UnionTy, union_name: &str) {
        let mut seen_labels = HashSet::new();
        for variant in &union_ty.variants {
            for label in &variant.labels {
                let label_key = self.numeric_to_string(&label.value);
                if !seen_labels.insert(label_key.clone()) {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        format!("union '{union_name}' has duplicate case label '{label_key}'"),
                        Label::new(label.span).message("duplicate case label"),
                    ) {
                        Self::report(self.ctx, diag);
                    }
                }
            }
        }
    }

    fn numeric_to_string(&self, num: &Numeric) -> String {
        match num {
            Numeric::Bool(v) => v.to_string(),
            Numeric::Char(v) => format!("'{v}'"),
            Numeric::Int8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::Int64(v) => v.to_string(),
            Numeric::Octet(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::UInt32(v) => v.to_string(),
            Numeric::UInt64(v) => v.to_string(),
            Numeric::Float(v) => v.to_string(),
            Numeric::Double(v) => v.to_string(),
            Numeric::String(v) => format!("\"{v}\""),
            Numeric::Const(id) => {
                let def = self.hir.context.definitions.get(*id);
                def.ident.name.clone()
            }
            _ => format!("{num:?}"),
        }
    }
}

impl<'a> Visitor<'a> for DuplicateCaseLabels<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, def: &'a Def, data: &'a UnionTy) {
        self.check_union(data, &def.ident.name);
        ic_hir::visit::walk_union(self, data);
    }
}
