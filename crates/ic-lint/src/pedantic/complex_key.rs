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
use ic_hir::hir::{DefKind, Ty};
use ic_hir::visit::Visitor;
use ic_vfs::Span;

use crate::{Category, Lint, LintCtx};

/// Lint that checks if map keys are primitive types. Produces a warning when
/// complex types are used.
pub struct ComplexMapKey<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for ComplexMapKey<'a> {
    fn category() -> Category {
        Category::Pedantic
    }
}

impl<'a> Visitor<'a> for ComplexMapKey<'a> {
    // TODO: span of Ty
    fn visit_ty(&mut self, ty: &'a Ty) {
        let ctx = ic_hir::Context::new();
        if let Ty::Map { key, .. } = ty {
            // TODO: base_type_of
            // let key_id = ctx.resolve_type(&ty);
            if let Ty::Adt(adt) = key.as_ref() {
                let diag = warn_span(
                    "complex types as map keys is not standard",
                    Label::new(Span::default()).message("non-primitive map key"),
                )
                .note("only integers, strings, and enums may be used as map keys");

                self.ctx.report(diag);
            }
        }
    }
}
