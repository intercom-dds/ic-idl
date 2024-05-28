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
use chumsky::{Error, Parser as _, Stream};

use crate::lexer::Token;
use crate::syntax::Definition;

// Workaround until trait aliases are stabilized
pub trait Parser<T>: chumsky::Parser<char, T, Error = Simple<char>> + Clone {}

// Blanket impl because we really just want an alias
impl<T, U: chumsky::Parser<char, T, Error = Simple<char>> + Clone> Parser<T> for U {}

/// Creates a parser that lazily constructs an AST as it gets fed tokens.
pub fn parse() -> Result<Vec<Definition>> {
    Ok(vec![])
}

/// Constructs an AST from the given tokens.
pub fn from_tokens(_tokens: &[Token]) -> Result<Vec<Definition>> {
    // let ast = parser::specification().parse(stream);
    Ok(vec![])
}

// fn specification() -> impl Parser<Vec<Definition>> {
//     todo!()
// }
