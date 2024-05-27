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

//! Syntax tree for IDL.
//!
//! This module provides the types used in the syntax tree produced by the IDL
//! parser. The syntax tree closely resembles the source code, but some things
//! -- such as whitespace and `//` comments -- are omitted.
//!
//! For performance reasons, all strings are interned and stored separately.
//! Each item's lifetime is bound by the lifetime of the [`Interner`].
//!
//! [`Interner`]: ../../ic_alloc/interner/index.html

use ic_alloc::inline_vec::InlineVec;
use ic_alloc::ptr::P;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol;

#[derive(Debug)]
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Span {
    pub index: u32,
    pub len: u32,
}

pub type AnnotationVec = InlineVec<AnnotationAppl>;

#[derive(Debug)]
pub struct Item<K> {
    pub ident: Ident,
    pub span: Span,
    pub annotations: AnnotationVec,
    pub kind: K,
}

#[derive(Debug)]
pub enum ItemKind {
    Annotation(P<AnnotationDef>),
    Module(P<ModuleDef>),
    Struct(P<StructDef>),
    Union(P<UnionDef>),
    Enum(P<EnumDef>),
    Exception(P<ExceptDef>),
    Bitmask(P<BitmaskDef>),
    Bitset(P<BitsetDef>),
    Const(P<ConstDef>),
    Decl(P<Decl>),
}

#[derive(Debug)]
pub struct Decl {
    pub kind: DeclKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclKind {
    Struct,
    Union,
    Native,
    Interface,
    Valuetype,
}

#[derive(Debug)]
pub enum Type {
    /// Array of another type, e.g. `int32 value[3]`.
    /// Only the type is included; the name of the member is omitted.
    Array(P<Type>),

    /// Sequence of another type, e.g. `sequence<string>`.
    Sequence(P<Type>),

    /// (key, value) pair of types, e.g. `map<string, string>`.
    Map(P<Type>, P<Type>),

    /// A possibly qualified identifier of a type, e.g. `foo::Bar`.
    Path(Ident),
}

/// A definition of an annotation, e.g. `@annotation foo {};`.
#[derive(Debug)]
pub struct AnnotationDef {
    pub fields: InlineVec<AnnotationField>,
}

/// The items that can be placed inside a definition of an annotation.
#[derive(Debug)]
pub enum AnnotationField {
    Enum(P<Item<EnumDef>>),
    Bitmask(P<Item<BitmaskDef>>),
    Const(P<Item<ConstDef>>),
    Field(P<Field>),
}

/// A parameter inside an applied annotation, e.g. `value=true` in
/// `@optional(value=true)`.
#[derive(Debug)]
pub struct AnnotationParam {
    /// Name of the parameter if one was specified.
    /// May be omitted for annotations with only a single,  non-default member.
    pub name: Option<Ident>,

    /// Span of the entire parameter.
    pub span: Span,

    /// The specified value of the parameter.
    pub value: Expr,
}

#[derive(Debug)]
pub struct AnnotationAppl {
    pub ident: Ident,
    pub span: Span,
    pub params: InlineVec<AnnotationParam>,
}

#[derive(Debug)]
pub struct ModuleDef {
    pub defs: InlineVec<Item<ItemKind>>,
}

#[derive(Debug)]
pub struct StructDef {
    pub members: InlineVec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: Ident,
    pub span: Span,
    pub ty: Type,
}

#[derive(Debug)]
pub struct ExceptDef {
    pub members: InlineVec<Field>,
}

#[derive(Debug)]
pub struct EnumDef {
    pub fields: InlineVec<Enumerator>,
}

#[derive(Debug)]
pub struct BitmaskDef {
    pub annotations: InlineVec<AnnotationAppl>,
    pub bits: InlineVec<Bit>,
}

#[derive(Debug)]
pub struct Bit {
    pub value: Ident,
    pub annotations: InlineVec<AnnotationAppl>,
}

#[derive(Debug)]
pub struct BitsetDef {
    pub annotations: InlineVec<AnnotationAppl>,
    pub bits: InlineVec<Bitfield>,
}

#[derive(Debug)]
pub struct Bitfield {
    pub annotations: InlineVec<AnnotationAppl>,
    pub size: Ident,
}

#[derive(Debug)]
pub struct Enumerator {
    pub annotations: InlineVec<AnnotationAppl>,
    pub name: Ident,

    /// An explicit value, e.g. `enum Foo { VALUE = 1 };`
    /// The `@value` annotation will *not* populate this field.
    pub value: Option<Expr>,
}

#[derive(Debug)]
pub struct UnionDef {
    /// The discriminator component of the union.
    pub disc: Discriminator,

    /// All variants of the union. The case labels that map to each variant can
    /// be found in `UnionField`.
    pub fields: InlineVec<UnionField>,
}

#[derive(Debug)]
pub struct Discriminator {
    pub annotations: InlineVec<AnnotationAppl>,
    pub ty: Type,
}

#[derive(Debug)]
pub struct UnionField {
    pub annotations: InlineVec<AnnotationAppl>,

    /// Case labels that map to this variant.
    pub labels: InlineVec<Label>,

    pub field: Field,
}

#[derive(Debug)]
pub enum Label {
    Case { value: Ident },
    Null,
    Default,
}

#[derive(Debug)]
pub struct ConstDef {
    pub value: Expr,
    pub annotations: InlineVec<AnnotationAppl>,
}

#[derive(Debug)]
pub struct Numeric {
    pub kind: NumericKind,
    pub span: Span,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumericKind {
    Bool(bool),
    Int,
    Float,
    Char(char),
    String(Symbol),
    Ident(Symbol),
}

#[derive(Debug)]
pub struct Op {
    pub span: Span,
    pub kind: OpKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpKind {
    // Arithmetic operations
    Add,
    Sub,
    Multply,
    Divide,
    Modulo,

    // Bitwise operations
    LShift,
    RShift,
    Or,
    Xor,
    And,
}

#[derive(Debug)]
pub enum Expr {
    Numeric(Numeric),
    Unary { op: Op, expr: P<Expr> },
    Binary { lhs: P<Expr>, op: Op, rhs: P<Expr> },
}

pub fn dummy() {
    let defs = Item {
        ident: Ident {
            name: Symbol,
            span: Span::default(),
        },
        span: Span::default(),
        annotations: vec![],
        kind: ItemKind::Module(P(ModuleDef {
            defs: vec![Item {
                ident: Ident {
                    name: Symbol,
                    span: Span::default(),
                },
                span: Span::default(),
                annotations: vec![AnnotationAppl {
                    ident: Ident {
                        name: Symbol,
                        span: Span::default(),
                    },
                    span: Span::default(),
                    params: vec![],
                }],
                kind: ItemKind::Const(P(ConstDef {
                    value: Expr::Numeric(Numeric {
                        kind: NumericKind::Bool(true),
                        span: Span::default(),
                    }),
                    annotations: vec![],
                })),
            }],
        })),
    };

    println!("{defs:#?}");
}
