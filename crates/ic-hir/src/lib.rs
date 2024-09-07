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

#![allow(clippy::all, warnings)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZero;
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_macros::EnumIter;
use ic_syntax::util::{path_name, type_name};
use ic_syntax::{AnnotationDef, AnnotationField, Expr, Ident, Item, Span};

// mod annotation;
pub mod fold;
pub mod hir;
pub mod keywords;
mod lower;
mod resolve;
pub mod visit;
// mod downcast;

use hir::{
    AliasTy, DeclTy, EnumTy, Enumerator, Member, ModuleTy, Numeric, PrimitiveTy, StructTy, TyFlags,
    Type, UnionTy,
};

mod embedded {
    pub const ANNOTATIONS: &str = include_str!("../idl/annotations.idl");
}

// TODO: some id that identifies the source file this belongs to
pub type TypeId = ic_alloc::arena::Id<Type>;

#[derive(Debug)]
pub struct Context {
    pub arena: Arena<Type>,
    pub items: Arena<hir::Item>,

    // Qualified type name => Type ID
    pub symbols: HashMap<String, TypeId>,
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
            arena: Arena::default(),
            items: Arena::default(),
            symbols: HashMap::new(),
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

    fn register_type<I>(&mut self, name: I, ty: Type) -> Id<Type>
    where
        I: Into<String>,
    {
        match self.symbols.entry(name.into()) {
            Entry::Occupied(v) => {
                panic!("type {} was registered multiple times", v.key());
            }
            Entry::Vacant(v) => {
                let id = self.arena.alloc(ty);
                v.insert(id);
                id
            }
        }
    }

    /// Returns the type definition of the specified type.
    ///
    /// # Panics
    ///
    /// Panics if the specified type does not exist in the arena. This can only
    /// happen if there are multiple `Context`s whose arenas have been mixed
    /// up.
    fn type_of(&self, ty: TypeId) -> &Type {
        self.arena
            .get(ty)
            .unwrap_or_else(|| panic!("type {ty:?} does not exist"))
    }

    /// Similar to `type_of`, but will resolve the underlying type.
    fn base_type_of(&self, ty: TypeId) -> &Type {
        let ty = self.type_of(ty);
        if let Type::Alias(alias) = ty {
            self.type_of(alias.ty)
        } else {
            ty
        }
    }

    // TODO: handle this in `Resolver` instead -- we should only operate on IDs.
    fn resolve_type(&self, name: &ic_syntax::Type) -> TypeId {
        let name = type_name(name);
        *self.symbols.get(&name).expect("unknown type")
    }

    fn resolve_path(&self, path: &ic_syntax::Path) -> TypeId {
        let name = path_name(path);
        *self.symbols.get(&name).expect("unknown type")
    }

    // TODO: or should it be TypeId? that must be a ConstTy?
    fn resolve_const(&self, path: &ic_syntax::Path) -> Type {
        todo!()
        // let name = path_name(path);
        // *self.symbols.get(&name).expect("unknown const")
    }
}

/// Inserts primitive types and built-in annotations into the context.
fn init_ctx_state(ctx: &mut Context) {
    for ty in PrimitiveTy::iter() {
        ctx.register_type(ty.name(), Type::Primitive(ty));
    }
}

pub fn resolve(tree: &[Item]) {
    let mut visitor = resolve::Resolver::default();
    ic_syntax::visit::visit_tree(&mut visitor, tree);
    println!("{visitor:#?}");
}

#[derive(Default)]
struct Resolver;

impl Resolver {
    fn resolve_expr(&mut self, expr: &Expr) -> i32 {
        match expr {
            Expr::Literal(v) => 9,
            Expr::Path(_) => todo!(),
            Expr::Unary(_) => todo!(),
            Expr::Binary(_) => todo!(),
            Expr::InitList(_) => todo!(),
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
        // (AnnotationField::Arg(lhs), AnnotationField::Arg(rhs)) => {
        //     decl_consistent(ctx, &lhs.names, &rhs.names)
        //         && is_type_consistent(ctx, &lhs.ty, &rhs.ty)
        // }
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
        (Type::String_(lhs), Type::String_(rhs)) => lhs.wide == rhs.wide,
        (Type::Map(lhs), Type::Map(rhs)) => {
            is_type_consistent(ctx, lhs.key.as_ref(), rhs.key.as_ref())
                && is_type_consistent(ctx, lhs.value.as_ref(), rhs.value.as_ref())
        }
        (Type::Path(lhs), Type::Path(rhs)) => ctx.resolve_path(lhs) == ctx.resolve_path(rhs),
        _ => lhs.disc() == rhs.disc(),
    }
}

#[derive(Debug)]
pub struct ResolvedGraph {
    /// The primary data structure that owns all the types.
    pub context: Context,

    /// Defines the order in which the types appeared in the syntax tree. This
    /// can be used to traverse the graph in the same order in which the types
    /// were defined.
    pub order: Vec<TypeId>,
}

pub fn lower_ast<I>(ast: I) -> ResolvedGraph
where
    I: IntoIterator<Item = ic_syntax::Item>,
{
    let mut context = Context::new();
    let mut resolver = Resolver::default();
    let order = lower::from_ast(&mut context, ast);

    ResolvedGraph { context, order }
}
