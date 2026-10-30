// Copyright 2026 KONGSBERG

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, AnnParam, Def, DefKind, Ident, Member, UnionTy, Variant};
use ic_hir::visit::{self, Visitor};
use ic_hir_analysis::annotation::{builtin_annotation, enabled_bool_annotation};

use crate::{Category, Lint, LintCtx};

const INCOMPATIBLE_PAIRS: [(&str, &str); 5] = [
    ("optional", "external"),
    ("optional", "shared"),
    ("optional", "default"),
    ("nested", "topic"),
    ("non_serialized", "key"),
];

pub struct IncompatibleAnnotations<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for IncompatibleAnnotations<'a> {
    fn name() -> &'static str {
        "incompatible-annotations"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "rejects mutually exclusive built-in annotations"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut visitor = Self { ctx, hir };
        visit::walk_tree(&mut visitor, hir);
    }
}

impl IncompatibleAnnotations<'_> {
    fn active_annotation<'a>(&self, annotations: &'a [Ann], name: &str) -> Option<&'a Ann> {
        match name {
            "default" | "topic" => builtin_annotation(&self.hir.context, annotations, name),
            _ => enabled_bool_annotation(&self.hir.context, annotations, name),
        }
    }

    fn check_annotations(&self, annotations: &[Ann], ident: &Ident, item_type: &str) {
        for (first_name, second_name) in INCOMPATIBLE_PAIRS {
            let (Some(first), Some(second)) = (
                self.active_annotation(annotations, first_name),
                self.active_annotation(annotations, second_name),
            ) else {
                continue;
            };

            Self::report(
                self.ctx,
                error_span(
                    format!(
                        "{item_type} `{}` cannot be both @{first_name} and @{second_name}",
                        ident.name
                    ),
                    Label::new(ident.span)
                        .message(format!("incompatible annotations on {item_type}")),
                )
                .label(
                    Label::new(first.ident.span).message(format!("@{first_name} annotation here")),
                )
                .label(
                    Label::new(second.ident.span)
                        .message(format!("@{second_name} annotation here")),
                )
                .help(format!("remove either @{first_name} or @{second_name}")),
            );
        }
    }
}

impl<'a> Visitor<'a> for IncompatibleAnnotations<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        match &def.kind {
            DefKind::Alias(_) => {
                self.check_annotations(&def.annotations, &def.ident, "typedef");
            }
            DefKind::Struct(_) | DefKind::Union(_) => {
                self.check_annotations(&def.annotations, &def.ident, "type");
            }
            _ => {}
        }
        visit::walk_def(self, def);
    }

    fn visit_member(&mut self, member: &'a Member) {
        self.check_annotations(&member.annotations, &member.ident, "member");
        visit::walk_member(self, member);
    }

    fn visit_variant(&mut self, variant: &'a Variant) {
        self.check_annotations(&variant.annotations, &variant.ident, "union variant");
        visit::walk_variant(self, variant);
    }

    fn visit_ann_param(&mut self, param: &'a AnnParam) {
        self.check_annotations(&param.annotations, &param.ident, "annotation member");
        visit::walk_ann_param(self, param);
    }

    fn visit_union(&mut self, def: &'a Def, union: &'a UnionTy) {
        self.check_annotations(&union.disc.annotations, &def.ident, "union discriminator");
        visit::walk_union(self, union);
    }
}
