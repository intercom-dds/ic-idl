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

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{BitsetTy, Def, InterfaceTy, Spanned, StructTy, ValueTy};
use ic_hir::visit::Visitor;
use ic_hir_analysis::annotation::{Extensibility, extensibility};

use crate::{Category, Lint, LintCtx};

pub struct InheritanceExtensibility<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for InheritanceExtensibility<'a> {
    fn name() -> &'static str {
        "inheritance-extensibility"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Requires derived types to use their parent's extensibility"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &'a ResolvedGraph) {
        let mut visitor = Self { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl InheritanceExtensibility<'_> {
    fn name_of(extensibility: Extensibility) -> &'static str {
        match extensibility {
            Extensibility::Final => "final",
            Extensibility::Appendable => "appendable",
            Extensibility::Mutable => "mutable",
        }
    }

    fn check_parent(&self, def: &Def, parent: Spanned<ic_hir::hir::DefId>) {
        let parent_def = self.hir.context.base_def_of(parent.def_id);
        let child_extensibility = extensibility(&self.hir.context, def);
        let parent_extensibility = extensibility(&self.hir.context, parent_def);
        if child_extensibility == parent_extensibility {
            return;
        }

        Self::report(
            self.ctx,
            error_span(
                format!(
                    "derived type `{}` is {}, but parent `{}` is {}",
                    def.ident.name,
                    Self::name_of(child_extensibility),
                    parent_def.ident.name,
                    Self::name_of(parent_extensibility),
                ),
                Label::new(def.ident.span).message("extensibility differs from parent"),
            )
            .label(Label::new(parent.span).message("parent type used here")),
        );
    }
}

impl<'a> Visitor<'a> for InheritanceExtensibility<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a Def, data: &'a StructTy) {
        if let Some(parent) = data.parent {
            self.check_parent(def, parent);
        }
        ic_hir::visit::walk_struct(self, data);
    }

    fn visit_interface(&mut self, def: &'a Def, data: &'a InterfaceTy) {
        for &parent in &data.parents {
            self.check_parent(def, parent);
        }
        ic_hir::visit::walk_interface(self, def, data);
    }

    fn visit_valuetype(&mut self, def: &'a Def, data: &'a ValueTy) {
        if let Some(parent) = data.parent {
            self.check_parent(def, parent);
        }
        ic_hir::visit::walk_valuetype(self, def, data);
    }

    fn visit_bitset(&mut self, def: &'a Def, data: &'a BitsetTy) {
        if let Some(parent) = data.parent {
            self.check_parent(def, parent);
        }
        ic_hir::visit::walk_bitset(self, data);
    }
}
