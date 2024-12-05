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

#[cfg(test)]
mod tests;

use anyhow::Result;
use chumsky::prelude::*;
use chumsky::text::{Character, TextParser};
use chumsky::{Error, Parser, Stream};
use ic_alloc::ptr::P;

use crate::lexer::{Kind, Token};
use crate::syntax::{Definition, Ident, ItemKind, ModuleDef, Span};

// Workaround until trait aliases are stabilized
pub trait IdlParser<T>: chumsky::Parser<Kind, T, Error = Simple<Kind>> + Clone {}

// Blanket impl because we really just want an alias
impl<T, U: chumsky::Parser<Kind, T, Error = Simple<Kind>> + Clone> IdlParser<T> for U {}

#[must_use]
pub fn specification() -> impl IdlParser<Vec<Definition>> {
    just(Kind::Module)
        .ignore_then(just(Kind::Ident).map(|_| Definition {
            name: Ident::default(),
            span: Span::default(),
            annotations: vec![],
            kind: ItemKind::Module(P(ModuleDef { defs: vec![] })),
        }))
        // .delimited_by(just(Kind::LBrace), just(Kind::RBrace))
        .then_ignore(just(Kind::LBrace))
        .then_ignore(definition().repeated())
        .then_ignore(just(Kind::RBrace))
        .then_ignore(just(Kind::Semi))
        // .then_ignore(just([Kind::LBrace, Kind::RBrace, Kind::Semi]))
        .repeated()
        .then_ignore(end())
}

fn definition() -> impl IdlParser<()> {
    just([Kind::Struct, Kind::Ident, Kind::Semi]).ignored()
}
