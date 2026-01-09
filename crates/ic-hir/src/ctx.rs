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

use std::collections::HashSet;

use ic_alloc::arena::Arena;

use crate::hir::{self, Def, DefId, DefKind, Numeric, Ty, TyKind};
use crate::scope::ScopeTree;

#[must_use]
#[derive(Clone, Debug)]
pub struct Context {
    pub definitions: Arena<hir::Def>,

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
    pub fn new() -> Self {
        Self {
            definitions: Arena::default(),
            scopes: ScopeTree::new(),
        }
    }

    /// Returns the underlying type definition of the given definition.
    ///
    /// This will resolve any intermediate typedefs.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    pub fn base_def_of(&self, id: DefId) -> &Def {
        self.type_of(self.base_id_of(id))
    }

    /// Returns the underlying `DefId` of the given definition, resolving any
    /// intermediate typedefs.
    ///
    /// This is similar to [`base_def_of`](Self::base_def_of) but returns the
    /// `DefId` instead of the `Def`.
    pub fn base_id_of(&self, mut id: DefId) -> DefId {
        loop {
            let def = self.type_of(id);
            match &def.kind {
                DefKind::Alias(v) => match v.ty.kind {
                    TyKind::Adt(next_id) => {
                        id = next_id;
                    }
                    _ => break,
                },
                DefKind::Const(v) => match v.ty.kind {
                    TyKind::Adt(next_id) => {
                        id = next_id;
                    }
                    _ => break,
                },
                _ => break,
            }
        }
        id
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

    /// Similar to `type_of`, but will resolve the underlying type.
    ///
    /// # Panics
    ///
    /// Panics if the given type ID does not exist, or if the ID came from a
    /// different arena. This can only ever happen if there are multiple
    /// `Context`s whose arenas have been mixed up.
    pub fn base_type_of(&self, mut id: DefId) -> Ty {
        loop {
            let ty = self.type_of(id);
            match &ty.kind {
                DefKind::Alias(v) => match v.ty.kind {
                    TyKind::Adt(next_id) => {
                        id = next_id;
                    }
                    _ => return v.ty.clone(),
                },
                DefKind::Const(v) => match v.ty.kind {
                    TyKind::Adt(next_id) => {
                        id = next_id;
                    }
                    _ => return v.ty.clone(),
                },
                _ => {
                    return Ty {
                        kind: TyKind::Adt(id),
                        span: ty.span,
                    };
                }
            }
        }
    }

    /// Resolves a type through aliases to get the underlying type.
    pub fn resolve_ty(&self, ty: &Ty) -> Ty {
        match &ty.kind {
            TyKind::Adt(id) => self.base_type_of(*id),
            _ => ty.clone(),
        }
    }

    /// Returns the root scope ID.
    pub fn root_scope(&self) -> crate::scope::ScopeId {
        self.scopes.root()
    }

    /// Resolves a name in the given scope.
    #[must_use]
    pub fn resolve_name(&self, scope: crate::scope::ScopeId, name: &str) -> Option<DefId> {
        self.scopes.resolve_name(scope, name)
    }

    /// Resolves a path starting from the given scope.
    #[must_use]
    pub fn resolve_path(&self, scope: crate::scope::ScopeId, path: &[&str]) -> Option<DefId> {
        self.scopes.resolve_path(scope, path)
    }

    /// Creates a child scope with the given name.
    pub fn create_child_scope(
        &mut self,
        parent: crate::scope::ScopeId,
        name: String,
        def_id: Option<DefId>,
    ) -> crate::scope::ScopeId {
        self.scopes.create_child_scope(parent, name, def_id)
    }

    /// Adds a definition to a scope.
    pub fn add_definition_to_scope(
        &mut self,
        scope: crate::scope::ScopeId,
        name: String,
        def_id: DefId,
    ) {
        self.scopes.add_definition(scope, name, def_id);
    }

    /// Adds an annotation definition to a scope.
    pub fn add_annotation_to_scope(
        &mut self,
        scope: crate::scope::ScopeId,
        name: &str,
        def_id: DefId,
    ) {
        self.scopes.add_annotation(scope, name, def_id);
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

    /// Returns the name of the given type.
    #[must_use]
    pub fn type_name(&self, ty: &Ty) -> String {
        match ty.kind {
            TyKind::Primitive(p) => p.name().to_string(),
            TyKind::String { .. } => "string".to_string(),
            TyKind::Sequence { .. } => "sequence".to_string(),
            TyKind::Array { .. } => "array".to_string(),
            TyKind::Map { .. } => "map".to_string(),
            TyKind::Any => "any".to_string(),
            TyKind::Fixed => "fixed".to_string(),
            TyKind::Null => "null".to_string(),
            TyKind::Adt(def_id) => self.type_of(def_id).ident.name.clone(),
        }
    }

    /// Returns the fully qualified name of the given definition.
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

    /// Looks up a symbol by its qualified name (e.g., `"DDS::XTypes"`).
    /// Always resolves from the root scope (absolute path).
    /// For reopened modules, returns the last (most recent) definition.
    #[must_use]
    pub fn lookup_symbol(&self, qualified_name: &str) -> Option<DefId> {
        let parts: Vec<&str> = qualified_name.split("::").collect();
        self.scopes
            .try_resolve_path(self.root_scope(), &parts, true)
            .ok()
    }

    /// Looks up all definitions for a module by its qualified name.
    /// This is useful for reopened modules which may have multiple `DefId`s.
    /// Returns empty vector if not found or if not a module.
    #[must_use]
    pub fn lookup_modules(&self, qualified_name: &str) -> Vec<DefId> {
        let parts: Vec<&str> = qualified_name.split("::").collect();
        self.scopes.lookup_all_modules(self.root_scope(), &parts)
    }

    /// Resolves a numeric value to a signed integer, recursively following
    /// `Const` references. Falls back to `0` if the value cannot be
    /// represented as an integer.
    #[must_use]
    pub fn integer_value(&self, numeric: &Numeric) -> i64 {
        self.unsigned_value(numeric) as i64
    }

    /// Resolves a numeric value to an unsigned integer, recursively following
    /// `Const` references. Falls back to `0` if the value cannot be
    /// represented as an unsigned integer.
    #[must_use]
    pub fn unsigned_value(&self, numeric: &Numeric) -> u64 {
        match numeric {
            Numeric::UInt8(v) => u64::from(*v),
            Numeric::UInt16(v) => u64::from(*v),
            Numeric::UInt32(v) => u64::from(*v),
            Numeric::UInt64(v) => *v,
            Numeric::Int8(v) => i64::from(*v) as u64,
            Numeric::Int16(v) => i64::from(*v) as u64,
            Numeric::Int32(v) => i64::from(*v) as u64,
            Numeric::Int64(v) => *v as u64,
            Numeric::Const(def_id) => {
                let def = self.type_of(*def_id);
                if let DefKind::Const(const_def) = &def.kind {
                    self.unsigned_value(&const_def.value)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Resolves a numeric value to a string, recursively following `Const`
    /// references. Returns `None` if the value cannot be represented as a
    /// string.
    #[must_use]
    pub fn string_value(&self, numeric: &Numeric) -> Option<String> {
        match numeric {
            Numeric::Bool(v) => Some(v.to_string()),
            Numeric::Char(v) => Some(v.to_string()),
            Numeric::UInt8(v) => Some(v.to_string()),
            Numeric::UInt16(v) => Some(v.to_string()),
            Numeric::UInt32(v) => Some(v.to_string()),
            Numeric::UInt64(v) => Some(v.to_string()),
            Numeric::Int8(v) => Some(v.to_string()),
            Numeric::Int16(v) => Some(v.to_string()),
            Numeric::Int32(v) => Some(v.to_string()),
            Numeric::Int64(v) => Some(v.to_string()),
            Numeric::Const(def_id) => {
                let def = self.type_of(*def_id);
                if let DefKind::Const(const_def) = &def.kind {
                    self.string_value(&const_def.value)
                } else {
                    None
                }
            }
            Numeric::Float(v) => Some(v.to_string()),
            Numeric::Double(v) => Some(v.to_string()),
            Numeric::String(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Returns all `DefId`s referenced by a definition.
    ///
    /// This includes types used in members, parent types, interface parents,
    /// constant references in union labels, etc. It does not include children
    /// (e.g., module definitions or enum fields).
    #[must_use]
    pub fn deps(&self, def_id: DefId) -> HashSet<DefId> {
        self.deps_where(def_id, |_| true)
    }

    /// Returns all `DefId`s referenced by a definition, filtered by a predicate.
    ///
    /// Only includes references where `include(def)` returns true. This allows
    /// backends to exclude types they don't support (e.g., valuetypes).
    #[must_use]
    pub fn deps_where<F>(&self, def_id: DefId, include: F) -> HashSet<DefId>
    where
        F: Fn(&Def) -> bool,
    {
        DepCollector { ctx: self }.deps_where(def_id, include)
    }
}

struct DepCollector<'a> {
    ctx: &'a Context,
}

impl DepCollector<'_> {
    fn deps_where<F>(&self, def_id: DefId, include: F) -> HashSet<DefId>
    where
        F: Fn(&Def) -> bool,
    {
        let mut deps = HashSet::new();
        let def = self.ctx.type_of(def_id);

        match &def.kind {
            DefKind::Struct(s) => {
                self.insert_if(s.parent, &include, &mut deps);
                for m in &s.members {
                    self.collect_ty_refs(&m.ty, &include, &mut deps);
                }
            }
            DefKind::Union(u) => {
                self.collect_ty_refs(&u.disc.ty, &include, &mut deps);
                for v in &u.variants {
                    self.collect_ty_refs(&v.ty, &include, &mut deps);
                    for label in &v.labels {
                        self.collect_numeric_refs(&label.value, &include, &mut deps);
                    }
                }
            }
            DefKind::Interface(i) => {
                self.extend_if(&i.parents, &include, &mut deps);
                for attr in &i.attributes {
                    self.collect_ty_refs(&attr.ty, &include, &mut deps);
                    self.extend_if(&attr.getraises, &include, &mut deps);
                    self.extend_if(&attr.setraises, &include, &mut deps);
                }
                for proto in &i.prototypes {
                    self.collect_ty_refs(&proto.ty, &include, &mut deps);
                    self.extend_if(&proto.raises, &include, &mut deps);
                    for param in &proto.params {
                        self.collect_ty_refs(&param.ty, &include, &mut deps);
                    }
                }
            }
            DefKind::Valuetype(v) => {
                self.insert_if(v.parent, &include, &mut deps);
                self.insert_if(v.supports, &include, &mut deps);
                for m in &v.members {
                    self.collect_ty_refs(&m.ty, &include, &mut deps);
                }
                for attr in &v.attributes {
                    self.collect_ty_refs(&attr.ty, &include, &mut deps);
                    self.extend_if(&attr.getraises, &include, &mut deps);
                    self.extend_if(&attr.setraises, &include, &mut deps);
                }
                for proto in &v.prototypes {
                    self.collect_ty_refs(&proto.ty, &include, &mut deps);
                    self.extend_if(&proto.raises, &include, &mut deps);
                    for param in &proto.params {
                        self.collect_ty_refs(&param.ty, &include, &mut deps);
                    }
                }
            }
            DefKind::Except(e) => {
                for m in &e.members {
                    self.collect_ty_refs(&m.ty, &include, &mut deps);
                }
            }
            DefKind::Alias(a) => {
                self.collect_ty_refs(&a.ty, &include, &mut deps);
            }
            DefKind::Const(c) => {
                self.collect_ty_refs(&c.ty, &include, &mut deps);
                self.collect_numeric_refs(&c.value, &include, &mut deps);
            }
            DefKind::Bitset(b) => {
                self.insert_if(b.parent, &include, &mut deps);
                for field in &b.fields {
                    self.collect_ty_refs(&field.ty, &include, &mut deps);
                }
            }
            DefKind::Annotation(a) => {
                for param in &a.params {
                    self.collect_ty_refs(&param.ty, &include, &mut deps);
                    if let Some(default) = &param.default {
                        self.collect_numeric_refs(default, &include, &mut deps);
                    }
                }
            }
            DefKind::Module(_) | DefKind::Enum(_) | DefKind::Bitmask(_) | DefKind::Decl(_) => {}
        }

        deps
    }

    fn insert_if<F>(&self, id: Option<DefId>, include: &F, deps: &mut HashSet<DefId>)
    where
        F: Fn(&Def) -> bool,
    {
        if let Some(id) = id
            && include(self.ctx.type_of(id))
        {
            deps.insert(id);
        }
    }

    fn extend_if<F>(&self, ids: &[DefId], include: &F, deps: &mut HashSet<DefId>)
    where
        F: Fn(&Def) -> bool,
    {
        for &id in ids {
            if include(self.ctx.type_of(id)) {
                deps.insert(id);
            }
        }
    }

    fn collect_ty_refs<F>(&self, ty: &Ty, include: &F, deps: &mut HashSet<DefId>)
    where
        F: Fn(&Def) -> bool,
    {
        match &ty.kind {
            TyKind::Adt(def_id) => {
                if include(self.ctx.type_of(*def_id)) {
                    deps.insert(*def_id);
                }
            }
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                self.collect_ty_refs(ty, include, deps);
            }
            TyKind::Map { key, elem, .. } => {
                self.collect_ty_refs(key, include, deps);
                self.collect_ty_refs(elem, include, deps);
            }
            _ => {}
        }
    }

    fn collect_numeric_refs<F>(&self, numeric: &Numeric, include: &F, deps: &mut HashSet<DefId>)
    where
        F: Fn(&Def) -> bool,
    {
        match numeric {
            Numeric::Const(def_id) => {
                if include(self.ctx.type_of(*def_id)) {
                    deps.insert(*def_id);
                }
            }
            Numeric::Array { values, .. } | Numeric::Sequence { values, .. } => {
                for v in &**values {
                    self.collect_numeric_refs(v, include, deps);
                }
            }
            Numeric::Map { entries, .. } => {
                for (k, v) in &**entries {
                    self.collect_numeric_refs(k, include, deps);
                    self.collect_numeric_refs(v, include, deps);
                }
            }
            Numeric::Struct { ty, fields } => {
                if include(self.ctx.type_of(*ty)) {
                    deps.insert(*ty);
                }
                for v in &**fields {
                    self.collect_numeric_refs(v, include, deps);
                }
            }
            Numeric::Union {
                ty,
                discriminant,
                value,
                ..
            } => {
                if include(self.ctx.type_of(*ty)) {
                    deps.insert(*ty);
                }
                self.collect_numeric_refs(discriminant, include, deps);
                self.collect_numeric_refs(value, include, deps);
            }
            _ => {}
        }
    }
}
