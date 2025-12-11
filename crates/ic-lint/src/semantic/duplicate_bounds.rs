// Copyright 2025 KONGSBERG
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

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, DefKind, Member, TyKind, Variant};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct DuplicateBounds<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for DuplicateBounds<'a> {
    fn name() -> &'static str {
        "duplicate-bounds"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "detects duplicate @min, @max, or @range annotations across typedef chains"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateBounds { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

#[derive(Default)]
struct BoundAnnotations<'a> {
    min: Option<&'a Ann>,
    max: Option<&'a Ann>,
    range: Option<&'a Ann>,
}

impl<'a> BoundAnnotations<'a> {
    fn has_any(&self) -> bool {
        self.min.is_some() || self.max.is_some() || self.range.is_some()
    }

    fn collect(annotations: &'a [Ann]) -> Self {
        let mut result = Self::default();
        for ann in annotations {
            match ann.ident.name.as_str() {
                "min" => result.min = Some(ann),
                "max" => result.max = Some(ann),
                "range" => result.range = Some(ann),
                _ => {}
            }
        }
        result
    }
}

impl DuplicateBounds<'_> {
    fn check_bounds(&self, annotations: &[Ann], ty: &ic_hir::hir::Ty, location: &str) {
        let local = BoundAnnotations::collect(annotations);

        if local.range.is_some() && (local.min.is_some() || local.max.is_some()) {
            self.report_range_with_min_max(&local);
            return;
        }

        if !local.has_any() {
            return;
        }

        if let TyKind::Adt(def_id) = ty.kind {
            self.check_typedef_chain(def_id, &local, location);
        }
    }

    fn check_typedef_chain(
        &self,
        mut def_id: ic_hir::hir::DefId,
        local: &BoundAnnotations<'_>,
        location: &str,
    ) {
        loop {
            let def = self.hir.context.type_of(def_id);

            let DefKind::Alias(alias) = &def.kind else {
                break;
            };

            let typedef_bounds = BoundAnnotations::collect(&def.annotations);

            if let Some(local_ann) = local.min {
                if let Some(typedef_ann) = typedef_bounds.min {
                    self.report_duplicate(
                        "@min",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
                if let Some(typedef_ann) = typedef_bounds.range {
                    self.report_conflict_with_range(
                        "@min",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
            }

            if let Some(local_ann) = local.max {
                if let Some(typedef_ann) = typedef_bounds.max {
                    self.report_duplicate(
                        "@max",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
                if let Some(typedef_ann) = typedef_bounds.range {
                    self.report_conflict_with_range(
                        "@max",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
            }

            if let Some(local_ann) = local.range {
                if let Some(typedef_ann) = typedef_bounds.range {
                    self.report_duplicate(
                        "@range",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
                if let Some(typedef_ann) = typedef_bounds.min {
                    self.report_conflict_with_range(
                        "@range",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
                if let Some(typedef_ann) = typedef_bounds.max {
                    self.report_conflict_with_range(
                        "@range",
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                    );
                }
            }

            match alias.ty.kind {
                TyKind::Adt(next_id) => def_id = next_id,
                _ => break,
            }
        }
    }

    fn report_range_with_min_max(&self, bounds: &BoundAnnotations<'_>) {
        let range_ann = bounds.range.unwrap();
        let other_ann = bounds.min.or(bounds.max).unwrap();
        let other_name = if bounds.min.is_some() { "@min" } else { "@max" };

        Self::report(
            self.ctx,
            error_span(
                format!("@range cannot be used together with {other_name}"),
                Label::new(range_ann.ident.span).message("@range annotation here"),
            )
            .label(
                Label::new(other_ann.ident.span).message(format!("{other_name} annotation here")),
            )
            .help("use either @range or @min/@max, not both"),
        );
    }

    fn report_duplicate(
        &self,
        name: &str,
        local: &Ann,
        typedef: &Ann,
        typedef_name: &str,
        location: &str,
    ) {
        let help = if location == "typedef" {
            format!("remove {name} from one of the typedefs")
        } else {
            format!("remove {name} from either the {location} or the typedef")
        };

        Self::report(
            self.ctx,
            error_span(
                format!("{name} is already defined on typedef `{typedef_name}`"),
                Label::new(local.ident.span).message(format!("{name} on {location}")),
            )
            .label(
                Label::new(typedef.ident.span)
                    .message(format!("{name} already defined on `{typedef_name}`")),
            )
            .help(help),
        );
    }

    fn report_conflict_with_range(
        &self,
        local_name: &str,
        local: &Ann,
        typedef: &Ann,
        typedef_name: &str,
        location: &str,
    ) {
        let (first, second) = if local_name == "@range" {
            ("@range", "@min or @max")
        } else {
            (local_name, "@range")
        };

        Self::report(
            self.ctx,
            error_span(
                format!("{first} conflicts with {second} on typedef `{typedef_name}`"),
                Label::new(local.ident.span).message(format!("{first} on {location}")),
            )
            .label(
                Label::new(typedef.ident.span)
                    .message(format!("conflicting annotation on `{typedef_name}`")),
            )
            .help("use either @range or @min/@max, not both"),
        );
    }
}

impl<'a> Visitor<'a> for DuplicateBounds<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_member(&mut self, member: &'a Member) {
        self.check_bounds(&member.annotations, &member.ty, "member");
    }

    fn visit_variant(&mut self, variant: &'a Variant) {
        self.check_bounds(&variant.annotations, &variant.ty, "variant");
    }

    fn visit_def(&mut self, def: &'a ic_hir::hir::Def) {
        if let DefKind::Alias(alias) = &def.kind {
            self.check_bounds(&def.annotations, &alias.ty, "typedef");
        }
        ic_hir::visit::walk_def(self, def);
    }
}
