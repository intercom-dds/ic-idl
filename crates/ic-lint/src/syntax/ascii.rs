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

use ic_diagnostic::{Label, error_span};
use ic_syntax::Item;
use ic_syntax::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Verifies that all identifiers and character literals are made up of
/// alphanumeric ASCII characters,
pub struct AsciiIdent<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for AsciiIdent<'_> {
    fn visit_ident(&mut self, ident: &'a ic_syntax::Ident) {
        let invalid = ident
            .name
            .chars()
            .any(|v| !v.is_ascii_alphanumeric() && v != '_' && !v.is_whitespace());

        if invalid {
            let diag = error_span(
                "identifiers can only consist of alphanumeric ASCII characters",
                Label::new(ident.span).message("non-ASCII identifier"),
            );
            Self::report(self.ctx, diag);
        }
    }

    fn visit_literal(&mut self, num: &'a ic_syntax::Literal) {
        if let ic_syntax::LiteralValue::Char(c) = &num.value {
            if !c.is_ascii() {
                let diag = error_span(
                    "character literals can only consist of alphanumeric ASCII characters",
                    Label::new(num.span).message("non-ASCII character"),
                );
                Self::report(self.ctx, diag);
            }
        }
    }
}

impl<'a> Lint<'a> for AsciiIdent<'a> {
    fn name() -> &'static str {
        "ascii"
    }

    fn category() -> crate::Category {
        Category::Syntax
    }

    fn description() -> &'static str {
        "Errors when identifiers contain non-ASCII characters"
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = AsciiIdent { ctx };
        walk_tree(&mut lint, ast);
    }
}
