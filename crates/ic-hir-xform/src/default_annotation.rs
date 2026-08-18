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
//! Initializer lists are shaped during lowering, where the member type is
//! known. Scalars are evaluated without a target type, so this transformation
//! coerces the values that only a type can resolve: a float literal for a
//! `float` member, and an integer literal for an enum member.

use std::collections::HashMap;

use ic_hir::fold::Fold;
use ic_hir::hir::{
    Ann, AnnotationTy, Def, DefId, DefKind, Member, Numeric, PrimitiveTy, Ty, TyKind, UnionTy,
    Variant,
};
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
            (TyKind::Primitive(PrimitiveTy::Float32), Numeric::Double(v)) => {
                Some(Numeric::Float(*v as f32))
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
            if let Some(const_value) = self.const_values.get(const_id)
                && self.is_valid_enum_const(const_value, fields)
            {
                return None;
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

    fn process_annotations(&self, annotations: &mut [Ann], ty: &Ty) {
        if let Some(default_ann) = annotations.iter_mut().find(|a| a.ident.name == "default")
            && let Some(arg) = default_ann.args.first_mut()
            && let Some(coerced) = self.coerce_numeric(&arg.value, ty)
        {
            arg.value = coerced;
            arg.ty = Some(ty.clone());
        }
    }
}

impl Fold for DefaultAnnotation {
    fn fold_def(&mut self, mut def: Def) -> Def {
        if let DefKind::Alias(alias_ty) = &def.kind {
            let ty = alias_ty.ty.clone();
            self.process_annotations(&mut def.annotations, &ty);
        }
        ic_hir::fold::fold_def(self, def)
    }

    fn fold_member(&mut self, mut member: Member) -> Member {
        let ty = member.ty.clone();
        self.process_annotations(&mut member.annotations, &ty);
        ic_hir::fold::fold_member(self, member)
    }

    fn fold_variant(&mut self, mut variant: Variant) -> Variant {
        let ty = variant.ty.clone();
        self.process_annotations(&mut variant.annotations, &ty);
        ic_hir::fold::fold_variant(self, variant)
    }

    fn fold_union_ty(&mut self, mut union_ty: UnionTy) -> UnionTy {
        let ty = union_ty.disc.ty.clone();
        self.process_annotations(&mut union_ty.disc.annotations, &ty);
        ic_hir::fold::fold_union_ty(self, union_ty)
    }

    fn fold_annotation_ty(&mut self, mut annotation_ty: AnnotationTy) -> AnnotationTy {
        for param in &mut annotation_ty.params {
            let ty = param.ty.clone();
            self.process_annotations(&mut param.annotations, &ty);
        }
        ic_hir::fold::fold_annotation_ty(self, annotation_ty)
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
