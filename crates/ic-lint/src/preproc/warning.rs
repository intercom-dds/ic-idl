// Copyright 2026 KONGSBERG
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

use crate::{Category, Lint, LintCtx, SyntaxInput};

/// Lint for `#warning` directives.
pub struct PreprocWarning;

impl<'a> Lint<'a> for PreprocWarning {
    fn name() -> &'static str {
        "preproc-warning"
    }

    fn category() -> Category {
        Category::Preprocessor
    }

    fn description() -> &'static str {
        "#warning directives in preprocessor"
    }

    fn check_syntax(ctx: &'a LintCtx<'_>, input: &SyntaxInput<'_>) {
        for error in input.preproc_warnings {
            if let ic_preproc::Error::Note { span, tokens } = error {
                let msg = if tokens.is_empty() {
                    "#warning directive".to_string()
                } else {
                    let text = tokens
                        .iter()
                        .map(|t| ctx.slice(t.span))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("#warning directive: {text}")
                };

                if let Some(diag) = ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    msg,
                    Label::new(*span).message("here"),
                ) {
                    Self::report(ctx, diag);
                }
            }
        }
    }
}
