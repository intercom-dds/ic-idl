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

use ic_diagnostic::{Label, error_span};
use ic_syntax::util::ItemTraits;
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{Item, Span};

use crate::{Category, Lint, LintCtx};

/// Verifies that enums, unions, and bitmasks have at least one member.
///
/// Support for empty structs is allowed in the extended data-types building
/// block.
pub struct EmptyTypes<'a> {
    ctx: &'a LintCtx<'a>,
}

impl EmptyTypes<'_> {
    fn diagnose<T: ItemTraits>(&mut self, span: Span, _def: &T, member: &str) {
        let ty = T::item_name();
        let note = format!("all {ty}s must have at least one {member}");
        let diag = error_span(
            format!("empty {ty}s are not allowed"),
            Label::new(span).message("defined here"),
        )
        .note(note);

        Self::report(self.ctx, diag);
    }
}

impl<'a> Visitor<'a> for EmptyTypes<'a> {
    fn visit_enum(&mut self, def: &'a ic_syntax::EnumDef) {
        if def.fields.is_empty() {
            self.diagnose(def.ident.span, def, "enumerator");
        }
    }

    fn visit_union(&mut self, def: &'a ic_syntax::UnionDef) {
        if def.fields.is_empty() {
            self.diagnose(def.ident.span, def, "variant");
        }
    }

    fn visit_bitmask(&mut self, def: &'a ic_syntax::BitmaskDef) {
        if def.bits.is_empty() {
            self.diagnose(def.ident.span, def, "flag");
        }
    }

    fn visit_bitset(&mut self, def: &'a ic_syntax::BitsetDef) {
        if def.fields.is_empty() {
            self.diagnose(def.ident.span, def, "bitfield");
        }
    }
}

impl<'a> Lint<'a> for EmptyTypes<'a> {
    fn name() -> &'static str {
        "empty"
    }

    fn category() -> crate::Category {
        Category::Syntax
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, ast);
    }
}
