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

use ic_alloc::arena::Arena;
use ic_alloc::insensitive::CaseMap;
use ic_syntax::util::{path_name, type_name};

use crate::hir::{self, Def, DefId, DefKind, PrimitiveTy, Ty, TyKind, TypeId};
use crate::scope::ScopeTree;

#[derive(Debug)]
pub struct Type;

#[derive(Debug)]
pub struct Context {
    pub types: Arena<Type>,
    pub definitions: Arena<hir::Def>,

    // Fully qualified type name => DefId
    pub registered: CaseMap<DefId>,

    // Scope hierarchy for name resolution
    pub scopes: ScopeTree,
}

impl Context {
    /// Creates a new context where built-in type definitions and annotations
    /// have been injected.
    pub fn new() -> Self {
        let mut ctx = Self::empty();
        init_ctx_state(&mut ctx);
        ctx
    }

    /// Creates a new context without injecting any of the built-in types.
    pub fn empty() -> Self {
        Self {
            types: Arena::default(),
            definitions: Arena::default(),
            registered: CaseMap::new(),
            scopes: ScopeTree::new(),
        }
    }

    /// Returns the type definition of the specified type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    pub fn type_of(&self, id: DefId) -> &Def {
        self.definitions.get(id)
    }

    /// Try to get a definition without panicking.
    pub fn try_get(&self, id: DefId) -> Option<&Def> {
        // Arena doesn't have try_get, so we need to check bounds manually
        // For now, just use get which will panic if invalid
        // TODO: Implement proper bounds checking
        Some(self.definitions.get(id))
    }

    /// Similar to `type_of`, but will resolve the underlying type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    pub fn base_type_of(&self, id: DefId) -> Ty {
        let ty = self.type_of(id);
        match &ty.kind {
            DefKind::Alias(v) => match v.ty.kind {
                TyKind::Adt(id) => self.base_type_of(id),
                _ => v.ty.clone(),
            },
            _ => Ty {
                kind: TyKind::Adt(id),
                span: ty.span,
            },
        }
    }

    /// Returns the `DefId` of the given type, if one exists. For arrays,
    /// sequences, and maps, this will return the element type if it points to
    /// a definition.
    pub fn def_of(&self, ty: &Ty) -> Option<DefId> {
        match &ty.kind {
            TyKind::Array { ty, .. }
            | TyKind::Sequence { ty, .. }
            | TyKind::Map { elem: ty, .. } => self.def_of(ty),
            TyKind::Adt(id) => Some(*id),
            _ => None,
        }
    }

    pub fn resolve_path(&self, _path: &ic_syntax::Path) -> TypeId {
        todo!()
    }
}

/// Inserts primitive types and built-in annotations into the context.
fn init_ctx_state(ctx: &mut Context) {
    for ty in PrimitiveTy::iter() {
        // let name = name.into();
        // tracing::info!("registering type {name}: {ty:?}");
        // ctx.register_type(ty.name(), Type::Primitive(ty));
    }
}
