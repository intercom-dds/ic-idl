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

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, InterfaceTy, Spanned, ValueTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct RedundantInheritance<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for RedundantInheritance<'a> {
    fn name() -> &'static str {
        "redundant-inheritance"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when interfaces inherit from same parent multiple times"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = RedundantInheritance { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl RedundantInheritance<'_> {
    fn check_parents<'p, I>(&self, parents: I, owner: &str)
    where
        I: IntoIterator<Item = &'p Spanned<DefId>>,
    {
        let mut seen = HashMap::new();
        for parent in parents {
            let name = &self.hir.context.type_of(parent.value).ident.name;
            if let Some(&first_span) = seen.get(&parent.value) {
                let diag = self
                    .ctx
                    .diag_span(
                        Self::name(),
                        Self::category(),
                        format!("{owner} inherits from `{name}` multiple times"),
                        Label::new(parent.span).message("redundant inheritance"),
                    )
                    .label(Label::new(first_span).message("first listed here"));
                Self::report(self.ctx, diag);
            } else {
                seen.insert(parent.value, parent.span);
            }
        }
    }
}

impl<'a> Visitor<'a> for RedundantInheritance<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_interface(&mut self, def: &'a ic_hir::hir::Def, data: &'a InterfaceTy) {
        self.check_parents(&data.parents, &format!("interface `{}`", def.ident.name));
        ic_hir::visit::walk_interface(self, def, data);
    }

    fn visit_valuetype(&mut self, def: &'a ic_hir::hir::Def, data: &'a ValueTy) {
        let parents = data.parent.iter().chain(data.supports.iter());
        self.check_parents(parents, &format!("valuetype `{}`", def.ident.name));
        ic_hir::visit::walk_valuetype(self, def, data);
    }
}
