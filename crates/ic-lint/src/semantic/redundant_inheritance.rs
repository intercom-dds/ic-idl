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

use std::collections::HashSet;

use ic_diagnostic::Label;
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{InterfaceDef, ValuetypeDef};

use crate::{Category, Lint, LintCtx};

pub struct RedundantInheritance<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for RedundantInheritance<'a> {
    fn name() -> &'static str {
        "RedundantInheritance"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[ic_syntax::Item]) {
        let mut visitor = RedundantInheritance { ctx };
        walk_tree(&mut visitor, ast);
    }
}

impl RedundantInheritance<'_> {
    fn check_inheritance_list(&mut self, inherits: &[ic_syntax::Path], item_name: &str) {
        let mut seen = HashSet::new();

        for parent_path in inherits {
            let parent_name = parent_path
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            if !seen.insert(parent_name.clone()) {
                // This parent was already seen
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    format!("{item_name} inherits from '{parent_name}' multiple times"),
                    Label::new(ic_syntax::util::path_span(parent_path))
                        .message("redundant inheritance"),
                ) {
                    self.ctx.report(Self::name(), Self::category(), diag);
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for RedundantInheritance<'a> {
    fn visit_interface(&mut self, def: &'a InterfaceDef) {
        self.check_inheritance_list(&def.inherits, &format!("interface '{}'", def.ident.name));
        ic_syntax::visit::walk_interface(self, def);
    }

    fn visit_valuetype(&mut self, def: &'a ValuetypeDef) {
        // Check both inherits and supports for valuetypes
        let mut all_parents = Vec::new();
        if let Some(inherits) = &def.inherits {
            all_parents.push(inherits.clone());
        }
        if let Some(supports) = &def.supports {
            all_parents.push(supports.clone());
        }

        self.check_inheritance_list(&all_parents, &format!("valuetype '{}'", def.ident.name));
        ic_syntax::visit::walk_valuetype(self, def);
    }
}
