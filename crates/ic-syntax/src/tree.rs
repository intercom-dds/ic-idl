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

//! Rust-native syntax tree for IDL.
//!
//! Unlike the legacy syntax tree, these types are designed as Rust types rather
//! than generated from an IDL schema. The module is intentionally independent
//! of the parser while the new representation is evaluated.

use crate::Span;

/// A value together with the span it was written at.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Spanned<T> {
    /// Span of the value in the source.
    pub span: Span,

    /// The value itself.
    pub value: T,
}

/// Source metadata shared by syntax nodes that may have annotations.
#[derive(Clone, Debug, PartialEq)]
pub struct Meta {
    /// Span of the entire syntax node.
    pub span: Span,

    /// Annotations applied to the syntax node.
    pub annotations: Vec<Annotation>,
}

/// An identifier in the source.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ident {
    /// Identifier text.
    pub name: String,

    /// Span of the identifier.
    pub span: Span,
}

/// A possibly absolute, scoped name.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Path {
    /// Span of the leading `::`, when this is an absolute path.
    pub leading_colons: Option<Span>,

    /// Path segments in source order.
    pub segments: Vec<Ident>,
}

/// The value and spelling category of an IDL literal.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Bool(bool),
    Int(u64),
    Float(f64),
    Char(char),
    WChar(char),
    String(String),
    WString(String),
}

/// An operator accepted in a unary or binary expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Op {
    /// `+`
    Add,

    /// `-`
    Sub,

    /// `*`
    Multiply,

    /// `/`
    Divide,

    /// `%`
    Modulo,

    /// `<<`
    LShift,

    /// `>>`
    RShift,

    /// `|`
    Or,

    /// `^`
    Xor,

    /// `&`
    And,

    /// `~`
    Not,
}

/// An IDL constant expression.
pub type Expr = Spanned<ExprKind>;

/// The kind and payload of an IDL constant expression.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Path(Path),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    InitList(Vec<NamedExpr>),
    Group(Box<Expr>),
}

/// A unary expression.
#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub op: Spanned<Op>,
    pub operand: Expr,
}

/// A binary expression.
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub lhs: Expr,
    pub op: Spanned<Op>,
    pub rhs: Expr,
}

/// An optionally designated initializer-list element.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedExpr {
    pub name: Option<Ident>,
    pub value: Expr,
}

/// A type as written in IDL source.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Sequence(Box<SequenceType>),
    String(StringType),
    Map(Box<MapType>),
    Fixed(FixedType),
    Named(Path),
}

/// A sequence type.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceType {
    pub element: Type,
    pub bound: Option<Expr>,
    pub span: Span,
    pub element_annotations: Vec<Annotation>,
}

/// A string type.
#[derive(Clone, Debug, PartialEq)]
pub struct StringType {
    pub kind: StringKind,
    pub bound: Option<Expr>,
    pub span: Span,
}

/// The character width of a string type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum StringKind {
    Narrow,
    Wide,
}

/// A map type.
#[derive(Clone, Debug, PartialEq)]
pub struct MapType {
    pub key: Type,
    pub value: Type,
    pub bound: Option<Expr>,
    pub span: Span,
    pub key_annotations: Vec<Annotation>,
    pub value_annotations: Vec<Annotation>,
}

/// A fixed-point type.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedType {
    pub span: Span,
    pub bounds: Option<FixedBounds>,
}

/// The total and fractional digit expressions of a fixed-point type.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedBounds {
    pub total: Expr,
    pub fractional: Expr,
}

/// A name declared with an optional array shape.
#[derive(Clone, Debug, PartialEq)]
pub enum Declarator {
    Name(Ident),
    Array(ArrayDeclarator),
}

/// An array declarator such as `value[3][4]`.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDeclarator {
    pub name: Ident,
    pub bounds: Vec<Expr>,
}

/// An annotation applied to a syntax node.
#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub path: Path,
    pub span: Span,
    pub arguments: Vec<AnnotationArg>,
}

/// An argument in an annotation application.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationArg {
    pub name: Option<Ident>,
    pub span: Span,
    pub value: Expr,
}

/// A definition of an annotation type.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationDef {
    pub meta: Meta,
    pub name: Ident,
    pub members: Vec<AnnotationMember>,
}

/// A member or nested definition in an annotation definition.
#[derive(Clone, Debug, PartialEq)]
pub enum AnnotationMember {
    Item(Item),
    Value(AnnotationValue),
}

/// A value member in an annotation definition.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationValue {
    pub meta: Meta,
    pub declarator: Declarator,
    pub ty: Type,
    pub default: Option<Expr>,
}

/// A module definition.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDef {
    pub meta: Meta,
    pub name: Ident,
    pub items: Vec<Item>,
}

/// A field in a struct or exception.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub meta: Meta,
    pub declarators: Vec<Declarator>,
    pub ty: Type,
}

/// A struct definition.
#[derive(Clone, Debug, PartialEq)]
pub struct StructDef {
    pub meta: Meta,
    pub name: Ident,
    pub parent: Option<Path>,
    pub fields: Vec<Field>,
}

/// A union discriminator.
#[derive(Clone, Debug, PartialEq)]
pub struct Disc {
    pub meta: Meta,
    pub ty: Type,
}

/// A label on a union case.
#[derive(Clone, Debug, PartialEq)]
pub enum Label {
    Value(Expr),
    Default(Span),
}

/// A case in a union definition.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionCase {
    pub meta: Meta,
    pub labels: Vec<Label>,
    pub ty: Type,
    pub declarator: Declarator,
}

/// A union definition.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionDef {
    pub meta: Meta,
    pub name: Ident,
    pub disc: Disc,
    pub cases: Vec<UnionCase>,
}

/// A constant definition.
#[derive(Clone, Debug, PartialEq)]
pub struct ConstDef {
    pub meta: Meta,
    pub declarator: Declarator,
    pub ty: Type,
    pub value: Expr,
}

/// An enumerator in an enum definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Enumerator {
    pub meta: Meta,
    pub name: Ident,
    pub value: Option<Expr>,
}

/// An enum definition.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumDef {
    pub meta: Meta,
    pub name: Ident,
    pub enumerators: Vec<Enumerator>,
}

/// An exception definition.
#[derive(Clone, Debug, PartialEq)]
pub struct ExceptDef {
    pub meta: Meta,
    pub name: Ident,
    pub fields: Vec<Field>,
}

/// A type alias definition.
#[derive(Clone, Debug, PartialEq)]
pub struct AliasDef {
    pub meta: Meta,
    pub declarators: Vec<Declarator>,
    pub ty: Type,
}

/// A bit in a bitmask definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Bit {
    pub meta: Meta,
    pub name: Ident,
    pub value: Option<Expr>,
}

/// A bitmask definition.
#[derive(Clone, Debug, PartialEq)]
pub struct BitmaskDef {
    pub meta: Meta,
    pub name: Ident,
    pub bits: Vec<Bit>,
}

/// A field in a bitset definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Bitfield {
    pub meta: Meta,
    pub size: Expr,
    pub ty: Option<Type>,
}

/// A bitset definition.
#[derive(Clone, Debug, PartialEq)]
pub struct BitsetDef {
    pub meta: Meta,
    pub name: Ident,
    pub parent: Option<Path>,
    pub fields: Vec<Bitfield>,
}

/// An interface or valuetype attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub meta: Meta,
    pub declarators: Vec<Declarator>,
    pub setraises: Vec<Path>,
    pub getraises: Vec<Path>,
    pub ty: Type,
    pub readonly: Option<Span>,
}

/// The direction explicitly written on an operation parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ParamKind {
    In,
    Out,
    InOut,
}

/// A parameter in an operation.
#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub meta: Meta,
    pub declarator: Declarator,
    pub ty: Type,
    pub kind: Option<ParamKind>,
}

/// An operation prototype in an interface or valuetype.
#[derive(Clone, Debug, PartialEq)]
pub struct Proto {
    pub meta: Meta,
    pub name: Ident,
    pub return_type: Type,
    pub parameters: Vec<Param>,
    pub raises: Vec<Path>,
    pub oneway: Option<Span>,
}

/// A member of an interface definition.
#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceMember {
    Attribute(Attribute),
    Proto(Proto),
    Item(Item),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InterfaceKind {
    Regular,
    Local(Span),
    Abstract(Span),
}

/// An interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceDef {
    pub meta: Meta,
    pub name: Ident,
    pub members: Vec<InterfaceMember>,
    pub inherits: Vec<Path>,
    pub kind: InterfaceKind,
}

/// The visibility of a valuetype state member.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Visibility {
    Public,
    Private,
}

/// A state member in a valuetype definition.
#[derive(Clone, Debug, PartialEq)]
pub struct StateMember {
    pub meta: Meta,
    pub declarators: Vec<Declarator>,
    pub ty: Type,
    pub visibility: Spanned<Visibility>,
}

/// A member of a valuetype definition.
#[derive(Clone, Debug, PartialEq)]
pub enum ValuetypeMember {
    State(StateMember),
    Attribute(Attribute),
    Proto(Proto),
    Item(Item),
}

/// A valuetype definition.
#[derive(Clone, Debug, PartialEq)]
pub struct ValuetypeDef {
    pub meta: Meta,
    pub name: Ident,
    pub members: Vec<ValuetypeMember>,
    pub inherits: Option<Path>,
    pub truncatable: Option<Span>,
    pub supports: Vec<Path>,
    pub abstract_: Option<Span>,
}

/// The kind of an incomplete or native declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DeclKind {
    Struct,
    Union,
    Native,
    Interface,
    Valuetype,
}

/// An incomplete or native declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct Decl {
    pub meta: Meta,
    pub name: Ident,
    pub kind: DeclKind,
    pub local: Option<Span>,
    pub abstract_: Option<Span>,
}

/// An item in an IDL source file or nested scope.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Annotation(AnnotationDef),
    Module(ModuleDef),
    Struct(StructDef),
    Union(UnionDef),
    Enum(EnumDef),
    Exception(ExceptDef),
    Bitmask(BitmaskDef),
    Bitset(BitsetDef),
    Const(ConstDef),
    Alias(AliasDef),
    Interface(InterfaceDef),
    Valuetype(ValuetypeDef),
    Decl(Decl),
}
