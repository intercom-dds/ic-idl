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

//! Coerce @default annotation values to match their target member types.
//!
//! The @default annotation value is initially evaluated without knowing the
//! member's type, so `{1, 2, 3}` becomes a Sequence even when the target is
//! an Array. This transformation coerces the Numeric values to match.

use std::collections::HashMap;

use ic_hir::fold::Fold;
use ic_hir::hir::{DefId, DefKind, Member, Numeric, PrimitiveTy, Ty, TyKind};
use ic_hir::{Context, ResolvedGraph};
use tracing::{debug, debug_span};

struct DefaultAnnotation {
    enum_fields: HashMap<DefId, Vec<(i64, DefId)>>,
    typedef_targets: HashMap<DefId, TyKind>,
    const_values: HashMap<DefId, Numeric>,
}

impl DefaultAnnotation {
    fn new(context: &Context) -> Self {
        let mut enum_fields = HashMap::new();
        let mut typedef_targets = HashMap::new();
        let mut const_values = HashMap::new();

        for (def_id, def) in &context.definitions {
            match &def.kind {
                DefKind::Enum(enum_ty) => {
                    enum_fields.insert(def_id, Self::collect_enum_fields(context, enum_ty));
                }
                DefKind::Alias(alias_ty) => {
                    let resolved = context.resolve_ty(&alias_ty.ty);
                    typedef_targets.insert(def_id, resolved.kind.clone());
                }
                DefKind::Const(const_ty) => {
                    const_values.insert(def_id, const_ty.value.clone());
                }
                _ => {}
            }
        }

        Self {
            enum_fields,
            typedef_targets,
            const_values,
        }
    }

    fn collect_enum_fields(context: &Context, enum_ty: &ic_hir::hir::EnumTy) -> Vec<(i64, DefId)> {
        enum_ty
            .fields
            .iter()
            .filter_map(|&field_id| {
                let field_def = context.type_of(field_id);
                if let DefKind::Const(const_ty) = &field_def.kind {
                    Self::numeric_to_i64(&const_ty.value).map(|v| (v, field_id))
                } else {
                    None
                }
            })
            .collect()
    }

    fn resolve_ty_kind<'a>(&'a self, ty: &'a TyKind) -> &'a TyKind {
        if let TyKind::Adt(def_id) = ty
            && let Some(resolved) = self.typedef_targets.get(def_id)
        {
            return self.resolve_ty_kind(resolved);
        }
        ty
    }

    #[allow(clippy::cast_possible_truncation)]
    fn coerce_numeric(&self, value: &Numeric, target_ty: &Ty) -> Option<Numeric> {
        let resolved_kind = self.resolve_ty_kind(&target_ty.kind);
        match (resolved_kind, value) {
            (TyKind::Array { ty: elem_ty, .. }, Numeric::Sequence { values, .. }) => {
                let coerced: Vec<_> = values
                    .iter()
                    .map(|v| self.coerce_numeric(v, elem_ty).unwrap_or_else(|| v.clone()))
                    .collect();
                Some(Numeric::Array {
                    ty: (**elem_ty).clone(),
                    values: coerced.into_boxed_slice(),
                })
            }

            (TyKind::Map { key, elem, .. }, Numeric::Sequence { values, .. }) => {
                let mut entries = Vec::new();
                for entry in values {
                    if let Numeric::Sequence { values: pair, .. } = entry
                        && pair.len() >= 2
                    {
                        let k = self
                            .coerce_numeric(&pair[0], key)
                            .unwrap_or_else(|| pair[0].clone());
                        let v = self
                            .coerce_numeric(&pair[1], elem)
                            .unwrap_or_else(|| pair[1].clone());
                        entries.push((k, v));
                    }
                }
                Some(Numeric::Map {
                    key: (**key).clone(),
                    value: (**elem).clone(),
                    entries: entries.into_boxed_slice(),
                })
            }

            (TyKind::Primitive(PrimitiveTy::Float32), Numeric::Double(v)) => {
                Some(Numeric::Float(*v as f32))
            }

            (TyKind::Sequence { ty: elem_ty, .. }, Numeric::Sequence { values, .. }) => {
                let coerced: Vec<_> = values
                    .iter()
                    .map(|v| self.coerce_numeric(v, elem_ty).unwrap_or_else(|| v.clone()))
                    .collect();
                Some(Numeric::Sequence {
                    ty: (**elem_ty).clone(),
                    values: coerced.into_boxed_slice(),
                })
            }

            (TyKind::Adt(def_id), _) => self.coerce_int_to_enum(value, *def_id),

            _ => None,
        }
    }

    fn coerce_int_to_enum(&self, value: &Numeric, def_id: DefId) -> Option<Numeric> {
        let fields = self.enum_fields.get(&def_id)?;

        if let Numeric::Const(const_id) = value {
            if fields.iter().any(|(_, fid)| fid == const_id) {
                return None;
            }
            if let Some(const_value) = self.const_values.get(const_id) {
                if self.is_valid_enum_const(const_value, fields) {
                    return None;
                }
            }
            return None;
        }

        let int_val = Self::numeric_to_i64(value)?;
        for &(field_val, field_id) in fields {
            if field_val == int_val {
                return Some(Numeric::Const(field_id));
            }
        }
        None
    }

    fn is_valid_enum_const(&self, value: &Numeric, fields: &[(i64, DefId)]) -> bool {
        if let Numeric::Const(const_id) = value {
            if fields.iter().any(|(_, fid)| fid == const_id) {
                return true;
            }
            if let Some(const_value) = self.const_values.get(const_id) {
                return self.is_valid_enum_const(const_value, fields);
            }
        }
        false
    }

    fn numeric_to_i64(value: &Numeric) -> Option<i64> {
        match value {
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

    fn process_member(&self, member: &mut Member) {
        if let Some(default_ann) = member
            .annotations
            .iter_mut()
            .find(|a| a.ident.name == "default")
            && let Some(arg) = default_ann.args.first_mut()
            && let Some(coerced) = self.coerce_numeric(&arg.value, &member.ty)
        {
            arg.value = coerced;
            arg.ty = Some(member.ty.clone());
        }
    }
}

impl Fold for DefaultAnnotation {
    fn fold_member(&mut self, mut member: Member) -> Member {
        self.process_member(&mut member);
        ic_hir::fold::fold_member(self, member)
    }
}

#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    let _span = debug_span!("xform", name = "default_annotation").entered();
    debug!("applying transform");

    let mut folder = DefaultAnnotation::new(&hir.context);
    let def_ids: Vec<_> = hir.context.definitions.iter().map(|(id, _)| id).collect();

    for id in def_ids {
        hir.context.definitions.fold(id, |def| folder.fold_def(def));
    }

    hir
}
