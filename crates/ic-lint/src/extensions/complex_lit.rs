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

use ic_diagnostic::{Color, Diag, Label, warn_span};
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{ConstDef, Expr, Span, util};

use crate::{Category, Lint, LintCtx};

/// Warns when an initializer list is used, e.g. for complex constants or
/// complex default values.
pub struct ComplexLit<'a> {
    ctx: &'a LintCtx<'a>,
}

impl ComplexLit<'_> {
    fn diagnose(&mut self, (diag, msg): (Span, &str), (label_span, label): (Span, &str)) {
        let diag = Diag::warning("complex literals are non-standard")
            .label(Label::new(diag).message(msg).color(Color::Yellow))
            .label(Label::new(label_span).message(label).color(Color::Cyan))
            .note("only literals of trivial types are allowed in standard IDL");

        Self::report(self.ctx, diag);
    }
}

impl<'a> Visitor<'a> for ComplexLit<'a> {
    fn visit_annotation_appl(&mut self, def: &'a ic_syntax::AnnotationAppl) {
        for arg in &def.args {
            if let Expr::InitList(_) = &arg.value {
                self.diagnose(
                    (arg.value.span(), "complex default values are non-standard"),
                    (util::path_span(&def.ident), "in this annotation"),
                );
            }
        }
    }

    fn visit_const(&mut self, def: &'a ConstDef) {
        if let Expr::InitList(_) = &def.value {
            self.diagnose(
                (def.value.span(), "complex constants are non-standard"),
                (util::decl_span(&def.decl), "const defined here"),
            );
        }
    }

    // Fallback in case we ever end up with an initializer list in another
    // place.
    fn visit_expr(&mut self, expr: &'a ic_syntax::Expr) {
        if let ic_syntax::Expr::InitList(_) = expr {
            let diag = warn_span(
                "initializer lists are non-standard",
                Label::new(expr.span()),
            );
            Self::report(self.ctx, diag);
        }
    }
}

impl<'a> Lint<'a> for ComplexLit<'a> {
    fn name() -> &'static str {
        "complex-lit"
    }

    fn category() -> Category {
        Category::Extensions
    }

    fn description() -> &'static str {
        "Complex literals used in constants/annotations"
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[ic_syntax::Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, tree);
    }
}
