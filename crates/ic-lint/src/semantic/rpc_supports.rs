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

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, ValueTy};
use ic_hir::visit::Visitor;
use ic_hir_analysis::annotation::is_rpc_service;

use crate::{Category, Lint, LintCtx};

pub struct RpcSupports<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for RpcSupports<'a> {
    fn name() -> &'static str {
        "rpc-supports"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when valuetypes support RPC services"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = RpcSupports { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for RpcSupports<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_valuetype(&mut self, def: &'a Def, data: &'a ValueTy) {
        if let Some(supports) = &data.supports {
            let supported = self.hir.context.base_def_of(supports.def_id);
            if is_rpc_service(&self.hir.context, supported) {
                let diag = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!(
                        "valuetype `{}` supports `@DDSService` interface `{}`",
                        def.ident.name, supported.ident.name
                    ),
                    Label::new(supports.span).message("supported here"),
                );
                Self::report(self.ctx, diag);
            }
        }

        ic_hir::visit::walk_valuetype(self, def, data);
    }
}
