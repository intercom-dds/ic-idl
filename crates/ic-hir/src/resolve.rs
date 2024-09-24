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
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use ic_alloc::arena::{self, Arena};
use ic_alloc::insensitive::{CaseMap, CaseSet};
use ic_syntax::{AnnotationDef, AnnotationField, Expr, Ident, ItemKind, Path, Span, util};
use tracing::{Instrument, debug};

use crate::{Context, Decl, Def, DefId};

pub type SymbolKind = ItemKind;

/// An ID of a lexical scope.
type ScopeId = arena::Id<ScopeKind>;

#[derive(Debug)]
pub enum Scope {
    /// Scope of an ADT. May contain nested symbols, but cannot contain
    /// nested definitions.
    Adt(DefId),

    /// A declaration of a type.
    Decl(DefId),

    /// A constant of some sort.
    Const,

    /// A module which may contain nested type definitions.
    Module { entries: CaseMap<'static, Scope> },

    /// A unique lexical scope that may contain nested type definitions, but
    /// which cannot be redefined. Used for interfaces, valuetypes, and
    /// annotations.
    Lexical {
        id: DefId,
        entries: CaseMap<'static, Scope>,
    },
}

#[derive(Debug)]
pub enum Symbol {
    /// Scope of an ADT. May contain nested symbols, but cannot contain
    /// nested definitions.
    Adt(DefId, SymbolKind),

    /// A declaration of a type.
    Decl(DefId, SymbolKind),

    /// A constant of some sort.
    Const,

    /// A module which may contain nested type definitions. The ID maps to the
    /// arena in the resolver, which can be used to retrieve the lexical scope
    /// of the module.
    Module(ScopeId),
}

#[derive(Debug)]
pub struct ScopeKind {
    /// Name of the current scope.
    name: String,

    /// Symbols registered in the scope.
    symbols: HashMap<String, Symbol>,

    /// A set of IDs of forward declarations that have not yet been defined.
    /// Once defined, the corresponding ID will be removed from this set.
    decls: HashSet<String>,
}

/// Contains all the logic required to resolve all types and expressions in the
/// AST. This will keep track of the current scope, and provides functions for
/// resolving symbols and paths.
///
/// The resolver does not know anything about the definition of each type; it
/// only cares about the name and lexical scope.
pub struct Resolver {
    /// Stores all the lexical scopes that have been defined, including the
    /// global scope.
    lexical_scopes: Arena<ScopeKind>,

    /// Stack of the current scope and all parents. This will never be empty,
    /// as the first entry is guaranteed to be the global scope.
    current_scope: Vec<ScopeId>,

    /// Maps fully qualified names to each type's respective definition. This
    /// is not used during type resolution, but it is useful in later stages.
    type_map: CaseMap<'static, DefId>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut arena = Arena::default();
        let global = arena.alloc(ScopeKind {
            name: "<global>".to_string(),
            symbols: HashMap::default(),
            decls: HashSet::default(),
        });

        Self {
            lexical_scopes: arena,
            current_scope: vec![global],
            type_map: CaseMap::default(),
        }
    }

    /// Returns the global scope.
    fn global_scope(&self) -> &ScopeKind {
        let id = *self.current_scope.first().unwrap();
        self.lexical_scopes.get(id)
    }

    /// Returns a mutable reference to the current scope.
    fn current_scope(&mut self) -> &mut ScopeKind {
        let current_scope = *self.current_scope.last().unwrap();
        self.lexical_scopes.get_mut(current_scope)
    }

    /// Performs a lookup of the given identifier in the current scope.
    fn local_symbol(&mut self, ident: &Ident) -> Option<&mut Symbol> {
        self.current_scope().symbols.get_mut(&ident.name)
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
            .find_map(|id| self.lexical_scopes.get(*id).symbols.get(&ident.name))
    }

    /// Determines if two symbols are compatible. Compatible in this case means
    /// they can co-exist in the same lexical scope, for example if a type was
    /// forward declared then later defined.
    fn is_compatible(ident: &Ident, lhs: &Symbol, rhs: &Symbol) -> bool {
        let eq = match (lhs, rhs) {
            (Symbol::Decl(_, prev), Symbol::Adt(_, kind))
            | (Symbol::Adt(_, prev), Symbol::Decl(_, kind))
            | (Symbol::Decl(_, prev), Symbol::Decl(_, kind)) => match (prev, kind) {
                (ItemKind::Struct, ItemKind::Struct)
                | (ItemKind::Union, ItemKind::Union)
                | (ItemKind::Interface, ItemKind::Interface)
                | (ItemKind::Valuetype, ItemKind::Valuetype) => true,
                _ => false,
            },
            v => false,
            // Symbol::Decl(_, prev) => match (prev, rhs) {
            //     (ItemKind::Struct, Decl::Struct)
            //     | (ItemKind::Union, Decl::Union)
            //     | (ItemKind::Interface, Decl::Interface)
            //     | (ItemKind::Valuetype, Decl::Valuetype) => true,
            //     _ => {
            //         tracing::error!("{} was previously declared as {rhs:?}", ident.name);
            //         false
            //     }
            // },
            // Symbol::Const | Symbol::Module(_) => {
            //     tracing::error!("{} was previously declared as {rhs:?}", ident.name);
            //     false
            // }
        };

        if !eq {
            tracing::error!(
                "incompatible: {} was previously defined as ....",
                ident.name,
            );
        }
        eq

        // match (lhs, rhs) {
        //     // Decls can be redefined so long as the type remains consistent
        //     (Symbol::Decl(l, _), Symbol::Decl(r, _)) => {
        //         // TODO:
        //         true
        //     }
        //
        //     // Declarations may appear before or after a definition, both
        //     // of which are fine so long as the kind doesn't change.
        //     (Symbol::Adt(_, def), Symbol::Decl(_, decl))
        //     | (Symbol::Decl(_, decl), Symbol::Adt(_, def)) => {
        //         // TODO:
        //         true
        //     }
        //
        //     // Modules are open-ended
        //     (Symbol::Module(_), Symbol::Module(_)) => true,
        //
        //     // All other symbols are never compatible as they cannot be redeclared
        //     _ => false,
        // }
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
    pub fn start_module(&mut self, ident: &Ident) {
        // TODO: should this return a result, or do we want to keep going even
        // in case of a collision? for AST lowering, that would imply skipping
        // all definitions inside the module. I don't think that's ideal since
        // it will cause spurious errors when the skipped types are used.
        if let Some(sym) = self.local_symbol(ident) {
            match sym {
                Symbol::Module(v) => {
                    let id = *v;
                    self.current_scope.push(id);
                }
                Symbol::Adt(_, _) | Symbol::Decl(_, _) => {
                    tracing::error!("symbol {} was previously registered as a type", ident.name);
                }
                Symbol::Const => {
                    tracing::error!(
                        "symbol {} was previously registered as a constant",
                        ident.name,
                    );
                }
            }
        } else {
            // TODO: might be a good idea to let each mod know what the parent ID is.
            let id = self.lexical_scopes.alloc(ScopeKind {
                name: ident.name.clone(),
                symbols: HashMap::default(),
                decls: HashSet::default(),
            });

            // Register the module
            self.current_scope()
                .symbols
                .insert(ident.name.clone(), Symbol::Module(id));

            self.current_scope.push(id);
        }
    }

    /// Wraps up the current module and restores the previous scope.
    pub fn finish_module(&mut self) {
        let last = self.current_scope.pop();
        tracing::info!(
            "finished module: {:?}",
            self.lexical_scopes.get(last.unwrap())
        );
        debug_assert!(
            !self.current_scope.is_empty(),
            "closed scope but stack is empty",
        );
    }

    /// Declares the existance of a type and its kind.
    pub fn declare_type(&mut self, ident: &Ident, symbol: Symbol) {
        if self.local_symbol(ident).is_none() {
            self.current_scope().decls.insert(ident.name.clone());
        }

        // Go through the motions of defining the type -- even if it has
        // already been defined -- so we can report type errors.
        self.define_type(ident, symbol);
    }

    // TODO: this is only ever used for Adts. We should have separate functions
    // for decls, types, consts, etc.
    //
    /// Registers a new type in the current scope.
    pub fn define_type(&mut self, ident: &Ident, symbol: Symbol) -> bool {
        let qualified = self.qualified_symbol(ident);
        tracing::info!("registering {qualified}");

        // If the type was previously declared, remove the declaration in
        // favor of the definition.
        if !matches!(symbol, Symbol::Decl(_, _)) {
            self.current_scope().decls.remove(&ident.name);
        }

        if let Some(prev) = self.local_symbol(ident) {
            // TODO: replace decls with def
            // TODO: returning a bool here is not sufficient. leads to weird errors
            Self::is_compatible(ident, prev, &symbol)
        } else {
            self.current_scope()
                .symbols
                .insert(ident.name.clone(), symbol)
                .is_none()
        }
    }

    /// Resolves a path. This operates relative to the current scope.
    ///
    /// # Errors
    ///
    /// Returns an error that contains the span of the identifier that did not
    /// resolve correctly.
    //
    // TODO: fn relative_path
    pub fn resolve_path(&self, path: &Path) -> Result<DefId, Span> {
        let mut segments = path.segments.iter();

        let scope = if path.leading_colons.is_some() {
            // Start at the global scope for fully qualified symbols
            None
        } else {
            // Resolve the first segment by looking at the current scope and
            // parent scopes. We pick the first that matches -- modules with
            // the same name in any of the upper scopes are ignored.
            if let Some(first) = segments.next() {
                let sym = self.parent_symbol(first).ok_or(first.span)?;
                match sym {
                    Symbol::Const => todo!(),
                    Symbol::Adt(v, _) | Symbol::Decl(v, _) => return Ok(*v),
                    Symbol::Module(v) => Some(self.lexical_scopes.get(*v)),
                }
            } else {
                None
            }
        };

        // Once the initial scope has been resolved, we work our way downwards
        // by only looking at items defined within said scope.
        let mut span = util::path_span(path);
        let mut scope = scope.unwrap_or_else(|| self.global_scope());

        for seg in segments {
            let entry = scope.symbols.get(&seg.name).ok_or(seg.span)?;
            match entry {
                Symbol::Adt(v, _) | Symbol::Decl(v, _) => return Ok(*v),
                Symbol::Const => todo!(),
                Symbol::Module(v) => {
                    scope = self.lexical_scopes.get(*v);
                    // Narrow down the span so we can provide better diagnostics
                    span = seg.span;
                }
            }
        }

        // Reaching this means all paths resolved, but the last segment
        // resolved to a module and not an actual type.
        Err(span)
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
        debug_assert_eq!(len, 1, "type resolution finished with a length of {len}");
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
