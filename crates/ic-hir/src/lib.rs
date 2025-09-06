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

//! High-level Intermediate Representation (HIR) for IDL compilation.

#![allow(clippy::cast_possible_truncation)] // We handle overflow appropriately
#![allow(clippy::cast_possible_wrap)] // We handle overflow appropriately
#![allow(clippy::cast_sign_loss)] // We handle sign conversion appropriately
#![allow(clippy::cast_precision_loss)] // Expected for float conversions
#![allow(clippy::cast_lossless)] // Explicit casts are clearer in this context
//!
//! This crate transforms the parse tree (AST) into a typed, resolved representation
//! suitable for semantic analysis and code generation. The HIR resolves names,
//! performs type checking, evaluates constant expressions, and validates IDL semantics.
//!
//! # Architecture
//!
//! The main entry point is [`from_ast`], which takes an AST and produces a [`ResolvedGraph`].
//! The graph contains:
//! - A [`Context`] with all type definitions and metadata
//! - The order in which types were defined
//! - Any errors or warnings encountered during lowering
//!
//! # Example
//!
//! ```ignore
//! use ic_hir::{from_ast, AstInput};
//!
//! let ast = parse_idl_file("example.idl")?;
//! let hir = from_ast(AstInput::User(ast));
//!
//! if !hir.errors.is_empty() {
//!     // Handle compilation errors
//! }
//!
//! // Use the HIR for code generation or analysis
//! for def in hir.iter() {
//!     println!("Found type: {}", def.ident.name);
//! }
//! ```

use std::fmt::Debug;

pub use crate::ctx::Context;

mod ctx;
mod lower;

/// Annotation processing and validation.
pub mod annotation;
/// HIR tree folding for transformations.
pub mod fold;
/// Core HIR type definitions and data structures.
pub mod hir;
/// IDL keywords and reserved identifiers.
pub mod keywords;
/// Merging multiple HIR graphs into a single graph.
pub mod merge;
/// Scope resolution and name lookup utilities.
pub mod scope;
/// Type size calculations for fixed-size types.
pub mod type_size;
/// HIR visitor pattern for traversal and analysis.
pub mod visit;

/// Input for HIR lowering, supporting both user-only and user+builtins scenarios.
pub enum AstInput<I> {
    /// Just user AST, no builtins
    User(I),

    /// User AST with builtin definitions
    WithBuiltins {
        /// Built-in definitions that will be marked with `IS_BUILTIN` flag
        builtins: I,

        /// User definitions
        user: I,

        /// If true, builtins are included in the output order.
        /// If false, they're available in context but not in the output.
        include_in_output: bool,
    },
}

/// The result of lowering AST to HIR.
///
/// This structure contains the fully resolved and type-checked HIR graph,
/// along with any diagnostics produced during the lowering process.
#[derive(Debug)]
pub struct ResolvedGraph {
    /// The primary data structure that owns all the types.
    pub context: Context,

    /// Defines the order in which the top-level types appeared in the syntax
    /// tree. This can be used to traverse the graph in the same order in which
    /// the types were defined.
    pub order: Vec<hir::TypeId>,

    /// Defines the order in which built-in types were defined.
    /// Empty if no built-ins were loaded.
    pub builtin_order: Vec<hir::TypeId>,

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

/// Convert AST to HIR with the specified input configuration.
pub fn from_ast<I>(input: AstInput<I>) -> ResolvedGraph
where
    I: IntoIterator<Item = ic_syntax::Item>,
{
    // For now, just skip builtins entirely
    let result = match input {
        AstInput::User(ast) => lower::lower(ast),
        AstInput::WithBuiltins {
            builtins: _, // Skip builtins
            user,
            include_in_output: _,
        } => {
            // Just process user items only
            lower::lower(user)
        }
    };

    ResolvedGraph {
        context: result.context,
        order: result.order,
        builtin_order: result.builtin_order,
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
