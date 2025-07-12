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

use std::fmt::Debug;
use std::num::NonZero;
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_macros::{EnumIter, ToString};
use ic_syntax::Path;
pub use ic_syntax::{Ident, ParamKind, Span};

pub type DefId = ic_alloc::arena::Id<Def>;

pub type TypeId = ic_alloc::arena::Id<Def>;

/// Built-in primitive types. These types are effectively stateless and have no
/// bounds or other attributes attached to them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, ToString)]
pub enum PrimitiveTy {
    Void,
    Bool,
    Char,
    WChar,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    Float128,
}

impl PrimitiveTy {
    pub fn name(&self) -> &str {
        match self {
            PrimitiveTy::Void => "void",
            PrimitiveTy::Bool => "boolean",
            PrimitiveTy::Char => "char",
            PrimitiveTy::WChar => "wchar",
            PrimitiveTy::Int8 => "int8",
            PrimitiveTy::UInt8 => "uint8",
            PrimitiveTy::Int16 => "int16",
            PrimitiveTy::UInt16 => "uint16",
            PrimitiveTy::Int32 => "int32",
            PrimitiveTy::UInt32 => "uint32",
            PrimitiveTy::Int64 => "int64",
            PrimitiveTy::UInt64 => "uint64",
            PrimitiveTy::Float32 => "float",
            PrimitiveTy::Float64 => "double",
            PrimitiveTy::Float128 => "long double",
        }
    }
}

intercom_cts::bitmask! {
    #[derive(Copy, Clone)]
    pub DefFlags: u32 {
        /// Indicates whether the type is recursive.
        IS_CIRCULAR = 1 << 0,

        /// Indicates whether the type is trivial, i.e. consists only of
        /// primitive types and arrays thereof.
        IS_TRIVIAL = 1 << 1,

        /// Marker for built-in types.
        IS_BUILTIN = 1 << 2,

        /// Marker for synthesized types.
        IS_SYNTHESIZED = 1 << 3,

        /// Marker for incomplete types.
        IS_INCOMPLETE = 1 << 4,

        /// Marker for non-suppressed types.
        IS_EMIT = 1 << 5,

        /// Indicates whether the type consists of members that can form an
        /// ordinal sequence, i.e. a well-ordered set.
        TOTAL_ORDER = 1 << 6,
    }
}

impl Default for DefFlags {
    fn default() -> Self {
        Self::IS_EMIT
    }
}

#[derive(Debug)]
pub struct Def {
    /// The ID of this definition.
    pub id: DefId,

    /// Name of the definition.
    pub ident: Ident,

    /// Parent definition, if any. None for top-level definitions.
    pub parent: Option<DefId>,

    /// Annotations attached to the definition.
    pub annotations: Vec<Ann>,

    /// Span of the whole definition of the type, typically from the type's
    /// keyword to the terminating semicolon.
    pub span: Span,

    /// Variant-specific data.
    pub kind: DefKind,

    pub flags: DefFlags,
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Annotation(AnnotationTy),
    Module(ModuleTy),
    Struct(StructTy),
    Except(ExceptTy),
    Union(UnionTy),
    Enum(EnumTy),
    Const(ConstTy),
    Bitmask(BitmaskTy),
    Alias(AliasTy),
    Interface(InterfaceTy),
    Valuetype(ValueTy),
    Decl(Decl),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Decl {
    Struct,
    Union,
    Native,
    Interface,
    Valuetype,
}

#[derive(Clone, Debug)]
pub struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

#[derive(Clone, Debug)]
pub enum TyKind {
    /// The `any` type.
    Any,

    /// Fixed<> types.
    Fixed,

    /// A primitive built-in type, such as `uint8` or `long long`.
    Primitive(PrimitiveTy),

    /// An array of type `ty` with bounds `len`.
    /// For multi-dimensional arrays, the type `ty` will point to another array.
    Array {
        ty: Box<Ty>,
        len: usize,
    },

    Sequence {
        ty: Box<Ty>,
        bound: Option<usize>,
    },

    String {
        wide: bool,
        bound: Option<usize>,
    },

    Map {
        key: Box<Ty>,
        elem: Box<Ty>,
        bound: Option<usize>,
    },

    /// An algebraic data type.
    Adt(DefId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Numeric {
    /// A `null` literal.
    Null,

    /// A boolean literal.
    Bool(bool),

    /// A char literal.
    Char(char),

    /// An i8 literal.
    Int8(i8),

    /// A u8 literal.
    Octet(u8),

    /// An i16 literal.
    Int16(i16),

    /// A u16 literal.
    UInt16(u16),

    /// An i32 literal.
    Int32(i32),

    /// A u32 literal.
    UInt32(u32),

    /// An i64 literal.
    Int64(i64),

    /// A u64 literal.
    UInt64(u64),

    /// An f32 literal.
    Float(f32),

    /// An f64 literal.
    Double(f64),

    /// A string literal.
    String(String),

    /// Value that points to another constant.
    /// To retrieve the fully resolved value, use [`Context::resolve_expr`].
    Const(DefId),

    /// Fixed-size array elements, e.g. `{1, 2, 3}`.
    Array { ty: TypeId, values: Box<[Numeric]> },

    /// Sequence elements, e.g. `{1, 2, 3}`.
    Sequence { ty: TypeId, values: Box<[Numeric]> },

    /// Map entries, eg. `{{key1, value1}, {key2, value2}}`.
    Map {
        ty: TypeId,
        values: Box<[(Numeric, Numeric)]>,
    },

    /// Struct initialization with named fields.
    Struct {
        ty: TypeId,
        fields: Box<[(Ident, Numeric)]>,
    },

    /// Union initialization with discriminant and value.
    Union {
        ty: TypeId,
        discriminant: Box<Numeric>,
        field: Ident,
        value: Box<Numeric>,
    },
}

#[derive(Debug, Clone)]
pub struct AnnotationTy {
    pub members: Vec<Member>,

    /// Types defined inside the annotation.
    pub types: Vec<DefId>,
}

#[derive(Debug, Clone)]
pub struct ModuleTy {
    pub definitions: Vec<DefId>,
}

#[derive(Debug, Clone)]
pub struct StructTy {
    /// Parent type, i.e. the type from which this type inherits.
    pub parent: Option<DefId>,

    /// Direct members of the struct. Does not include inherited members.
    pub members: Vec<Member>,
}

#[derive(Debug, Clone)]
pub struct Member {
    pub ident: Ident,
    pub ty: Ty,
    pub annotations: Vec<Ann>,
}

#[derive(Debug, Clone)]
pub struct ExceptTy {
    /// Direct members of the exception.
    pub members: Vec<Member>,
}

#[derive(Debug, Clone)]
pub struct UnionTy {
    /// The type of the union's discriminator.
    pub disc: Ty,

    /// The union's variants, i.e. its members.
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    /// Annotations attached to the variant.
    pub annotations: Vec<Ann>,

    /// Name of the variant.
    pub ident: Ident,

    /// Type of the variant.
    pub ty: Ty,

    /// All switch cases that map to this variant.
    // TODO: should this be a const? It can have a name, so maybe?
    pub labels: Vec<Numeric>,

    /// Indicates whether this variant has a default label.
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct EnumTy {
    pub fields: Vec<EnumLit>,

    /// Underlying primitive type of the enum.
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct EnumLit {
    pub ident: Ident,
    pub value: isize,
    pub annotations: Vec<Ann>,
}

#[derive(Debug, Clone)]
pub struct ConstTy {
    /// The value of the constant.
    pub value: Numeric,

    /// Type of the constant.
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct BitmaskTy {
    /// The bitmask flags.
    pub flags: Vec<BitFlag>,

    /// Underlying primitive type of the bitmask.
    // TODO: consider replacing with `PrimitiveTy`
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct BitFlag {
    /// Name of the bitmask flag.
    pub ident: Ident,

    /// Value of the flag.
    // TODO: numeric instead?
    // pub value: Numeric,
    pub value: usize,

    pub annotations: Vec<Ann>,
}

#[derive(Debug, Clone, Default)]
pub struct InterfaceTy {
    pub parents: Vec<DefId>,
    pub prototypes: Vec<ProtoTy>,
    pub attributes: Vec<()>,
    pub definitions: Vec<DefId>,
    pub is_local: bool,
}

#[derive(Debug, Clone)]
pub struct ValueTy {
    pub parent: Option<DefId>,
    pub extends: Option<DefId>,
    pub prototypes: Vec<ProtoTy>,
    pub members: Vec<()>,
    pub definitions: Vec<DefId>,
}

#[derive(Debug, Clone)]
pub struct ProtoTy {
    pub ident: Ident,
    pub ty: Ty,
    pub params: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ident: Ident,
    pub ty: Ty,
    pub kind: ParamKind,
}

#[derive(Debug, Clone)]
pub struct AliasTy {
    /// The type to which this alias points.
    pub ty: Ty,
}

/// An applied annotation
#[derive(Clone, Debug)]
pub struct Ann {
    pub path: Path,
    pub ty: Option<Ty>,
    pub args: Vec<AnnArg>,
}

// TODO: we should create a better representation that more cleanly maps from
// an applied annotation to an annotation def.
#[derive(Clone, Debug)]
pub struct AnnArg {
    // TODO: should never be none, we should insert the name
    pub ident: Option<Ident>,

    // TODO: we should resolve these as well
    // pub ty: Ty,
    pub value: Numeric,
}

macro_rules! numeric_from {
    ($($ty:ty => $var:ident),+ $(,)?) => {
        $(
            impl From<$ty> for Numeric {
                fn from(value: $ty) -> Self {
                    Self::$var(value)
                }
            }
        )+
    }
}

numeric_from! {
    i8 => Int8,
    i16 => Int16,
    i32 => Int32,
    i64 => Int64,
    u8 => Octet,
    u16 => UInt16,
    u32 => UInt32,
    u64 => UInt64,
    f32 => Float,
    f64 => Double,
    String => String,
    DefId => Const,
}
