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

use ic_diagnostic::{error_span, Diag, Label};
use ic_syntax::util::{item_name, ItemTraits};
use ic_syntax::visit::{visit_tree, Visitor};
use ic_syntax::{Ident, Item, Span};

use crate::{Category, Lint};

/// Verifies that enums, unions, bitmasks, exceptions and valuetypes have at
/// least one member.
///
/// Support for empty structs is allowed in the extended data-types building
/// block.
#[derive(Default)]
pub struct EmptyTypes(Vec<Diag>);

impl EmptyTypes {
    fn diagnose<T: ItemTraits>(&mut self, span: Span, def: &T, member: &str) {
        let ty = T::item_name();
        let note = format!("all {ty}s must have at least one {member}");
        let diag = error_span(
            format!("empty {ty}s are not allowed"),
            Label::new(span).message("defined here"),
        )
        .note(note);

        self.0.push(diag);
    }
}

impl<'a> Visitor<'a> for EmptyTypes {
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

    fn visit_valuetype(&mut self, def: &'a ic_syntax::ValuetypeDef) {
        if def.prototypes.is_empty() {
            self.diagnose(def.ident.span, def, "member or prototype");
        }
    }

    fn visit_bitmask(&mut self, def: &'a ic_syntax::BitmaskDef) {
        if def.bits.is_empty() {
            self.diagnose(def.ident.span, def, "flag");
        }
    }

    fn visit_exception(&mut self, def: &'a ic_syntax::ExceptDef) {
        if def.members.is_empty() {
            self.diagnose(def.ident.span, def, "member");
        }
    }
}

impl Lint for EmptyTypes {
    fn new() -> Box<dyn Lint>
    where
        Self: Sized,
    {
        Box::<Self>::default()
    }

    fn category(&self) -> crate::Category {
        Category::Syntax
    }

    fn check(mut self: Box<Self>, ast: &[Item]) -> Vec<Diag> {
        visit_tree(&mut *self, ast);
        self.0
    }
}
