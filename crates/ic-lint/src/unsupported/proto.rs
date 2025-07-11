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

use ic_diagnostic::{Diag, Label, error_span};
use ic_hir::hir;
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct Proto<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for Proto<'_> {
    fn visit_enum(&mut self, def: &'a hir::Def, ty: &'a hir::EnumTy) {
        if let Some(field) = ty.fields.first() {
            if field.value != 0 {
                let diag = error_span(
                    "the first enum value must be zero in proto3",
                    Label::new(field.ident.span)
                        .message(format!("this field has the value {}", field.value)),
                );
                self.ctx.report_error(diag);
            }
        }
    }
}

impl<'a> Lint<'a> for Proto<'_> {
    fn name() -> &'static str {
        "proto"
    }

    fn category() -> Category {
        Category::Unsupported
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ic_hir::ResolvedGraph) {
        let mut res = Proto { ctx };
        ic_hir::visit::walk_tree(&mut res, &hir.context.definitions);
    }
}
