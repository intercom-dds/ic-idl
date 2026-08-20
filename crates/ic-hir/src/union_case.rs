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

use crate::Context;
use crate::hir::{DefKind, Label, Numeric, PrimitiveTy, Ty, TyKind, UnionTy, Variant};

#[derive(Clone, Copy, Debug)]
pub struct UnionCase<'a> {
    pub variant: &'a Variant,
    pub label: Option<&'a Label>,
}

#[must_use]
pub fn default_discriminator(ctx: &Context, union_ty: &UnionTy) -> Numeric {
    default_value(ctx, &union_ty.disc.ty).expect("union discriminator must have a default value")
}

fn default_value(ctx: &Context, ty: &Ty) -> Option<Numeric> {
    match ctx.resolve_ty(ty).kind {
        TyKind::Primitive(primitive) => primitive_value(primitive, 0),
        TyKind::Adt(def_id) => {
            let def = ctx.type_of(def_id);
            let DefKind::Enum(enum_ty) = &def.kind else {
                return None;
            };

            enum_ty
                .fields
                .iter()
                .copied()
                .find(|field_id| {
                    ctx.type_of(*field_id)
                        .annotations
                        .iter()
                        .any(|annotation| annotation.ident.name == "default_literal")
                })
                .or_else(|| enum_ty.fields.first().copied())
                .map(Numeric::Const)
        }
        _ => None,
    }
}

#[must_use]
pub fn union_case<'a>(
    ctx: &Context,
    union_ty: &'a UnionTy,
    discriminator: &Numeric,
) -> Option<UnionCase<'a>> {
    for variant in &union_ty.variants {
        for label in &variant.labels {
            if numerics_equal(ctx, &label.value, discriminator) {
                return Some(UnionCase {
                    variant,
                    label: Some(label),
                });
            }
        }
    }

    union_ty
        .variants
        .iter()
        .find(|variant| variant.is_default)
        .map(|variant| UnionCase {
            variant,
            label: None,
        })
}

#[must_use]
pub fn default_union_case<'a>(ctx: &Context, union_ty: &'a UnionTy) -> UnionCase<'a> {
    let discriminator = default_discriminator(ctx, union_ty);
    union_case(ctx, union_ty, &discriminator)
        .expect("union must have a case for its default discriminator")
}

#[must_use]
pub fn unused_discriminator(ctx: &Context, union_ty: &UnionTy) -> Option<Numeric> {
    match ctx.resolve_ty(&union_ty.disc.ty).kind {
        TyKind::Primitive(primitive) => unused_primitive(ctx, union_ty, primitive),
        TyKind::Adt(def_id) => {
            let def = ctx.type_of(def_id);
            let DefKind::Enum(enum_ty) = &def.kind else {
                return None;
            };

            enum_ty
                .fields
                .iter()
                .copied()
                .map(Numeric::Const)
                .find(|value| !is_used(ctx, union_ty, value))
        }
        _ => None,
    }
}

fn unused_primitive(ctx: &Context, union_ty: &UnionTy, primitive: PrimitiveTy) -> Option<Numeric> {
    let (min, max) = primitive_range(primitive)?;
    let mut value = 0;

    while value <= max {
        if let Some(candidate) = primitive_value(primitive, value)
            && !is_used(ctx, union_ty, &candidate)
        {
            return Some(candidate);
        }
        value += 1;
    }

    value = -1;
    while value >= min {
        if let Some(candidate) = primitive_value(primitive, value)
            && !is_used(ctx, union_ty, &candidate)
        {
            return Some(candidate);
        }
        value -= 1;
    }

    None
}

fn primitive_range(primitive: PrimitiveTy) -> Option<(i128, i128)> {
    Some(match primitive {
        PrimitiveTy::Bool => (0, 1),
        PrimitiveTy::Char | PrimitiveTy::UInt8 => (0, i128::from(u8::MAX)),
        PrimitiveTy::WChar | PrimitiveTy::UInt16 => (0, i128::from(u16::MAX)),
        PrimitiveTy::Int8 => (i128::from(i8::MIN), i128::from(i8::MAX)),
        PrimitiveTy::Int16 => (i128::from(i16::MIN), i128::from(i16::MAX)),
        PrimitiveTy::Int32 => (i128::from(i32::MIN), i128::from(i32::MAX)),
        PrimitiveTy::UInt32 => (0, i128::from(u32::MAX)),
        PrimitiveTy::Int64 => (i128::from(i64::MIN), i128::from(i64::MAX)),
        PrimitiveTy::UInt64 => (0, i128::from(u64::MAX)),
        _ => return None,
    })
}

fn primitive_value(primitive: PrimitiveTy, value: i128) -> Option<Numeric> {
    Some(match primitive {
        PrimitiveTy::Bool => Numeric::Bool(value != 0),
        PrimitiveTy::Char => Numeric::Char(char::from_u32(u32::try_from(value).ok()?)?),
        PrimitiveTy::WChar => Numeric::WChar(char::from_u32(u32::try_from(value).ok()?)?),
        PrimitiveTy::Int8 => Numeric::Int8(i8::try_from(value).ok()?),
        PrimitiveTy::UInt8 => Numeric::UInt8(u8::try_from(value).ok()?),
        PrimitiveTy::Int16 => Numeric::Int16(i16::try_from(value).ok()?),
        PrimitiveTy::UInt16 => Numeric::UInt16(u16::try_from(value).ok()?),
        PrimitiveTy::Int32 => Numeric::Int32(i32::try_from(value).ok()?),
        PrimitiveTy::UInt32 => Numeric::UInt32(u32::try_from(value).ok()?),
        PrimitiveTy::Int64 => Numeric::Int64(i64::try_from(value).ok()?),
        PrimitiveTy::UInt64 => Numeric::UInt64(u64::try_from(value).ok()?),
        _ => return None,
    })
}

fn is_used(ctx: &Context, union_ty: &UnionTy, candidate: &Numeric) -> bool {
    union_ty
        .variants
        .iter()
        .flat_map(|variant| &variant.labels)
        .any(|label| numerics_equal(ctx, &label.value, candidate))
}

fn numerics_equal(ctx: &Context, lhs: &Numeric, rhs: &Numeric) -> bool {
    matches!(
        (numeric_value(ctx, lhs), numeric_value(ctx, rhs)),
        (Some(lhs), Some(rhs)) if lhs == rhs
    )
}

fn numeric_value(ctx: &Context, numeric: &Numeric) -> Option<i128> {
    match numeric {
        Numeric::Bool(value) => Some(i128::from(*value)),
        Numeric::Char(value) | Numeric::WChar(value) => Some(i128::from(u32::from(*value))),
        Numeric::Int8(value) => Some(i128::from(*value)),
        Numeric::UInt8(value) => Some(i128::from(*value)),
        Numeric::Int16(value) => Some(i128::from(*value)),
        Numeric::UInt16(value) => Some(i128::from(*value)),
        Numeric::Int32(value) => Some(i128::from(*value)),
        Numeric::UInt32(value) => Some(i128::from(*value)),
        Numeric::Int64(value) => Some(i128::from(*value)),
        Numeric::UInt64(value) => Some(i128::from(*value)),
        Numeric::Const(def_id) => {
            let def = ctx.type_of(*def_id);
            let DefKind::Const(const_ty) = &def.kind else {
                return None;
            };

            numeric_value(ctx, &const_ty.value)
        }
        _ => None,
    }
}
