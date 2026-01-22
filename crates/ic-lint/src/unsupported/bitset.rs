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

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::DefKind;
use ic_hir::visit::{Visitor, walk_def, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Warns when `bitset`s are used.
pub struct Bitset<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for Bitset<'a> {
    fn name() -> &'static str {
        "bitset"
    }

    fn category() -> Category {
        Category::Unsupported
    }

    fn description() -> &'static str {
        "Checks for use of bitsets"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut lint = Self { ctx, hir };
        walk_tree(&mut lint, hir);
    }
}

impl<'a> Visitor<'a> for Bitset<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a ic_hir::hir::Def) {
        if let DefKind::Bitset(_) = &def.kind {
            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    "bitsets are not supported",
                    Label::new(def.ident.span).message("defined here"),
                )
                .note("the bitset will be skipped during codegen");
            Self::report(self.ctx, diag);
        } else {
            walk_def(self, def);
        }
    }
}
