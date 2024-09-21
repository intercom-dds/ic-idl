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

use ic_alloc::arena::Arena;
use ic_alloc::insensitive::CaseMap;
use ic_syntax::util::{path_name, type_name};

use crate::resolve::Lc;
use crate::{hir, Def, DefId, PrimitiveTy, TypeId};

// TODO: should a Type point to the definition instead?
//
// So type {
//     def_id: Id,
//     ??? whatever else
// }
#[derive(Debug)]
pub struct Type;

// No reason for the namespace to not be monomorphed -- we retain module-level
// definitions, so this is just for type resolution (which is fine!).
#[derive(Debug, Default)]
pub struct Namespace {
    /// All symbols defined in the current namespace.
    pub symbols: CaseMap<'static, DefId>,
}

#[derive(Debug)]
pub struct Context {
    pub types: Arena<Type>,
    pub definitions: Arena<hir::Def>,

    // Qualified type name => Type ID
    pub symbols: HashMap<String, TypeId>,
    pub registered: CaseMap<'static, DefId>,

    pub namespaces: CaseMap<'static, Namespace>,
}

impl Context {
    /// Creates a new context where primitive types and built-in annotations
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
            symbols: HashMap::new(),
            registered: CaseMap::default(),
            namespaces: CaseMap::default(),
        }
    }

    /// # Panics
    ///
    /// Panics if the given type ID does not exist. This can only ever happen
    /// if there are multiple `Context`s whose arenas have been mixed up.
    // pub fn ty(&self, id: Id<Type>) -> &Type {
    //     self.try_ty(id)
    //         .expect(&format!("type {id:?} does not exist"))
    // }

    /// # Panics
    // pub fn try_ty(&self, id: Id<Type>) -> Option<&Type> {
    //     self.arena.get(id)
    // }

    pub fn primitive_type(&self, _kind: PrimitiveTy) -> &Type {
        todo!()
    }

    // pub fn lookup_type(&self, name: &str) -> Option<&Type> {
    //     let ty = self.types.get(name)?;
    //     self.try_ty(*ty)
    // }

    // fn register_type<I>(&mut self, name: I, ty: Type) -> TypeId
    // where
    //     I: Into<String>,
    // {
    //     let name = name.into();
    //     tracing::info!("registering type {name}: {ty:?}");
    //
    //     match self.symbols.entry(name) {
    //         Entry::Occupied(v) => {
    //             panic!("type {} was registered multiple times", v.key());
    //         }
    //         Entry::Vacant(v) => {
    //             let id = self.types.alloc(ty);
    //             v.insert(id);
    //             id
    //         }
    //     }
    // }

    /// Returns the type definition of the specified type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist. This can only ever happen
    /// if there are multiple `Context`s whose arenas have been mixed up.
    pub fn type_of(&self, id: TypeId) -> &Def {
        self.definitions.get(id)
        // self.types
        //     .get(ty)
        //     .unwrap_or_else(|| panic!("type {ty:?} does not exist"))
    }

    pub fn base_type_of(&self, _ty: TypeId) -> &Def {
        todo!()
        // self.types
        //     .get(ty)
        //     .unwrap_or_else(|| panic!("type {ty:?} does not exist"))
    }

    /// Similar to `type_of`, but will resolve the underlying type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist. This can only ever happen
    /// if there are multiple `Context`s whose arenas have been mixed up.
    // pub fn base_type_of(&self, ty: TypeId) -> &Type {
    //     let ty = self.type_of(ty);
    //     if let Type::Alias(alias) = ty {
    //         self.type_of(alias.ty)
    //     } else {
    //         ty
    //     }
    // }

    pub fn declare(&mut self, scope: Option<DefId>, def: &Def) {
        let name = def.ident.name.clone();
        // TODO: check that kind + capitalization matches
        self.registered.insert(name, def.id);
    }

    pub fn define(&mut self, scope: Option<DefId>, def: DefId) {
        let def = self.definitions.get(def);
        match self.registered.entry(def.ident.name.clone()) {
            Entry::Occupied(v) => {
                tracing::error!(
                    "duplicate registration of `{}`, first registered as `{}`",
                    def.ident.name,
                    v.key(),
                );
            }
            Entry::Vacant(v) => {
                tracing::info!("registered type");
                v.insert(def.id);
            }
        }
    }

    // TODO: handle this in `Resolver` instead -- we should only operate on IDs.
    pub fn resolve_type(&self, name: &ic_syntax::Type) -> TypeId {
        let name = type_name(name);
        let id = *self.symbols.get(&name).expect("unknown type");
        tracing::trace!("resolving type `{name}` => {id:?}");
        id
    }

    pub fn resolve_path(&self, path: &ic_syntax::Path) -> TypeId {
        let name = path_name(path);
        let ty = *self.symbols.get(&name).expect("unknown type");
        tracing::trace!("resolving path `{name}` => {ty:?}");
        ty
    }

    // TODO: or should it be TypeId? that must be a ConstTy?
    // fn resolve_const(&self, path: &ic_syntax::Path) -> Type {
    //     todo!()
    //     // let name = path_name(path);
    //     // *self.symbols.get(&name).expect("unknown const")
    // }
}

/// Inserts primitive types and built-in annotations into the context.
fn init_ctx_state(ctx: &mut Context) {
    for ty in PrimitiveTy::iter() {
        // let name = name.into();
        // tracing::info!("registering type {name}: {ty:?}");
        // ctx.register_type(ty.name(), Type::Primitive(ty));
    }
}
