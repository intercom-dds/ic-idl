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
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    Ann, AnnParam, Attribute, BitsetTy, Def, DefId, Member, ProtoTy, UnionTy, Variant,
};
use ic_hir::visit::{self, Visitor};
use ic_hir_analysis::annotation::{builtin_annotation_def, is_builtin_annotation};

use crate::{Category, Lint, LintCtx};

/// HIR-based duplicate annotations lint that properly handles annotation resolution
pub struct DuplicateAnnotations<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum AnnotationKey {
    Definition(DefId),
    Extensibility,
    External,
    MemberId,
}

impl<'a> Lint<'a> for DuplicateAnnotations<'a> {
    fn name() -> &'static str {
        "duplicate-ann"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Detects duplicate built-in annotations on the same item"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateAnnotations { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl DuplicateAnnotations<'_> {
    fn check_annotation_list(&self, annotations: &[Ann]) {
        let mut seen = HashSet::new();

        for ann in annotations {
            let Some(def) = builtin_annotation_def(&self.hir.context, ann) else {
                continue;
            };
            if ["doc", "verbatim", "derive"]
                .iter()
                .any(|name| is_builtin_annotation(&self.hir.context, ann, name))
            {
                continue;
            }

            let key = if ["final", "mutable", "appendable", "extensibility"]
                .iter()
                .any(|name| is_builtin_annotation(&self.hir.context, ann, name))
            {
                AnnotationKey::Extensibility
            } else if ["external", "shared"]
                .iter()
                .any(|name| is_builtin_annotation(&self.hir.context, ann, name))
            {
                AnnotationKey::External
            } else if ["id", "hashid"]
                .iter()
                .any(|name| is_builtin_annotation(&self.hir.context, ann, name))
            {
                AnnotationKey::MemberId
            } else {
                AnnotationKey::Definition(def.id)
            };

            if seen.insert(key) {
                continue;
            }

            let message = match key {
                AnnotationKey::Extensibility => "multiple extensibility annotations".to_string(),
                AnnotationKey::External => "multiple external annotations".to_string(),
                AnnotationKey::MemberId => "multiple member ID annotations".to_string(),
                AnnotationKey::Definition(_) => {
                    format!("duplicate annotation '@{}'", ann.ident.name)
                }
            };
            let diag = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                message,
                Label::new(ann.ident.span).message("duplicate annotation"),
            );
            Self::report(self.ctx, diag);
        }
    }
}

impl<'a> Visitor<'a> for DuplicateAnnotations<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        self.check_annotation_list(&def.annotations);
        visit::walk_def(self, def);
    }

    fn visit_member(&mut self, member: &'a Member) {
        self.check_annotation_list(&member.annotations);
        visit::walk_member(self, member);
    }

    fn visit_ann_param(&mut self, param: &'a AnnParam) {
        self.check_annotation_list(&param.annotations);
        visit::walk_ann_param(self, param);
    }

    fn visit_variant(&mut self, variant: &'a Variant) {
        self.check_annotation_list(&variant.annotations);
        visit::walk_variant(self, variant);
    }

    fn visit_proto(&mut self, proto: &'a ProtoTy) {
        self.check_annotation_list(&proto.annotations);
        visit::walk_proto(self, proto);
    }

    fn visit_attribute(&mut self, attribute: &'a Attribute) {
        self.check_annotation_list(&attribute.annotations);
        visit::walk_attribute(self, attribute);
    }

    fn visit_union(&mut self, _def: &'a Def, union: &'a UnionTy) {
        self.check_annotation_list(&union.disc.annotations);
        visit::walk_union(self, union);
    }

    fn visit_bitset(&mut self, _def: &'a Def, bitset: &'a BitsetTy) {
        for field in &bitset.fields {
            self.check_annotation_list(&field.annotations);
        }
        visit::walk_bitset(self, bitset);
    }
}
