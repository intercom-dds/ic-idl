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

use std::fmt::Debug;

pub use crate::ctx::Context;

mod builtin;
mod ctx;
mod lower;

pub mod annotation;
pub mod fold;
pub mod hir;
pub mod keywords;
pub mod merge;
pub mod scope;
pub mod type_size;
pub mod visit;

/// Input for HIR lowering, supporting both user-only and user+builtins scenarios.
pub enum AstInput<U, B = std::iter::Empty<ic_syntax::Item>> {
    /// Just user AST, no builtins
    User(U),
    /// User AST with builtin definitions
    WithBuiltins {
        /// Built-in definitions that will be marked with IS_BUILTIN flag
        builtins: B,
        /// User definitions
        user: U,
        /// If true, builtins are included in the output order.
        /// If false, they're available in context but not in the output.
        include_in_output: bool,
    },
}

#[derive(Debug)]
pub struct ResolvedGraph {
    /// The primary data structure that owns all the types.
    pub context: Context,

    /// Defines the order in which the top-level types appeared in the syntax
    /// tree. This can be used to traverse the graph in the same order in which
    /// the types were defined.
    pub order: Vec<hir::TypeId>,

    /// Errors accumulated during type resolution, type checking, etc.
    pub errors: Vec<ic_diagnostic::Diag>,

    /// Warnings accumulated during type resolution, type checking, etc.
    pub warnings: Vec<ic_diagnostic::Diag>,
}

impl ResolvedGraph {
    /// Returns an iterator of the definitions, iterating over all top-level
    /// definitions in the order they were defined.
    #[must_use]
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> DefIter<'_> {
        DefIter::new(self)
    }
}

/// Lower AST to HIR with the specified input configuration.
pub fn lower<U, B>(input: AstInput<U, B>) -> ResolvedGraph
where
    U: IntoIterator<Item = ic_syntax::Item>,
    B: IntoIterator<Item = ic_syntax::Item>,
{
    let result = match input {
        AstInput::User(ast) => lower::lower(ast),
        AstInput::WithBuiltins {
            builtins,
            user,
            include_in_output,
        } => {
            if include_in_output {
                lower::lower_with_builtins(builtins, user)
            } else {
                lower::lower_with_builtin_context(builtins, user)
            }
        }
    };

    ResolvedGraph {
        context: result.context,
        order: result.order,
        errors: result.errors,
        warnings: result.warnings,
    }
}

pub struct DefIter<'a> {
    ctx: &'a Context,
    iter: std::slice::Iter<'a, hir::DefId>,
}

impl<'a> DefIter<'a> {
    #[must_use]
    pub fn new(hir: &'a ResolvedGraph) -> Self {
        Self::with_order(&hir.context, &hir.order)
    }

    #[must_use]
    pub fn with_order(ctx: &'a Context, order: &'a [hir::DefId]) -> Self {
        let iter = order.iter();
        Self { ctx, iter }
    }
}

impl<'a> Iterator for DefIter<'a> {
    type Item = &'a hir::Def;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| self.ctx.type_of(*v))
    }
}
