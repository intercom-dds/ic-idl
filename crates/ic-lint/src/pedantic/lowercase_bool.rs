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

use ic_cli::color::Colorize as _;
use ic_diagnostic::Label;
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{Item, Literal, LiteralValue};

use crate::{Category, Lint, LintCtx};

/// Lint that checks for uses of lowercase `true` or `false`, neither of which
/// are standard IDL. Only `TRUE` and `FALSE` are specified in the standard.
pub struct LowercaseBool<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for LowercaseBool<'a> {
    fn name() -> &'static str {
        "lowercase-bool"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Lowercase 'true' or 'false' used"
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, ast);
    }
}

impl<'a> Visitor<'a> for LowercaseBool<'a> {
    fn visit_literal(&mut self, num: &'a Literal) {
        if let LiteralValue::Bool(_lit) = num.value {
            let slice = self.ctx.slice(num.span);
            if slice.chars().any(char::is_lowercase) {
                let fixed = slice.to_uppercase().green();
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    "lowercase boolean literals are non-standard",
                    Label::new(num.span).message("lowercase boolean literal"),
                ) {
                    let diag = diag.help(format!("use `{fixed}` instead"));
                    Self::report(self.ctx, diag);
                }
            }
        }
    }
}
