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
use ic_hir::hir::{
    Def, DefId, DefKind, EnumTy, Member, Numeric, PrimitiveTy, StructTy, Ty, TyKind,
};
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
        if let Numeric::Const(const_id) = value {
            return self.is_const_compatible(*const_id, ty);
        }

        let resolved_ty = self.hir.context.resolve_ty(ty);
        match (&resolved_ty.kind, value) {
            (_, Numeric::Null)
            | (TyKind::String { .. }, Numeric::String(_) | Numeric::WString(_)) => true,

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

            (TyKind::Adt(def_id), _) => {
                let def = self.hir.context.type_of(*def_id);
                match &def.kind {
                    DefKind::Struct(_) => matches!(value, Numeric::Struct { .. }),
                    DefKind::Enum(enum_ty) => self.is_valid_enum_value(value, enum_ty),
                    DefKind::Bitmask(bitmask_ty) => {
                        Self::is_primitive_compatible(value, bitmask_ty.ty)
                    }
                    _ => false,
                }
            }

            _ => false,
        }
    }

    fn is_const_compatible(&self, const_id: DefId, ty: &Ty) -> bool {
        let const_def = self.hir.context.type_of(const_id);
        let DefKind::Const(const_ty) = &const_def.kind else {
            return false;
        };
        let const_resolved = self.hir.context.resolve_ty(&const_ty.ty);
        let target_resolved = self.hir.context.resolve_ty(ty);
        Self::types_compatible(&const_resolved.kind, &target_resolved.kind)
    }

    fn types_compatible(a: &TyKind, b: &TyKind) -> bool {
        match (a, b) {
            (TyKind::String { .. }, TyKind::String { .. }) => true,
            (TyKind::Primitive(pa), TyKind::Primitive(pb)) => pa == pb,
            (TyKind::Adt(id_a), TyKind::Adt(id_b)) => id_a == id_b,
            _ => false,
        }
    }

    fn is_valid_enum_value(&self, value: &Numeric, enum_ty: &EnumTy) -> bool {
        if let Numeric::Const(const_id) = value {
            if enum_ty.fields.contains(const_id) {
                return true;
            }

            let const_def = self.hir.context.type_of(*const_id);
            if let DefKind::Const(const_ty) = &const_def.kind
                && let Numeric::Const(_) = &const_ty.value
            {
                return self.is_valid_enum_value(&const_ty.value, enum_ty);
            }
            return false;
        }

        if let Some(int_val) = Self::numeric_to_i64(value) {
            enum_ty.fields.iter().any(|field_id| {
                let field_def = self.hir.context.type_of(*field_id);
                if let DefKind::Const(const_ty) = &field_def.kind {
                    Self::numeric_to_i64(&const_ty.value) == Some(int_val)
                } else {
                    false
                }
            })
        } else {
            false
        }
    }

    fn numeric_to_i64(value: &Numeric) -> Option<i64> {
        match value {
            Numeric::Char(value) | Numeric::WChar(value) => Some(i64::from(u32::from(*value))),
            Numeric::Int8(v) => Some(i64::from(*v)),
            Numeric::UInt8(v) => Some(i64::from(*v)),
            Numeric::Int16(v) => Some(i64::from(*v)),
            Numeric::UInt16(v) => Some(i64::from(*v)),
            Numeric::Int32(v) => Some(i64::from(*v)),
            Numeric::UInt32(v) => Some(i64::from(*v)),
            Numeric::Int64(v) => Some(*v),
            Numeric::UInt64(v) => i64::try_from(*v).ok(),
            _ => None,
        }
    }

    fn is_primitive_compatible(value: &Numeric, prim: PrimitiveTy) -> bool {
        matches!(
            (prim, value),
            (PrimitiveTy::Bool, Numeric::Bool(_))
                | (
                    PrimitiveTy::Char | PrimitiveTy::WChar,
                    Numeric::Char(_)
                        | Numeric::WChar(_)
                        | Numeric::Int8(_)
                        | Numeric::UInt8(_)
                        | Numeric::Int16(_)
                        | Numeric::UInt16(_)
                        | Numeric::Int32(_)
                        | Numeric::UInt32(_)
                )
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
