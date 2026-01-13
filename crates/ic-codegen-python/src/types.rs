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
use ic_hir::hir::{DefId, PrimitiveTy, Ty, TyKind};

pub fn needs_decimal(hir: &ResolvedGraph, ty: &Ty) -> bool {
    let resolved = hir.context.resolve_ty(ty);
    match &resolved.kind {
        TyKind::Primitive(PrimitiveTy::Float128) | TyKind::Fixed => true,
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => needs_decimal(hir, ty),
        TyKind::Map { key, elem, .. } => needs_decimal(hir, key) || needs_decimal(hir, elem),
        _ => false,
    }
}

pub fn primitive_type(prim: PrimitiveTy) -> &'static str {
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

pub fn primitive_default(prim: PrimitiveTy) -> &'static str {
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

#[allow(clippy::only_used_in_recursion)]
pub fn py_type(hir: &ResolvedGraph, ty: &Ty, relative_def: DefId) -> String {
    let resolved = hir.context.resolve_ty(ty);
    match &resolved.kind {
        TyKind::Primitive(prim) => primitive_type(*prim).to_string(),
        TyKind::String { .. } => "str".to_string(),
        TyKind::Adt(def_id) => {
            let def = hir.context.definitions.get(*def_id);
            def.ident.name.clone()
        }
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
            let inner = py_type(hir, ty, relative_def);
            format!("list[{inner}]")
        }
        TyKind::Map { key, elem, .. } => {
            let key_ty = py_type(hir, key, relative_def);
            let elem_ty = py_type(hir, elem, relative_def);
            format!("dict[{key_ty}, {elem_ty}]")
        }
        TyKind::Any => "object".to_string(),
        TyKind::Fixed => "_decimal_.Decimal".to_string(),
        TyKind::Null => "None".to_string(),
    }
}

pub fn default_value(hir: &ResolvedGraph, ty: &Ty, relative_def: DefId) -> String {
    let resolved = hir.context.resolve_ty(ty);
    match &resolved.kind {
        TyKind::Primitive(prim) => primitive_default(*prim).to_string(),
        TyKind::String { .. } => "\"\"".to_string(),
        TyKind::Adt(_) => {
            let type_name = py_type(hir, ty, relative_def);
            format!("_dataclasses_.field(default_factory={type_name})")
        }
        TyKind::Any | TyKind::Null => "None".to_string(),
        TyKind::Array { .. } | TyKind::Sequence { .. } => {
            "_dataclasses_.field(default_factory=list)".to_string()
        }
        TyKind::Map { .. } => "_dataclasses_.field(default_factory=dict)".to_string(),
        TyKind::Fixed => "_decimal_.Decimal(0)".to_string(),
    }
}
