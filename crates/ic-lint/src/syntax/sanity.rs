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

use ic_syntax::visit::{Visitor, walk_attribute, walk_struct_field, walk_tree, walk_union_variant};
use ic_syntax::{Declarator, Ident, Item};

use crate::{Category, Lint, LintCtx};

/// Collection of sanity checks that verifies the AST is correctly constructed.
/// This ensures that all types have names, all arrays have bounds, etc.
///
/// Unlike most lints, this one will panic. These checks should never fail for
/// any of the ASTs we've constructed, and if they do, it's best to flag those
/// as an internal error.
#[derive(Default)]
pub struct Sanity;

impl<'a> Lint<'a> for Sanity {
    fn name() -> &'static str {
        "sanity"
    }

    fn category() -> Category {
        Category::Syntax
    }

    fn description() -> &'static str {
        "Internal checks for AST structural validity"
    }

    fn check(_ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = Self;
        walk_tree(&mut lint, ast);
    }
}

impl<'a> Visitor<'a> for Sanity {
    fn visit_ident(&mut self, ident: &'a Ident) {
        assert!(!ident.name.is_empty());
    }

    fn visit_path(&mut self, path: &'a ic_syntax::Path) {
        assert!(!path.segments.is_empty());
    }

    fn visit_declarator(&mut self, decl: &'a ic_syntax::Declarator) {
        if let Declarator::Array(array) = decl {
            assert!(!array.bounds.is_empty());
        }
    }

    fn visit_struct_field(&mut self, def: &'a ic_syntax::Field) {
        assert!(!def.declarators.is_empty());
        walk_struct_field(self, def);
    }

    fn visit_union_variant(&mut self, variant: &'a ic_syntax::UnionCase) {
        assert!(!variant.labels.is_empty());
        walk_union_variant(self, variant);
    }

    fn visit_attribute(&mut self, def: &'a ic_syntax::Attribute) {
        assert!(!def.declarators.is_empty());
        walk_attribute(self, def);
    }
}
