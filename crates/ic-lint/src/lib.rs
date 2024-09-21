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

use std::cell::RefCell;

use ic_diagnostic::Diag;
use ic_syntax::{Item, Span};
use ic_vfs::SourceMap;

// mod annotation;
mod pedantic;
mod semantic;
mod syntax;
mod unsupported;

mod iter;

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
    // TODO: spans can go across files, this has to be accounted for
    pub fn slice(&self, span: Span) -> &str {
        &self.vfs.source_str(span.start.file_id)[span.range()]
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
pub fn lint_syntax(tree: &[Item], vfs: &SourceMap) -> Report {
    let ctx = LintCtx {
        vfs,
        diagnostics: RefCell::default(),
    };

    {
        pedantic::array_param::ArrayParam::check(&ctx, tree);
        pedantic::assign_expr::AssignExpr::check(&ctx, tree);
        pedantic::bitmask_ann::BitmaskAnn::check(&ctx, tree);
        pedantic::complex_lit::ComplexLit::check(&ctx, tree);
        pedantic::empty_mod::EmptyMod::check(&ctx, tree);
        pedantic::lowercase_bool::LowercaseBool::check(&ctx, tree);
        pedantic::null::NullVariant::check(&ctx, tree);
        pedantic::omitted_in::OmittedIn::check(&ctx, tree);
        pedantic::scoped_lit::ScopedLit::check(&ctx, tree);
        semantic::oneway::NonVoidOneway::check(&ctx, tree);
        semantic::unsupported::Unsupported::check(&ctx, tree);
        syntax::ann_members::AnnMembers::check(&ctx, tree);
        syntax::ascii::AsciiIdent::check(&ctx, tree);
        syntax::empty::EmptyTypes::check(&ctx, tree);
        syntax::sanity::Sanity::check(&ctx, tree);
    }

    Report {
        diagnostics: ctx.diagnostics.take(),
    }
}

/// Set of lints that operates on the HIR.
pub fn lint_hir() {}
