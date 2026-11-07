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
use ic_hir_analysis::annotation::{builtin_annotation, builtin_annotation_def};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoundEdge {
    Lower,
    Upper,
}

const BOUND_EDGES: [(&str, BoundEdge); 4] = [
    ("min", BoundEdge::Lower),
    ("exclusive_minimum", BoundEdge::Lower),
    ("max", BoundEdge::Upper),
    ("exclusive_maximum", BoundEdge::Upper),
];

fn bound_edge(name: &str) -> Option<BoundEdge> {
    BOUND_EDGES
        .iter()
        .find_map(|&(edge_name, edge)| (edge_name == name).then_some(edge))
}

#[derive(Default)]
struct BoundAnnotations<'a> {
    min: Option<&'a Ann>,
    max: Option<&'a Ann>,
    exclusive_minimum: Option<&'a Ann>,
    exclusive_maximum: Option<&'a Ann>,
    range: Option<&'a Ann>,
    collisions: Vec<(&'a Ann, &'a Ann, BoundEdge)>,
}

impl<'a> BoundAnnotations<'a> {
    fn has_any(&self) -> bool {
        self.min.is_some()
            || self.max.is_some()
            || self.exclusive_minimum.is_some()
            || self.exclusive_maximum.is_some()
            || self.range.is_some()
    }

    fn collect(ctx: &ic_hir::Context, annotations: &'a [Ann]) -> Self {
        let mut result = Self {
            min: builtin_annotation(ctx, annotations, "min"),
            max: builtin_annotation(ctx, annotations, "max"),
            exclusive_minimum: builtin_annotation(ctx, annotations, "exclusive_minimum"),
            exclusive_maximum: builtin_annotation(ctx, annotations, "exclusive_maximum"),
            range: builtin_annotation(ctx, annotations, "range"),
            collisions: Vec::new(),
        };

        let mut lower: Option<&'a Ann> = None;
        let mut upper: Option<&'a Ann> = None;
        for annotation in annotations {
            let Some(def) = builtin_annotation_def(ctx, annotation) else {
                continue;
            };
            let Some(edge) = bound_edge(def.ident.name.as_str()) else {
                continue;
            };
            let (occupied, collisions) = match edge {
                BoundEdge::Lower => (&mut lower, &mut result.collisions),
                BoundEdge::Upper => (&mut upper, &mut result.collisions),
            };
            if let Some(previous) = occupied
                && builtin_annotation_def(ctx, previous)
                    .is_some_and(|previous_def| previous_def.ident.name != def.ident.name)
            {
                collisions.push((previous, annotation, edge));
            }
            *occupied = Some(annotation);
        }

        result
    }
}

impl DuplicateBounds<'_> {
    fn check_bounds(&self, annotations: &[Ann], ty: &ic_hir::hir::Ty, location: &str) {
        let local = BoundAnnotations::collect(&self.hir.context, annotations);

        for &(first, second, edge) in &local.collisions {
            self.report_same_edge_conflict(first, second, edge, location);
        }

        if local.range.is_some()
            && (local.min.is_some()
                || local.exclusive_minimum.is_some()
                || local.max.is_some()
                || local.exclusive_maximum.is_some())
        {
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

            let typedef_bounds = BoundAnnotations::collect(&self.hir.context, &def.annotations);

            for local_ann in [local.min, local.exclusive_minimum].into_iter().flatten() {
                for typedef_ann in [typedef_bounds.min, typedef_bounds.exclusive_minimum]
                    .into_iter()
                    .flatten()
                {
                    self.report_edge_pair(
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                        "lower",
                    );
                }
                if let Some(typedef_ann) = typedef_bounds.range {
                    self.report_edge_pair(
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                        "lower",
                    );
                }
            }

            for local_ann in [local.max, local.exclusive_maximum].into_iter().flatten() {
                for typedef_ann in [typedef_bounds.max, typedef_bounds.exclusive_maximum]
                    .into_iter()
                    .flatten()
                {
                    self.report_edge_pair(
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                        "upper",
                    );
                }
                if let Some(typedef_ann) = typedef_bounds.range {
                    self.report_edge_pair(
                        local_ann,
                        typedef_ann,
                        &def.ident.name,
                        location,
                        "upper",
                    );
                }
            }

            if let Some(local_ann) = local.range {
                for typedef_ann in [
                    typedef_bounds.range,
                    typedef_bounds.min,
                    typedef_bounds.exclusive_minimum,
                    typedef_bounds.max,
                    typedef_bounds.exclusive_maximum,
                ]
                .into_iter()
                .flatten()
                {
                    self.report_edge_pair(local_ann, typedef_ann, &def.ident.name, location, "");
                }
            }

            match alias.ty.kind {
                TyKind::Adt(next_id) => def_id = next_id,
                _ => break,
            }
        }
    }

    fn report_same_edge_conflict(
        &self,
        first: &Ann,
        second: &Ann,
        edge: BoundEdge,
        location: &str,
    ) {
        let first_name = format!("@{}", first.ident.name);
        let second_name = format!("@{}", second.ident.name);
        let edge_word = match edge {
            BoundEdge::Lower => "lower",
            BoundEdge::Upper => "upper",
        };

        Self::report(
            self.ctx,
            error_span(
                format!("{first_name} conflicts with {second_name} on the {edge_word} bound"),
                Label::new(first.ident.span).message(format!("{first_name} on {location}")),
            )
            .label(
                Label::new(second.ident.span).message(format!("{second_name} also on {location}")),
            )
            .help(format!(
                "a {location} carries one {edge_word}-bound annotation"
            )),
        );
    }

    fn report_edge_pair(
        &self,
        local: &Ann,
        typedef: &Ann,
        typedef_name: &str,
        location: &str,
        edge_word: &str,
    ) {
        let local_name = format!("@{}", local.ident.name);
        let typedef_label = format!("@{}", typedef.ident.name);
        let ctx = &self.hir.context;
        let local_def = builtin_annotation_def(ctx, local);
        let typedef_def = builtin_annotation_def(ctx, typedef);

        if local_def.is_some_and(|local_def| {
            typedef_def.is_some_and(|typedef_def| typedef_def.ident.name == local_def.ident.name)
        }) {
            self.report_duplicate(&local_name, local, typedef, typedef_name, location);
        } else if local_def.is_some_and(|def| def.ident.name == "range")
            || typedef_def.is_some_and(|def| def.ident.name == "range")
        {
            self.report_conflict_with_range(
                &local_name,
                local,
                typedef,
                typedef_name,
                location,
                &typedef_label,
            );
        } else {
            Self::report(
                self.ctx,
                error_span(
                    format!(
                        "{local_name} conflicts with {typedef_label} on the {edge_word} bound of \
                         typedef `{typedef_name}`"
                    ),
                    Label::new(local.ident.span).message(format!("{local_name} on {location}")),
                )
                .label(
                    Label::new(typedef.ident.span)
                        .message(format!("{typedef_label} on `{typedef_name}`")),
                )
                .help(format!(
                    "remove {local_name} or {typedef_label}; the {edge_word} bound is occupied"
                )),
            );
        }
    }

    fn report_range_with_min_max(&self, bounds: &BoundAnnotations<'_>) {
        let range_ann = bounds.range.unwrap();
        let other_ann = bounds
            .min
            .or(bounds.exclusive_minimum)
            .or(bounds.max)
            .or(bounds.exclusive_maximum)
            .unwrap();
        let other_name = format!("@{}", other_ann.ident.name);

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
        typedef_label: &str,
    ) {
        let (first, second) = if local_name == "@range" {
            let second = match typedef_label {
                "@min" | "@max" => "@min or @max".to_string(),
                other => other.to_string(),
            };
            ("@range".to_string(), second)
        } else {
            (local_name.to_string(), "@range".to_string())
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
