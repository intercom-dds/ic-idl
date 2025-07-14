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
use ic_hir::hir::{Def, DefId, Numeric, Ty, TyKind};
use ic_hir::visit::Visitor;
use ic_hir::{Context, ResolvedGraph};

use crate::{Category, Lint, LintCtx};

pub struct Deprecated<'a> {
    ctx: &'a LintCtx<'a>,
    hir_ctx: &'a Context,
    deprecated_items: HashSet<DefId>,
}

impl<'a> Lint<'a> for Deprecated<'a> {
    fn name() -> &'static str {
        "Deprecated"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut deprecated_items = HashSet::new();

        // First pass: collect all deprecated items
        for (id, def) in hir.context.definitions.iter() {
            if is_deprecated(def) {
                deprecated_items.insert(id);
            }
        }

        // Second pass: check for usage of deprecated items
        let mut visitor = Deprecated {
            ctx,
            hir_ctx: &hir.context,
            deprecated_items,
        };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

fn is_deprecated(def: &Def) -> bool {
    def.annotations.iter().any(|ann| {
        ann.path.segments.last().map(|s| s.name.as_str()) == Some("deprecated")
            || ann.path.segments.last().map(|s| s.name.as_str()) == Some("obsolete")
    })
}

impl<'a> Deprecated<'a> {
    fn check_type_usage(&mut self, ty: &Ty, usage_context: &str) {
        if let TyKind::Adt(def_id) = &ty.kind {
            if self.deprecated_items.contains(def_id) {
                let def = self.hir_ctx.definitions.get(*def_id);
                let message = self.get_deprecation_message(&def);

                if let Some(mut diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    &format!(
                        "use of deprecated type '{}' {}",
                        def.ident.name, usage_context
                    ),
                    Label::new(ty.span).message("deprecated type used here"),
                ) {
                    diag = diag.help(&message);
                    self.ctx.report(Self::name(), Self::category(), diag);
                }
            }
        }
    }

    fn check_const_usage(&mut self, def_id: DefId, span: ic_syntax::Span) {
        if self.deprecated_items.contains(&def_id) {
            let def = self.hir_ctx.definitions.get(def_id);
            let message = self.get_deprecation_message(&def);

            if let Some(mut diag) = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                &format!("use of deprecated constant '{}'", def.ident.name),
                Label::new(span).message("deprecated constant used here"),
            ) {
                diag = diag.help(&message);
                self.ctx.report(Self::name(), Self::category(), diag);
            }
        }
    }

    fn get_deprecation_message(&self, def: &Def) -> String {
        // Try to find a deprecation message in the annotation
        for ann in &def.annotations {
            let name = ann
                .path
                .segments
                .last()
                .map(|s| s.name.as_str())
                .unwrap_or("");
            if name == "deprecated" || name == "obsolete" {
                // Look for a "reason" or "message" argument, or the first string argument
                for arg in &ann.args {
                    if let Some(ident) = &arg.ident {
                        if ident.name == "reason" || ident.name == "message" {
                            if let Numeric::String(msg) = &arg.value {
                                return msg.clone();
                            }
                        }
                    } else if let Numeric::String(msg) = &arg.value {
                        return msg.clone();
                    }
                }
            }
        }

        // Default message
        format!("'{}' has been deprecated", def.ident.name)
    }
}

impl<'a> Visitor<'a> for Deprecated<'a> {
    fn visit_ty(&mut self, ty: &'a Ty) {
        self.check_type_usage(ty, "");
        ic_hir::visit::walk_ty(self, ty);
    }

    fn visit_numeric(&mut self, num: &'a Numeric) {
        // Check for const references
        if let Numeric::Const(def_id) = num {
            // TODO: We need a span here, but Numeric doesn't carry one
            // For now, use a dummy span
            let span = ic_syntax::Span::default();
            self.check_const_usage(*def_id, span);
        }
        ic_hir::visit::walk_numeric(self, num);
    }
}
