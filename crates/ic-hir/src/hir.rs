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

//! Type definitions of the HIR.

use std::fmt::Debug;

pub use ic_syntax::{Ident, ParamKind, Span};

pub type DefId = ic_alloc::arena::Id<Def>;

/// Built-in primitive types. These types are effectively stateless and have no
/// bounds or other attributes attached to them.
#[must_use]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    #[must_use]
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

    #[must_use]
    pub fn legacy_name(&self) -> &str {
        match self {
            PrimitiveTy::Void => "void",
            PrimitiveTy::Bool => "boolean",
            PrimitiveTy::Char => "char",
            PrimitiveTy::WChar => "wchar",
            PrimitiveTy::Int8 => "int8",
            PrimitiveTy::UInt8 => "octet",
            PrimitiveTy::Int16 => "short",
            PrimitiveTy::UInt16 => "unsigned short",
            PrimitiveTy::Int32 => "long",
            PrimitiveTy::UInt32 => "unsigned long",
            PrimitiveTy::Int64 => "long long",
            PrimitiveTy::UInt64 => "unsigned long long",
            PrimitiveTy::Float32 => "float",
            PrimitiveTy::Float64 => "double",
            PrimitiveTy::Float128 => "long double",
        }
    }

    #[must_use]
    pub fn size(&self) -> usize {
        match self {
            PrimitiveTy::Void => 0,
            PrimitiveTy::Bool | PrimitiveTy::Char | PrimitiveTy::Int8 | PrimitiveTy::UInt8 => 1,
            PrimitiveTy::WChar | PrimitiveTy::Int16 | PrimitiveTy::UInt16 => 2,
            PrimitiveTy::Int32 | PrimitiveTy::UInt32 | PrimitiveTy::Float32 => 4,
            PrimitiveTy::Int64 | PrimitiveTy::UInt64 | PrimitiveTy::Float64 => 8,
            PrimitiveTy::Float128 => 16,
        }
    }
}

intercom_cts::bitmask! {
    #[must_use]
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

        /// An enumerator or bitmask position explicitly set.
        IS_ENUMERATED = 1 << 6,

        /// Indicates that another type inherits from this type.
        HAS_CHILDREN = 1 << 7,

        /// Indicates whether the type consists of members that can form an
        /// ordinal sequence, i.e. a well-ordered set.
        TOTAL_ORDER = 1 << 8,

        /// Indicates whether the type comes from a file
        /// that was not specified on the command line.
        IS_INCLUDED = 1 << 9,
    }
}

impl Default for DefFlags {
    fn default() -> Self {
        Self::IS_EMIT
    }
}

#[must_use]
#[derive(Clone, Debug)]
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

#[must_use]
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum DefKind {
    Annotation(AnnotationTy),
    Module(ModuleTy),
    Struct(StructTy),
    Except(ExceptTy),
    Union(UnionTy),
    Enum(EnumTy),
    Const(ConstTy),
    Bitmask(BitmaskTy),
    Bitset(BitsetTy),
    Alias(AliasTy),
    Interface(InterfaceTy),
    Valuetype(ValueTy),
    Decl(Decl),
}

impl DefKind {
    /// Returns a human-readable name for the definition kind.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            DefKind::Annotation(_) => "annotation",
            DefKind::Module(_) => "module",
            DefKind::Struct(_) => "struct",
            DefKind::Except(_) => "exception",
            DefKind::Union(_) => "union",
            DefKind::Enum(_) => "enum",
            DefKind::Const(_) => "const",
            DefKind::Bitmask(_) => "bitmask",
            DefKind::Bitset(_) => "bitset",
            DefKind::Alias(_) => "typedef",
            DefKind::Interface(_) => "interface",
            DefKind::Valuetype(_) => "valuetype",
            DefKind::Decl(decl) => match decl {
                Decl::Struct => "struct forward declaration",
                Decl::Union => "union forward declaration",
                Decl::Native => "native",
                Decl::Interface => "interface forward declaration",
                Decl::Valuetype => "valuetype forward declaration",
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Decl {
    Struct,
    Union,
    Native,
    Interface,
    Valuetype,
}

#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

#[must_use]
#[derive(Clone, Debug, PartialEq)]
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
        /// Span of the array bound expression (e.g., the "0" in "long[0]")
        len_span: Span,
    },

    Sequence {
        ty: Box<Ty>,
        bound: Option<usize>,
        /// Span of the sequence bound expression (if bounded)
        bound_span: Option<Span>,
    },

    String {
        wide: bool,
        bound: Option<usize>,
        /// Span of the string bound expression (if bounded)
        bound_span: Option<Span>,
    },

    Map {
        key: Box<Ty>,
        elem: Box<Ty>,
        bound: Option<usize>,
        /// Span of the map bound expression (if bounded)
        bound_span: Option<Span>,
    },

    /// The null type.
    Null,

    /// An algebraic data type.
    Adt(DefId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Numeric {
    /// A `null` literal.
    Null,

    /// A boolean literal.
    Bool(bool),

    /// A narrow char literal.
    Char(char),

    /// A wide char literal.
    WChar(char),

    /// An i8 literal.
    Int8(i8),

    /// A u8 literal.
    UInt8(u8),

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

    /// A narrow string literal.
    String(String),

    /// A wide string literal.
    WString(String),

    /// Value that points to another constant.
    /// To retrieve the fully resolved value, use [`Context::resolve_expr`].
    Const(DefId),

    /// Fixed-size array elements, e.g. `{1, 2, 3}`.
    Array { ty: Ty, values: Box<[Numeric]> },

    /// Sequence elements, e.g. `{1, 2, 3}`.
    Sequence { ty: Ty, values: Box<[Numeric]> },

    /// Map entries, eg. `{{key1, value1}, {key2, value2}}`.
    Map {
        key: Ty,
        value: Ty,
        entries: Box<[(Numeric, Numeric)]>,
    },

    /// Struct initialization with field values in declaration order.
    /// Field names are not stored; look them up from the struct definition.
    Struct { ty: DefId, fields: Box<[Numeric]> },

    /// Union initialization with discriminant and value.
    /// The field index refers to the variant in the union definition.
    Union {
        ty: DefId,
        discriminant: Box<Numeric>,
        field_index: usize,
        value: Box<Numeric>,
    },
}

#[derive(Debug, Clone)]
pub struct AnnotationTy {
    /// Parameters of the annotation
    pub params: Vec<AnnParam>,

    /// Types defined inside the annotation.
    pub types: Vec<DefId>,
}

#[derive(Debug, Clone)]
pub struct AnnParam {
    /// Parameter name
    pub ident: Ident,

    /// Parameter type
    pub ty: Ty,

    /// Default value for the parameter
    pub default: Option<Numeric>,

    /// Annotations attached to the annotation member.
    pub annotations: Vec<Ann>,
}

#[derive(Debug, Clone)]
pub struct ModuleTy {
    pub definitions: Vec<DefId>,
}

#[derive(Debug, Clone)]
pub struct StructTy {
    /// Parent type, i.e. the type from which this type inherits.
    pub parent: Option<Spanned<DefId>>,

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
pub struct Disc {
    /// Annotations applied to the discriminator.
    pub annotations: Vec<Ann>,

    /// The type of the union's discriminator.
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct UnionTy {
    /// The type of the union's discriminator.
    pub disc: Disc,

    /// The union's variants, i.e. its members.
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct Label {
    /// The evaluated value of the case label.
    pub value: Numeric,

    /// The span of the label expression/identifier.
    pub span: Span,
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
    pub labels: Vec<Label>,

    /// Indicates whether this variant has a default label.
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct EnumTy {
    /// `DefIds` of the enum constants
    pub fields: Vec<DefId>,

    /// Underlying primitive type of the enum.
    pub ty: PrimitiveTy,
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
    /// `DefIds` of the bitmask flag constants
    pub flags: Vec<DefId>,

    /// Underlying primitive type of the bitmask.
    pub ty: PrimitiveTy,
}

#[derive(Debug, Clone)]
pub struct BitsetTy {
    /// Parent bitset for inheritance.
    pub parent: Option<Spanned<DefId>>,

    /// The bitset fields.
    pub fields: Vec<BitsetField>,
}

#[derive(Debug, Clone)]
pub struct BitsetField {
    /// Name of the bitfield.
    pub ident: Ident,

    /// Size in bits (evaluated expression).
    pub size: usize,

    /// Type for the field.
    pub ty: Ty,

    /// Annotations on this field.
    pub annotations: Vec<Ann>,
}

#[derive(Debug, Clone, Default)]
pub struct InterfaceTy {
    pub parents: Vec<Spanned<DefId>>,
    pub prototypes: Vec<ProtoTy>,
    pub attributes: Vec<Attribute>,
    pub definitions: Vec<DefId>,
    pub is_local: bool,
}

#[derive(Debug, Clone)]
pub struct ValueTy {
    pub parent: Option<Spanned<DefId>>,
    pub supports: Option<Spanned<DefId>>,
    pub prototypes: Vec<ProtoTy>,
    pub attributes: Vec<Attribute>,
    pub members: Vec<Member>,
    pub definitions: Vec<DefId>,
}

#[derive(Debug, Clone)]
pub struct ProtoTy {
    pub ident: Ident,
    pub ty: Ty,
    pub params: Vec<Parameter>,
    pub raises: Vec<Spanned<DefId>>,
    pub annotations: Vec<Ann>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ident: Ident,
    pub ty: Ty,
    pub kind: ParamKind,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub ident: Ident,
    pub ty: Ty,
    pub is_readonly: bool,
    pub getraises: Vec<Spanned<DefId>>,
    pub setraises: Vec<Spanned<DefId>>,
    pub annotations: Vec<Ann>,
}

#[derive(Copy, Clone, Debug)]
pub struct Spanned<T> {
    pub def_id: T,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AliasTy {
    /// The type to which this alias points.
    pub ty: Ty,
}

/// An applied annotation
#[derive(Clone, Debug)]
pub struct Ann {
    /// The annotation name
    pub ident: Ident,

    /// Reference to the @annotation definition
    pub def_id: Option<DefId>,

    /// Arguments passed to the annotation
    pub args: Vec<AnnArg>,
}

#[derive(Clone, Debug)]
pub struct AnnArg {
    /// Argument name
    pub ident: Ident,

    /// The argument value
    pub value: Numeric,

    /// The resolved type of the parameter (from the annotation definition)
    pub ty: Option<Ty>,
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
    u8 => UInt8,
    u16 => UInt16,
    u32 => UInt32,
    u64 => UInt64,
    f32 => Float,
    f64 => Double,
    String => String,
    DefId => Const,
}

impl std::fmt::Display for Def {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident.name)
    }
}
