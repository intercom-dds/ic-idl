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

use ic_diagnostic::{warn_span, Color, Diag, Label};
use ic_syntax::visit::{visit_tree, Visitor};
use ic_syntax::{ConstDef, Expr, Item, Span, UnionNull};

use crate::{Category, Lint};

/// Warns when an initializer list is used, e.g. for complex constants or
/// complex default values.
#[derive(Default)]
pub struct ComplexDefaultValue(Vec<Diag>);

impl ComplexDefaultValue {
    fn diagnose(&mut self, (diag, msg): (Span, &str), (label_span, label): (Span, &str)) {
        let diag = Diag::warning("complex literals are an InterCOM extension")
            .label(Label::new(diag).message(msg).color(Color::Yellow))
            .label(Label::new(label_span).message(label).color(Color::Cyan))
            .note("only trivial literals are standardized");

        self.0.push(diag);
    }
}

impl<'a> Visitor<'a> for ComplexDefaultValue {
    fn visit_annotation_appl(&mut self, def: &'a ic_syntax::AnnotationAppl) {
        for arg in &def.args {
            if let Expr::InitList(_) = &arg.value {
                self.diagnose(
                    (
                        arg.value.span(),
                        "complex default values are an InterCOM extension",
                    ),
                    (def.span, "in this annotation"),
                );
            }
        }
    }

    fn visit_const(&mut self, def: &'a ConstDef) {
        if let Expr::InitList(_) = &def.value {
            self.diagnose(
                (
                    def.value.span(),
                    "complex constants are an InterCOM extension",
                ),
                (def.ident.span, "const defined here"),
            );
        }
    }

    // Fallback in case we ever end up with an initializer list in another
    // place.
    fn visit_expr(&mut self, expr: &'a ic_syntax::Expr) {
        if let ic_syntax::Expr::InitList(_) = expr {
            let diag = warn_span(
                "initializer lists are an InterCOM extension",
                Label::new(expr.span()),
            );
            self.0.push(diag);
        }
    }
}

impl Lint for ComplexDefaultValue {
    fn new() -> Box<dyn Lint>
    where
        Self: Sized,
    {
        Box::<Self>::default()
    }

    fn category(&self) -> Category {
        Category::Pedantic
    }

    fn check(mut self: Box<Self>, ast: &[Item]) -> Vec<Diag> {
        visit_tree(&mut *self, ast);
        self.0
    }
}
