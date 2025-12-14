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

use ic_diagnostic::{Label, warn_span};
use ic_syntax::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

pub struct BitmaskAnn<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for BitmaskAnn<'_> {
    fn visit_annotation_field(&mut self, def: &'a ic_syntax::AnnotationField) {
        if let ic_syntax::AnnotationField::Item(item) = def
            && let ic_syntax::Item::BitmaskValue(bitmask) = item.as_ref()
        {
            let diag = warn_span(
                "defining bitmasks in annotations is non-standard",
                Label::new(bitmask.ident.span).message("defined here"),
            );
            Self::report(self.ctx, diag);
        }
    }
}

impl<'a> Lint<'a> for BitmaskAnn<'_> {
    fn name() -> &'static str {
        "bitmask_ann"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Bitmasks defined inside annotations"
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[ic_syntax::Item]) {
        let mut lint = BitmaskAnn { ctx };
        walk_tree(&mut lint, tree);
    }
}
