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

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind, PrimitiveTy, Ty, TyKind};

use crate::codegen::PyGen;
use crate::imports::parent_module;
use crate::writer::PyWriter;

pub fn needs_decimal(hir: &ResolvedGraph, ty: &Ty) -> bool {
    let resolved = hir.context.resolve_ty(ty);
    match &resolved.kind {
        TyKind::Primitive(PrimitiveTy::Float128) | TyKind::Fixed => true,
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => needs_decimal(hir, ty),
        TyKind::Map { key, elem, .. } => needs_decimal(hir, key) || needs_decimal(hir, elem),
        _ => false,
    }
}

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
        PrimitiveTy::Char | PrimitiveTy::WChar => "\"\"",
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

impl PyGen<'_> {
    pub fn py_def(&self, w: &PyWriter, def_id: DefId) -> String {
        let def = self.hir.context.type_of(def_id);
        let type_name = def.ident.name.clone();

        if let Some(module_id) = parent_module(self.hir, def_id)
            && let Some(style) = w.import_context.module_imports.get(&module_id)
        {
            let prefix = style.type_prefix();
            format!("{prefix}.{type_name}")
        } else {
            type_name
        }
    }

    pub fn py_type(&self, w: &PyWriter, ty: &Ty) -> String {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Primitive(prim) => primitive_type(*prim).to_string(),
            TyKind::String { .. } => "str".to_string(),
            TyKind::Adt(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                let type_name = def.ident.name.clone();

                if let Some(module_id) = parent_module(self.hir, *def_id)
                    && let Some(style) = w.import_context.module_imports.get(&module_id)
                {
                    let prefix = style.type_prefix();
                    return format!("{prefix}.{type_name}");
                }

                type_name
            }
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                let inner = self.py_type(w, ty);
                format!("list[{inner}]")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.py_type(w, key);
                let elem_ty = self.py_type(w, elem);
                format!("dict[{key_ty}, {elem_ty}]")
            }
            TyKind::Any => "object".to_string(),
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
                if let Some(enum_val) = self.enum_default(w, *def_id) {
                    enum_val
                } else {
                    let type_name = self.py_type(w, ty);
                    format!("_dataclasses_.field(default_factory={type_name})")
                }
            }
            TyKind::Any | TyKind::Null => "None".to_string(),
            TyKind::Array { .. } | TyKind::Sequence { .. } => {
                "_dataclasses_.field(default_factory=list)".to_string()
            }
            TyKind::Map { .. } => "_dataclasses_.field(default_factory=dict)".to_string(),
            TyKind::Fixed => "_decimal_.Decimal(0)".to_string(),
        }
    }

    fn enum_default(&self, w: &PyWriter, def_id: DefId) -> Option<String> {
        let def = self.hir.context.type_of(def_id);
        if let DefKind::Enum(enum_ty) = &def.kind
            && let Some(&first_field) = enum_ty.fields.first()
        {
            let first_def = self.hir.context.type_of(first_field);
            let enum_name = self.py_def(w, def_id);
            return Some(format!("{}.{}", enum_name, first_def.ident.name));
        }
        None
    }
}
