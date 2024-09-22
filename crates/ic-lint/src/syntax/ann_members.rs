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
use ic_syntax::visit::{Visitor, visit_tree};
use ic_syntax::{AnnotationField, Item, util};

use crate::{Category, Lint, LintCtx};

/// Verifies that all definitions in annotations are either enums, consts,
/// aliases or bitmasks.
pub struct AnnMembers<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for AnnMembers<'_> {
    fn visit_annotation_field(&mut self, def: &'a AnnotationField) {
        if let AnnotationField::Item(item) = def {
            let span = util::item_span(item);

            match item.as_ref() {
                Item::ConstValue(_)
                | Item::AliasValue(_)
                | Item::BitmaskValue(_)
                | Item::EnumValue(_) => (),
                v => {
                    // Bitmasks are not standard and deliberately omitted from
                    // the message.
                    let name = util::item_variant_name(v);
                    let diag = error_span(
                        "only consts, typedefs and enums can be defined in annotations",
                        Label::new(span)
                            .message(format!("`{name}`s are not allowed in annotations")),
                    );
                    self.ctx.report(diag);
                }
            }
        }
    }
}

impl<'a> Lint<'a> for AnnMembers<'a> {
    fn category() -> crate::Category {
        Category::Syntax
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = AnnMembers { ctx };
        visit_tree(&mut lint, ast);
    }
}
