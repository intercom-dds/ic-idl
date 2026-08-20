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

use ic_emit::printer::Twine;
use ic_emit::w;
use ic_hir::hir::{Ann, DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};

use crate::codegen::RustGen;
use crate::helpers::{format_integer, is_trivial};

fn is_integer(value: &Numeric) -> bool {
    matches!(
        value,
        Numeric::Int8(_)
            | Numeric::UInt8(_)
            | Numeric::Int16(_)
            | Numeric::UInt16(_)
            | Numeric::Int32(_)
            | Numeric::UInt32(_)
            | Numeric::Int64(_)
            | Numeric::UInt64(_)
    )
}

impl RustGen<'_> {
    pub(crate) fn format_numeric(num: &Numeric) -> String {
        match num {
            Numeric::Int8(v) => format_integer(i128::from(*v)),
            Numeric::UInt8(v) => format_integer(i128::from(*v)),
            Numeric::Int16(v) => format_integer(i128::from(*v)),
            Numeric::UInt16(v) => format_integer(i128::from(*v)),
            Numeric::Int32(v) => format_integer(i128::from(*v)),
            Numeric::UInt32(v) => format_integer(i128::from(*v)),
            Numeric::Int64(v) => format_integer(i128::from(*v)),
            Numeric::UInt64(v) => format_integer(i128::from(*v)),
            Numeric::String(v) | Numeric::WString(v) => format!("\"{}\"", v.escape_default()),
            _ => "0".to_string(),
        }
    }

    fn array_default(&self, ty: &Ty, ctx_id: DefId, w: &mut Twine) {
        if let TyKind::Array {
            ty: elem_ty, len, ..
        } = &ty.kind
        {
            if self.is_copy_type(elem_ty) {
                w!(w, "[");
                self.emit_default_value(elem_ty, ctx_id, w);
                w!(w, "; ", len.to_string(), "]");
            } else {
                w!(w, "std::array::from_fn(|_| ");
                self.array_default(elem_ty, ctx_id, w);
                w!(w, ")");
            }
        } else {
            self.emit_default_value(ty, ctx_id, w);
        }
    }

    pub(crate) fn emit_default_value(&self, ty: &Ty, ctx_id: DefId, w: &mut Twine) {
        match &ty.kind {
            TyKind::Primitive(prim) => {
                let val = match prim {
                    PrimitiveTy::Bool => "false",
                    PrimitiveTy::Char | PrimitiveTy::WChar => "'\\0'",
                    PrimitiveTy::Float32 => "0_f32",
                    PrimitiveTy::Float64 | PrimitiveTy::Float128 => "0_f64",
                    _ => "0",
                };
                w!(w, val);
            }
            TyKind::Sequence { .. } => {
                w!(w, "vec![]");
            }
            _ => {
                let ty_str = self.rust_type(ty, ctx_id);
                w!(w, "<", ty_str, ">::default()");
            }
        }
    }

    pub(crate) fn emit_annotated_default_value(
        &self,
        ty: &Ty,
        annotations: &[Ann],
        ctx_id: DefId,
        w: &mut Twine,
    ) {
        let is_shared = self.is_shared_annotations(annotations);
        if is_shared {
            w!(w, "::std::boxed::Box::new(");
        }

        self.emit_default_value(ty, ctx_id, w);

        if is_shared {
            w!(w, ")");
        }
    }

    fn emit_struct_literal(
        &self,
        struct_id: DefId,
        fields: &[Numeric],
        ctx_id: DefId,
        w: &mut Twine,
    ) {
        let ty_str = self.scoped_name(struct_id, ctx_id);
        w!(w, ty_str, " {\n");

        let struct_def = self.hir.context.definitions.get(struct_id);
        if let DefKind::Struct(struct_ty) = &struct_def.kind {
            let members = self.struct_members(struct_ty);
            for (member, field_value) in members.iter().zip(fields.iter()) {
                w!(w, member.ident.name, ": ");
                self.emit_annotated_const_value(
                    field_value,
                    &member.ty,
                    &member.annotations,
                    ctx_id,
                    w,
                );
                w!(w, ",\n");
            }
        }
        w!(w, "}");
    }

    pub(crate) fn emit_annotated_const_value(
        &self,
        val: &Numeric,
        ty: &Ty,
        annotations: &[Ann],
        ctx_id: DefId,
        w: &mut Twine,
    ) {
        let is_shared = self.is_shared_annotations(annotations);
        if is_shared {
            w!(w, "::std::boxed::Box::new(");
        }

        self.emit_const_value(val, ty, ctx_id, w);

        if is_shared {
            w!(w, ")");
        }
    }

    fn const_newtype(&self, val: &Numeric) -> Option<DefId> {
        let Numeric::Const(def_id) = val else {
            return None;
        };
        let DefKind::Const(const_ty) = &self.hir.context.definitions.get(*def_id).kind else {
            return None;
        };

        self.newtype_alias(&const_ty.ty).map(|(id, _)| id)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn emit_const_value(&self, val: &Numeric, ty: &Ty, ctx_id: DefId, w: &mut Twine) {
        if let Some((newtype_id, alias_ty)) = self.newtype_alias(ty)
            && self.const_newtype(val) != Some(newtype_id)
        {
            let newtype_ty = self.scoped_name(newtype_id, ctx_id);
            w!(w, newtype_ty, "(");
            self.emit_const_value(val, &alias_ty.ty, ctx_id, w);
            w!(w, ")");
            return;
        }

        // Rust bitmasks are newtypes, so raw integer values require the
        // underlying bitmask constructor.
        if let TyKind::Adt(def_id) = ty.kind {
            let base_def = self.hir.context.base_def_of(def_id);
            if matches!(base_def.kind, DefKind::Bitmask(_)) && is_integer(val) {
                let bitmask_ty = self.scoped_name(base_def.id, ctx_id);
                w!(w, bitmask_ty, "(", Self::format_numeric(val), ")");
                return;
            }
        }

        match val {
            Numeric::Null => {
                if matches!(ty.kind, TyKind::Array { .. }) {
                    self.array_default(ty, ctx_id, w);
                } else {
                    self.emit_default_value(ty, ctx_id, w);
                }
            }
            Numeric::Bool(val) => {
                w!(w, val);
            }
            Numeric::Char(c) | Numeric::WChar(c) => {
                w!(w, "'", c.escape_default(), "'");
            }
            Numeric::Int8(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::UInt8(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::Int16(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::UInt16(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::Int32(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::UInt32(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::Int64(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::UInt64(v) => w!(w, format_integer(i128::from(*v))),
            Numeric::Float(v) => w!(w, format!("{v:.7}_f32")),
            Numeric::Double(v) => w!(w, format!("{v:.16}_f64")),
            Numeric::String(s) | Numeric::WString(s) => {
                w!(w, "\"", s.escape_default(), "\"");
                let base_ty = self.hir.context.base_type_of(ctx_id);
                if !matches!(base_ty.kind, TyKind::String { .. }) {
                    w!(w, ".into()");
                }
            }
            Numeric::Const(def_id) => {
                let const_def = self.hir.context.definitions.get(*def_id);
                if let DefKind::Const(const_ty) = &const_def.kind {
                    let name = self.scoped_name(*def_id, ctx_id);
                    w!(w, name);
                    let is_str_const_lit =
                        self.is_string_const_literal(&const_ty.ty, &const_ty.value);
                    if matches!(ty.kind, TyKind::String { .. }) {
                        if is_str_const_lit {
                            let base_ty = self.hir.context.base_type_of(ctx_id);
                            if !matches!(base_ty.kind, TyKind::String { .. }) {
                                w!(w, ".into()");
                            }
                        } else if !matches!(const_ty.ty.kind, TyKind::String { .. }) {
                            w!(w, ".into()");
                        }
                    } else if !is_trivial(const_def) && !is_str_const_lit {
                        w!(w, ".clone()");
                    }
                }
            }
            Numeric::Array { ty: arr_ty, values } => {
                w!(w, "[");
                for (i, v) in values.iter().enumerate() {
                    self.emit_const_value(v, arr_ty, ctx_id, w);
                    if i + 1 < values.len() {
                        w!(w, ", ");
                    }
                }
                w!(w, "]");
            }
            Numeric::Sequence { ty: seq_ty, values } => {
                w!(w, "vec![");
                for (i, v) in values.iter().enumerate() {
                    self.emit_const_value(v, seq_ty, ctx_id, w);
                    if i + 1 < values.len() {
                        w!(w, ", ");
                    }
                }
                w!(w, "]");
            }
            Numeric::Map {
                key,
                value,
                entries,
            } => {
                w!(w, "::std::collections::BTreeMap::from([\n");
                for (k, v) in entries {
                    w!(w, "(");
                    self.emit_const_value(k, key, ctx_id, w);
                    w!(w, ", ");
                    self.emit_const_value(v, value, ctx_id, w);
                    w!(w, "),\n");
                }
                w!(w, "])");
            }
            Numeric::Struct {
                ty: struct_id,
                fields,
            } => {
                self.emit_struct_literal(*struct_id, fields, ctx_id, w);
            }
            Numeric::Union { .. } => {
                let ty_str = self.rust_type(ty, ctx_id);
                w!(w, ty_str, "::new()");
            }
        }
    }
}
