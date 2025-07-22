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
use ic_syntax::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Warns when unsupported language items are used.
pub struct Unsupported<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for Unsupported<'a> {
    fn visit_bitset(&mut self, bitset: &'a ic_syntax::BitsetDef) {
        let diag = warn_span(
            "bitsets are not supported",
            Label::new(bitset.ident.span).message("defined here"),
        )
        .note("the bitset will be skipped during codegen");

        Self::report(self.ctx, diag);
    }

    fn visit_path(&mut self, path: &'a ic_syntax::Path) {
        if path.segments.len() == 1 {
            let ty = &path.segments[0];
            if ty.name == "long double" {
                let diag = warn_span(
                    "long double is not supported",
                    Label::new(ty.span).message("used here"),
                )
                .note("long double will be treated as a normal double during codegen");

                Self::report(self.ctx, diag);
            }
        }
    }
}

impl<'a> Lint<'a> for Unsupported<'a> {
    fn name() -> &'static str {
        "items"
    }

    fn category() -> Category {
        Category::Unsupported
    }

    fn description() -> &'static str {
        "Warns when unsupported IDL features are used"
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, ast);
    }
}
