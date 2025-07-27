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

use crate::hir::{self, Def, DefId, DefKind, Ty, TyKind};
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

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Creates a new context.
    #[must_use]
    pub fn new() -> Self {
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
    #[must_use]
    pub fn type_of(&self, id: DefId) -> &Def {
        self.definitions.get(id)
    }

    /// Try to get a definition without panicking.
    #[must_use]
    pub fn try_get(&self, id: DefId) -> Option<&Def> {
        let index: usize = id.into();
        if index < self.definitions.len() {
            Some(self.definitions.get(id))
        } else {
            None
        }
    }

    /// Similar to `type_of`, but will resolve the underlying type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    #[must_use]
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
    #[must_use]
    #[allow(clippy::only_used_in_recursion)]
    pub fn def_of(&self, ty: &Ty) -> Option<DefId> {
        match &ty.kind {
            TyKind::Array { ty, .. }
            | TyKind::Sequence { ty, .. }
            | TyKind::Map { elem: ty, .. } => self.def_of(ty),
            TyKind::Adt(id) => Some(*id),
            _ => None,
        }
    }

    #[must_use]
    pub fn qualified_name(&self, id: DefId) -> String {
        let def = self.type_of(id);
        let mut parts = vec![def.ident.name.clone()];
        let mut current = def.parent;
        while let Some(parent_id) = current {
            let parent_def = self.type_of(parent_id);
            parts.push(parent_def.ident.name.clone());
            current = parent_def.parent;
        }

        parts.reverse();
        parts.join("::")
    }
}
