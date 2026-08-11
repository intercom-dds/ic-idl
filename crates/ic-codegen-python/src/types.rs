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

use ic_hir::hir::{DefId, DefKind, PrimitiveTy, Ty, TyKind};

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
        if let Some(file_import) = w.import_context.file_imports.get(&def_id) {
            return file_import
                .alias
                .as_ref()
                .unwrap_or(&file_import.type_name)
                .clone();
        }

        let type_path = self.nested_type_path(def_id);
        if let Some(module_id) = parent_module(self.hir, def_id)
            && let Some(style) = w.import_context.module_imports.get(&module_id)
        {
            let prefix = style.type_prefix();
            format!("{prefix}.{type_path}")
        } else {
            type_path
        }
    }

    pub(crate) fn nested_type_path(&self, def_id: DefId) -> String {
        let mut path = vec![];
        let mut current = Some(def_id);

        while let Some(id) = current {
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
        match &ty.kind {
            TyKind::Primitive(prim) => primitive_type(*prim).to_string(),
            TyKind::String { .. } => "str".to_string(),
            TyKind::Adt(def_id) => self.py_def(w, *def_id),
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                let inner = self.py_type(w, ty);
                format!("list[{inner}]")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.py_type(w, key);
                let elem_ty = self.py_type(w, elem);
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
            TyKind::Array { .. } | TyKind::Sequence { .. } => "[]".to_string(),
            TyKind::Map { .. } => "{}".to_string(),
            TyKind::Fixed => "_decimal_.Decimal(0)".to_string(),
        }
    }

    pub fn field_default(&self, w: &PyWriter, ty: &Ty) -> String {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Primitive(prim) => primitive_default(*prim).to_string(),
            TyKind::String { .. } => "\"\"".to_string(),
            TyKind::Any | TyKind::Null => "None".to_string(),
            TyKind::Fixed => "_decimal_.Decimal(0)".to_string(),
            TyKind::Array { .. } | TyKind::Sequence { .. } => {
                "_dataclasses_.field(default_factory=list)".to_string()
            }
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

    fn needs_lambda_default(&self, w: &PyWriter, def_id: DefId) -> bool {
        let def = self.hir.context.type_of(def_id);
        if matches!(
            def.kind,
            DefKind::Enum(_) | DefKind::Bitmask(_) | DefKind::Const(_)
        ) {
            return true;
        }

        if let Some(module_id) = parent_module(self.hir, def_id) {
            w.import_context.module_imports.contains_key(&module_id)
        } else {
            false
        }
    }

    fn adt_default(&self, w: &PyWriter, def_id: DefId) -> Option<String> {
        let def = self.hir.context.type_of(def_id);
        match &def.kind {
            DefKind::Enum(enum_ty) => {
                let first = *enum_ty.fields.first()?;
                let first_def = self.hir.context.type_of(first);
                let enum_path = self.py_def(w, def_id);
                Some(format!("{}.{}", enum_path, first_def.ident.name))
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
