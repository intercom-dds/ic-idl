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

//! Lint that checks @default annotation values are compatible with member types.

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, Member, Numeric, PrimitiveTy, StructTy, Ty, TyKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct DefaultTypeMismatch<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for DefaultTypeMismatch<'a> {
    fn name() -> &'static str {
        "default-type-mismatch"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Detects when @default values are incompatible with member types"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DefaultTypeMismatch { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl DefaultTypeMismatch<'_> {
    fn check_member(&self, member: &Member) {
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

        if !self.is_compatible(&arg.value, &member.ty) {
            let diag = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                format!(
                    "@default value is not compatible with member type `{}`",
                    self.hir.context.type_name(&member.ty),
                ),
                Label::new(arg.ident.span).message("incompatible default value"),
            );
            Self::report(self.ctx, diag);
        }
    }

    fn is_compatible(&self, value: &Numeric, ty: &Ty) -> bool {
        match (&ty.kind, value) {
            (_, Numeric::Null) | (TyKind::String { .. }, Numeric::String(_)) => true,

            (TyKind::Primitive(prim), _) => Self::is_primitive_compatible(value, *prim),

            (
                TyKind::Sequence { ty: elem_ty, .. } | TyKind::Array { ty: elem_ty, .. },
                Numeric::Sequence { values, .. } | Numeric::Array { values, .. },
            ) => values.iter().all(|v| self.is_compatible(v, elem_ty)),

            (TyKind::Map { key, elem, .. }, Numeric::Map { entries, .. }) => entries
                .iter()
                .all(|(k, v)| self.is_compatible(k, key) && self.is_compatible(v, elem)),

            (TyKind::Map { key, elem, .. }, Numeric::Sequence { values, .. }) => {
                values.iter().all(|entry| {
                    if let Numeric::Sequence { values: pair, .. } = entry {
                        pair.len() >= 2
                            && self.is_compatible(&pair[0], key)
                            && self.is_compatible(&pair[1], elem)
                    } else {
                        false
                    }
                })
            }

            (TyKind::Adt(def_id), Numeric::Const(const_id)) => {
                let const_def = self.hir.context.type_of(*const_id);
                const_def
                    .parent
                    .is_some_and(|parent_id| parent_id == *def_id)
            }

            (TyKind::Adt(def_id), _) => {
                let def = self.hir.context.type_of(*def_id);
                match &def.kind {
                    DefKind::Struct(_) => matches!(value, Numeric::Struct { .. }),
                    DefKind::Enum(_) | DefKind::Bitmask(_) => {
                        Self::is_primitive_compatible(value, PrimitiveTy::Int32)
                    }
                    _ => false,
                }
            }

            _ => false,
        }
    }

    fn is_primitive_compatible(value: &Numeric, prim: PrimitiveTy) -> bool {
        matches!(
            (prim, value),
            (PrimitiveTy::Bool, Numeric::Bool(_))
                | (PrimitiveTy::Char | PrimitiveTy::WChar, Numeric::Char(_))
                | (
                    PrimitiveTy::Int8
                        | PrimitiveTy::UInt8
                        | PrimitiveTy::Int16
                        | PrimitiveTy::UInt16
                        | PrimitiveTy::Int32
                        | PrimitiveTy::UInt32
                        | PrimitiveTy::Int64
                        | PrimitiveTy::UInt64,
                    Numeric::Int8(_)
                        | Numeric::UInt8(_)
                        | Numeric::Int16(_)
                        | Numeric::UInt16(_)
                        | Numeric::Int32(_)
                        | Numeric::UInt32(_)
                        | Numeric::Int64(_)
                        | Numeric::UInt64(_),
                )
                | (
                    PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128,
                    Numeric::Float(_) | Numeric::Double(_),
                )
        )
    }
}

impl<'a> Visitor<'a> for DefaultTypeMismatch<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a StructTy) {
        for member in &data.members {
            self.check_member(member);
        }
        ic_hir::visit::walk_struct(self, data);
    }
}
