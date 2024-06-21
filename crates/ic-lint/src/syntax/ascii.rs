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

use ic_diagnostic::{error_span, Diag, Label};
use ic_syntax::visit::{visit_tree, Visitor};
use ic_syntax::Item;

use crate::{Category, Lint};

/// Verifies that all identifiers are made up of alphanumeric ASCII characters,
/// and that all character literals only consist of ASCII characters.
#[derive(Default)]
pub struct AsciiIdent(Vec<Diag>);

impl<'a> Visitor<'a> for AsciiIdent {
    fn visit_ident(&mut self, ident: &'a ic_syntax::Ident) {
        let invalid = ident
            .name
            .chars()
            .any(|v| !v.is_ascii_alphanumeric() && v != '_');

        if invalid {
            let diag = error_span(
                "identifiers can only consist of alphanumeric ASCII characters",
                Label::new(ident.span).message("defined here"),
            );
            self.0.push(diag);
        }
    }

    fn visit_literal(&mut self, _num: &'a ic_syntax::Literal) {
        // match &num.kind {
        //     ic_syntax::LitKind::LitBool => todo!(),
        //     ic_syntax::LitKind::LitInt => todo!(),
        //     ic_syntax::LitKind::LitFloat => todo!(),
        //     ic_syntax::LitKind::LitChar => todo!(),
        //     ic_syntax::LitKind::LitString => todo!(),
        // }
    }
}

impl Lint for AsciiIdent {
    fn new() -> Box<dyn Lint>
    where
        Self: Sized,
    {
        Box::<Self>::default()
    }

    fn category(&self) -> crate::Category {
        Category::Syntax
    }

    fn check(mut self: Box<Self>, ast: &[Item]) -> Vec<Diag> {
        visit_tree(&mut *self, ast);
        self.0
    }
}
