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
use ic_hir::hir::{DefId, DefKind};
use ic_hir::{Context, ResolvedGraph};

use crate::{Category, Lint, LintCtx};

pub struct CircularInheritance<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for CircularInheritance<'a> {
    fn name() -> &'static str {
        "CircularInheritance"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let checker = CircularInheritance { ctx };
        checker.check_all_inheritance(&hir.context);
    }
}

impl CircularInheritance<'_> {
    fn check_all_inheritance(&self, hir_ctx: &Context) {
        for (id, def) in &hir_ctx.definitions {
            match &def.kind {
                DefKind::Struct(struct_ty) => {
                    if let Some(parent) = struct_ty.parent {
                        self.check_inheritance_chain(id, parent, hir_ctx, "struct");
                    }
                }
                DefKind::Interface(interface_ty) => {
                    for &parent in &interface_ty.parents {
                        self.check_inheritance_chain(id, parent, hir_ctx, "interface");
                    }
                }
                DefKind::Valuetype(valuetype_ty) => {
                    if let Some(parent) = valuetype_ty.parent {
                        self.check_inheritance_chain(id, parent, hir_ctx, "valuetype");
                    }
                    if let Some(extends) = valuetype_ty.extends {
                        self.check_inheritance_chain(id, extends, hir_ctx, "valuetype");
                    }
                }
                _ => {}
            }
        }
    }

    fn check_inheritance_chain(
        &self,
        start_id: DefId,
        parent_id: DefId,
        hir_ctx: &Context,
        kind: &str,
    ) {
        let mut visited = HashSet::new();
        let mut path = vec![start_id];

        if self.has_cycle(parent_id, &mut visited, &mut path, hir_ctx) {
            // Found a cycle
            let start_def = hir_ctx.definitions.get(start_id);
            let parent_def = hir_ctx.definitions.get(parent_id);

            if let Some(diag) = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                format!(
                    "{} '{}' has circular inheritance through '{}'",
                    kind, start_def.ident.name, parent_def.ident.name
                ),
                Label::new(start_def.ident.span).message("circular inheritance detected"),
            ) {
                self.ctx.report(Self::name(), Self::category(), diag);
            }
        }
    }

    fn has_cycle(
        &self,
        current_id: DefId,
        visited: &mut HashSet<DefId>,
        path: &mut Vec<DefId>,
        hir_ctx: &Context,
    ) -> bool {
        if path.contains(&current_id) {
            // Found a cycle
            return true;
        }

        if visited.contains(&current_id) {
            // Already visited this node without finding a cycle
            return false;
        }

        visited.insert(current_id);
        path.push(current_id);

        // Get parents of current node
        let parents = Self::get_parents(current_id, hir_ctx);

        for parent in parents {
            if self.has_cycle(parent, visited, path, hir_ctx) {
                return true;
            }
        }

        path.pop();
        false
    }

    fn get_parents(id: DefId, hir_ctx: &Context) -> Vec<DefId> {
        let def = hir_ctx.definitions.get(id);
        match &def.kind {
            DefKind::Struct(struct_ty) => struct_ty.parent.into_iter().collect(),
            DefKind::Interface(interface_ty) => interface_ty.parents.clone(),
            DefKind::Valuetype(valuetype_ty) => {
                let mut parents = Vec::new();
                if let Some(parent) = valuetype_ty.parent {
                    parents.push(parent);
                }
                if let Some(extends) = valuetype_ty.extends {
                    parents.push(extends);
                }
                parents
            }
            _ => Vec::new(),
        }
    }
}
