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

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZero;
use std::rc::Rc;

use ic_alloc::arena::{self, Arena};
use ic_macros::EnumIter;
use ic_syntax::util::{path_name, type_name};
use ic_syntax::{AnnotationDef, AnnotationField, Expr, Ident, Item, Span};

// mod annotation;
mod ctx;
pub mod fold;
pub mod hir;
pub mod keywords;
mod lower;
mod resolve;
pub mod visit;
pub use ctx::Context;
// mod downcast;
use hir::*;
use resolve::Resolver;

#[derive(Debug)]
pub struct ResolvedGraph {
    /// The primary data structure that owns all the types.
    pub context: Context,

    /// Defines the order in which the types appeared in the syntax tree. This
    /// can be used to traverse the graph in the same order in which the types
    /// were defined.
    pub order: Vec<TypeId>,
}

pub fn lower_ast<I>(ast: I) -> ResolvedGraph
where
    I: IntoIterator<Item = ic_syntax::Item>,
{
    let mut context = Context::new();
    tracing::info!("lowering AST -> HIR: {context:?}");
    let order = lower::from_ast(&mut context, ast);

    ResolvedGraph { context, order }
}

// pub fn resolve(tree: &[Item]) {
//     let mut visitor = Resolver::default();
//     ic_syntax::visit::visit_tree(&mut visitor, tree);
//     println!("{visitor:#?}");
// }
