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
use crate::hir::{Ann, DefId, DefKind, Numeric, Ty, TyKind};

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

/// Replaces every `DefId` reference in a definition according to the mapping:
/// type/inheritance references, child-definition lists, enum fields, bitmask
/// flags, annotation parameters, and def-level annotations. Ids not present
/// in the mapping are left untouched.
pub fn replace_all_def_ids_in_def<S: std::hash::BuildHasher>(
    ctx: &mut Context,
    def_id: DefId,
    mapping: &HashMap<DefId, DefId, S>,
) {
    replace_def_ids_in_def(ctx, def_id, mapping);

    let def = ctx.definitions.get_mut(def_id);

    for ann in &mut def.annotations {
        replace_def_ids_in_ann(ann, mapping);
    }

    match &mut def.kind {
        DefKind::Module(m) => replace_def_ids_in_list(&mut m.definitions, mapping),
        DefKind::Interface(i) => replace_def_ids_in_list(&mut i.definitions, mapping),
        DefKind::Enum(e) => replace_def_ids_in_list(&mut e.fields, mapping),
        DefKind::Bitmask(b) => replace_def_ids_in_list(&mut b.flags, mapping),
        DefKind::Annotation(a) => {
            replace_def_ids_in_list(&mut a.types, mapping);
            for param in &mut a.params {
                replace_def_ids_in_ty(&mut param.ty, mapping);
                if let Some(default) = &mut param.default {
                    replace_def_ids_in_numeric(default, mapping);
                }
            }
        }
        DefKind::Bitset(b) => {
            replace_def_ids_in_spanned(&mut b.parent, mapping);
            for field in &mut b.fields {
                for ann in &mut field.annotations {
                    replace_def_ids_in_ann(ann, mapping);
                }
            }
        }
        DefKind::Struct(s) => {
            for member in &mut s.members {
                for ann in &mut member.annotations {
                    replace_def_ids_in_ann(ann, mapping);
                }
            }
        }
        DefKind::Except(e) => {
            for member in &mut e.members {
                for ann in &mut member.annotations {
                    replace_def_ids_in_ann(ann, mapping);
                }
            }
        }
        DefKind::Union(u) => {
            for ann in &mut u.disc.annotations {
                replace_def_ids_in_ann(ann, mapping);
            }
            for variant in &mut u.variants {
                for ann in &mut variant.annotations {
                    replace_def_ids_in_ann(ann, mapping);
                }
                for label in &mut variant.labels {
                    replace_def_ids_in_numeric(&mut label.value, mapping);
                }
            }
        }
        DefKind::Valuetype(v) => {
            replace_def_ids_in_list(&mut v.definitions, mapping);
            for member in &mut v.members {
                for ann in &mut member.annotations {
                    replace_def_ids_in_ann(ann, mapping);
                }
            }
        }
        DefKind::Alias(_) | DefKind::Const(_) | DefKind::Decl(_) => {}
    }
}

fn replace_def_ids_in_ann<S: std::hash::BuildHasher>(
    ann: &mut Ann,
    mapping: &HashMap<DefId, DefId, S>,
) {
    if let Some(def_id) = &mut ann.def_id
        && let Some(new_id) = mapping.get(def_id)
    {
        *def_id = *new_id;
    }

    for arg in &mut ann.args {
        replace_def_ids_in_numeric(&mut arg.value, mapping);
        if let Some(ty) = &mut arg.ty {
            replace_def_ids_in_ty(ty, mapping);
        }
    }
}

fn replace_def_ids_in_list<S: std::hash::BuildHasher>(
    ids: &mut [DefId],
    mapping: &HashMap<DefId, DefId, S>,
) {
    for id in ids {
        if let Some(new_id) = mapping.get(id) {
            *id = *new_id;
        }
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
            replace_def_ids_in_spanned(&mut s.parent, mapping);

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
            replace_def_ids_in_spanned(&mut i.parents, mapping);

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
            replace_def_ids_in_spanned(&mut v.parent, mapping);
            replace_def_ids_in_spanned(&mut v.supports, mapping);

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

fn replace_def_ids_in_spanned<'a, I, S>(refs: I, mapping: &HashMap<DefId, DefId, S>)
where
    I: IntoIterator<Item = &'a mut crate::hir::Spanned<DefId>>,
    S: std::hash::BuildHasher,
{
    for def_id in refs {
        if let Some(new_id) = mapping.get(&def_id.def_id) {
            def_id.def_id = *new_id;
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
        | Numeric::WChar(_)
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
        | Numeric::String(_)
        | Numeric::WString(_) => {}
    }
}
