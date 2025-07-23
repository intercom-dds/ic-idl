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
use ic_hir::hir::{Ann, Def, DefKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Checks that annotations are applied to valid targets
pub struct InvalidAnnotationTarget<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for InvalidAnnotationTarget<'a> {
    fn name() -> &'static str {
        "invalid_annotation_target"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when annotations are used on invalid targets"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = InvalidAnnotationTarget { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl InvalidAnnotationTarget<'_> {
    fn check_annotations(&mut self, annotations: &[Ann], target: &str, def_kind: &DefKind) {
        for ann in annotations {
            match ann.ident.name.as_str() {
                // @key is only valid on struct fields
                "key" => {
                    if target != "field" {
                        self.report_invalid_target(
                            ann,
                            "@key can only be applied to struct fields",
                        );
                    }
                }
                // @optional is only valid on struct/union fields
                "optional" => {
                    if target != "field" {
                        self.report_invalid_target(ann, "@optional can only be applied to fields");
                    }
                }
                // @oneway is only valid on interface methods
                "oneway" => {
                    if target != "method" {
                        self.report_invalid_target(
                            ann,
                            "@oneway can only be applied to interface methods",
                        );
                    }
                }
                // @bit is only valid on bitmask flags
                "bit" => {
                    if target != "bitmask_flag" {
                        self.report_invalid_target(
                            ann,
                            "@bit can only be applied to bitmask flags",
                        );
                    }
                }
                // @range, @min, @max are only valid on numeric types/fields
                "range" | "min" | "max" => {
                    if !matches!(
                        def_kind,
                        DefKind::Struct(_) | DefKind::Const(_) | DefKind::Alias(_)
                    ) && target != "field"
                    {
                        self.report_invalid_target(
                            ann,
                            &format!(
                                "@{} can only be applied to numeric types or fields",
                                ann.ident.name
                            ),
                        );
                    }
                }
                // @id is typically used on fields or constants
                "id" => {
                    if target != "field" && !matches!(def_kind, DefKind::Const(_)) {
                        self.report_invalid_target(
                            ann,
                            "@id can only be applied to fields or constants",
                        );
                    }
                }
                // @annotation is only valid on annotation definitions
                "annotation" => {
                    if !matches!(def_kind, DefKind::Annotation(_)) {
                        self.report_invalid_target(
                            ann,
                            "@annotation can only be applied to annotation type definitions",
                        );
                    }
                }
                _ => {
                    // User-defined annotations typically have fewer restrictions
                }
            }
        }
    }

    fn report_invalid_target(&mut self, ann: &Ann, message: &str) {
        if let Some(diag) = self.ctx.diag_span(
            Self::name(),
            Self::category(),
            message,
            Label::new(ann.ident.span).message("invalid annotation target"),
        ) {
            Self::report(self.ctx, diag);
        }
    }
}

impl<'a> Visitor<'a> for InvalidAnnotationTarget<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        // Check annotations on the definition itself
        let target = match &def.kind {
            DefKind::Struct(_) => "struct",
            DefKind::Union(_) => "union",
            DefKind::Enum(_) => "enum",
            DefKind::Interface(_) => "interface",
            DefKind::Const(_) => "const",
            DefKind::Alias(_) => "typedef",
            DefKind::Annotation(_) => "annotation",
            DefKind::Bitmask(_) => "bitmask",
            DefKind::Bitset(_) => "bitset",
            DefKind::Valuetype(_) => "valuetype",
            DefKind::Except(_) => "exception",
            _ => "unknown",
        };

        self.check_annotations(&def.annotations, target, &def.kind);

        // Continue visiting
        ic_hir::visit::walk_def(self, def);
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a ic_hir::hir::StructTy) {
        // Check field annotations
        for member in &data.members {
            self.check_annotations(&member.annotations, "field", &DefKind::Struct(data.clone()));
        }
        ic_hir::visit::walk_struct(self, data);
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a ic_hir::hir::UnionTy) {
        // Check variant annotations
        for variant in &data.variants {
            self.check_annotations(
                &variant.annotations,
                "union_variant",
                &DefKind::Union(data.clone()),
            );
        }
        ic_hir::visit::walk_union(self, data);
    }

    fn visit_interface(&mut self, def: &'a Def, data: &'a ic_hir::hir::InterfaceTy) {
        // Note: interface methods are checked via definitions
        ic_hir::visit::walk_interface(self, def, data);
    }

    fn visit_bitmask(&mut self, _def: &'a Def, data: &'a ic_hir::hir::BitmaskTy) {
        // Check flag annotations
        for flag in &data.flags {
            self.check_annotations(
                &flag.annotations,
                "bitmask_flag",
                &DefKind::Bitmask(data.clone()),
            );
        }
        ic_hir::visit::walk_bitmask(self, data);
    }
}
