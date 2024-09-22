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
use ic_syntax::Item;
use ic_syntax::util::ty_span;
use ic_syntax::visit::{Visitor, visit_tree};

use crate::{Category, Lint, LintCtx};

/// Warns when the `in` keyword is omitted in prototypes.
pub struct OmittedIn<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for OmittedIn<'a> {
    fn visit_prototype_param(&mut self, def: &'a ic_syntax::Param) {
        if def.kind.is_none() {
            let diag = warn_span(
                "parameters must be declared with `in`, `out`, or `inout`",
                Label::new(ty_span(&def.ty))
                    .message("expected parameter specifier before this type"),
            )
            .help("prefix the parameter with `in`");

            self.ctx.report(diag);
        }
    }
}

impl<'a> Lint<'a> for OmittedIn<'a> {
    fn category() -> Category {
        Category::Pedantic
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = Self { ctx };
        visit_tree(&mut lint, ast);
    }
}
