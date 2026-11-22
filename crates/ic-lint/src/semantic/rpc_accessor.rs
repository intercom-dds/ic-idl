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

use std::collections::HashSet;

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Attribute, Def, DefKind, InterfaceTy, ProtoTy};
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

impl RpcAccessor<'_> {
    fn check_hierarchy(&self, def: &Def, data: &InterfaceTy) {
        let ctx = &self.hir.context;

        let bases: Vec<HashSet<_>> = data
            .parents
            .iter()
            .map(|parent| {
                ctx.hierarchy_of(parent.def_id)
                    .map(|base| base.id)
                    .collect()
            })
            .collect();

        let mut attributes = vec![];
        let mut prototypes = vec![];
        for base in ctx.hierarchy_of(def.id) {
            if let DefKind::Interface(iface) = &base.kind {
                attributes.extend(iface.attributes.iter().map(|attr| (base.id, attr)));
                prototypes.extend(iface.prototypes.iter().map(|proto| (base.id, proto)));
            }
        }

        for &(attr_owner, attr) in &attributes {
            for &(proto_owner, proto) in &prototypes {
                if !implies(attr, &proto.ident.name) {
                    continue;
                }

                let from_one_base = bases
                    .iter()
                    .any(|base| base.contains(&attr_owner) && base.contains(&proto_owner));
                if from_one_base {
                    continue;
                }

                let inherited = attr_owner != def.id && proto_owner != def.id;
                self.report_collision(def, attr, proto, inherited);
            }
        }
    }

    fn report_collision(&self, def: &Def, attr: &Attribute, proto: &ProtoTy, inherited: bool) {
        let mut diag = self
            .ctx
            .diag_span(
                Self::name(),
                Self::category(),
                format!(
                    "operation `{}` collides with an accessor implied by attribute `{}`",
                    proto.ident.name, attr.ident.name
                ),
                Label::new(proto.ident.span).message("operation declared here"),
            )
            .label(Label::new(attr.ident.span).message("attribute declared here"));

        if inherited {
            diag = diag.label(Label::new(def.ident.span).message("both inherited here"));
        }

        Self::report(self.ctx, diag);
    }
}

impl<'a> Visitor<'a> for RpcAccessor<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_interface(&mut self, def: &'a Def, data: &'a InterfaceTy) {
        if is_rpc_service(&self.hir.context, def) {
            self.check_hierarchy(def, data);
        }

        ic_hir::visit::walk_interface(self, def, data);
    }
}

fn implies(attr: &Attribute, operation: &str) -> bool {
    let getter = format!("get_attribute_{}", attr.ident.name);
    let setter = format!("set_attribute_{}", attr.ident.name);

    operation.eq_ignore_ascii_case(&getter)
        || (!attr.is_readonly && operation.eq_ignore_ascii_case(&setter))
}
