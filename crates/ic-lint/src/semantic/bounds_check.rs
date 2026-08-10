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

//! Lint that checks that const types or default values are within bounds

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, Member, Numeric, StructTy, Ty, TyKind, UnionTy, Variant};
use ic_hir::visit::Visitor;
use ic_vfs::Span;

use crate::{Category, Lint, LintCtx};

pub struct BoundsCheck<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for BoundsCheck<'a> {
    fn name() -> &'static str {
        "bounds-check"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Detects when default or const values of bounded types are violated"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = BoundsCheck { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl BoundsCheck<'_> {
    fn check_struct_member(&self, member: &Member) {
        let Some(default_ann) = member
            .annotations
            .iter()
            .find(|a| a.ident.name == "default")
        else {
            return;
        };

        let Some(arg) = default_ann.args.first() else {
            return;
        };

        self.bounds_check(&arg.value, &member.ty, arg.ident.span);
    }

    fn check_union_varaint(&self, variant: &Variant) {
        let Some(default_ann) = variant
            .annotations
            .iter()
            .find(|a| a.ident.name == "default")
        else {
            return;
        };

        let Some(arg) = default_ann.args.first() else {
            return;
        };

        self.bounds_check(&arg.value, &variant.ty, arg.ident.span);
    }

    fn bounds_check(&self, value: &Numeric, ty: &Ty, span: Span) {
        if !self.is_in_bounds(value, ty) {
            let diag = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                format!("value violates bounds of type `{}`", self.type_name(ty)),
                Label::new(span).message("value violates type bound"),
            );
            Self::report(self.ctx, diag);
        }
    }

    fn is_in_bounds(&self, value: &Numeric, ty: &Ty) -> bool {
        let resolved_ty = self.hir.context.resolve_ty(ty);

        // Resolve const
        if let Numeric::Const(const_id) = value {
            let const_def = self.hir.context.type_of(*const_id);
            let DefKind::Const(const_ty) = &const_def.kind else {
                return false;
            };

            return self.is_in_bounds(&const_ty.value, ty);
        }

        match (&resolved_ty.kind, value) {
            (
                TyKind::String {
                    wide: false, bound, ..
                },
                Numeric::String(str),
            )
            | (
                TyKind::String {
                    wide: true, bound, ..
                },
                Numeric::WString(str),
            ) => bound.is_none_or(|b| b >= str.len()),

            (
                TyKind::Sequence {
                    ty: elem_ty, bound, ..
                },
                Numeric::Sequence { values, .. } | Numeric::Array { values, .. },
            ) => {
                values.iter().all(|v| self.is_in_bounds(v, elem_ty))
                    && bound.is_none_or(|b| b >= values.len())
            }
            (
                TyKind::Array {
                    ty: elem_ty, len, ..
                },
                Numeric::Sequence { values, .. } | Numeric::Array { values, .. },
            ) => values.iter().all(|v| self.is_in_bounds(v, elem_ty)) && *len >= values.len(),

            (
                TyKind::Map {
                    key, elem, bound, ..
                },
                Numeric::Map { entries, .. },
            ) => {
                entries
                    .iter()
                    .all(|(k, v)| self.is_in_bounds(k, key) && self.is_in_bounds(v, elem))
                    && bound.is_none_or(|b| b >= entries.len())
            }

            (
                TyKind::Map {
                    key, elem, bound, ..
                },
                Numeric::Sequence { values, .. },
            ) => values.iter().all(|entry| {
                if let Numeric::Sequence { values: pair, .. } = entry {
                    pair.len() == 2
                        && self.is_in_bounds(&pair[0], key)
                        && self.is_in_bounds(&pair[1], elem)
                        && bound.is_none_or(|b| b >= values.len())
                } else {
                    true
                }
            }),

            (TyKind::Adt(def_id), Numeric::Struct { fields, .. }) => {
                let def = self.hir.context.type_of(*def_id);

                match &def.kind {
                    DefKind::Struct(struct_ty) => {
                        for (member, value) in struct_ty.members.iter().zip(fields.iter()) {
                            if !self.is_in_bounds(value, &member.ty) {
                                return false;
                            }
                        }

                        true
                    }
                    _ => true,
                }
            }

            (
                TyKind::Adt(def_id),
                Numeric::Union {
                    value, field_index, ..
                },
            ) => {
                let def = self.hir.context.type_of(*def_id);

                match &def.kind {
                    DefKind::Union(union_ty) => union_ty
                        .variants
                        .get(*field_index)
                        .is_none_or(|variant| self.is_in_bounds(value, &variant.ty)),
                    _ => true,
                }
            }

            _ => true,
        }
    }

    pub fn type_name(&self, ty: &Ty) -> String {
        match ty.kind {
            TyKind::String {
                bound: Some(bound),
                wide: false,
                ..
            } => format!("string<{bound}>"),
            TyKind::String {
                bound: Some(bound),
                wide: true,
                ..
            } => format!("wstring<{bound}>"),
            TyKind::Sequence {
                bound: Some(bound),
                ref ty,
                ..
            } => format!("sequence<{}, {bound}>", self.type_name(ty)),
            TyKind::Array { len, ref ty, .. } => format!("{}[{}]", self.type_name(ty), len),
            TyKind::Map {
                ref key,
                ref elem,
                bound: Some(bound),
                ..
            } => format!(
                "map<{}, {}, {bound}>",
                self.type_name(key),
                self.type_name(elem)
            ),
            _ => self.hir.context.type_name(ty),
        }
    }
}

impl<'a> Visitor<'a> for BoundsCheck<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_const(&mut self, def: &'a Def, data: &'a ic_hir::hir::ConstTy) {
        self.bounds_check(&data.value, &data.ty, def.span);

        ic_hir::visit::walk_const(self, data);
    }

    fn visit_except(&mut self, _def: &'a Def, data: &'a ic_hir::hir::ExceptTy) {
        for member in &data.members {
            self.check_struct_member(member);
        }
        ic_hir::visit::walk_except(self, data);
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a StructTy) {
        for member in &data.members {
            self.check_struct_member(member);
        }
        ic_hir::visit::walk_struct(self, data);
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a UnionTy) {
        for variant in &data.variants {
            self.check_union_varaint(variant);
        }
        ic_hir::visit::walk_union(self, data);
    }
}
