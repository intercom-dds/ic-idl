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

//! HIR rewriting utilities for replacing `DefId` references.
//!
//! This module provides functions to rewrite `DefId` references throughout the
//! HIR, typically used to resolve forward declarations by replacing them with
//! their actual definitions.

use std::collections::HashMap;

use crate::Context;
use crate::hir::{DefId, DefKind, Numeric, Ty, TyKind};

/// Replaces all `DefId` references in the context according to the given mapping.
///
/// Used to resolve forward declarations by replacing their `DefId`s with the
/// actual definitions.
pub fn replace_def_ids<S: std::hash::BuildHasher>(
    ctx: &mut Context,
    mapping: &HashMap<DefId, DefId, S>,
) {
    if mapping.is_empty() {
        return;
    }

    let all_defs: Vec<DefId> = ctx.definitions.iter().map(|(id, _)| id).collect();
    for def_id in all_defs {
        replace_def_ids_in_def(ctx, def_id, mapping);
    }
}

fn replace_def_ids_in_def<S: std::hash::BuildHasher>(
    ctx: &mut Context,
    def_id: DefId,
    mapping: &HashMap<DefId, DefId, S>,
) {
    let def = ctx.definitions.get_mut(def_id);

    match &mut def.kind {
        DefKind::Struct(s) => {
            if let Some(parent) = &mut s.parent
                && let Some(new_id) = mapping.get(parent)
            {
                *parent = *new_id;
            }

            for member in &mut s.members {
                replace_def_ids_in_ty(&mut member.ty, mapping);
            }
        }
        DefKind::Union(u) => {
            replace_def_ids_in_ty(&mut u.disc.ty, mapping);

            for variant in &mut u.variants {
                replace_def_ids_in_ty(&mut variant.ty, mapping);
            }
        }
        DefKind::Interface(i) => {
            for parent in &mut i.parents {
                if let Some(new_id) = mapping.get(parent) {
                    *parent = *new_id;
                }
            }

            for proto in &mut i.prototypes {
                replace_def_ids_in_ty(&mut proto.ty, mapping);
                replace_def_ids_in_spanned(&mut proto.raises, mapping);
                for param in &mut proto.params {
                    replace_def_ids_in_ty(&mut param.ty, mapping);
                }
            }

            for attr in &mut i.attributes {
                replace_def_ids_in_ty(&mut attr.ty, mapping);
                replace_def_ids_in_spanned(&mut attr.getraises, mapping);
                replace_def_ids_in_spanned(&mut attr.setraises, mapping);
            }
        }
        DefKind::Valuetype(v) => {
            if let Some(parent) = &mut v.parent
                && let Some(new_id) = mapping.get(parent)
            {
                *parent = *new_id;
            }

            if let Some(supports) = &mut v.supports
                && let Some(new_id) = mapping.get(supports)
            {
                *supports = *new_id;
            }

            for member in &mut v.members {
                replace_def_ids_in_ty(&mut member.ty, mapping);
            }

            for proto in &mut v.prototypes {
                replace_def_ids_in_ty(&mut proto.ty, mapping);
                replace_def_ids_in_spanned(&mut proto.raises, mapping);
                for param in &mut proto.params {
                    replace_def_ids_in_ty(&mut param.ty, mapping);
                }
            }

            for attr in &mut v.attributes {
                replace_def_ids_in_ty(&mut attr.ty, mapping);
                replace_def_ids_in_spanned(&mut attr.getraises, mapping);
                replace_def_ids_in_spanned(&mut attr.setraises, mapping);
            }
        }
        DefKind::Alias(a) => {
            replace_def_ids_in_ty(&mut a.ty, mapping);
        }
        DefKind::Const(c) => {
            replace_def_ids_in_ty(&mut c.ty, mapping);
            replace_def_ids_in_numeric(&mut c.value, mapping);
        }
        DefKind::Except(e) => {
            for member in &mut e.members {
                replace_def_ids_in_ty(&mut member.ty, mapping);
            }
        }
        DefKind::Enum(_)
        | DefKind::Bitmask(_)
        | DefKind::Bitset(_)
        | DefKind::Module(_)
        | DefKind::Decl(_)
        | DefKind::Annotation(_) => {}
    }
}

fn replace_def_ids_in_spanned<S: std::hash::BuildHasher>(
    refs: &mut [crate::hir::Spanned<DefId>],
    mapping: &HashMap<DefId, DefId, S>,
) {
    for def_id in refs {
        if let Some(new_id) = mapping.get(&def_id.value) {
            def_id.value = *new_id;
        }
    }
}

fn replace_def_ids_in_ty<S: std::hash::BuildHasher>(
    ty: &mut Ty,
    mapping: &HashMap<DefId, DefId, S>,
) {
    match &mut ty.kind {
        TyKind::Adt(def_id) => {
            if let Some(new_id) = mapping.get(def_id) {
                *def_id = *new_id;
            }
        }
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
            replace_def_ids_in_ty(ty, mapping);
        }
        TyKind::Map { key, elem, .. } => {
            replace_def_ids_in_ty(key, mapping);
            replace_def_ids_in_ty(elem, mapping);
        }
        TyKind::Primitive(_)
        | TyKind::String { .. }
        | TyKind::Any
        | TyKind::Fixed
        | TyKind::Null => {}
    }
}

fn replace_def_ids_in_numeric<S: std::hash::BuildHasher>(
    numeric: &mut Numeric,
    mapping: &HashMap<DefId, DefId, S>,
) {
    match numeric {
        Numeric::Const(def_id) => {
            if let Some(new_id) = mapping.get(def_id) {
                *def_id = *new_id;
            }
        }
        Numeric::Array { ty, values } | Numeric::Sequence { ty, values } => {
            replace_def_ids_in_ty(ty, mapping);
            for value in values.iter_mut() {
                replace_def_ids_in_numeric(value, mapping);
            }
        }
        Numeric::Map {
            key,
            value,
            entries,
        } => {
            replace_def_ids_in_ty(key, mapping);
            replace_def_ids_in_ty(value, mapping);
            for (k, v) in entries.iter_mut() {
                replace_def_ids_in_numeric(k, mapping);
                replace_def_ids_in_numeric(v, mapping);
            }
        }
        Numeric::Struct { ty, fields } => {
            if let Some(new_id) = mapping.get(ty) {
                *ty = *new_id;
            }
            for field_value in fields.iter_mut() {
                replace_def_ids_in_numeric(field_value, mapping);
            }
        }
        Numeric::Union {
            ty,
            discriminant,
            field_index: _,
            value,
        } => {
            if let Some(new_id) = mapping.get(ty) {
                *ty = *new_id;
            }
            replace_def_ids_in_numeric(discriminant, mapping);
            replace_def_ids_in_numeric(value, mapping);
        }
        Numeric::Null
        | Numeric::Bool(_)
        | Numeric::Char(_)
        | Numeric::Int8(_)
        | Numeric::UInt8(_)
        | Numeric::Int16(_)
        | Numeric::UInt16(_)
        | Numeric::Int32(_)
        | Numeric::UInt32(_)
        | Numeric::Int64(_)
        | Numeric::UInt64(_)
        | Numeric::Float(_)
        | Numeric::Double(_)
        | Numeric::String(_) => {}
    }
}
