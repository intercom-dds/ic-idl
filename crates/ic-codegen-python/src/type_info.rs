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

use std::fmt::Write;

use ic_hir::hir::{Def, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir_analysis::annotation::{
    Extensibility, extensibility, is_external, is_key, is_must_understand, is_nested, is_optional,
};
use ic_hir_analysis::member_id::{Autoid, effective_autoid, member_ids};

use crate::codegen::{PyGen, escape_python_string};
use crate::py;
use crate::types::{MemberKind, collect_all_members};
use crate::writer::PyWriter;

impl PyGen<'_, '_> {
    pub fn emit_type_info_def(&self, w: &mut PyWriter, def: &Def) {
        let orig_def = self.original_hir.context.definitions.get(def.id);

        let kind = match &orig_def.kind {
            DefKind::Union(_) => "UNION",
            DefKind::Enum(_) => "ENUM",
            DefKind::Bitmask(_) => "BITMASK",
            DefKind::Struct(_)
            | DefKind::Valuetype(_)
            | DefKind::Except(_)
            | DefKind::Module(_)
            | DefKind::Const(_)
            | DefKind::Bitset(_)
            | DefKind::Alias(_)
            | DefKind::Interface(_)
            | DefKind::Annotation(_)
            | DefKind::Decl(_) => "STRUCT",
        };

        let key_info = match &def.kind {
            DefKind::Union(union_ty) => Some(union_ty.disc.ty.clone()),
            _ => None,
        };
        let element_info = match &def.kind {
            DefKind::Enum(enum_ty) => Some(enum_ty.ty),
            DefKind::Bitmask(bitmask_ty) => Some(bitmask_ty.ty),
            _ => None,
        };
        let name = escape_python_string(&self.original_hir.context.qualified_name(def.id));

        py!(w, "\n");
        py!(w, "@_builtins_.staticmethod\n");
        py!(w, "@_cache_\n");
        py!(w, "def __type_info__() -> _type_info_.TypeInfo:\n");
        w.indent();
        py!(w, "return _type_info_.TypeInfo(\n");
        w.indent();
        py!(w, "name=\"", name, "\",\n");
        py!(w, "ty=", self.py_def(w, def.id), ",\n");
        py!(w, "kind=_type_info_.TypeKind.", kind, ",\n");
        py!(w, "flags=", type_flags(&self.hir.context, def), ",\n");
        if let Some(key_info) = key_info {
            py!(w, "key_info=");
            self.emit_type_info(w, &key_info);
            py!(w, ",\n");
        } else {
            py!(w, "key_info=None,\n");
        }
        if let Some(element_info) = element_info {
            py!(w, "element_info=_type_info_.BuiltinTypes.", primitive_type_info(element_info), ",\n");
        } else {
            py!(w, "element_info=None,\n");
        }
        w.dedent();
        py!(w, ")\n");
        w.dedent();
        py!(w, "\n");

        match &def.kind {
            DefKind::Struct(_) | DefKind::Except(_) | DefKind::Valuetype(_) => {
                self.emit_struct_member_info(w, def);
            }
            DefKind::Bitmask(bitmask_ty) => self.emit_bitmask_member_info(w, def, bitmask_ty),
            DefKind::Enum(enum_ty) => self.emit_enum_member_info(w, def, enum_ty),
            DefKind::Union(union_ty) => self.emit_union_member_info(w, def, union_ty),
            _ => {}
        }

        if let Some(type_info_cb) = self.type_info_cb.borrow_mut().as_mut() {
            type_info_cb(w, orig_def);
        }
    }

    pub fn emit_type_info(&self, w: &mut PyWriter, ty: &Ty) {
        const BUILTIN_PREFIX: &str = "_type_info_.BuiltinTypes.";
        let inner_ty = self.original_hir.context.resolve_ty(ty);
        match &inner_ty.kind {
            TyKind::Primitive(prim) => return py!(w, BUILTIN_PREFIX, primitive_type_info(*prim)),
            TyKind::String {
                wide, bound: None, ..
            } => {
                return py!(w, BUILTIN_PREFIX, if *wide { "STRING16" } else { "STRING8" });
            }
            TyKind::String {
                wide,
                bound: Some(bound),
                ..
            } => {
                let wide = if *wide { "True" } else { "False" };
                return py!(w, "_type_info_.bounded_string(", wide,", ", bound, ")");
            }
            TyKind::Adt(adt_id)
                if !matches!(
                    self.hir.context.definitions.get(*adt_id).kind,
                    DefKind::Alias(_)
                ) =>
            {
                return py!(w, "_type_info_.type_info(", self.py_def(w, *adt_id), ")");
            }
            _ => {}
        }

        let py_ty = self.py_type(w, ty);

        let kind: &str = match &inner_ty.kind {
            TyKind::Map { .. } => "MAP",
            TyKind::Array { .. } => "ARRAY",
            TyKind::Sequence { .. } => "SEQUENCE",
            _ => "ANY",
        };

        py!(w, "_type_info_.TypeInfo(\n");
        w.indent();
        py!(w, "name=\"", kind.to_lowercase(), "\",\n");
        py!(w, "ty=", py_ty, ",\n");
        py!(w, "kind=_type_info_.TypeKind.", kind, ",\n");
        py!(w, "flags=_type_info_.TypeFlag.IS_FINAL,\n");

        match &inner_ty.kind {
            TyKind::Map {
                key, elem, bound, ..
            } => {
                py!(w, "key_info=");
                self.emit_type_info(w, key);
                py!(w, ",\n");
                py!(w, "element_info=");
                self.emit_type_info(w, elem);
                py!(w, ",\n");
                if let Some(bound) = bound {
                    py!(w, "length=", bound, ",\n");
                }
            }
            TyKind::Array { ty, len, .. } => {
                py!(w, "key_info=None,\n");
                py!(w, "element_info=");
                self.emit_type_info(w, ty);
                py!(w, ",\n");
                py!(w, "length=", len, ",\n");
            }
            TyKind::Sequence { ty, bound, .. } => {
                py!(w, "key_info=None,\n");
                py!(w, "element_info=");
                self.emit_type_info(w, ty);
                py!(w, ",\n");
                if let Some(bound) = bound {
                    py!(w, "length=", bound, ",\n");
                }
            }
            _ => {
                py!(w, "key_info=None,\n");
                py!(w, "element_info=None,\n");
            }
        }

        w.dedent();
        py!(w, ")");
    }

    pub fn emit_struct_member_info(&self, w: &mut PyWriter, def: &Def) {
        let orig_def = self.original_hir.context.base_def_of(def.id);

        let orig_members = match &orig_def.kind {
            DefKind::Struct(_) | DefKind::Except(_) | DefKind::Valuetype(_) => {
                collect_all_members(&self.original_hir.context, def.id)
            }
            _ => vec![],
        };
        let members = match &orig_def.kind {
            DefKind::Struct(_) | DefKind::Except(_) | DefKind::Valuetype(_) => {
                collect_all_members(&self.hir.context, def.id)
            }
            _ => vec![],
        };

        let ids = if matches!(
            orig_def.kind,
            DefKind::Struct(_) | DefKind::Except(_) | DefKind::Valuetype(_)
        ) {
            member_ids(&self.original_hir.context, def.id)
        } else {
            (0u32..).take(orig_members.len()).collect()
        };

        py!(w, "@_builtins_.staticmethod\n");
        py!(w, "@_cache_\n");
        py!(w, "def __member_info__() -> list[_type_info_.MemberInfo]:\n");
        w.indent();
        py!(w, "return [\n");
        w.indent();
        for ((member, orig_member), id) in members.iter().zip(&orig_members).zip(ids) {
            let ty = self.hir.context.base_type_of(def.id);
            py!(w, "_type_info_.MemberInfo(\n");
            w.indent();
            py!(w, "name=\"", orig_member.name(), "\",\n");
            if matches!(orig_def.kind, DefKind::Except(_)) {
                py!(w, "descriptor=_type_info_.AttributeDescriptor(\"", member.name(), "\"),\n");
            } else {
                py!(w, "descriptor=", self.py_type_relative_to(w, &ty, Some(def.id)), ".", member.name(), ",\n");
            }
            py!(w, "member_id=", id,",\n");
            py!(w, "flags=", member_flags(&self.original_hir.context, orig_member),",\n");
            py!(w, "type_info=");
            self.emit_type_info(w, orig_member.ty());
            py!(w, ",\n");

            w.dedent();
            py!(w, "),\n");
        }
        w.dedent();
        py!(w, "]\n");
        w.dedent();
    }

    pub fn emit_bitmask_member_info(
        &self,
        w: &mut PyWriter,
        def: &Def,
        bitmask_ty: &ic_hir::hir::BitmaskTy,
    ) {
        py!(w, "@_builtins_.staticmethod\n");
        py!(w, "@_cache_\n");
        py!(w, "def __member_info__() -> list[_type_info_.MemberInfo]:\n");
        w.indent();
        py!(w, "return [\n");
        w.indent();
        for &flag_id in &bitmask_ty.flags {
            let flag_def = self.hir.context.definitions.get(flag_id);
            let orig_flag_def = self.original_hir.context.definitions.get(flag_id);
            let position = if let DefKind::Const(const_ty) = &orig_flag_def.kind {
                self.original_hir
                    .context
                    .unsigned_value(&const_ty.value)
                    .trailing_zeros()
            } else {
                0
            };

            let ty = self.hir.context.base_type_of(flag_id);
            py!(w, "_type_info_.MemberInfo(\n");
            w.indent();
            py!(w, "name=\"", orig_flag_def.ident.name, "\",\n");
            py!(w, "descriptor=", self.py_type_relative_to(w, &ty, Some(def.id)),",\n");
            py!(w, "member_id=", position,",\n");
            py!(w, "flags=_type_info_.MemberFlag.NONE,\n");
            py!(w, "type_info=");
            self.emit_type_info(w, &ty);
            py!(w, ",\n");
            py!(w, "key=", self.py_type_relative_to(w, &ty, Some(def.id)),".", flag_def.ident.name,",\n");
            w.dedent();
            py!(w, "),\n");
        }
        w.dedent();
        py!(w, "]\n");
        w.dedent();
    }

    pub fn emit_enum_member_info(
        &self,
        w: &mut PyWriter,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
    ) {
        py!(w, "@_builtins_.staticmethod\n");
        py!(w, "@_cache_\n");
        py!(w, "def __member_info__() -> list[_type_info_.MemberInfo]:\n");
        w.indent();
        py!(w, "return [\n");
        w.indent();
        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            let orig_field_def = self.original_hir.context.definitions.get(field_id);
            let ty = self.hir.context.base_type_of(field_id);

            #[allow(clippy::cast_possible_wrap)]
            let member_id = if let DefKind::Const(c) = &field_def.kind {
                self.hir.context.integer_value(&c.value)
            } else {
                i as i64
            };

            py!(w, "_type_info_.MemberInfo(\n");
            w.indent();
            py!(w, "name=\"", orig_field_def.ident.name, "\",\n");
            py!(w, "descriptor=", self.py_type_relative_to(w, &ty, Some(def.id)),",\n");
            py!(w, "member_id=", member_id,",\n");
            py!(w, "flags=_type_info_.MemberFlag.NONE,\n");
            py!(w, "type_info=");
            self.emit_type_info(w, &ty);
            py!(w, ",\n");
            py!(w, "key=", self.py_type_relative_to(w, &ty, Some(def.id)),".", field_def.ident.name,",\n");
            w.dedent();
            py!(w, "),\n");
        }
        w.dedent();
        py!(w, "]\n");
        w.dedent();
    }

    pub fn emit_union_member_info(
        &self,
        w: &mut PyWriter,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
    ) {
        let original_def = self.original_hir.context.definitions.get(def.id);
        let original_union = match &original_def.kind {
            DefKind::Union(u) => u,
            _ => union_ty,
        };

        let ids = member_ids(&self.original_hir.context, def.id);
        let variants: Vec<_> = union_ty
            .variants
            .iter()
            .zip(&original_union.variants)
            .zip(ids.into_iter().skip(1))
            .filter(|((v, _), _)| !matches!(v.ty.kind, TyKind::Null))
            .collect();

        py!(w, "@_builtins_.staticmethod\n");
        py!(w, "@_cache_\n");
        py!(w, "def __member_info__() -> list[_type_info_.MemberInfo]:\n");
        w.indent();
        if variants.is_empty() {
            py!(w, "return []\n");
            w.dedent();
            return;
        }

        py!(w, "return [\n");
        w.indent();
        for ((variant, orig_variant), id) in variants {
            let ty = self.hir.context.base_type_of(def.id);
            py!(w, "_type_info_.MemberInfo(\n");
            w.indent();
            py!(w, "name=\"", orig_variant.ident.name, "\",\n");
            py!(w, "descriptor=", self.py_type_relative_to(w, &ty, Some(def.id)),".", variant.ident.name, ",\n");
            py!(w, "member_id=", id,",\n");
            py!(w, "flags=", member_flags(&self.original_hir.context, &MemberKind::Variant(variant)),",\n");
            py!(w, "type_info=");
            self.emit_type_info(w, &orig_variant.ty);
            py!(w, ",\n");
            py!(w, "key=[\n");
            w.indent();
            if !variant.is_default {
                for label in &variant.labels {
                    py!(w, self.format_numeric(w, Some(&union_ty.disc.ty), &label.value), ",\n");
                }
            }
            w.dedent();
            py!(w, "],\n");
            w.dedent();
            py!(w, "),\n");
        }
        w.dedent();
        py!(w, "]\n");
        w.dedent();
    }
}

fn primitive_type_info(ty: PrimitiveTy) -> &'static str {
    match ty {
        PrimitiveTy::Void => "NONE",
        PrimitiveTy::Bool => "BOOL",
        PrimitiveTy::Int8 => "I8",
        PrimitiveTy::UInt8 => "U8",
        PrimitiveTy::Int16 => "I16",
        PrimitiveTy::UInt16 => "U16",
        PrimitiveTy::Int32 => "I32",
        PrimitiveTy::UInt32 => "U32",
        PrimitiveTy::Int64 => "I64",
        PrimitiveTy::UInt64 => "U64",
        PrimitiveTy::Float32 => "F32",
        PrimitiveTy::Float64 => "F64",
        PrimitiveTy::Float128 => "F128",
        PrimitiveTy::Char => "CHAR8",
        PrimitiveTy::WChar => "CHAR16",
    }
}

fn has_key_member(ctx: &ic_hir::Context, def: &Def) -> bool {
    let members = match &def.kind {
        DefKind::Struct(s) => &s.members,
        DefKind::Valuetype(v) => &v.members,
        DefKind::Except(e) => &e.members,
        _ => return false,
    };
    members.iter().any(|member| is_key(ctx, member))
}

fn type_flags(ctx: &ic_hir::Context, def: &Def) -> String {
    let mut flags = Vec::new();

    match extensibility(ctx, def) {
        Extensibility::Final => flags.push("IS_FINAL"),
        Extensibility::Appendable => flags.push("IS_APPENDABLE"),
        Extensibility::Mutable => flags.push("IS_MUTABLE"),
    }

    if is_nested(ctx, def) {
        flags.push("IS_NESTED");
    }

    if effective_autoid(ctx, def) == Autoid::Hash {
        flags.push("IS_AUTOID_HASH");
    }

    if has_key_member(ctx, def) {
        flags.push("IS_KEYED");
    }

    let mut result = format!("_type_info_.TypeFlag.{}", flags[0]);
    for flag in &flags[1..] {
        _ = write!(result, " | _type_info_.TypeFlag.{flag}");
    }
    result
}

fn member_flags(ctx: &ic_hir::Context, member: &MemberKind<'_>) -> String {
    let mut flags = Vec::new();

    if is_key(ctx, member) {
        flags.push("IS_KEY");
    }

    if is_optional(ctx, member) {
        flags.push("IS_OPTIONAL");
    }

    if is_external(ctx, member) {
        flags.push("IS_EXTERNAL");
    }

    if is_must_understand(ctx, member) {
        flags.push("IS_MUST_UNDERSTAND");
    }

    if let MemberKind::Variant(variant) = member
        && variant.is_default
    {
        flags.push("IS_DEFAULT");
    }

    if flags.is_empty() {
        "_type_info_.MemberFlag.NONE".into()
    } else {
        let mut result = format!("_type_info_.MemberFlag.{}", flags[0]);
        for flag in &flags[1..] {
            _ = write!(result, " | _type_info_.MemberFlag.{flag}");
        }
        result
    }
}
