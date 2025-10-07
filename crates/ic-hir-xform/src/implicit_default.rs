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

#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

//! Add implicit default case with null member for incomplete unions
//!
//! This transformation adds a default case with a null member to unions that
//! don't cover all possible discriminator values.

use std::collections::{HashMap, HashSet};

use ic_hir::fold::Fold;
use ic_hir::hir::{
    DefId, DefKind, EnumTy, Ident, Label, Numeric, PrimitiveTy, Span, Ty, TyKind, UnionTy, Variant,
};
use ic_hir::{Context, ResolvedGraph};

struct ImplicitDefault {
    enum_values: HashMap<DefId, Vec<(i64, Numeric)>>,
    const_values: HashMap<DefId, i64>,
}

impl ImplicitDefault {
    fn new(context: &Context) -> Self {
        let mut enum_values = HashMap::new();
        let mut const_values = HashMap::new();

        // Pre-compute all enum values and const values
        for (def_id, def) in &context.definitions {
            match &def.kind {
                DefKind::Enum(enum_ty) => {
                    let mut values = Vec::new();
                    let mut next_value = 0i32;

                    for &field_id in &enum_ty.fields {
                        let const_def = context.type_of(field_id);
                        if let DefKind::Const(const_ty) = &const_def.kind {
                            if let Some(v) = Self::numeric_to_i64(&const_ty.value) {
                                values.push((v, const_ty.value.clone()));
                                const_values.insert(field_id, v);
                                next_value = v.wrapping_add(1) as i32;
                            } else {
                                values.push((i64::from(next_value), Numeric::Int32(next_value)));
                                const_values.insert(field_id, i64::from(next_value));
                                next_value = next_value.wrapping_add(1);
                            }
                        }
                    }

                    enum_values.insert(def_id, values);
                }
                DefKind::Const(const_ty) => {
                    if let Some(v) = Self::numeric_to_i64(&const_ty.value) {
                        const_values.insert(def_id, v);
                    }
                }
                _ => {}
            }
        }

        Self {
            enum_values,
            const_values,
        }
    }

    /// Check if a union covers all possible values of its discriminator
    fn is_exhaustive(&self, union_ty: &UnionTy, disc_ty: &TyKind) -> bool {
        // If there's already a default case, it's exhaustive
        if union_ty.variants.iter().any(|v| v.is_default) {
            return true;
        }

        // Count total case labels
        let label_count: usize = union_ty.variants.iter().map(|v| v.labels.len()).sum();

        // Check based on discriminator type
        match disc_ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => label_count >= 2,
                PrimitiveTy::UInt8 | PrimitiveTy::Int8 | PrimitiveTy::Char => label_count >= 256,
                _ => false, // For larger types, assume not exhaustive
            },
            TyKind::Adt(def_id) => {
                // Check if this is an enum ADT
                if let Some(values) = self.enum_values.get(def_id) {
                    label_count >= values.len()
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Convert Numeric to i64 for comparison
    fn numeric_to_i64(numeric: &Numeric) -> Option<i64> {
        match numeric {
            Numeric::Bool(b) => Some(i64::from(*b)),
            Numeric::Char(c) => Some(*c as i64),
            Numeric::Int8(v) => Some(i64::from(*v)),
            Numeric::Octet(v) => Some(i64::from(*v)),
            Numeric::Int16(v) => Some(i64::from(*v)),
            Numeric::UInt16(v) => Some(i64::from(*v)),
            Numeric::Int32(v) => Some(i64::from(*v)),
            Numeric::UInt32(v) => Some(i64::from(*v)),
            Numeric::Int64(v) => Some(*v),
            Numeric::UInt64(v) => {
                if i64::try_from(*v).is_ok() {
                    Some(*v as i64)
                } else {
                    None
                }
            }
            Numeric::Null
            | Numeric::Float(_)
            | Numeric::Double(_)
            | Numeric::String(_)
            | Numeric::Const(_)
            | Numeric::Array { .. }
            | Numeric::Sequence { .. }
            | Numeric::Map { .. }
            | Numeric::Struct { .. }
            | Numeric::Union { .. } => None,
        }
    }

    /// Resolve Numeric to i64, handling Const references
    fn resolve_numeric_to_i64(&self, numeric: &Numeric) -> Option<i64> {
        match numeric {
            Numeric::Const(def_id) => self.const_values.get(def_id).copied(),
            _ => Self::numeric_to_i64(numeric),
        }
    }

    /// Find the first available value for the default case
    fn find_first_available_value(&self, union_ty: &UnionTy, disc_ty: &TyKind) -> Option<Numeric> {
        // Collect all used values
        let mut used_values = HashSet::new();
        for variant in &union_ty.variants {
            for label in &variant.labels {
                if let Some(value) = self.resolve_numeric_to_i64(&label.value) {
                    used_values.insert(value);
                }
            }
        }

        // Find first available value based on discriminator type
        match disc_ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => {
                    if !used_values.contains(&0) {
                        Some(Numeric::Bool(false))
                    } else if !used_values.contains(&1) {
                        Some(Numeric::Bool(true))
                    } else {
                        None
                    }
                }
                PrimitiveTy::UInt8 => (0u8..=255)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(Numeric::Octet),
                PrimitiveTy::Char => (0u8..=255)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(|v| Numeric::Char(v as char)),
                PrimitiveTy::Int8 => (-128i8..=127)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(Numeric::Int8),
                PrimitiveTy::UInt16 => (0u16..=1000)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(Numeric::UInt16),
                PrimitiveTy::Int16 => (0i16..=1000)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(Numeric::Int16),
                PrimitiveTy::UInt32 => (0u32..=1000)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(Numeric::UInt32),
                PrimitiveTy::Int32 => (0i32..=1000)
                    .find(|v| !used_values.contains(&i64::from(*v)))
                    .map(Numeric::Int32),
                PrimitiveTy::UInt64 => (0u64..=1000)
                    .find(|v| !used_values.contains(&(*v as i64)))
                    .map(Numeric::UInt64),
                PrimitiveTy::Int64 => (0i64..=1000)
                    .find(|v| !used_values.contains(v))
                    .map(Numeric::Int64),
                _ => None,
            },
            TyKind::Adt(def_id) => {
                // Check if this is an enum ADT
                if let Some(values) = self.enum_values.get(def_id) {
                    values
                        .iter()
                        .find(|(val, _)| !used_values.contains(val))
                        .map(|(_, numeric)| numeric.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Fold for ImplicitDefault {
    fn fold_union_ty(&mut self, mut u: UnionTy) -> UnionTy {
        // First, fold the inner types
        u = ic_hir::fold::fold_union_ty(self, u);

        // Check if this union needs an implicit default
        if !self.is_exhaustive(&u, &u.disc.ty.kind) {
            // Find the first available value
            if let Some(value) = self.find_first_available_value(&u, &u.disc.ty.kind) {
                // Use the discriminator's span as a base
                let disc_span = u.disc.ty.span;

                // Create a new variant with null type
                let null_variant = Variant {
                    annotations: Vec::new(),
                    ident: Ident {
                        name: "_implicit_default".into(),
                        span: disc_span,
                    },
                    ty: Ty {
                        kind: TyKind::Null,
                        span: disc_span,
                    },
                    labels: vec![Label {
                        value,
                        span: disc_span,
                    }],
                    is_default: true,
                };

                u.variants.push(null_variant);
            }
        }

        u
    }
}

/// Transform HIR to add implicit default cases to incomplete unions
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    let mut folder = ImplicitDefault::new(&hir.context);
    let def_ids: Vec<_> = hir.context.definitions.iter().map(|(id, _)| id).collect();

    for id in def_ids {
        hir.context.definitions.fold(id, |def| folder.fold_def(def));
    }

    hir
}
