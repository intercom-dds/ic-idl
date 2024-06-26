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

//! Type definitions of the HIR.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZero;
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_macros::EnumIter;
use ic_syntax::util::{path_name, type_name};
use ic_syntax::{AnnotationDef, AnnotationField, Expr, Ident, Item, Span};

/// Dependency graph
pub type TyGraph = ic_alloc::graph::DiGraph<TypeId>;

use crate::TypeId;

/// Built-in primitive types. These types are effectively stateless and have no
/// bounds or other attributes attached to them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum PrimitiveTy {
    Bool,
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

impl PrimitiveTy {
    pub fn name(self) -> &'static str {
        match self {
            PrimitiveTy::Bool => "boolean",
            PrimitiveTy::Char => "char",
            PrimitiveTy::Int8 => "int8",
            PrimitiveTy::UInt8 => "octet",
            PrimitiveTy::Int16 => "int16",
            PrimitiveTy::UInt16 => "uint16",
            PrimitiveTy::Int32 => "int32",
            PrimitiveTy::UInt32 => "uint32",
            PrimitiveTy::Int64 => "int64",
            PrimitiveTy::UInt64 => "uint64",
            PrimitiveTy::Float => "float",
            PrimitiveTy::Double => "double",
            PrimitiveTy::String => "string",
        }
    }
}

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
pub enum Type {
    Primitive(PrimitiveTy),
    Annotation(AnnTy),
    Module(ModuleTy),
    Alias(AliasTy),
    Const(ConstTy),
    Struct(StructTy),
    Except(ExceptTy),
    Union(UnionTy),
    Enum(EnumTy),
    Bitmask(BitmaskTy),
    Interface(InterfaceTy),
    Decl(DeclTy),
}

intercom_cts::bitmask! {
    #[derive(Copy, Clone)]
    pub TyFlags: u16 {
        /// Indicates whether the type is recursive.
        IS_CIRCULAR  = 1 << 0,

        /// Indicates whether the type is trivial, i.e. consists only of
        /// primitive types and arrays thereof.
        IS_TRIVIAL   = 1 << 1,

        /// Indicates whether the type is anonymous.
        IS_ANONYMOUS = 1 << 2,

        /// Marker for built-in types.
        IS_BUILTIN   = 1 << 3,

        /// Indicates whether the type consists of members that can form a
        /// total order.
        TOTAL_ORDER  = 1 << 4,
    }
}

#[derive(Debug)]
pub struct Node {
    pub id: TypeId,
    pub ident: Ident,
    pub scope: Option<TypeId>,
    pub annotations: Vec<GenericAnn>,
    pub span: Span,
    pub data: Type,
}

pub enum Kind {
    Enum {
        enumerators: Vec<Enumerator>,
    },
    Struct {
        members: Vec<Member>,
    },
    Union {
        disc: Discriminator,
        variants: Vec<Variant>,
    },
    Module {
        defs: Vec<Node>,
    },
}

#[derive(Debug)]
pub struct AnnTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub struct ModuleTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
    pub definitions: Vec<TypeId>,
}

#[derive(Debug)]
pub struct AliasTy {
    pub id: TypeId,
    pub ident: Ident,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug)]
pub struct ConstTy {
    pub id: TypeId,
    pub ident: Ident,
    pub ty: TypeId,
    pub value: Numeric,
    pub span: Span,
}

#[derive(Debug)]
pub struct StructTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
    pub members: Vec<Member>,
    pub flags: TyFlags,
}

pub type ExceptTy = StructTy;

/// Member of a struct or union.
#[derive(Debug)]
pub struct Member {
    pub ident: Ident,
    pub ty: TypeId,
}

pub enum MemberKind {
    Type(TypeId),
    String {
        bound: Option<usize>,
    },
    Sequence {
        ty: Box<MemberKind>,
        bound: Option<usize>,
    },
    Array {
        ty: TypeId,
        bounds: Vec<usize>,
    },
    Map {
        key: Box<MemberKind>,
        element: Box<MemberKind>,
        bound: Option<usize>,
    },
}

#[derive(Debug)]
pub struct UnionTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
    pub disc: Discriminator,
    pub variants: Vec<Variant>,
}

#[derive(Debug)]
pub struct Discriminator {
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug)]
pub enum Variant {
    Member(Member, Vec<Numeric>),
    Null(Vec<Numeric>),
}

#[derive(Debug)]
pub struct EnumTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
    pub enumerators: Vec<Enumerator>,
}

#[derive(Debug)]
pub struct Enumerator {
    pub ident: Ident,
    pub value: i32,
}

#[derive(Debug)]
pub struct BitmaskTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
    pub bits: Vec<(Ident, usize)>,
}

#[derive(Debug)]
pub struct InterfaceTy {
    pub id: TypeId,
    pub ident: Ident,
    pub span: Span,
    pub prototypes: Vec<Proto>,
    pub attributes: Vec<Attr>,
}

#[derive(Debug)]
pub struct Proto {
    pub ident: Ident,
    pub return_ty: Option<TypeId>,
}

#[derive(Debug)]
pub struct Attr {
    pub ident: Ident,
    pub ty: TypeId,
    pub read_only: bool,
}

#[derive(Debug)]
pub struct DeclTy {
    pub ident: Ident,
    pub ty: TypeId,
}

#[derive(Debug)]
pub enum Numeric {
    Boolean(bool),
    Char(char),
    Int8(i8),
    Octet(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float(f32),
    Double(f64),
    String(String),

    /// Value that points to another constant.
    /// To retrieve the fully resolved value, use `Context::resolve_expr`.
    Const(TypeId),

    /// Fixed-size array elements, e.g. `{1, 2, 3}`.
    Array {
        ty: TypeId,
        values: Box<[Numeric]>,
    },

    /// Sequence elements, e.g. `{1, 2, 3}`.
    Sequence {
        ty: TypeId,
        values: Vec<Numeric>,
    },

    /// Map entries, eg. `{{key1, value1}, {key2, value2}}`.
    Map {
        ty: TypeId,
        values: Vec<(Numeric, Numeric)>,
    },
}
