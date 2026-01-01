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

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

//! High-level Intermediate Representation (HIR) for IDL compilation.
//!
//! This crate provides the core HIR data structures and utilities. The HIR is a
//! typed, resolved representation suitable for semantic analysis and code generation.
//!
//! # Architecture
//!
//! The HIR consists of:
//! - [`ResolvedGraph`] - The top-level result containing all definitions
//! - [`Context`] - Owns all type definitions and provides lookup
//! - [`hir`] module - Core type definitions (`Def`, `DefKind`, `Ty`, etc.)
//!
//! To create HIR from AST, use `ic-hir-lower`:
//!
//! ```ignore
//! use ic_hir_lower::{from_ast, AstInput};
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

mod ctx;
pub use crate::ctx::Context;

/// Diagnostic collection for HIR lowering.
pub mod diagnostics;

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

/// HIR rewriting utilities for replacing DefId references.
pub mod rewrite;

/// Scope resolution and name lookup utilities.
pub mod scope;

/// Type size calculations for fixed-size types.
pub mod type_size;

/// HIR validation utilities.
pub mod validate;

/// HIR visitor pattern for traversal and analysis.
pub mod visit;

/// The result of lowering AST to HIR.
///
/// This structure contains the fully resolved and type-checked HIR graph,
/// along with any diagnostics produced during the lowering process.
#[derive(Clone, Debug)]
pub struct ResolvedGraph {
    /// The primary data structure that owns all the types.
    pub context: Context,

    /// Defines the order in which the top-level types appeared in the syntax
    /// tree. This can be used to traverse the graph in the same order in which
    /// the types were defined.
    pub order: Vec<hir::DefId>,

    /// Defines the order in which built-in types were defined.
    /// Empty if no built-ins were loaded.
    pub builtin_order: Vec<hir::DefId>,

    /// Errors accumulated during type resolution, type checking, etc.
    pub errors: Vec<ic_diagnostic::Diag>,

    /// Warnings accumulated during type resolution, type checking, etc.
    pub warnings: Vec<ic_diagnostic::Diag>,
}

impl ResolvedGraph {
    /// Returns an iterator of the definitions, iterating over all top-level
    /// definitions in the order they were defined.
    #[must_use]
    pub fn iter(&self) -> DefIter<'_> {
        DefIter::new(self)
    }
}

impl<'a> IntoIterator for &'a ResolvedGraph {
    type Item = &'a hir::Def;

    type IntoIter = DefIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
