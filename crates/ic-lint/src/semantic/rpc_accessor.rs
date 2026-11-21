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
use ic_hir::hir::{Def, InterfaceTy};
use ic_hir::visit::Visitor;
use ic_hir_analysis::annotation::is_rpc_service;

use crate::{Category, Lint, LintCtx};

pub struct RpcAccessor<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for RpcAccessor<'a> {
    fn name() -> &'static str {
        "rpc-accessor"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when RPC service operations collide with implied attribute accessors"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = RpcAccessor { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for RpcAccessor<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_interface(&mut self, def: &'a Def, data: &'a InterfaceTy) {
        if is_rpc_service(&self.hir.context, def) {
            for attr in &data.attributes {
                let mut accessors = vec![format!("get_attribute_{}", attr.ident.name)];
                if !attr.is_readonly {
                    accessors.push(format!("set_attribute_{}", attr.ident.name));
                }

                for proto in &data.prototypes {
                    if !accessors
                        .iter()
                        .any(|accessor| accessor.eq_ignore_ascii_case(&proto.ident.name))
                    {
                        continue;
                    }

                    let diag = self
                        .ctx
                        .diag_span(
                            Self::name(),
                            Self::category(),
                            format!(
                                "operation `{}` collides with an accessor implied by attribute \
                                 `{}`",
                                proto.ident.name, attr.ident.name
                            ),
                            Label::new(proto.ident.span).message("operation declared here"),
                        )
                        .label(Label::new(attr.ident.span).message("attribute declared here"));
                    Self::report(self.ctx, diag);
                }
            }
        }

        ic_hir::visit::walk_interface(self, def, data);
    }
}
