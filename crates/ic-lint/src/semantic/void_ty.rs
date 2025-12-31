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
use ic_hir::hir::{self, PrimitiveTy, TyKind};
use ic_hir::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Lint that checks for invalid uses of the `void` type.
/// The `void` type is only valid as a return type in function prototypes.
pub struct VoidTy<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Visitor<'a> for VoidTy<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_ty(&mut self, ty: &'a hir::Ty) {
        if let TyKind::Primitive(PrimitiveTy::Void) = ty.kind
            && let Some(diag) = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                "`void` is only allowed as a return type in prototypes",
                Label::new(ty.span).message("invalid use of `void`"),
            )
        {
            Self::report(self.ctx, diag);
        }

        // Continue visiting nested types
        ic_hir::visit::walk_ty(self, ty);
    }

    fn visit_proto(&mut self, proto: &'a hir::ProtoTy) {
        // Skip the return type since `void` is allowed there
        // TOOD: check exceptions as well
        for param in &proto.params {
            self.visit_parameter(param);
        }
    }
}

impl<'a> Lint<'a> for VoidTy<'a> {
    fn name() -> &'static str {
        "void-ty"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when `void` is used outside function prototypes"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &'a ic_hir::ResolvedGraph) {
        let mut lint = Self { ctx, hir };
        walk_tree(&mut lint, hir);
    }
}
