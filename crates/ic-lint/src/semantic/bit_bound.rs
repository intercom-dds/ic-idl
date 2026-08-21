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
use ic_hir::hir::Def;
use ic_hir::visit::Visitor;
use ic_hir_analysis::annotation::bit_bound_annotation;

use crate::{Category, Lint, LintCtx};

pub struct BitBound<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for BitBound<'a> {
    fn name() -> &'static str {
        "bit-bound"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when @bit_bound is outside the supported range"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = BitBound { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl BitBound<'_> {
    fn check_bit_bound(&self, def: &Def, maximum: u64) {
        let Some(annotation) = bit_bound_annotation(&self.hir.context, def) else {
            return;
        };
        let Some(argument) = annotation.args.first() else {
            return;
        };

        let bit_bound = self.hir.context.unsigned_value(&argument.value);
        let message = if bit_bound == 0 {
            Some("@bit_bound must be at least 1".to_string())
        } else if bit_bound > maximum {
            Some(format!(
                "@bit_bound({bit_bound}) exceeds maximum of {maximum}"
            ))
        } else {
            None
        };
        let Some(message) = message else {
            return;
        };

        let diag = self.ctx.diag_span(
            Self::name(),
            Self::category(),
            message,
            Label::new(argument.ident.span).message("bit bound out of bounds"),
        );
        Self::report(self.ctx, diag);
    }
}

impl<'a> Visitor<'a> for BitBound<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_enum(&mut self, def: &'a Def, data: &'a ic_hir::hir::EnumTy) {
        self.check_bit_bound(def, 32);
        ic_hir::visit::walk_enum(self, data);
    }

    fn visit_bitmask(&mut self, def: &'a Def, data: &'a ic_hir::hir::BitmaskTy) {
        self.check_bit_bound(def, 64);
        ic_hir::visit::walk_bitmask(self, data);
    }
}
