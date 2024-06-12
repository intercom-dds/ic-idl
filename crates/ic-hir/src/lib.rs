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

#![allow(unused, dead_code, clippy::all, clippy::must_use_candidate)]

use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_alloc::interner::{Interner, SymbolId};
use ic_macros::EnumIter;
use ic_syntax::{Expr, Ident, Item, Span};

// mod annotation;
pub mod keywords;
pub mod visit;

// mod downcast;

// TODO: some id that identifies the source file this belongs to
pub type NodeId = ic_alloc::arena::Id<Type>;

/// A dynamic representation of an applied annotation.
#[derive(Debug)]
pub struct GenericAnn {
    pub ident: Ident,
    pub span: Span,
    pub fields: Vec<AnnParam>,
}

#[derive(Debug)]
pub struct AnnParam {
    pub ident: Option<Ident>,
    pub span: Span,
    pub value: Expr,
}

impl GenericAnn {
    /// Attempts to "downcast" the annotation to a concrete annotation type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ic_hir::GenericAnn;
    /// use ic_hir::annotations::MustUnderstand;
    ///
    /// let ann = GenericAnn { ... };
    /// let concrete = ann.try_get::<MustUnderstand>().unwrap();
    /// assert_eq!(concrete.value, true);
    /// ```
    pub fn try_get<T>(&self) -> T {
        todo!()
    }
}

#[derive(Debug)]
pub struct Context {
    pub arena: Arena<Type>,
    pub interner: Interner,

    // Qualified name => Type ID
    pub types: HashMap<String, Id<Type>>,
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
            interner: Interner::default(),
            types: HashMap::new(),
        }
    }

    /// # Panics
    ///
    /// Panics if the given type ID does not exist. This can only ever happen
    /// if there are multiple `Context`s whose arenas have been mixed up.
    pub fn ty(&self, id: Id<Type>) -> &Type {
        self.try_ty(id)
            .expect(&format!("type {id:?} does not exist"))
    }

    /// # Panics
    pub fn str(&self, id: SymbolId) -> &str {
        self.try_str(id)
            .expect(&format!("symbol {id:?} does not exist"))
    }

    pub fn try_ty(&self, id: Id<Type>) -> Option<&Type> {
        self.arena.get(id)
    }

    pub fn try_str(&self, id: SymbolId) -> Option<&str> {
        self.interner.get(id)
    }

    pub fn primitive_type(&self, _kind: Primitive) -> &Type {
        todo!()
    }

    pub fn lookup_type(&self, name: &str) -> Option<&Type> {
        let ty = self.types.get(name)?;
        self.try_ty(*ty)
    }

    fn register_type<I>(&mut self, name: I, ty: Type) -> Id<Type>
    where
        I: Into<String>,
    {
        match self.types.entry(name.into()) {
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
}

/// Inserts primitive types and built-in annotations into the context.
fn init_ctx_state(ctx: &mut Context) {
    for ty in Primitive::iter() {
        ctx.register_type(ty.name(), Type::Primitive(ty));
    }
}

#[derive(Debug)]
pub enum Type {
    Primitive(Primitive),
    Typedef(Typedef),
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::Primitive(v) => v.name(),
            Type::Typedef(v) => v.name(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum Primitive {
    Boolean,
    Char,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float,
    Double,
    String,
}

impl Primitive {
    pub fn name(&self) -> &'static str {
        match self {
            Primitive::Boolean => "boolean",
            Primitive::Char => "char",
            Primitive::Int8 => "int8",
            Primitive::UInt8 => "octet",
            Primitive::Int16 => "int16",
            Primitive::UInt16 => "uint16",
            Primitive::Int32 => "int32",
            Primitive::UInt32 => "uint32",
            Primitive::Int64 => "int64",
            Primitive::UInt64 => "uint64",
            Primitive::Float => "float",
            Primitive::Double => "double",
            Primitive::String => "string",
        }
    }
}

#[derive(Debug)]
pub struct Typedef {
    ident: Ident,
    ty: NodeId,
    scope: Option<NodeId>,
    span: Span,
    state: Rc<Context>,
    flags: usize,
    pub annotations: Vec<GenericAnn>,
}

pub enum Scope {
    Global,
    Module,
    Interface,
}

impl Typedef {
    pub fn name(&self) -> &str {
        &self.ident.name
    }

    pub fn ty(&self) -> &Type {
        self.state.ty(self.ty)
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn annotations(&self) -> &[GenericAnn] {
        &self.annotations
    }

    /// The scope in which this node was defined. Returns `None` if the type
    /// was defined in the global scope.
    pub fn scope(&self) -> Option<&Type> {
        self.scope.map(|id| self.state.ty(id))
    }

    pub fn flags(&self) -> usize {
        self.flags
    }
}

pub fn qualified_name(ty: &Typedef) -> String {
    let mut path = vec![ty.name()];
    let mut node = ty;

    while let Some(v) = ty.scope() {
        path.push(v.name());

        match v {
            Type::Typedef(v) => {
                node = v;
            }
            Type::Primitive(_) => (),
        }
    }
    path.reverse();
    path.join("::")
}

#[test]
fn test_typedef() {
    let mut ctx = Context::new();
    let ty = *ctx.types.get("int32").unwrap();
    let state = Rc::new(ctx);

    let typedef = Typedef {
        ident: Ident {
            name: "foobar".to_string(),
            span: Span::default(),
        },
        ty,
        scope: None,
        span: Span::default(),
        state,
        annotations: vec![],
        flags: 0,
    };

    println!("{}", typedef.ty().name());
    println!("{}", qualified_name(&typedef));
}
