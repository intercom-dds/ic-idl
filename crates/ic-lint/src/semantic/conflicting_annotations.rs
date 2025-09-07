// Copyright 2025 KONGSBERG

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:

// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.

// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.

// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.

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

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Member, Variant};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Checks for conflicting annotations on struct and union members.
/// Specifically, @optional and @key are mutually exclusive.
pub struct ConflictingAnnotations<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for ConflictingAnnotations<'a> {
    fn name() -> &'static str {
        "conflicting-annotations"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "detects conflicting annotations"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = ConflictingAnnotations { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> ConflictingAnnotations<'a> {
    fn check_annotations(
        &self,
        annotations: &[ic_hir::hir::Ann],
        ident: &ic_hir::hir::Ident,
        item_type: &str,
    ) {
        let mut has_optional = None;
        let mut has_key = None;

        for ann in annotations {
            if ann.ident.name == "optional" {
                has_optional = Some(ann);
            } else if ann.ident.name == "key" {
                has_key = Some(ann);
            }
        }

        if let (Some(optional_ann), Some(key_ann)) = (has_optional, has_key) {
            Self::report(
                self.ctx,
                error_span(
                    format!(
                        "{} `{}` cannot be both @optional and @key",
                        item_type, ident.name
                    ),
                    Label::new(ident.span)
                        .message(format!("conflicting annotations on {}", item_type)),
                )
                .label(Label::new(optional_ann.ident.span).message("@optional annotation here"))
                .label(Label::new(key_ann.ident.span).message("@key annotation here"))
                .help("remove either @optional or @key"),
            );
        }
    }
}

impl<'a> Visitor<'a> for ConflictingAnnotations<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_member(&mut self, member: &'a Member) {
        self.check_annotations(&member.annotations, &member.ident, "struct member");
    }

    fn visit_variant(&mut self, variant: &'a Variant) {
        self.check_annotations(&variant.annotations, &variant.ident, "union variant");
    }
}
