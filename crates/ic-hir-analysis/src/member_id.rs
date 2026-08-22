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

use ic_alloc::md5;
use ic_hir::Context;
use ic_hir::hir::{Ann, Def, DefId, DefKind, Member};

use crate::annotation::builtin_annotation;

const MEMBER_ID_MASK: u32 = 0x0fff_ffff;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Autoid {
    #[default]
    Sequential,
    Hash,
}

#[must_use]
pub fn effective_autoid(ctx: &Context, def: &Def) -> Autoid {
    if !matches!(
        def.kind,
        DefKind::Struct(_) | DefKind::Union(_) | DefKind::Except(_) | DefKind::Valuetype(_)
    ) {
        return Autoid::Sequential;
    }

    if let Some(autoid) = autoid(&def.annotations, ctx) {
        return autoid;
    }

    let mut parent = def.parent;
    while let Some(parent_id) = parent {
        let parent_def = ctx.type_of(parent_id);
        if matches!(parent_def.kind, DefKind::Module(_))
            && let Some(autoid) = autoid(&parent_def.annotations, ctx)
        {
            return autoid;
        }
        parent = parent_def.parent;
    }

    Autoid::Sequential
}

#[must_use]
pub fn member_ids(ctx: &Context, def_id: DefId) -> Vec<u32> {
    let def = ctx.type_of(def_id);
    match &def.kind {
        DefKind::Struct(_) => {
            let mut ids = vec![];
            struct_member_ids(ctx, def, &mut ids);
            ids
        }
        DefKind::Union(union_ty) => {
            let autoid = effective_autoid(ctx, def);
            let mut current = 0;
            std::iter::once(current)
                .chain(union_ty.variants.iter().map(|variant| {
                    current = assign_member_id(
                        ctx,
                        autoid,
                        current,
                        &variant.ident.name,
                        &variant.annotations,
                    );
                    current
                }))
                .collect()
        }
        DefKind::Except(except_ty) => {
            let mut ids = vec![];
            append_member_ids(
                ctx,
                effective_autoid(ctx, def),
                u32::MAX,
                &except_ty.members,
                &mut ids,
            );
            ids
        }
        DefKind::Valuetype(_) => {
            let mut ids = vec![];
            valuetype_member_ids(ctx, def, &mut ids);
            ids
        }
        _ => vec![],
    }
}

fn valuetype_member_ids(ctx: &Context, def: &Def, ids: &mut Vec<u32>) -> u32 {
    let DefKind::Valuetype(valuetype) = &def.kind else {
        return u32::MAX;
    };

    let current = valuetype.parent.map_or(u32::MAX, |parent| {
        valuetype_member_ids(ctx, ctx.type_of(parent.def_id), ids)
    });

    append_member_ids(
        ctx,
        effective_autoid(ctx, def),
        current,
        &valuetype.members,
        ids,
    )
}

fn struct_member_ids(ctx: &Context, def: &Def, ids: &mut Vec<u32>) -> u32 {
    let DefKind::Struct(struct_ty) = &def.kind else {
        return u32::MAX;
    };

    let current = struct_ty.parent.map_or(u32::MAX, |parent| {
        struct_member_ids(ctx, ctx.type_of(parent.def_id), ids)
    });
    append_member_ids(
        ctx,
        effective_autoid(ctx, def),
        current,
        &struct_ty.members,
        ids,
    )
}

fn append_member_ids(
    ctx: &Context,
    autoid: Autoid,
    mut current: u32,
    members: &[Member],
    ids: &mut Vec<u32>,
) -> u32 {
    for member in members {
        current = assign_member_id(
            ctx,
            autoid,
            current,
            &member.ident.name,
            &member.annotations,
        );
        ids.push(current);
    }

    current
}

#[allow(clippy::cast_possible_truncation)]
fn assign_member_id(
    ctx: &Context,
    autoid: Autoid,
    current: u32,
    name: &str,
    annotations: &[Ann],
) -> u32 {
    if let Some(id) = builtin_annotation(ctx, annotations, "id") {
        return id
            .args
            .first()
            .map_or(0, |arg| ctx.unsigned_value(&arg.value) as u32);
    }

    if let Some(hashid) = builtin_annotation(ctx, annotations, "hashid") {
        let hash_name = hashid
            .args
            .first()
            .and_then(|arg| ctx.string_value(&arg.value))
            .filter(|value| !value.is_empty());

        return hash_member_name(hash_name.as_deref().unwrap_or(name));
    }

    match autoid {
        Autoid::Sequential => current.wrapping_add(1),
        Autoid::Hash => hash_member_name(name),
    }
}

fn hash_member_name(name: &str) -> u32 {
    let digest = md5::digest(name);
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]) & MEMBER_ID_MASK
}

fn autoid(annotations: &[Ann], ctx: &Context) -> Option<Autoid> {
    let annotation = builtin_annotation(ctx, annotations, "autoid")?;
    let autoid = annotation.args.first().map_or(Autoid::Hash, |arg| {
        if ctx.unsigned_value(&arg.value) == 1 {
            Autoid::Hash
        } else {
            Autoid::Sequential
        }
    });

    Some(autoid)
}
