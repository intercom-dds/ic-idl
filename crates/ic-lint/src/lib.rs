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

#![allow(dead_code, unused)]
#![allow(clippy::new_ret_no_self)]

use std::cell::RefCell;

use ic_diagnostic::Diag;
use ic_syntax::{Item, Span};
use ic_vfs::SourceMap;
use syntax::sanity;

// mod annotation;
mod pedantic;
// mod semantic;
mod syntax;
mod unsupported;
//

// macro_rules! lints {
//     ($($lint:ty),* $(,)?) => {
//         type LintFn = fn() -> Box<dyn Lint>;
//
//         const LINTS: &[LintFn] = &[
//             $(<$lint>::new,)*
//         ];
//     };
// }
//
// lints! {
//     // pedantic::assign_expr::AssignExpr,
//     // pedantic::complex_lit::ComplexLit,
//     // pedantic::empty_mod::EmptyMod,
//     pedantic::lowercase_bool::LowercaseBool,
//     // pedantic::null::NullVariant,
//     // pedantic::omitted_in::OmittedIn,
//     // semantic::oneway::NonVoidOneway,
//     // semantic::unsupported::Unsupported,
//     // syntax::ascii::AsciiIdent,
//     // syntax::empty::EmptyTypes,
// }

/// The supported lint categories.
#[derive(Copy, Clone, Debug)]
pub enum Category {
    /// Annotation-related lints
    Annotation,

    /// Deprecated language items
    Deprecated,

    /// Unsupported language items
    Unsupported,

    /// Lint for language extensions
    Pedantic,

    // Syntax errors or other semantic issues that should always be hard errors
    Syntax,
}

#[derive(Debug)]
pub struct LintCtx<'a> {
    vfs: &'a SourceMap,
    diagnostics: RefCell<Vec<Diag>>,
}

impl LintCtx<'_> {
    /// Emit a diagnostic.
    ///
    /// Diagnostics will be collected and emitted after all lints have been
    /// ran.
    pub fn report(&self, diag: Diag) {
        self.diagnostics.borrow_mut().push(diag);
    }

    /// Returns a slice of the given span.
    pub fn slice(&self, span: Span) -> &str {
        todo!()
    }
}

pub trait Lint<'a>: Sized {
    /// Category of the lint.
    fn category() -> Category;

    /// Runs the lint on the given AST.
    ///
    /// A lint should never fail in a way that prevents further traversal. Any
    /// potential errors should be gracefully ignored.
    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {}

    /// Runs the lint on the given HIR.
    ///
    /// A lint should never fail in a way that prevents further traversal. Any
    /// potential errors should be gracefully ignored.
    #[must_use]
    fn check_hir(context: &ic_hir::Context, graph: &[ic_hir::hir::Def]) -> Vec<Diag> {
        todo!()
    }
}

#[must_use]
#[derive(Debug)]
pub struct Report {
    pub diagnostics: Vec<Diag>,
}

/// Traverses the AST and produces diagnostics for all enabled lints.
///
/// Lints that operate on the AST are mostly syntactic. Other lints that
/// require more in-depth semantic analysis is typically done on the HIR with
/// [`lint_hir`].
pub fn lint_syntax(tree: &[Item]) -> Report {
    let vfs = SourceMap::default();
    let mut ctx = LintCtx {
        vfs: &vfs,
        diagnostics: RefCell::default(),
    };

    {
        sanity::Sanity::check(&ctx, tree);
        pedantic::lowercase_bool::LowercaseBool::check(&ctx, tree);
        syntax::ascii::AsciiIdent::check(&ctx, tree);
    }

    // for lint in LINTS {
    //     let pass = lint().check(&ctx, tree);
    //     diagnostics.extend(pass.into_iter());
    // }

    Report {
        diagnostics: ctx.diagnostics.take(),
    }
}

/// Set of lints that operates on the HIR.
pub fn lint_hir() {}
