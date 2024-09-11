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

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

use ic_syntax::{AnnotationDef, AnnotationField, Expr};

use crate::{Context, Def, DefId};

/// Wrapper around `String` that performs case-insensitive hashing of the
/// underlying string.
#[derive(Debug, Eq)]
pub struct Lc(pub String);

impl PartialEq for Lc {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl std::hash::Hash for Lc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state)
    }
}

#[derive(Default)]
pub struct Resolver {
    // TODO: insert decl and then replace with def?
    pub definitions: HashMap<Lc, DefId>,
}

impl Resolver {
    pub fn declare(&mut self, _scope: Option<DefId>, _def: &Def) {
        // Already defined? Check kind
        // if self.definitions.get(&(_def.ident.name.clone())).is_some() {
        // TODO: check kind
        // }
        todo!()
    }

    pub fn define(&mut self, _scope: Option<DefId>, def: &Def) {
        let lc = Lc(def.ident.name.clone());
        match self.definitions.entry(lc) {
            Entry::Occupied(v) => {
                tracing::error!(
                    "duplicate registration of `{}`, first registered as `{}`",
                    def.ident.name,
                    v.key().0,
                );
            }
            Entry::Vacant(v) => {
                tracing::info!("registered type");
                v.insert(def.id);
            }
        }
    }
}

/// Determines if two annotation definitions are consistent. The standard
/// doesn't clarify what "consistent" means, but I've interpreted it as the two
/// definitions being identical.
fn is_consistent(ctx: &mut Context, lhs: &AnnotationDef, rhs: &AnnotationDef) -> bool {
    if !lhs.ident.name.eq_ignore_ascii_case(&rhs.ident.name) || lhs.params.len() != rhs.params.len()
    {
        return false;
    }

    lhs.params.iter().zip(rhs.params.iter()).all(|v| match v {
        (AnnotationField::Member(lhs), AnnotationField::Member(rhs)) => {
            decl_consistent(ctx, &lhs.names, &rhs.names)
                && is_type_consistent(ctx, &lhs.ty, &rhs.ty)
        }
        // (AnnotationField::Const(lhs), AnnotationField::Const(rhs)) => {
        //     // TODO: check value
        //     lhs.ident.name.eq_ignore_ascii_case(&rhs.ident.name)
        //         && is_type_consistent(ctx, &lhs.ty, &rhs.ty)
        // }
        (lhs, rhs) => lhs.disc() == rhs.disc(),
    })
}

/// Determines if two sets of declarators are semantically consistent. They
/// must resolve to the same types with the same bounds for them to be
/// considered consistent.
fn decl_consistent(
    ctx: &mut Context,
    lhs: &[ic_syntax::Declarator],
    rhs: &[ic_syntax::Declarator],
) -> bool {
    use ic_syntax::Declarator;

    lhs.iter().zip(rhs.iter()).all(|v| match v {
        (Declarator::Simple(lhs), Declarator::Simple(rhs)) => {
            lhs.name.eq_ignore_ascii_case(&rhs.name)
        }
        (Declarator::Array(lhs), Declarator::Array(rhs)) => {
            // TODO: should check each expr
            lhs.bounds.len() == rhs.bounds.len()
                && lhs.ident.name.eq_ignore_ascii_case(&rhs.ident.name)
        }
        _ => false,
    })
}

/// Determines if two types are semantically consistent. Collection types are
/// treated as consistent if they have the same bound and resolve to the same
/// element type.
fn is_type_consistent(ctx: &mut Context, lhs: &ic_syntax::Type, rhs: &ic_syntax::Type) -> bool {
    use ic_syntax::Type;

    // TODO: eval and check bounds
    match (lhs, rhs) {
        (Type::Sequence(lhs), Type::Sequence(rhs)) => {
            is_type_consistent(ctx, lhs.ty.as_ref(), rhs.ty.as_ref())
        }
        (Type::String(lhs), Type::String(rhs)) => lhs.wide == rhs.wide,
        (Type::Map(lhs), Type::Map(rhs)) => {
            is_type_consistent(ctx, lhs.key.as_ref(), rhs.key.as_ref())
                && is_type_consistent(ctx, lhs.value.as_ref(), rhs.value.as_ref())
        }
        (Type::Path(lhs), Type::Path(rhs)) => ctx.resolve_path(lhs) == ctx.resolve_path(rhs),
        _ => lhs.disc() == rhs.disc(),
    }
}
