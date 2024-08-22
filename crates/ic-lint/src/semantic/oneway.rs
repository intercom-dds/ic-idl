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

use ic_diagnostic::{error_span, warn_span, Color, Diag, Label};
use ic_syntax::visit::{visit_tree, Visitor};
use ic_syntax::{util, Item, ParamKind, Prototype, Type, UnionNull};

use crate::{Category, Lint};

/// Checks that all prototypes marked as `oneway` have:
///   1. A `void` return type.
///   2. No exception specifications.
///   3. No out/inout parameters.
#[derive(Default)]
pub struct NonVoidOneway(Vec<Diag>);

impl NonVoidOneway {
    fn chk_return_ty(&mut self, proto: &Prototype) {
        if util::type_name(&proto.ret) != "void" {
            let diag = Diag::error("oneway operations must have a `void` return type")
                .label(
                    Label::new(util::ty_span(&proto.ret))
                        .message("non-void return type")
                        .color(Color::Blue),
                )
                .label(
                    Label::new(proto.oneway.unwrap())
                        .message("declared as oneway here")
                        .color(Color::Red),
                )
                .help("change the return type to `void`");

            self.0.push(diag);
        }
    }

    fn chk_exception(&mut self, proto: &Prototype) {
        if let Some(raises) = proto.raises.first() {
            let diag = Diag::error("oneway operations cannot have exception specifiers")
                .label(
                    Label::new(util::path_span(raises))
                        .message("exception specifier")
                        .color(Color::Blue),
                )
                .label(
                    Label::new(proto.oneway.unwrap())
                        .message("declared as oneway here")
                        .color(Color::Red),
                );

            self.0.push(diag);
        }
    }

    fn chk_out_params(&mut self, proto: &Prototype) {
        for param in &proto.params {
            if let Some(ParamKind::ParamOut | ParamKind::ParamInout) = param.kind {
                let diag = Diag::error("oneway operations cannot have `out` parameters")
                    .label(
                        Label::new(param.ident.span)
                            .message("`out` parameter")
                            .color(Color::Blue),
                    )
                    .label(
                        Label::new(proto.oneway.unwrap())
                            .message("declared as oneway here")
                            .color(Color::Red),
                    );

                self.0.push(diag);
            }
        }
    }
}

impl<'a> Visitor<'a> for NonVoidOneway {
    fn visit_prototype(&mut self, def: &'a ic_syntax::Prototype) {
        if let Some(oneway) = def.oneway {
            self.chk_return_ty(def);
            self.chk_exception(def);
            self.chk_out_params(def);
        }
    }
}

impl Lint for NonVoidOneway {
    fn new() -> Box<dyn Lint>
    where
        Self: Sized,
    {
        Box::<Self>::default()
    }

    fn category(&self) -> Category {
        Category::Syntax
    }

    fn check(mut self: Box<Self>, ast: &[Item]) -> Vec<Diag> {
        visit_tree(&mut *self, ast);
        self.0
    }
}
