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

use std::collections::HashMap;

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, Member, StructTy, UnionTy};
use ic_hir::visit::Visitor;
use ic_hir_analysis::member_id::member_ids;
use ic_vfs::Span;

use crate::{Category, Lint, LintCtx};

const MAX_MEMBER_ID: u32 = 0x0fff_ffff;

pub struct MemberId<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for MemberId<'a> {
    fn name() -> &'static str {
        "member-id"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Validates effective member IDs"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &'a ResolvedGraph) {
        let mut visitor = Self { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl MemberId<'_> {
    fn struct_members<'a>(&'a self, struct_ty: &'a StructTy, members: &mut Vec<&'a Member>) {
        if let Some(parent) = struct_ty.parent
            && let DefKind::Struct(parent_ty) = &self.hir.context.type_of(parent.def_id).kind
        {
            self.struct_members(parent_ty, members);
        }

        members.extend(&struct_ty.members);
    }

    fn check_ids<'a>(&self, owner: &Def, members: impl Iterator<Item = (&'a str, Span, u32)>) {
        let mut seen = HashMap::new();

        for (name, span, id) in members {
            if id > MAX_MEMBER_ID {
                Self::report(
                    self.ctx,
                    error_span(
                        format!(
                            "member `{name}` has ID {id}, which exceeds maximum {MAX_MEMBER_ID:#X}"
                        ),
                        Label::new(span).message("member ID out of range"),
                    ),
                );
            }

            if let Some(first_span) = seen.get(&id) {
                Self::report(
                    self.ctx,
                    error_span(
                        format!("duplicate member ID {id} in `{}`", owner.ident.name),
                        Label::new(span).message("duplicate member ID"),
                    )
                    .label(Label::new(*first_span).message("first used here")),
                );
            } else {
                seen.insert(id, span);
            }
        }
    }
}

impl<'a> Visitor<'a> for MemberId<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a Def, struct_ty: &'a StructTy) {
        let ids = member_ids(&self.hir.context, def.id);
        let mut members = Vec::new();
        self.struct_members(struct_ty, &mut members);

        self.check_ids(
            def,
            members
                .into_iter()
                .zip(ids)
                .map(|(member, id)| (member.ident.name.as_str(), member.ident.span, id)),
        );
        ic_hir::visit::walk_struct(self, struct_ty);
    }

    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        let ids = member_ids(&self.hir.context, def.id);

        let discriminator = std::iter::once(("discriminator", union_ty.disc.ty.span));
        let members = discriminator.chain(
            union_ty
                .variants
                .iter()
                .map(|variant| (variant.ident.name.as_str(), variant.ident.span)),
        );

        self.check_ids(
            def,
            members.zip(ids).map(|((name, span), id)| (name, span, id)),
        );
        ic_hir::visit::walk_union(self, union_ty);
    }
}
