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

#![allow(non_local_definitions)]
#![allow(clippy::new_ret_no_self)]

//! Syntax tree for IDL.
//!
//! This crate provides the types used in the syntax tree produced by the IDL
//! parser. The syntax tree closely resembles the source code, but some things
//! -- such as whitespace and `//` comments -- are omitted.
//!
//! For performance reasons, all strings are interned and stored separately.
//! Each item's lifetime is bound by the lifetime of the [`Interner`].
//!
//! [`Interner`]: ../../ic_alloc/interner/index.html

/// Defines visitors for all AST nodes.
pub mod visit;

use ic_alloc::inline_vec::InlineVec;
use ic_alloc::interner::SymbolId;
use ic_alloc::ptr::P;
use intercom_cts::{Marshal, Unmarshal};

pub type Symbol = SymbolId;

pub type Span = std::ops::Range<usize>;

pub type AnnotationVec = InlineVec<AnnotationAppl>;

pub type Definition = Item<ItemKind>;

#[must_use]
#[derive(Debug)]
pub struct Document {
    /// Name of the file
    pub name: Symbol,

    /// Definitions found inside the file
    pub definitions: InlineVec<Definition>,
}

#[must_use]
#[derive(Default, Debug, Marshal, Unmarshal)]
pub struct Ident {
    /// The acutal identifier.
    pub name: Symbol,

    /// Span of the symbol.
    pub span: Span,
}

#[must_use]
#[derive(Debug)]
pub struct Item<K> {
    /// Name and span of the item.
    pub name: Ident,

    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: Span,

    /// Annotations that were applied to this item.
    pub annotations: AnnotationVec,

    /// Data of the underlying item type.
    pub kind: K,
}

impl Item<ItemKind> {
    pub fn decl(name: Ident, kind: DeclKind, span: Span) -> Self {
        Self {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Decl(Decl { kind }),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }
}

#[must_use]
#[derive(Debug)]
pub enum ItemKind {
    /// A definition of an annotation
    Annotation(P<AnnotationDef>),

    /// Module declaration
    Module(P<ModuleDef>),

    /// Struct definition
    Struct(P<StructDef>),

    /// Union definition
    Union(P<UnionDef>),

    /// Enum definition
    Enum(P<EnumDef>),

    /// Exception definition
    Exception(P<ExceptDef>),

    /// Bitmask definition
    Bitmask(P<BitmaskDef>),

    /// Bitset definition
    Bitset(P<BitsetDef>),

    /// Declaration of a `const`
    Const(P<ConstDef>),

    /// Typedef definition
    Typedef(P<Typedef>),

    /// Interface definition
    Interface(P<InterfaceDef>),

    /// Valuetype definition
    Valuetype(P<ValuetypeDef>),

    /// A forward declaration
    Decl(Decl),
}

#[must_use]
#[derive(Debug, Marshal, Unmarshal)]
pub struct Decl {
    pub kind: DeclKind,
}

#[must_use]
#[derive(Copy, Clone, Debug, Marshal, Unmarshal)]
pub enum DeclKind {
    Struct,
    Union,
    Native,
    Interface,
    Valuetype,
}

#[must_use]
#[derive(Debug, Marshal, Unmarshal)]
pub struct Path {
    pub leading_colons: Option<Span>,
    pub segments: InlineVec<Ident>,
}

impl Path {
    pub fn new(segments: InlineVec<Ident>) -> Self {
        Self {
            leading_colons: None,
            segments,
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct Fixed {
    pub total: usize,
    pub fractional: usize,
}

#[must_use]
#[derive(Debug)]
pub enum Type {
    /// The `any` type.
    Any { span: Span },

    /// Array of another type, e.g. `int32 value[3]`.
    /// Only the type is included; the name of the member is omitted.
    Array { ty: Path, bound: InlineVec<usize> },

    /// Sequence of another type, e.g. `sequence<string>`.
    Sequence { ty: P<Type>, bound: Option<usize> },

    /// A possibly bounded string.
    String { wide: bool, bound: Option<usize> },

    /// (key, value) pair of types, e.g. `map<string, string>`.
    Map {
        key: P<Type>,
        value: P<Type>,
        bound: Option<usize>,
    },

    /// Fixed-point type, e.g. `fixed` or `fixed<4, 2>`.
    Fixed { span: Span, bounds: Option<Fixed> },

    /// A possibly qualified identifier of a type, e.g. `foo::Bar`.
    Path(Path),
}

/// A definition of an annotation, e.g. `@annotation foo {};`.
#[must_use]
#[derive(Debug)]
pub struct AnnotationDef {
    pub params: InlineVec<AnnotationField>,
}

impl AnnotationDef {
    pub fn new(name: Ident, params: InlineVec<AnnotationField>, span: Span) -> Definition {
        let body = Self { params };

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Annotation(P(body)),
        }
    }
}

/// The items that can be placed inside a definition of an annotation.
#[must_use]
#[derive(Debug)]
pub enum AnnotationField {
    Enum(P<Item<EnumDef>>),
    Bitmask(P<Item<BitmaskDef>>),
    Const(P<Item<ConstDef>>),
    Field(P<Field>),
}

/// A parameter inside an applied annotation, e.g. `value=true` in
/// `@optional(value=true)`.
#[must_use]
#[derive(Debug)]
pub struct AnnotationArg {
    /// Name of the parameter if one was specified.
    /// May be omitted for annotations with only a single, non-default member.
    pub name: Option<Ident>,

    /// Span of the entire parameter.
    pub span: Span,

    /// The specified value of the parameter.
    pub value: Expr,
}

#[must_use]
#[derive(Debug)]
pub struct AnnotationAppl {
    pub name: Ident,
    pub span: Span,
    pub args: InlineVec<AnnotationArg>,
}

#[must_use]
#[derive(Debug)]
pub struct ModuleDef {
    pub defs: InlineVec<Item<ItemKind>>,
}

impl ModuleDef {
    pub fn new(name: Ident, defs: InlineVec<Definition>, span: Span) -> Definition {
        let body = Self { defs };

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Module(P(body)),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct StructDef {
    pub members: InlineVec<Field>,
    pub parent: Option<Path>,
}

impl StructDef {
    pub fn new(
        name: Ident,
        members: InlineVec<Field>,
        parent: Option<Path>,
        span: Span,
    ) -> Definition {
        let body = StructDef { members, parent };

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Struct(P(body)),
        }
    }

    pub fn with_parent(mut self, parent: Path) -> Self {
        self.parent = Some(parent);
        self
    }
}

#[must_use]
#[derive(Debug)]
pub struct Field {
    pub names: InlineVec<Ident>,
    pub ty: Type,
}

#[must_use]
#[derive(Debug)]
pub struct ExceptDef {
    pub members: InlineVec<Field>,
}

impl ExceptDef {
    pub fn new(name: Ident, members: InlineVec<Field>, span: Span) -> Definition {
        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Exception(P(ExceptDef { members })),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct BitmaskDef {
    pub annotations: InlineVec<AnnotationAppl>,
    pub bits: InlineVec<Bit>,
}

#[must_use]
#[derive(Debug)]
pub struct Bit {
    pub name: Ident,
    pub annotations: InlineVec<AnnotationAppl>,

    /// An explicit value, e.g. `bitmask Foo { VALUE = 1 };`
    /// The `@position` annotation will *not* populate this field.
    pub value: Option<Expr>,
}

#[must_use]
#[derive(Debug, Marshal, Unmarshal)]
pub struct BitsetDef {
    pub fields: InlineVec<Bitfield>,
}

#[must_use]
#[derive(Debug, Default, Marshal, Unmarshal)]
pub struct Bitfield {
    // pub annotations: InlineVec<AnnotationAppl>,
    pub size: Ident,
}

#[must_use]
#[derive(Debug)]
pub struct EnumDef {
    pub fields: InlineVec<Enumerator>,
}

impl EnumDef {
    pub fn new(name: Ident, enumerators: InlineVec<Enumerator>, span: Span) -> Definition {
        let body = P(Self {
            fields: enumerators,
        });

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Enum(body),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct Enumerator {
    pub annotations: InlineVec<AnnotationAppl>,
    pub name: Ident,

    /// An explicit value, e.g. `enum Foo { VALUE = 1 };`
    /// The `@value` annotation will *not* populate this field.
    pub value: Option<Expr>,
}

impl Enumerator {
    pub fn new(name: Ident, _span: Span) -> Self {
        Self {
            annotations: vec![],
            name,
            value: None,
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct UnionDef {
    /// The discriminator component of the union.
    pub disc: Discriminator,

    /// All variants of the union. The case labels that map to each variant can
    /// be found in `UnionField`.
    pub fields: InlineVec<UnionField>,
}

impl UnionDef {
    pub fn new(name: Ident, fields: InlineVec<UnionField>, span: Span) -> Definition {
        let body = Self {
            disc: Discriminator {
                annotations: vec![],
                ty: Type::Path(Path {
                    leading_colons: None,
                    segments: vec![],
                }),
            },
            fields,
        };

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Union(P(body)),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct Discriminator {
    pub annotations: InlineVec<AnnotationAppl>,
    pub ty: Type,
}

#[must_use]
#[derive(Debug)]
pub struct UnionField {
    pub annotations: InlineVec<AnnotationAppl>,

    /// Case labels that map to this variant.
    pub labels: InlineVec<Label>,

    pub field: Field,
}

#[must_use]
#[derive(Debug)]
pub enum Label {
    Case { ident: Path },
    Null,
    Default,
}

#[must_use]
#[derive(Debug)]
pub struct ConstDef {
    pub value: Expr,
    pub annotations: InlineVec<AnnotationAppl>,
}

impl ConstDef {
    pub fn new(name: Ident, ty: Type, span: Span) -> Definition {
        drop(ty);

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Const(P(Self {
                value: Expr::Lit(Literal {
                    kind: LitKind::Bool,
                    span: Span::default(),
                }),
                annotations: vec![],
            })),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct Typedef {
    /// The underlying type of the typedef.
    pub ty: Type,

    /// Annotations that are applied to the underlying type.
    pub annotations: InlineVec<AnnotationAppl>,
}

impl Typedef {
    pub fn new(name: Ident, ty: Type, span: Span) -> Definition {
        let body = Self {
            ty,
            annotations: vec![],
        };

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Typedef(P(body)),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct InterfaceDef {
    pub local: Option<Span>,
    pub definitions: InlineVec<()>,
}

#[must_use]
#[derive(Debug)]
pub enum InterfaceMember {
    /// An interface attribute.
    Attribute(Attribute),

    /// Function definition.
    Prototype(Prototype),

    /// Type definition nested inside the interface.
    Definition(Definition),
}

#[must_use]
#[derive(Debug)]
pub struct Attribute {
    /// Name of the attribute.
    pub name: Ident,

    /// The type of the attribute.
    pub ty: Type,

    /// Indicates whether this attribute was marked as `readonly`, and if
    /// so, the span of the keyword.
    pub readonly: Option<Span>,
}

#[must_use]
#[derive(Debug)]
pub struct Prototype {
    pub name: Ident,
    pub params: InlineVec<Param>,
    pub raises: InlineVec<Path>,
    pub oneway: Option<Span>,
}

#[must_use]
#[derive(Debug)]
pub enum ParamKind {
    /// Explicitly marked as `in`
    In,

    /// Explicitly marked as `out`
    Out,

    /// Explicitly marked as `inout`
    InOut,
}

#[must_use]
#[derive(Debug)]
pub struct Param {
    /// Name of the parameter.
    pub name: Ident,

    /// Type of the parameter.
    pub ty: Type,

    /// Specifies whether this is an `in`, `out`, or `inout` parameter.
    pub kind: Option<ParamKind>,
}

#[must_use]
#[derive(Debug)]
pub struct ValuetypeDef {
    pub members: InlineVec<ValueMember>,
    pub prototypes: InlineVec<Prototype>,
    pub inherits: Option<Path>,
    pub supports: Option<Path>,
}

impl ValuetypeDef {
    pub fn new(
        name: Ident,
        members: InlineVec<ValueMember>,
        inherits: Option<Path>,
        supports: Option<Path>,
        span: Span,
    ) -> Definition {
        let body = ValuetypeDef {
            members,
            inherits,
            supports,
            prototypes: vec![],
        };

        Definition {
            name,
            span,
            annotations: vec![],
            kind: ItemKind::Valuetype(P(body)),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct ValueMember {
    pub name: Ident,
    pub ty: Type,
    pub public: Option<Span>,
}

#[must_use]
#[derive(Debug, Marshal, Unmarshal)]
pub struct Literal {
    pub kind: LitKind,
    pub span: Span,
}

#[must_use]
#[derive(Copy, Clone, Debug, Marshal, Unmarshal)]
pub enum LitKind {
    Bool,
    Int,
    Float,
    Char,
    String,
    Ident,
}

#[must_use]
#[derive(Debug, Marshal, Unmarshal)]
pub struct Op {
    /// Span of the token.
    pub span: Span,

    /// The operation kind.
    pub kind: OpKind,
}

#[must_use]
#[derive(Copy, Clone, Debug, Marshal, Unmarshal)]
pub enum OpKind {
    // Arithmetic operations
    Add,
    Sub,
    Multiply,
    Divide,
    Modulo,

    // Bitwise operations
    LShift,
    RShift,
    Or,
    Xor,
    And,
    Not,
}

#[must_use]
#[derive(Debug)]
pub enum Expr {
    /// A single literal like `1` or `"foo"`
    Lit(Literal),

    /// `-a` or `a`
    Unary { op: Op, expr: P<Expr> },

    /// `a + b`
    Binary { lhs: P<Expr>, op: Op, rhs: P<Expr> },

    /// Initializer list for complex types, e.g. `{1, 2, {3}}`
    InitList(InlineVec<Expr>),
}
