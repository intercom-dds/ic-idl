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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_diagnostic::{Label, warn_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefKind, Ty, TyKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that checks if map keys are primitive types. Produces a warning when
/// complex types are used.
pub struct ComplexMapKey<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for ComplexMapKey<'a> {
    fn name() -> &'static str {
        "complex_key"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Non-primitive types used as map keys"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ic_hir::ResolvedGraph) {
        let mut res = ComplexMapKey { ctx, hir };
        ic_hir::visit::walk_tree(&mut res, &hir.context.definitions);
    }
}

fn is_complex(ctx: &ic_hir::Context, ty: &Ty) -> bool {
    match &ty.kind {
        TyKind::Primitive(_) | TyKind::String { .. } => false,
        TyKind::Adt(id) => match &ctx.type_of(*id).kind {
            DefKind::Enum(_) | DefKind::Bitmask(_) => false,
            DefKind::Alias(v) => is_complex(ctx, &v.ty),
            _ => true,
        },
        _ => true,
    }
}

impl<'a> Visitor<'a> for ComplexMapKey<'a> {
    fn visit_ty(&mut self, ty: &'a Ty) {
        if let TyKind::Map { key, .. } = &ty.kind {
            if is_complex(&self.hir.context, key) {
                let diag = warn_span(
                    "complex types as map keys are not standard",
                    Label::new(key.span).message("non-primitive map key"),
                )
                .note("only integers, strings, and enums may be used as map keys");
                Self::report(self.ctx, diag);
            }
        }
    }
}
