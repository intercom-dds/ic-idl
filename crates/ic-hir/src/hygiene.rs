// Copyright 2024 KONGSBERG
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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_alloc::insensitive::CaseSet;

use crate::Context;
use crate::hir::{
    BitmaskTy, Def, DefId, DefKind, EnumTy, ExceptTy, Ident, InterfaceTy, ProtoTy, StructTy,
    UnionTy,
};
use crate::visit::{Visitor, walk_def};

struct Hygiene<'a> {
    ctx: &'a Context,
}

impl Hygiene<'_> {
    fn check_def<'a>(elems: impl Iterator<Item = &'a Ident>) {
        let mut seen = CaseSet::default();
        Self::with_set(&mut seen, elems)
    }

    fn with_set<'a>(seen: &mut CaseSet<'a>, elems: impl Iterator<Item = &'a Ident>) {
        for ident in elems {
            if !seen.insert(ident.name.as_str()) {
                tracing::error!("collision occurred: {}", ident.name);
            }
        }
    }
}

impl<'a> Visitor<'a> for Hygiene<'_> {
    fn visit_struct(&mut self, def: &'a Def, data: &'a StructTy) {
        let mut seen = CaseSet::default();

        // Iterate over members of all parents first
        let mut parent_id = data.parent;
        while let Some(parent) = parent_id {
            let parent = self.ctx.type_of(parent);
            // TODO: move this to a separate function and re-use for
            // valuetype/interfaces.
            match &parent.kind {
                DefKind::Struct(v) => {
                    Self::with_set(&mut seen, v.members.iter().map(|v| &v.ident));
                    parent_id = v.parent;
                }
                _ => (),
            }
        }

        Self::with_set(&mut seen, data.members.iter().map(|v| &v.ident));
    }

    fn visit_except(&mut self, def: &'a Def, data: &'a ExceptTy) {
        Self::check_def(data.members.iter().map(|v| &v.ident));
    }

    fn visit_enum(&mut self, def: &'a Def, data: &'a EnumTy) {
        Self::check_def(data.fields.iter().map(|v| &v.ident));
    }

    fn visit_bitmask(&mut self, def: &'a Def, data: &'a BitmaskTy) {
        Self::check_def(data.flags.iter().map(|v| &v.ident));
    }

    fn visit_union(&mut self, def: &'a Def, data: &'a UnionTy) {
        // TODO: check for duplicated case labels too?
        Self::check_def(data.variants.iter().map(|v| &v.ident));
    }

    fn visit_interface(&mut self, def: &'a Def, data: &'a InterfaceTy) {
        // TODO: attributes and nested items
        Self::check_def(data.prototypes.iter().map(|v| &v.ident));
    }
}

pub fn check(ctx: &Context, order: &[DefId]) {
    let mut hygiene = Hygiene { ctx };
    for id in order {
        let def = ctx.type_of(*id);
        walk_def(&mut hygiene, def);
    }
}
