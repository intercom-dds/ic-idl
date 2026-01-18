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

use ic_diagnostic::{Color, Diag, Label};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{self, DefKind};
use ic_hir::visit::walk_tree;

use crate::{Category, Lint, LintCtx};

pub struct ExceptionMember<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for ExceptionMember<'a> {
    fn name() -> &'static str {
        "exception-member"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Exceptions may only be used in `raises` expressions"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut lint = Self { ctx, hir };
        walk_tree(&mut lint, hir);
    }
}

impl<'a> ic_hir::visit::Visitor<'a> for ExceptionMember<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_ty(&mut self, ty: &'a hir::Ty) {
        if let hir::TyKind::Adt(id) = ty.kind
            && let def = self.hir.context.base_def_of(id)
            && let DefKind::Except(_) = &def.kind
        {
            self.ctx.report(
                ExceptionMember::name(),
                ExceptionMember::category(),
                Diag::error("exceptions can only be used in `raises` expressions")
                    .label(
                        Label::new(ty.span)
                            .message("invalid exception use")
                            .color(Color::Red),
                    )
                    .label(Label::new(def.ident.span).message("defined as an exception here")),
            );
        } else {
            ic_hir::visit::walk_ty(self, ty);
        }
    }

    fn visit_alias(&mut self, _: &'a hir::Def, _: &'a hir::AliasTy) {}

    fn visit_attribute(&mut self, attr: &'a hir::Attribute) {
        self.visit_ty(&attr.ty);
    }

    fn visit_proto(&mut self, proto: &'a hir::ProtoTy) {
        self.visit_ty(&proto.ty);
        for param in &proto.params {
            self.visit_parameter(param);
        }
    }
}
