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

use ic_hir::Context;
use ic_hir::hir::{
    Attribute, Def, DefId, DefKind, Member, Numeric, PrimitiveTy, Ty, TyKind, UnionTy, Variant,
};
use ic_hir_analysis::annotation::{MemberLike, default_value, is_optional};
use ic_hir_analysis::enum_value::default_enumerator;
use ic_hir_analysis::union_case::{default_discriminator, default_union_case, union_case};

use crate::codegen::PyGen;
use crate::imports::parent_module;
use crate::writer::PyWriter;

fn primitive_type(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Void => "None",
        PrimitiveTy::Bool => "bool",
        PrimitiveTy::Char | PrimitiveTy::WChar => "str",
        PrimitiveTy::Int8
        | PrimitiveTy::UInt8
        | PrimitiveTy::Int16
        | PrimitiveTy::UInt16
        | PrimitiveTy::Int32
        | PrimitiveTy::UInt32
        | PrimitiveTy::Int64
        | PrimitiveTy::UInt64 => "int",
        PrimitiveTy::Float32 | PrimitiveTy::Float64 => "float",
        PrimitiveTy::Float128 => "_decimal_.Decimal",
    }
}

fn primitive_default(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Void => "None",
        PrimitiveTy::Bool => "False",
        PrimitiveTy::Char | PrimitiveTy::WChar => "\"\\0\"",
        PrimitiveTy::Int8
        | PrimitiveTy::UInt8
        | PrimitiveTy::Int16
        | PrimitiveTy::UInt16
        | PrimitiveTy::Int32
        | PrimitiveTy::UInt32
        | PrimitiveTy::Int64
        | PrimitiveTy::UInt64 => "0",
        PrimitiveTy::Float32 | PrimitiveTy::Float64 => "0.0",
        PrimitiveTy::Float128 => "_decimal_.Decimal(0)",
    }
}

pub(crate) fn default_union_variants<'a>(
    ctx: &Context,
    union_ty: &'a UnionTy,
) -> ((Numeric, &'a Variant), (Numeric, &'a Variant)) {
    let default_disc = default_discriminator(ctx, union_ty);
    let default_case = default_union_case(ctx, union_ty);

    let default_disc_init = default_value(ctx, &union_ty.disc)
        .unwrap_or(&default_disc)
        .clone();
    let default_variant_init = union_case(ctx, union_ty, &default_disc_init)
        .expect("union must have a case for its default discriminator")
        .variant;

    (
        (default_disc_init, default_variant_init),
        (default_disc, default_case.variant),
    )
}

pub(crate) fn wrapping_def<'a>(
    ctx: &'a Context,
    ty: Option<&Ty>,
    value: &Numeric,
) -> Option<&'a Def> {
    let TyKind::Adt(def_id) = &ty?.kind else {
        return None;
    };
    if matches!(value, Numeric::Const(_)) {
        return None;
    }
    let def = ctx.base_def_of(*def_id);
    matches!(def.kind, DefKind::Bitmask(_) | DefKind::Enum(_)).then_some(def)
}

pub(crate) fn collect_all_members(ctx: &Context, def_id: DefId) -> Vec<MemberKind<'_>> {
    let def = ctx.base_def_of(def_id);
    let mut all_members = Vec::new();

    match &def.kind {
        DefKind::Struct(struct_ty) => {
            if let Some(parent) = struct_ty.parent {
                all_members.extend(collect_all_members(ctx, parent.def_id));
            }
            all_members.extend(struct_ty.members.iter().map(MemberKind::Member));
        }
        DefKind::Valuetype(valuetype_ty) => {
            if let Some(parent) = valuetype_ty.parent {
                all_members.extend(collect_all_members(ctx, parent.def_id));
            }
            all_members.extend(valuetype_ty.members.iter().map(MemberKind::Member));
            all_members.extend(valuetype_ty.attributes.iter().map(MemberKind::Attrib));
        }
        DefKind::Except(except_ty) => {
            all_members.extend(except_ty.members.iter().map(MemberKind::Member));
        }
        _ => {}
    }

    all_members
}

pub(crate) enum MemberKind<'a> {
    Member(&'a Member),
    Attrib(&'a Attribute),
    Variant(&'a Variant),
}

impl MemberKind<'_> {
    pub fn name(&self) -> &str {
        match self {
            MemberKind::Member(m) => &m.ident.name,
            MemberKind::Attrib(a) => &a.ident.name,
            MemberKind::Variant(v) => &v.ident.name,
        }
    }

    pub fn ty(&self) -> &Ty {
        match self {
            MemberKind::Member(m) => &m.ty,
            MemberKind::Attrib(a) => &a.ty,
            MemberKind::Variant(v) => &v.ty,
        }
    }
}

impl MemberLike for MemberKind<'_> {
    fn annotations(&self) -> &[ic_hir::hir::Ann] {
        match self {
            MemberKind::Member(m) => m.annotations(),
            MemberKind::Attrib(a) => a.annotations(),
            MemberKind::Variant(v) => v.annotations(),
        }
    }
}

impl PyGen<'_> {
    pub fn py_def(&self, w: &PyWriter, def_id: DefId) -> String {
        self.py_def_relative_to(w, def_id, None)
    }

    pub(crate) fn is_imported(&self, w: &PyWriter, def_id: DefId) -> bool {
        w.import_context.file_imports.contains_key(&def_id)
            || parent_module(self.hir, def_id)
                .is_some_and(|module_id| w.import_context.module_imports.contains_key(&module_id))
    }

    fn py_def_relative_to(
        &self,
        w: &PyWriter,
        def_id: DefId,
        relative_to: Option<DefId>,
    ) -> String {
        if let Some(file_import) = w.import_context.file_imports.get(&def_id) {
            return file_import
                .alias
                .as_ref()
                .unwrap_or(&file_import.type_name)
                .clone();
        }

        let type_path = self.nested_type_path(def_id, relative_to);
        if let Some(module_id) = parent_module(self.hir, def_id)
            && let Some(style) = w.import_context.module_imports.get(&module_id)
        {
            let prefix = style.type_prefix();
            format!("{prefix}.{type_path}")
        } else {
            type_path
        }
    }

    pub(crate) fn nested_type_path(&self, def_id: DefId, relative_to: Option<DefId>) -> String {
        let mut path = vec![];
        let mut current = Some(def_id);

        while let Some(id) = current {
            if relative_to == Some(id) && !path.is_empty() {
                break;
            }

            let def = self.hir.context.type_of(id);
            match &def.kind {
                DefKind::Module(_) => break,
                _ => path.push(def.ident.name.clone()),
            }
            current = def.parent;
        }

        path.reverse();
        path.join(".")
    }

    pub fn py_type(&self, w: &PyWriter, ty: &Ty) -> String {
        self.py_type_relative_to(w, ty, None)
    }

    pub fn py_member_type(&self, w: &PyWriter, ty: &Ty, member: &impl MemberLike) -> String {
        if is_optional(&self.hir.context, member) {
            format!("{} | None", self.py_type(w, ty))
        } else {
            self.py_type(w, ty)
        }
    }

    pub(crate) fn py_type_relative_to(
        &self,
        w: &PyWriter,
        ty: &Ty,
        relative_to: Option<DefId>,
    ) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => primitive_type(*prim).to_string(),
            TyKind::String { .. } => "str".to_string(),
            TyKind::Adt(def_id) => self.py_def_relative_to(w, *def_id, relative_to),
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                let inner = self.py_type_relative_to(w, ty, relative_to);
                format!("list[{inner}]")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.py_type_relative_to(w, key, relative_to);
                let elem_ty = self.py_type_relative_to(w, elem, relative_to);
                format!("dict[{key_ty}, {elem_ty}]")
            }
            TyKind::Any => "_typing_.Any".to_string(),
            TyKind::Fixed => "_decimal_.Decimal".to_string(),
            TyKind::Null => "None".to_string(),
        }
    }

    pub fn default_value(&self, w: &PyWriter, ty: &Ty) -> String {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Primitive(prim) => primitive_default(*prim).to_string(),
            TyKind::String { .. } => "\"\"".to_string(),
            TyKind::Adt(def_id) => {
                if let Some(val) = self.adt_default(w, *def_id) {
                    val
                } else {
                    let type_name = self.value_type_name(w, ty, &resolved);
                    format!("{type_name}()")
                }
            }
            TyKind::Any | TyKind::Null => "None".to_string(),
            TyKind::Array { ty, len, .. } => format!(
                "[{} for _ in _builtins_.range({len})]",
                self.default_value(w, ty)
            ),
            TyKind::Sequence { .. } => "[]".to_string(),
            TyKind::Map { .. } => "{}".to_string(),
            TyKind::Fixed => "_decimal_.Decimal(0)".to_string(),
        }
    }

    pub(crate) fn numeric_contains_const(numeric: &Numeric) -> bool {
        match numeric {
            Numeric::Const(_) => true,
            Numeric::Array { values, .. } | Numeric::Sequence { values, .. } => {
                values.iter().any(Self::numeric_contains_const)
            }
            Numeric::Map { entries, .. } => entries.iter().any(|(key, value)| {
                Self::numeric_contains_const(key) || Self::numeric_contains_const(value)
            }),
            Numeric::Struct { fields, .. } => fields.iter().any(Self::numeric_contains_const),
            Numeric::Union {
                discriminant,
                value,
                ..
            } => Self::numeric_contains_const(discriminant) || Self::numeric_contains_const(value),
            _ => false,
        }
    }

    pub(crate) fn field_default(&self, w: &PyWriter, ty: &Ty, member: &impl MemberLike) -> String {
        if let Some(default_value) = default_value(&self.hir.context, member) {
            if matches!(
                default_value,
                Numeric::Array { .. }
                    | Numeric::Const(_)
                    | Numeric::Sequence { .. }
                    | Numeric::Map { .. }
                    | Numeric::Struct { .. }
                    | Numeric::Union { .. }
            ) || matches!(ty.kind, TyKind::Adt(def_id) if self.needs_lambda_default(w, def_id))
            {
                let initializer = self.format_numeric(w, Some(ty), default_value);

                return format!(
                    "_dataclasses_.field(default_factory=lambda: {})",
                    if Self::numeric_contains_const(default_value) {
                        format!("_copy_.deepcopy({initializer})")
                    } else {
                        initializer
                    }
                );
            }

            return self.format_numeric(w, Some(ty), default_value);
        }

        if is_optional(&self.hir.context, member) {
            return "None".into();
        }

        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Primitive(prim) => primitive_default(*prim).to_string(),
            TyKind::String { .. } => "\"\"".to_string(),
            TyKind::Any | TyKind::Null => "None".to_string(),
            TyKind::Fixed => "_decimal_.Decimal(0)".to_string(),
            TyKind::Array { .. } => format!(
                "_dataclasses_.field(default_factory=lambda: {})",
                self.default_value(w, ty)
            ),
            TyKind::Sequence { .. } => "_dataclasses_.field(default_factory=list)".to_string(),
            TyKind::Map { .. } => "_dataclasses_.field(default_factory=dict)".to_string(),
            TyKind::Adt(def_id) => {
                if self.needs_lambda_default(w, *def_id)
                    || matches!(&ty.kind, TyKind::Adt(id) if w.deferred_aliases.contains(id))
                {
                    let val = self.default_value(w, ty);
                    format!("_dataclasses_.field(default_factory=lambda: {val})")
                } else {
                    let type_name = self.value_type_name(w, ty, &resolved);
                    format!("_dataclasses_.field(default_factory={type_name})")
                }
            }
        }
    }

    fn value_type_name(&self, w: &PyWriter, ty: &Ty, resolved: &Ty) -> String {
        if let TyKind::Adt(def_id) = &ty.kind
            && w.deferred_aliases.contains(def_id)
        {
            self.py_type(w, resolved)
        } else {
            self.py_type(w, ty)
        }
    }

    pub(crate) fn needs_lambda_default(&self, w: &PyWriter, def_id: DefId) -> bool {
        let def = self.hir.context.base_def_of(def_id);
        if matches!(
            def.kind,
            DefKind::Enum(_) | DefKind::Bitmask(_) | DefKind::Const(_)
        ) {
            return true;
        }

        if w.import_context.file_imports.contains_key(&def_id) {
            return false;
        }

        if parent_module(self.hir, def_id)
            .is_some_and(|module_id| w.import_context.module_imports.contains_key(&module_id))
        {
            return true;
        }

        !w.decleared_defs.contains(&self.path_root(def_id))
    }

    fn adt_default(&self, w: &PyWriter, def_id: DefId) -> Option<String> {
        let def = self.hir.context.type_of(def_id);
        match &def.kind {
            DefKind::Enum(enum_ty) => {
                let field_id = default_enumerator(&self.hir.context, enum_ty);
                let field_def = self.hir.context.type_of(field_id);
                let enum_path = self.py_def(w, def_id);
                Some(format!("{}.{}", enum_path, field_def.ident.name))
            }
            DefKind::Bitmask(bitmask_ty) => {
                let first = *bitmask_ty.flags.first()?;
                let first_def = self.hir.context.type_of(first);
                let bitmask_path = self.py_def(w, def_id);
                Some(format!("{}.{}", bitmask_path, first_def.ident.name))
            }
            DefKind::Const(_) => Some(self.py_def(w, def_id)),
            _ => None,
        }
    }
}
