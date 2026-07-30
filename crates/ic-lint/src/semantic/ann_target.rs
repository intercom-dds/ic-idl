// Copyright 2026 KONGSBERG
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
use ic_hir::hir::{
    Ann, AnnotationTy, Attribute, BitsetTy, Decl, Def, DefFlags, DefKind, ExceptTy, Member,
    ProtoTy, StructTy, UnionTy, ValueTy,
};
use ic_hir::visit::{self, Visitor};

use crate::{Category, Lint, LintCtx};

type Target = (u64, &'static str);

const ANNOTATION_DEF: Target = (1 << 0, "annotations");
const ANNOTATION_MEMBER: Target = (1 << 1, "annotation members");
const MODULE_DEF: Target = (1 << 2, "modules");
const STRUCT_DEF: Target = (1 << 3, "structs");
const STRUCT_MEMBER: Target = (1 << 4, "struct members");
const UNION_DEF: Target = (1 << 5, "unions");
const UNION_DISC: Target = (1 << 6, "union discriminators");
const UNION_MEMBER: Target = (1 << 7, "union members");
const ENUM_DEF: Target = (1 << 8, "enums");
const ENUMERATOR: Target = (1 << 9, "enumerators");
const BITMASK_DEF: Target = (1 << 10, "bitmasks");
const BITMASK_FLAG: Target = (1 << 11, "bitmask flags");
const BITSET_DEF: Target = (1 << 12, "bitsets");
const BITSET_MEMBER: Target = (1 << 13, "bitfields");
const TYPE_ALIAS: Target = (1 << 14, "typedefs");
const CONST_DEF: Target = (1 << 15, "constants");
const EXCEPTION_DEF: Target = (1 << 16, "exceptions");
const EXCEPTION_MEMBER: Target = (1 << 17, "exception members");
const INTERFACE_DEF: Target = (1 << 18, "interfaces");
const PROTOTYPE: Target = (1 << 19, "prototypes");
const ATTRIBUTE_DEF: Target = (1 << 20, "attributes");
const VALUETYPE_DEF: Target = (1 << 21, "valuetypes");
const VALUETYPE_MEMBER: Target = (1 << 22, "valuetype members");
const NATIVE_DEF: Target = (1 << 23, "native definitions");

const ALL_TARGETS: [Target; 24] = [
    ANNOTATION_DEF,
    ANNOTATION_MEMBER,
    MODULE_DEF,
    STRUCT_DEF,
    STRUCT_MEMBER,
    UNION_DEF,
    UNION_DISC,
    UNION_MEMBER,
    ENUM_DEF,
    ENUMERATOR,
    BITMASK_DEF,
    BITMASK_FLAG,
    BITSET_DEF,
    BITSET_MEMBER,
    TYPE_ALIAS,
    CONST_DEF,
    EXCEPTION_DEF,
    EXCEPTION_MEMBER,
    INTERFACE_DEF,
    PROTOTYPE,
    ATTRIBUTE_DEF,
    VALUETYPE_DEF,
    VALUETYPE_MEMBER,
    NATIVE_DEF,
];

pub struct AnnotationTarget<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl AnnotationTarget<'_> {
    fn check_annotations(&self, annotations: &[Ann], target: Target) {
        for annotation in annotations {
            let Some(allowed) = self.allowed_targets(annotation) else {
                continue;
            };

            let allowed_targets = ALL_TARGETS
                .iter()
                .copied()
                .filter(|candidate| allowed & candidate.0 != 0)
                .collect::<Vec<_>>();

            if allowed_targets.contains(&target) {
                continue;
            }

            let message = format!(
                "`@{}` cannot be applied to {}",
                annotation.ident.name, target.1,
            );

            let mut descriptions = allowed_targets
                .into_iter()
                .map(|candidate| candidate.1)
                .collect::<Vec<_>>();

            let help = match descriptions.pop() {
                None => "annotation cannot be applied to any target".to_string(),
                Some(last) if descriptions.is_empty() => {
                    format!("the annotationt  can only be applied to {last}")
                }
                Some(last) => format!(
                    "the annotation only can be applied to {} or {last}",
                    descriptions.join(", ")
                ),
            };

            let diag = self
                .ctx
                .diag_span(
                    Self::name(),
                    Self::category(),
                    message,
                    Label::new(annotation.ident.span).message("invalid annotation target"),
                )
                .help(help);

            Self::report(self.ctx, diag);
        }
    }

    fn allowed_targets(&self, annotation: &Ann) -> Option<u64> {
        let annotation_def = self.hir.context.definitions.get(annotation.def_id?);
        let target = annotation_def.annotations.iter().find(|meta| {
            meta.def_id.is_some_and(|def_id| {
                let def = self.hir.context.type_of(def_id);
                def.flags.contains(DefFlags::IS_BUILTIN) && def.ident.name == "annotation_target"
            })
        })?;

        target
            .args
            .first()
            .map(|v| self.hir.context.unsigned_value(&v.value))
    }

    fn check_members(&self, members: &[Member], target: Target) {
        for member in members {
            self.check_annotations(&member.annotations, target);
        }
    }
}

impl<'a> Lint<'a> for AnnotationTarget<'a> {
    fn name() -> &'static str {
        "annotation-target"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Annotations applied to targets disallowed by their definition"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut visitor = Self { ctx, hir };
        visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for AnnotationTarget<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        let target = match &def.kind {
            DefKind::Annotation(_) => Some(ANNOTATION_DEF),
            DefKind::Module(_) => Some(MODULE_DEF),
            DefKind::Struct(_) => Some(STRUCT_DEF),
            DefKind::Except(_) => Some(EXCEPTION_DEF),
            DefKind::Union(_) => Some(UNION_DEF),
            DefKind::Enum(_) => Some(ENUM_DEF),
            DefKind::Const(_) => match def.parent.map(|id| &self.hir.context.type_of(id).kind) {
                Some(DefKind::Enum(_)) => Some(ENUMERATOR),
                Some(DefKind::Bitmask(_)) => Some(BITMASK_FLAG),
                _ => Some(CONST_DEF),
            },
            DefKind::Bitmask(_) => Some(BITMASK_DEF),
            DefKind::Bitset(_) => Some(BITSET_DEF),
            DefKind::Alias(_) => Some(TYPE_ALIAS),
            DefKind::Interface(_) => Some(INTERFACE_DEF),
            DefKind::Valuetype(_) => Some(VALUETYPE_DEF),
            DefKind::Decl(Decl::Native) => Some(NATIVE_DEF),
            DefKind::Decl(_) => None,
        };

        if let Some(target) = target {
            self.check_annotations(&def.annotations, target);
        }
        visit::walk_def(self, def);
    }

    fn visit_annotation_def(&mut self, def: &'a Def, data: &'a AnnotationTy) {
        for member in &data.params {
            self.check_annotations(&member.annotations, ANNOTATION_MEMBER);
        }
        visit::walk_annotation_def(self, def, data);
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a StructTy) {
        self.check_members(&data.members, STRUCT_MEMBER);
        visit::walk_struct(self, data);
    }

    fn visit_except(&mut self, _def: &'a Def, data: &'a ExceptTy) {
        self.check_members(&data.members, EXCEPTION_MEMBER);
        visit::walk_except(self, data);
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a UnionTy) {
        self.check_annotations(&data.disc.annotations, UNION_DISC);
        for member in &data.variants {
            self.check_annotations(&member.annotations, UNION_MEMBER);
        }
        visit::walk_union(self, data);
    }

    fn visit_bitset(&mut self, _def: &'a Def, data: &'a BitsetTy) {
        for member in &data.fields {
            self.check_annotations(&member.annotations, BITSET_MEMBER);
        }
        visit::walk_bitset(self, data);
    }

    fn visit_proto(&mut self, proto: &'a ProtoTy) {
        self.check_annotations(&proto.annotations, PROTOTYPE);
        visit::walk_proto(self, proto);
    }

    fn visit_attribute(&mut self, attribute: &'a Attribute) {
        self.check_annotations(&attribute.annotations, ATTRIBUTE_DEF);
        visit::walk_attribute(self, attribute);
    }

    fn visit_valuetype(&mut self, def: &'a Def, data: &'a ValueTy) {
        self.check_members(&data.members, VALUETYPE_MEMBER);
        visit::walk_valuetype(self, def, data);
    }
}
