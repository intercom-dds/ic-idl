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
use ic_syntax::AnnotationAppl;
use ic_syntax::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

pub struct DuplicateAnnotations<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for DuplicateAnnotations<'a> {
    fn name() -> &'static str {
        "DuplicateAnnotations"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[ic_syntax::Item]) {
        let mut visitor = DuplicateAnnotations { ctx };
        walk_tree(&mut visitor, ast);
    }
}

impl<'a> DuplicateAnnotations<'a> {
    fn check_annotation_list(&mut self, annotations: &[AnnotationAppl]) {
        let mut seen = HashSet::new();
        let conflicting_pairs = vec![("optional", "required"), ("readonly", "readwrite")];

        for ann in annotations {
            let ann_name = ann
                .ident
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            // Check for exact duplicates
            if !seen.insert(ann_name.clone()) {
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    &format!("duplicate annotation '@{}'", ann_name),
                    Label::new(ic_syntax::util::path_span(&ann.ident))
                        .message("duplicate annotation"),
                ) {
                    self.ctx.report(Self::name(), Self::category(), diag);
                }
            }

            // Check for conflicting annotations
            for (ann1, ann2) in &conflicting_pairs {
                if ann_name == *ann1 && seen.contains(*ann2) {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        &format!("conflicting annotations '@{}' and '@{}'", ann1, ann2),
                        Label::new(ic_syntax::util::path_span(&ann.ident))
                            .message("conflicts with previous annotation"),
                    ) {
                        self.ctx.report(Self::name(), Self::category(), diag);
                    }
                } else if ann_name == *ann2 && seen.contains(*ann1) {
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        &format!("conflicting annotations '@{}' and '@{}'", ann1, ann2),
                        Label::new(ic_syntax::util::path_span(&ann.ident))
                            .message("conflicts with previous annotation"),
                    ) {
                        self.ctx.report(Self::name(), Self::category(), diag);
                    }
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for DuplicateAnnotations<'a> {
    fn visit_struct(&mut self, def: &'a ic_syntax::StructDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_struct(self, def);
    }

    fn visit_union(&mut self, def: &'a ic_syntax::UnionDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_union(self, def);
    }

    fn visit_enum(&mut self, def: &'a ic_syntax::EnumDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_enum(self, def);
    }

    fn visit_interface(&mut self, def: &'a ic_syntax::InterfaceDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_interface(self, def);
    }

    fn visit_struct_field(&mut self, field: &'a ic_syntax::Field) {
        self.check_annotation_list(&field.annotations);
        ic_syntax::visit::walk_struct_field(self, field);
    }

    fn visit_const(&mut self, def: &'a ic_syntax::ConstDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_const(self, def);
    }

    fn visit_typedef(&mut self, def: &'a ic_syntax::AliasDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_typedef(self, def);
    }

    fn visit_exception(&mut self, def: &'a ic_syntax::ExceptDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_exception(self, def);
    }

    fn visit_bitmask(&mut self, def: &'a ic_syntax::BitmaskDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_bitmask(self, def);
    }

    fn visit_bitset(&mut self, def: &'a ic_syntax::BitsetDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_bitset(self, def);
    }

    fn visit_valuetype(&mut self, def: &'a ic_syntax::ValuetypeDef) {
        self.check_annotation_list(&def.annotations);
        ic_syntax::visit::walk_valuetype(self, def);
    }

    fn visit_discriminant(&mut self, disc: &'a ic_syntax::Discriminator) {
        self.check_annotation_list(&disc.annotations);
        ic_syntax::visit::walk_discriminant(self, disc);
    }

    fn visit_enum_variant(&mut self, e: &'a ic_syntax::Enumerator) {
        self.check_annotation_list(&e.annotations);
        ic_syntax::visit::walk_enum_variant(self, e);
    }
}
