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

#![allow(clippy::all, warnings)]

use std::fmt::Debug;

use ic_alloc::arena::{self, Arena};

pub use crate::ctx::Context;

mod ctx;
mod hygiene;
mod interp;
mod lower;
mod resolve;
mod typechk;

pub mod fold;
pub mod hir;
pub mod keywords;
pub mod scope;
pub mod visit;

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
}

impl ResolvedGraph {
    /// Returns an iterator of the definitions, iterating over all top-level
    /// definitions in the order they were defined.
    pub fn iter(&self) -> DefIter<'_> {
        DefIter::new(self)
    }
}

pub fn from_ast<I>(ast: I) -> ResolvedGraph
where
    I: IntoIterator<Item = ic_syntax::Item>,
{
    let result = lower::lower(ast);

    // Check for non-type name collisions, like struct members, etc.
    let mut errors = result.errors;
    hygiene::check(&result.context, &result.order, &mut errors);

    ResolvedGraph {
        context: result.context,
        order: result.order,
        errors,
    }
}

pub struct DefIter<'a> {
    ctx: &'a Context,
    iter: std::slice::Iter<'a, hir::DefId>,
}

impl<'a> DefIter<'a> {
    pub fn new(hir: &'a ResolvedGraph) -> Self {
        Self::with_order(&hir.context, &hir.order)
    }

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
