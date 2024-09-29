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

use std::collections::{HashMap, HashSet};

use ic_alloc::arena::{self, Arena};
use ic_syntax::{AnnotationDef, AnnotationField, Ident, ItemKind, Path, Span, util};

use crate::Context;
use crate::hir::{Decl, Def, DefId};

pub type SymbolKind = ItemKind;

/// An ID of a lexical scope.
pub type ScopeId = arena::Id<Scope>;

pub type Result<T> = std::result::Result<T, ResolveError>;

#[derive(Debug)]
pub enum Symbol {
    /// Scope of an ADT. May contain nested symbols, but cannot contain
    /// nested definitions.
    Adt(DefId, SymbolKind),

    /// A declaration of a type.
    Decl(DefId, SymbolKind),

    /// A constant of some sort.
    Const,

    /// A unique lexical scope that may contain nested type definitions, but
    /// which cannot be redefined. Used for interfaces, valuetypes, and
    /// annotations.
    Lexical {
        def: DefId,
        scope: ScopeId,
        kind: SymbolKind,
    },

    /// A module which may contain nested type definitions. The ID maps to the
    /// arena in the resolver, which can be used to retrieve the lexical scope
    /// of the module.
    Module(ScopeId),
}

#[derive(Debug)]
pub enum ResolveError {
    /// The requested type failed to resolve because it was not defined.
    Undefined(Span),

    /// Type registration failed because another item with the same name
    /// already exists in the same scope.
    Redefined(Span),

    /// A mismatch between a declaration and definition occurred, for example
    /// if a type was declared as a `struct` but later defined as a `union`.
    DeclMismatch {
        decl: SymbolKind,
        def: SymbolKind,
        span: Span,
    },

    /// A part of the path resolved to a type, but there were superfluous
    /// segments. For example `a::b::c`, where `b` is a `struct`.
    Superfluous(Span),

    /// A path resolved to a module when a type was expected.
    Module(Span),
}

impl ResolveError {
    pub fn primary_span(&self) -> Span {
        match self {
            Self::Undefined(span)
            | Self::Redefined(span)
            | Self::Module(span)
            | Self::Superfluous(span)
            | Self::DeclMismatch { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    /// Name of the current scope.
    name: String,

    /// Symbols registered in the scope.
    symbols: HashMap<String, Symbol>,

    /// A set of IDs of forward declarations that have not yet been defined.
    /// Once defined, the corresponding ID will be removed from this set.
    decls: HashSet<String>,
}

impl Scope {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            symbols: HashMap::default(),
            decls: HashSet::default(),
        }
    }
}

/// Contains all the logic required to resolve all types and expressions in the
/// AST. This will keep track of the current scope, and provides functions for
/// resolving symbols and paths.
///
/// The resolver does not know anything about the definition of each type; it
/// only cares about the name and lexical scope.
#[derive(Debug)]
pub struct Resolver {
    /// Stores all the lexical scopes that have been defined, including the
    /// global scope.
    lexical_scopes: Arena<Scope>,

    /// Stack of the current scope and all parents. This will never be empty,
    /// as the first entry is guaranteed to be the global scope.
    current_scope: Vec<ScopeId>,

    /// Maps fully qualified names to each type's respective definition. This
    /// is not used during type resolution, but it is useful in later stages.
    type_map: HashMap<String, DefId>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut arena = Arena::default();
        let global = arena.alloc(Scope::new("<global>"));

        Self {
            lexical_scopes: arena,
            current_scope: vec![global],
            type_map: HashMap::default(),
        }
    }

    /// Returns the global scope.
    fn global_scope(&self) -> &Scope {
        let id = *self.current_scope.first().unwrap();
        self.lexical_scopes.get(id)
    }

    /// Returns a mutable reference to the current scope.
    fn current_scope(&mut self) -> &mut Scope {
        let current_scope = *self.current_scope.last().unwrap();
        self.lexical_scopes.get_mut(current_scope)
    }

    /// Performs a lookup of the given identifier in the current scope.
    fn local_symbol(&mut self, ident: &Ident) -> Option<&mut Symbol> {
        self.current_scope().symbols.get_mut(&ident.name)
    }

    /// Returns the local scope if it is a module or non-ADT type.
    fn local_scope(&mut self, ident: &Ident) -> Option<ScopeId> {
        match self.local_symbol(ident)? {
            Symbol::Module(scope) | Symbol::Lexical { scope, .. } => Some(*scope),
            _ => None,
        }
    }

    /// Returns a fully qualified name of the given identifier. This assumes
    /// the identifier exists in the current scope.
    fn qualified_symbol(&self, ident: &Ident) -> String {
        let path = self
            .current_scope
            .iter()
            .skip(1)
            .map(|id| self.lexical_scopes.get(*id).name.as_str())
            .collect::<Vec<_>>()
            .join("::");

        format!("{path}::{}", ident.name)
    }

    /// Search the current + parent scopes for a symbol.
    fn parent_symbol(&self, ident: &Ident) -> Option<&Symbol> {
        self.current_scope
            .iter()
            .rev()
            .find_map(move |id| self.lexical_scopes.get(*id).symbols.get(&ident.name))
    }

    /// Determines if two symbols are compatible. Compatible in this case means
    /// they can co-exist in the same lexical scope, for example if a type was
    /// forward declared then later defined.
    fn is_compatible(ident: &Ident, lhs: &Symbol, rhs: &Symbol) -> bool {
        let eq = match (lhs, rhs) {
            // Modules are open-ended
            (Symbol::Module(_), Symbol::Module(_)) => true,

            // Types can be declared before or after they are defined
            (Symbol::Decl(_, prev), Symbol::Adt(_, kind))
            | (Symbol::Adt(_, prev), Symbol::Decl(_, kind))
            | (Symbol::Decl(_, prev), Symbol::Decl(_, kind)) => match (prev, kind) {
                (ItemKind::Struct, ItemKind::Struct)
                | (ItemKind::Union, ItemKind::Union)
                | (ItemKind::Interface, ItemKind::Interface)
                | (ItemKind::Valuetype, ItemKind::Valuetype) => true,
                _ => false,
            },
            _ => false,
        };

        if !eq {
            tracing::error!(
                "incompatible: {} was previously defined as ....",
                ident.name,
            );
        }
        eq
    }

    /// Performs a downward search for a symbol. This will never look at parent
    /// scopes, it will only search the specified scope and its children.
    fn symbol_in_scope<'a>(&'a self, mut scope: &'a Scope, path: &'a Path) -> Result<&'a Symbol> {
        let mut segments = path.segments.iter();
        while let Some(seg) = segments.next() {
            let entry = scope
                .symbols
                .get(&seg.name)
                .ok_or(ResolveError::Undefined(seg.span))?;

            match entry {
                Symbol::Module(v) => {
                    scope = self.lexical_scopes.get(*v);

                    if segments.next().is_none() {
                        return Ok(entry);
                    }
                }
                _ => {
                    if segments.next().is_some() {
                        tracing::error!("path resolved to type with superfluous segments");
                        return Err(ResolveError::Superfluous(seg.span));
                    }
                    return Ok(entry);
                }
            }
        }

        panic!("empty path");
        // Ok(scope)
    }

    /// Tries to resolve the given path starting from the global scope.
    fn global_symbol<'a>(&'a self, path: &'a Path) -> Result<&'a Symbol> {
        let global = self.global_scope();
        self.symbol_in_scope(global, path)
    }

    /// Verifies that all declared types have since been defined.
    fn all_defined(&self) -> bool {
        for (_id, _scope) in &self.lexical_scopes {
            // if scope.decls.h
        }
        true
    }

    /// Creates a new module if it did not already exist. Future type
    /// registrations will be registered in the scope of the newly created
    /// module.
    ///
    /// # Errors
    ///
    /// If an error is returned, a scope is not created and the implementation
    /// should skip processing any items found within the module.
    pub fn start_module(&mut self, ident: &Ident) -> Result<()> {
        if let Some(sym) = self.local_symbol(ident) {
            match sym {
                Symbol::Module(v) => {
                    let id = *v;
                    self.current_scope.push(id);
                    Ok(())
                }
                _ => {
                    tracing::error!("symbol {} was previously registered as a type", ident.name);
                    Err(ResolveError::Redefined(ident.span))
                }
            }
        } else {
            // TODO: might be a good idea to let each mod know what the parent ID is.
            let id = self.lexical_scopes.alloc(Scope::new(ident.name.clone()));

            // Register the module
            self.current_scope()
                .symbols
                .insert(ident.name.clone(), Symbol::Module(id));

            self.current_scope.push(id);
            Ok(())
        }
    }

    /// Returns `true` if the given symbol has been fully defined. This will
    /// return `false` for declarations and incomplete types.
    pub fn is_defined(&mut self, ident: &Ident) -> bool {
        if let Some(sym) = self.local_symbol(ident) {
            match sym {
                Symbol::Decl(_, _) => false,
                _ => true,
            }
        } else {
            false
        }
    }

    fn alloc_scope(&mut self, ident: &Ident, def: DefId, kind: SymbolKind) -> ScopeId {
        let scope = self.lexical_scopes.alloc(Scope::new(ident.name.clone()));
        let symbol = match kind {
            SymbolKind::Module => Symbol::Module(scope),
            SymbolKind::Interface | SymbolKind::Valuetype => Symbol::Lexical { def, scope, kind },
            _ => unreachable!(),
        };

        self.current_scope()
            .symbols
            .insert(ident.name.clone(), symbol);
        scope
    }

    pub fn start_scope(&mut self, ident: &Ident, def: DefId, kind: SymbolKind) -> Result<()> {
        // Does the given symbol already exist? If not, create it.
        let mut err = None;
        let scope_id = if let Some(sym) = self.local_symbol(ident) {
            match sym {
                Symbol::Module(v) => *v,
                Symbol::Decl(_, decl) => {
                    if *decl != kind {
                        err = Some(ResolveError::DeclMismatch {
                            decl: *decl,
                            def: kind,
                            span: ident.span,
                        });
                    }

                    // TODO: must replace decl with def
                    self.alloc_scope(ident, def, kind)
                }
                _ => {
                    panic!("already defined");
                }
            }
        } else {
            self.alloc_scope(ident, def, kind)
        };

        // Create a symbol for the definition
        self.current_scope.push(scope_id);
        err.map_or(Ok(()), Err)
    }

    /// Wraps up the current module and restores the previous scope.
    pub fn finish_scope(&mut self) {
        let last = self.current_scope.pop();
        tracing::info!(
            "finished scope: {:?}",
            self.lexical_scopes.get(last.unwrap()),
        );
        debug_assert!(
            !self.current_scope.is_empty(),
            "closed scope but stack is empty",
        );
    }

    /// Declares the existance of a type and its kind.
    pub fn declare_type(&mut self, ident: &Ident, symbol: Symbol) -> Result<()> {
        if self.local_symbol(ident).is_none() {
            self.current_scope().decls.insert(ident.name.clone());
        }

        if let Some(prev) = self.local_symbol(ident) {
            if Self::is_compatible(ident, prev, &symbol) {
                Ok(())
            } else {
                // TODO: should be mismatch
                Err(ResolveError::Redefined(ident.span))
            }
        } else {
            self.current_scope()
                .symbols
                .insert(ident.name.clone(), symbol);

            Ok(())
        }
    }

    /// Registers a new type in the current scope.
    pub fn define_type_old(&mut self, ident: &Ident, symbol: Symbol) -> bool {
        let qualified = self.qualified_symbol(ident);
        tracing::info!("registering {qualified}");

        // If the type was previously declared, remove the declaration in
        // favor of the definition.
        if !matches!(symbol, Symbol::Decl(_, _)) {
            self.current_scope().decls.remove(&ident.name);
        }

        if let Some(prev) = self.local_symbol(ident) {
            Self::is_compatible(ident, prev, &symbol)
        } else {
            self.current_scope()
                .symbols
                .insert(ident.name.clone(), symbol)
                .is_none()
        }
    }

    /// Registers a new type in the current scope.
    pub fn define_type(&mut self, ident: &Ident, id: DefId, kind: SymbolKind) -> Result<()> {
        if let Some(sym) = self.local_symbol(ident) {
            match sym {
                Symbol::Decl(_, decl) => {
                    if *decl == kind {
                        *sym = Symbol::Adt(id, kind);
                        self.current_scope().decls.remove(&ident.name);
                        Ok(())
                    } else {
                        Err(ResolveError::DeclMismatch {
                            decl: *decl,
                            def: kind,
                            span: ident.span,
                        })
                    }
                }
                Symbol::Const => Ok(()),
                Symbol::Adt(_, _) => Err(ResolveError::Redefined(ident.span)),
                _ => unreachable!("non-ADT registered as a type"),
            }
        } else {
            let symbol = match kind {
                ItemKind::Decl => Symbol::Adt(id, kind),
                ItemKind::Interface | ItemKind::Valuetype | ItemKind::Module => {
                    unreachable!("non-ADT was registered as a type")
                }
                _ => Symbol::Adt(id, kind),
            };

            self.current_scope()
                .symbols
                .insert(ident.name.to_string(), symbol);
            Ok(())
        }
    }

    /// Resolves a path. This operates relative to the current scope.
    ///
    /// # Errors
    ///
    /// Returns an error that contains the span of the identifier that did not
    /// resolve correctly.
    pub fn resolve_path(&self, path: &Path) -> Result<DefId> {
        let mut segments = path.segments.iter().peekable();

        let scope = if path.leading_colons.is_some() {
            // Start at the global scope for fully qualified symbols
            None
        } else {
            // Resolve the first segment by looking at the current scope and
            // parent scopes. We pick the first that matches -- modules with
            // the same name in any of the upper scopes are ignored.
            if let Some(first) = segments.next() {
                let sym = self
                    .parent_symbol(first)
                    .ok_or(ResolveError::Undefined(first.span))?;

                match sym {
                    Symbol::Const => todo!(),
                    Symbol::Adt(v, _) | Symbol::Decl(v, _) => {
                        if segments.peek().is_some() {
                            tracing::error!("path resolved to type with superfluous segments");
                            return Err(ResolveError::Superfluous(first.span));
                        }
                        return Ok(*v);
                    }
                    // TODO: if this is last segment, return id of lexical
                    Symbol::Lexical { scope, def, .. } => {
                        if segments.peek().is_none() {
                            return Ok(*def);
                        }
                        Some(self.lexical_scopes.get(*scope))
                    }
                    Symbol::Module(scope) => Some(self.lexical_scopes.get(*scope)),
                }
            } else {
                None
            }
        };

        // Once the initial scope has been resolved, we work our way downwards
        // by only looking at items defined within said scope.
        let mut span = util::path_span(path);
        let mut scope = scope.unwrap_or_else(|| self.global_scope());

        while let Some(seg) = segments.next() {
            let entry = scope
                .symbols
                .get(&seg.name)
                .ok_or(ResolveError::Undefined(seg.span))?;

            match entry {
                Symbol::Adt(v, _) | Symbol::Decl(v, _) => {
                    if segments.next().is_some() {
                        panic!();
                        tracing::error!("path resolved to type with superfluous segments");
                        return Err(ResolveError::Superfluous(seg.span));
                    }
                    return Ok(*v);
                }
                Symbol::Const => todo!(),
                Symbol::Module(v) => {
                    scope = self.lexical_scopes.get(*v);
                    // Narrow down the span so we can provide better diagnostics
                    span = seg.span;
                }
                Symbol::Lexical { def, scope: id, .. } => {
                    if segments.peek().is_some() {
                        scope = self.lexical_scopes.get(*id);
                        span = seg.span;
                    } else {
                        return Ok(*def);
                    }
                }
            }
        }

        // Reaching this means all paths resolved, but the last segment
        // resolved to a module and not an actual type.
        Err(ResolveError::Module(span))
    }

    /// Resolves the path `other` relative to `origin`. Unlike `resolve_path`,
    /// this does not account for the current scope.
    ///
    /// # Errors
    ///
    /// Returns an error that contains the span of the identifier that did not
    /// resolve correctly. This applies to both `origin` and `other`.
    pub fn relative_path(&self, origin: &Path, other: &Path) -> Result<DefId> {
        let origin = match self.global_symbol(origin)? {
            Symbol::Adt(_, _) => todo!(),
            Symbol::Decl(_, _) => todo!(),
            Symbol::Const => todo!(),
            Symbol::Module(v) => self.lexical_scopes.get(*v),
            Symbol::Lexical { .. } => todo!(),
        };

        match self.symbol_in_scope(&origin, other)? {
            Symbol::Adt(v, _) | Symbol::Decl(v, _) => Ok(*v),
            Symbol::Const => todo!(),
            Symbol::Module(_) => todo!(),
            Symbol::Lexical { .. } => todo!(),
        }
    }

    /// Verifies that all declarations have been defined, and that all modules
    /// and other lexical scopes have been correctly closed.
    pub fn finish(self) {
        for (_, scope) in &self.lexical_scopes {
            for decl in &scope.decls {
                tracing::error!("type {decl} was declared but not defined");
            }
        }

        let len = self.current_scope.len();
        debug_assert_eq!(
            len, 1,
            "type resolution finished with a scope size of {len}"
        );
    }
}

/// Determines if two annotation definitions are consistent. The standard
/// doesn't clarify what "consistent" means, but I've interpreted it as the two
/// definitions being identical.
fn is_consistent(ctx: &Context, lhs: &AnnotationDef, rhs: &AnnotationDef) -> bool {
    if !lhs.ident.name.eq_ignore_ascii_case(&rhs.ident.name) || lhs.params.len() != rhs.params.len()
    {
        return false;
    }

    lhs.params.iter().zip(rhs.params.iter()).all(|v| match v {
        (AnnotationField::Member(lhs), AnnotationField::Member(rhs)) => {
            true
            // decl_consistent(ctx, &lhs.names, &rhs.names)
            //     && is_type_consistent(ctx, &lhs.ty, &rhs.ty)
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
    ctx: &Context,
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
fn is_type_consistent(ctx: &Context, lhs: &ic_syntax::Type, rhs: &ic_syntax::Type) -> bool {
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

// /// Inserts primitive types and built-in annotations into the context.
// fn init_ctx_state(ctx: &mut Resolver) {
//     for ty in PrimitiveTy::iter() {
//         let name = ty.name();
//         // ctx.current_scope().symbols.insert(ty.name(), Symbol::)
//         // let name = name.into();
//         tracing::info!("registering type {name}: {ty:?}");
//         // ctx.register_type(ty.name(), Type::Primitive(ty));
//     }
// }
