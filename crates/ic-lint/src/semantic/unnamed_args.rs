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
use ic_hir::hir::{Ann, DefId, DefKind};
use ic_hir::visit::Visitor;
use ic_hir::{Context, ResolvedGraph};

use crate::{Category, Lint, LintCtx};

pub struct UnnamedArgs<'a> {
    ctx: &'a LintCtx<'a>,
    hir_ctx: &'a Context,
}

impl<'a> Lint<'a> for UnnamedArgs<'a> {
    fn name() -> &'static str {
        "unnamed_args"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = UnnamedArgs {
            ctx,
            hir_ctx: &hir.context,
        };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl UnnamedArgs<'_> {
    fn check_annotation(&mut self, ann: &Ann) {
        // Skip if all arguments are named
        if ann.args.iter().all(|arg| arg.ident.is_some()) {
            return;
        }

        // Skip if no arguments
        if ann.args.is_empty() {
            return;
        }

        // Check if this is a known annotation that requires named args
        let name = &ann.ident.name;
        match name.as_str() {
            // These annotations accept single unnamed argument
            "min" | "max" | "bit" | "id" | "optional" | "key" => {
                if ann.args.len() > 1 && ann.args.iter().any(|arg| arg.ident.is_none()) {
                    self.report_unnamed_args(ann);
                }
            }
            // These annotations require named arguments when multiple are present
            "range" => {
                if ann.args.len() > 2 && ann.args.iter().any(|arg| arg.ident.is_none()) {
                    self.report_unnamed_args(ann);
                }
            }
            // For user-defined annotations, check if they have multiple parameters
            _ => {
                // Try to resolve the annotation definition
                if let Some(ann_def_id) = self.resolve_annotation(name) {
                    let def = self.hir_ctx.definitions.get(ann_def_id);
                    if let DefKind::Annotation(ann_ty) = &def.kind {
                        // If annotation has multiple members, require named args
                        if ann_ty.members.len() > 1
                            && ann.args.iter().any(|arg| arg.ident.is_none())
                        {
                            self.report_unnamed_args(ann);
                        }
                    }
                }
            }
        }
    }

    fn report_unnamed_args(&mut self, ann: &Ann) {
        let name = &ann.ident.name;
        if let Some(mut diag) = self.ctx.diag_span(
            Self::name(),
            Self::category(),
            "annotation arguments should be named when multiple parameters exist",
            Label::new(ann.ident.span).message("use named arguments"),
        ) {
            diag = diag.help(format!(
                "use named arguments like @{name}(param1=value1, param2=value2)"
            ));
            Self::report(self.ctx, diag);
        }
    }

    fn resolve_annotation(&self, name: &str) -> Option<DefId> {
        // TODO: This is a simplified lookup. In reality, we'd need to use the scope tree
        // to properly resolve the annotation name considering imports and scoping
        for (id, def) in &self.hir_ctx.definitions {
            if def.ident.name == name && matches!(def.kind, DefKind::Annotation(_)) {
                return Some(id);
            }
        }
        None
    }
}

impl<'a> Visitor<'a> for UnnamedArgs<'a> {
    fn visit_annotation(&mut self, ann: &'a Ann) {
        self.check_annotation(ann);
        ic_hir::visit::walk_annotation(self, ann);
    }
}
