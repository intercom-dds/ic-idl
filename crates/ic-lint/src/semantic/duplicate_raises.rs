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

use std::collections::HashMap;

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Attribute, DefId, ProtoTy, Spanned};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct DuplicateRaises<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for DuplicateRaises<'a> {
    fn name() -> &'static str {
        "duplicate-raises"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when raises lists contain duplicate exceptions"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateRaises { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl DuplicateRaises<'_> {
    fn check_exceptions(&self, exceptions: &[Spanned<DefId>], clause: &str, owner: &str) {
        let mut seen = HashMap::new();

        for exception in exceptions {
            let def = self.hir.context.type_of(exception.value);

            if let Some(&first_span) = seen.get(&exception.value) {
                let diag = self
                    .ctx
                    .diag_span(
                        Self::name(),
                        Self::category(),
                        format!(
                            "duplicate exception `{}` in {clause} clause of `{owner}`",
                            def.ident.name
                        ),
                        Label::new(exception.span).message("duplicate exception"),
                    )
                    .label(Label::new(first_span).message("first listed here"))
                    .help("remove the duplicate exception from the clause");
                Self::report(self.ctx, diag);
            } else {
                seen.insert(exception.value, exception.span);
            }
        }
    }
}

impl<'a> Visitor<'a> for DuplicateRaises<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_proto(&mut self, proto: &'a ProtoTy) {
        self.check_exceptions(&proto.raises, "raises", &proto.ident.name);
        ic_hir::visit::walk_proto(self, proto);
    }

    fn visit_attribute(&mut self, attr: &'a Attribute) {
        self.check_exceptions(&attr.getraises, "getraises", &attr.ident.name);
        self.check_exceptions(&attr.setraises, "setraises", &attr.ident.name);
        ic_hir::visit::walk_attribute(self, attr);
    }
}
