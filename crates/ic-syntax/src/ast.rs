// @generated
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

#![allow(clippy::match_wildcard_for_single_variants)]

pub use ic_vfs::Span;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ident {
    /// The actual identifier.
    pub name: String,
    /// Span of the symbol.
    pub span: crate::ast::Span,
}

impl Ident {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: <String>::default(),
            span: <crate::ast::Span>::default(),
        }
    }
}

impl ::std::default::Default for Ident {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Ident {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Ident")?;
        state.encode_field(0, "name", &self.name)?;
        state.encode_field(1, "span", &self.span)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Ident {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Ident")?;
        state.decode_field(0, "name", &mut self.name)?;
        state.decode_field(1, "span", &mut self.span)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Path {
    pub leading_colons: Option<crate::ast::Span>,
    pub segments: Vec<crate::ast::Ident>,
}

impl Path {
    #[must_use]
    pub fn new() -> Self {
        Self {
            leading_colons: None,
            segments: <Vec<crate::ast::Ident>>::default(),
        }
    }
}

impl ::std::default::Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Path {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Path")?;
        state.encode_field(0, "leading_colons", &self.leading_colons)?;
        state.encode_field(1, "segments", &self.segments)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Path {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Path")?;
        state.decode_field(0, "leading_colons", &mut self.leading_colons)?;
        state.decode_field(1, "segments", &mut self.segments)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum LitKind {
    LitBool,
    LitInt,
    LitFloat,
    LitChar,
    LitString,
}

impl LitKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::LitKind::LitBool
    }
}

impl ::std::str::FromStr for LitKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "LIT_BOOL" => Ok(Self::LitBool),
            "LIT_INT" => Ok(Self::LitInt),
            "LIT_FLOAT" => Ok(Self::LitFloat),
            "LIT_CHAR" => Ok(Self::LitChar),
            "LIT_STRING" => Ok(Self::LitString),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for LitKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::LitBool => f.write_str("LIT_BOOL"),
            Self::LitInt => f.write_str("LIT_INT"),
            Self::LitFloat => f.write_str("LIT_FLOAT"),
            Self::LitChar => f.write_str("LIT_CHAR"),
            Self::LitString => f.write_str("LIT_STRING"),
        }
    }
}

impl ::std::default::Default for LitKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for LitKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("LitKind")?;
        match self {
            Self::LitBool => state.encode_variant::<i32>("LIT_BOOL", 0),
            Self::LitInt => state.encode_variant::<i32>("LIT_INT", 1),
            Self::LitFloat => state.encode_variant::<i32>("LIT_FLOAT", 2),
            Self::LitChar => state.encode_variant::<i32>("LIT_CHAR", 3),
            Self::LitString => state.encode_variant::<i32>("LIT_STRING", 4),
        }
    }
}

impl ::intercom_cts::Unmarshal for LitKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("LitKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for LitKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::LitBool,
            1 => Self::LitInt,
            2 => Self::LitFloat,
            3 => Self::LitChar,
            4 => Self::LitString,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "LIT_BOOL" => Self::LitBool,
            "LIT_INT" => Self::LitInt,
            "LIT_FLOAT" => Self::LitFloat,
            "LIT_CHAR" => Self::LitChar,
            "LIT_STRING" => Self::LitString,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LiteralValue {
    Bool(bool),
    Int(u64),
    Float(f64),
    Char(char),
    String(String),
}

impl LiteralValue {
    #[must_use]
    pub fn new() -> Self {
        Self::Bool(false)
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::LitKind {
        match self {
            Self::Bool(_) => crate::ast::LitKind::LitBool,
            Self::Int(_) => crate::ast::LitKind::LitInt,
            Self::Float(_) => crate::ast::LitKind::LitFloat,
            Self::Char(_) => crate::ast::LitKind::LitChar,
            Self::String(_) => crate::ast::LitKind::LitString,
        }
    }
}

impl From<crate::ast::LitKind> for LiteralValue {
    fn from(disc: crate::ast::LitKind) -> Self {
        match disc {
            crate::ast::LitKind::LitBool => Self::Bool(false),
            crate::ast::LitKind::LitInt => Self::Int(0),
            crate::ast::LitKind::LitFloat => Self::Float(0_f64),
            crate::ast::LitKind::LitChar => Self::Char('\x00'),
            crate::ast::LitKind::LitString => Self::String(<String>::default()),
        }
    }
}

impl ::std::default::Default for LiteralValue {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for LiteralValue {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("LiteralValue")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Bool(v) => state.encode_variant(0, "bool", v),
            Self::Int(v) => state.encode_variant(1, "int", v),
            Self::Float(v) => state.encode_variant(2, "float", v),
            Self::Char(v) => state.encode_variant(3, "char", v),
            Self::String(v) => state.encode_variant(4, "string", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for LiteralValue {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("LiteralValue")?;
        let mut disc = crate::ast::LitKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::LitKind::LitBool => {
                let mut value = false;
                state.decode_variant(0, "bool", &mut value)?;
                Self::Bool(value)
            }
            crate::ast::LitKind::LitInt => {
                let mut value = 0;
                state.decode_variant(1, "int", &mut value)?;
                Self::Int(value)
            }
            crate::ast::LitKind::LitFloat => {
                let mut value = 0_f64;
                state.decode_variant(2, "float", &mut value)?;
                Self::Float(value)
            }
            crate::ast::LitKind::LitChar => {
                let mut value = '\x00';
                state.decode_variant(3, "char", &mut value)?;
                Self::Char(value)
            }
            crate::ast::LitKind::LitString => {
                let mut value = <String>::default();
                state.decode_variant(4, "string", &mut value)?;
                Self::String(value)
            }
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Literal {
    pub span: crate::ast::Span,
    pub value: crate::ast::LiteralValue,
}

impl Literal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            value: <crate::ast::LiteralValue>::default(),
        }
    }
}

impl ::std::default::Default for Literal {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Literal {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Literal")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "value", &self.value)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Literal {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Literal")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "value", &mut self.value)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum OpKind {
    /// `+`
    OpAdd,
    /// `-`
    OpSub,
    /// `*`
    OpMultiply,
    /// `/`
    OpDivide,
    /// `%`
    OpModulo,
    /// `<<`
    OpLshift,
    /// `>>`
    OpRshift,
    /// `|`
    OpOr,
    /// `^`
    OpXor,
    /// `&`
    OpAnd,
    /// `~`
    OpNot,
}

impl OpKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::OpKind::OpAdd
    }
}

impl ::std::str::FromStr for OpKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "OP_ADD" => Ok(Self::OpAdd),
            "OP_SUB" => Ok(Self::OpSub),
            "OP_MULTIPLY" => Ok(Self::OpMultiply),
            "OP_DIVIDE" => Ok(Self::OpDivide),
            "OP_MODULO" => Ok(Self::OpModulo),
            "OP_LSHIFT" => Ok(Self::OpLshift),
            "OP_RSHIFT" => Ok(Self::OpRshift),
            "OP_OR" => Ok(Self::OpOr),
            "OP_XOR" => Ok(Self::OpXor),
            "OP_AND" => Ok(Self::OpAnd),
            "OP_NOT" => Ok(Self::OpNot),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for OpKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::OpAdd => f.write_str("OP_ADD"),
            Self::OpSub => f.write_str("OP_SUB"),
            Self::OpMultiply => f.write_str("OP_MULTIPLY"),
            Self::OpDivide => f.write_str("OP_DIVIDE"),
            Self::OpModulo => f.write_str("OP_MODULO"),
            Self::OpLshift => f.write_str("OP_LSHIFT"),
            Self::OpRshift => f.write_str("OP_RSHIFT"),
            Self::OpOr => f.write_str("OP_OR"),
            Self::OpXor => f.write_str("OP_XOR"),
            Self::OpAnd => f.write_str("OP_AND"),
            Self::OpNot => f.write_str("OP_NOT"),
        }
    }
}

impl ::std::default::Default for OpKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for OpKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("OpKind")?;
        match self {
            Self::OpAdd => state.encode_variant::<i32>("OP_ADD", 0),
            Self::OpSub => state.encode_variant::<i32>("OP_SUB", 1),
            Self::OpMultiply => state.encode_variant::<i32>("OP_MULTIPLY", 2),
            Self::OpDivide => state.encode_variant::<i32>("OP_DIVIDE", 3),
            Self::OpModulo => state.encode_variant::<i32>("OP_MODULO", 4),
            Self::OpLshift => state.encode_variant::<i32>("OP_LSHIFT", 5),
            Self::OpRshift => state.encode_variant::<i32>("OP_RSHIFT", 6),
            Self::OpOr => state.encode_variant::<i32>("OP_OR", 7),
            Self::OpXor => state.encode_variant::<i32>("OP_XOR", 8),
            Self::OpAnd => state.encode_variant::<i32>("OP_AND", 9),
            Self::OpNot => state.encode_variant::<i32>("OP_NOT", 10),
        }
    }
}

impl ::intercom_cts::Unmarshal for OpKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("OpKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for OpKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::OpAdd,
            1 => Self::OpSub,
            2 => Self::OpMultiply,
            3 => Self::OpDivide,
            4 => Self::OpModulo,
            5 => Self::OpLshift,
            6 => Self::OpRshift,
            7 => Self::OpOr,
            8 => Self::OpXor,
            9 => Self::OpAnd,
            10 => Self::OpNot,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "OP_ADD" => Self::OpAdd,
            "OP_SUB" => Self::OpSub,
            "OP_MULTIPLY" => Self::OpMultiply,
            "OP_DIVIDE" => Self::OpDivide,
            "OP_MODULO" => Self::OpModulo,
            "OP_LSHIFT" => Self::OpLshift,
            "OP_RSHIFT" => Self::OpRshift,
            "OP_OR" => Self::OpOr,
            "OP_XOR" => Self::OpXor,
            "OP_AND" => Self::OpAnd,
            "OP_NOT" => Self::OpNot,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Op {
    /// Span of the token.
    pub span: crate::ast::Span,
    /// The operation kind.
    pub kind: crate::ast::OpKind,
}

impl Op {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            kind: crate::ast::OpKind::OpAdd,
        }
    }
}

impl ::std::default::Default for Op {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Op {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Op")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "kind", &self.kind)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Op {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Op")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "kind", &mut self.kind)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum ExprKind {
    /// A single literal like `1` or `"foo"`
    ExprLiteral,
    /// A possibly scoped identifier like `foo` or `::foo::bar`
    ExprPath,
    /// `-a` or `a`
    ExprUnary,
    /// `a + b`
    ExprBinary,
    /// Initializer list for complex types, e.g. `{1, 2, {3}}`
    ExprInitList,
}

impl ExprKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::ExprKind::ExprLiteral
    }
}

impl ::std::str::FromStr for ExprKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "EXPR_LITERAL" => Ok(Self::ExprLiteral),
            "EXPR_PATH" => Ok(Self::ExprPath),
            "EXPR_UNARY" => Ok(Self::ExprUnary),
            "EXPR_BINARY" => Ok(Self::ExprBinary),
            "EXPR_INIT_LIST" => Ok(Self::ExprInitList),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for ExprKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::ExprLiteral => f.write_str("EXPR_LITERAL"),
            Self::ExprPath => f.write_str("EXPR_PATH"),
            Self::ExprUnary => f.write_str("EXPR_UNARY"),
            Self::ExprBinary => f.write_str("EXPR_BINARY"),
            Self::ExprInitList => f.write_str("EXPR_INIT_LIST"),
        }
    }
}

impl ::std::default::Default for ExprKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ExprKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("ExprKind")?;
        match self {
            Self::ExprLiteral => state.encode_variant::<i32>("EXPR_LITERAL", 0),
            Self::ExprPath => state.encode_variant::<i32>("EXPR_PATH", 1),
            Self::ExprUnary => state.encode_variant::<i32>("EXPR_UNARY", 2),
            Self::ExprBinary => state.encode_variant::<i32>("EXPR_BINARY", 3),
            Self::ExprInitList => state.encode_variant::<i32>("EXPR_INIT_LIST", 4),
        }
    }
}

impl ::intercom_cts::Unmarshal for ExprKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("ExprKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for ExprKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::ExprLiteral,
            1 => Self::ExprPath,
            2 => Self::ExprUnary,
            3 => Self::ExprBinary,
            4 => Self::ExprInitList,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "EXPR_LITERAL" => Self::ExprLiteral,
            "EXPR_PATH" => Self::ExprPath,
            "EXPR_UNARY" => Self::ExprUnary,
            "EXPR_BINARY" => Self::ExprBinary,
            "EXPR_INIT_LIST" => Self::ExprInitList,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct InitList {
    pub values: Vec<crate::ast::NamedExpr>,
}

impl InitList {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: <Vec<crate::ast::NamedExpr>>::default(),
        }
    }
}

impl ::std::default::Default for InitList {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for InitList {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("InitList")?;
        state.encode_field(0, "values", &self.values)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for InitList {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("InitList")?;
        state.decode_field(0, "values", &mut self.values)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Expr {
    Literal(crate::ast::Literal),
    Path(crate::ast::Path),
    Unary(Box<crate::ast::Unary>),
    Binary(Box<crate::ast::Binary>),
    InitList(crate::ast::InitList),
}

impl Expr {
    #[must_use]
    pub fn new() -> Self {
        Self::Literal(<crate::ast::Literal>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::ExprKind {
        match self {
            Self::Literal(_) => crate::ast::ExprKind::ExprLiteral,
            Self::Path(_) => crate::ast::ExprKind::ExprPath,
            Self::Unary(_) => crate::ast::ExprKind::ExprUnary,
            Self::Binary(_) => crate::ast::ExprKind::ExprBinary,
            Self::InitList(_) => crate::ast::ExprKind::ExprInitList,
        }
    }
}

impl From<crate::ast::ExprKind> for Expr {
    fn from(disc: crate::ast::ExprKind) -> Self {
        match disc {
            crate::ast::ExprKind::ExprLiteral => Self::Literal(<crate::ast::Literal>::default()),
            crate::ast::ExprKind::ExprPath => Self::Path(<crate::ast::Path>::default()),
            crate::ast::ExprKind::ExprUnary => {
                Self::Unary(Box::new(<crate::ast::Unary>::default()))
            }
            crate::ast::ExprKind::ExprBinary => {
                Self::Binary(Box::new(<crate::ast::Binary>::default()))
            }
            crate::ast::ExprKind::ExprInitList => Self::InitList(<crate::ast::InitList>::default()),
        }
    }
}

impl ::std::default::Default for Expr {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Expr {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("Expr")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Literal(v) => state.encode_variant(0, "literal", v),
            Self::Path(v) => state.encode_variant(1, "path", v),
            Self::Unary(v) => state.encode_variant(2, "unary", v),
            Self::Binary(v) => state.encode_variant(3, "binary", v),
            Self::InitList(v) => state.encode_variant(4, "init_list", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for Expr {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("Expr")?;
        let mut disc = crate::ast::ExprKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::ExprKind::ExprLiteral => {
                let mut value = <crate::ast::Literal>::default();
                state.decode_variant(0, "literal", &mut value)?;
                Self::Literal(value)
            }
            crate::ast::ExprKind::ExprPath => {
                let mut value = <crate::ast::Path>::default();
                state.decode_variant(1, "path", &mut value)?;
                Self::Path(value)
            }
            crate::ast::ExprKind::ExprUnary => {
                let mut value = Box::new(<crate::ast::Unary>::default());
                state.decode_variant(2, "unary", &mut value)?;
                Self::Unary(value)
            }
            crate::ast::ExprKind::ExprBinary => {
                let mut value = Box::new(<crate::ast::Binary>::default());
                state.decode_variant(3, "binary", &mut value)?;
                Self::Binary(value)
            }
            crate::ast::ExprKind::ExprInitList => {
                let mut value = <crate::ast::InitList>::default();
                state.decode_variant(4, "init_list", &mut value)?;
                Self::InitList(value)
            }
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct NamedExpr {
    pub ident: Option<crate::ast::Ident>,
    pub value: crate::ast::Expr,
}

impl NamedExpr {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: None,
            value: <crate::ast::Expr>::default(),
        }
    }
}

impl ::std::default::Default for NamedExpr {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for NamedExpr {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("NamedExpr")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "value", &self.value)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for NamedExpr {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("NamedExpr")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "value", &mut self.value)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Unary {
    pub op: crate::ast::Op,
    pub expr: crate::ast::Expr,
}

impl Unary {
    #[must_use]
    pub fn new() -> Self {
        Self {
            op: <crate::ast::Op>::default(),
            expr: <crate::ast::Expr>::default(),
        }
    }
}

impl ::std::default::Default for Unary {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Unary {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Unary")?;
        state.encode_field(0, "op", &self.op)?;
        state.encode_field(1, "expr", &self.expr)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Unary {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Unary")?;
        state.decode_field(0, "op", &mut self.op)?;
        state.decode_field(1, "expr", &mut self.expr)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Binary {
    pub lhs: crate::ast::Expr,
    pub op: crate::ast::Op,
    pub rhs: crate::ast::Expr,
}

impl Binary {
    #[must_use]
    pub fn new() -> Self {
        Self {
            lhs: <crate::ast::Expr>::default(),
            op: <crate::ast::Op>::default(),
            rhs: <crate::ast::Expr>::default(),
        }
    }
}

impl ::std::default::Default for Binary {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Binary {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Binary")?;
        state.encode_field(0, "lhs", &self.lhs)?;
        state.encode_field(1, "op", &self.op)?;
        state.encode_field(2, "rhs", &self.rhs)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Binary {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Binary")?;
        state.decode_field(0, "lhs", &mut self.lhs)?;
        state.decode_field(1, "op", &mut self.op)?;
        state.decode_field(2, "rhs", &mut self.rhs)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AnyType {
    pub span: crate::ast::Span,
}

impl AnyType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
        }
    }
}

impl ::std::default::Default for AnyType {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AnyType {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("AnyType")?;
        state.encode_field(0, "span", &self.span)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for AnyType {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("AnyType")?;
        state.decode_field(0, "span", &mut self.span)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SequenceType {
    pub ty: Box<crate::ast::Type>,
    pub bound: Option<crate::ast::Expr>,
    pub span: crate::ast::Span,
}

impl SequenceType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ty: Box::new(<crate::ast::Type>::default()),
            bound: None,
            span: <crate::ast::Span>::default(),
        }
    }
}

impl ::std::default::Default for SequenceType {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for SequenceType {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("SequenceType")?;
        state.encode_field(0, "ty", &self.ty)?;
        state.encode_field(1, "bound", &self.bound)?;
        state.encode_field(2, "span", &self.span)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for SequenceType {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("SequenceType")?;
        state.decode_field(0, "ty", &mut self.ty)?;
        state.decode_field(1, "bound", &mut self.bound)?;
        state.decode_field(2, "span", &mut self.span)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct StringType {
    pub wide: bool,
    pub bound: Option<crate::ast::Expr>,
    pub span: crate::ast::Span,
}

impl StringType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            wide: false,
            bound: None,
            span: <crate::ast::Span>::default(),
        }
    }
}

impl ::std::default::Default for StringType {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for StringType {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("StringType")?;
        state.encode_field(0, "wide", &self.wide)?;
        state.encode_field(1, "bound", &self.bound)?;
        state.encode_field(2, "span", &self.span)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for StringType {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("StringType")?;
        state.decode_field(0, "wide", &mut self.wide)?;
        state.decode_field(1, "bound", &mut self.bound)?;
        state.decode_field(2, "span", &mut self.span)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MapType {
    pub key: Box<crate::ast::Type>,
    pub value: Box<crate::ast::Type>,
    pub bound: Option<crate::ast::Expr>,
    pub span: crate::ast::Span,
}

impl MapType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            key: Box::new(<crate::ast::Type>::default()),
            value: Box::new(<crate::ast::Type>::default()),
            bound: None,
            span: <crate::ast::Span>::default(),
        }
    }
}

impl ::std::default::Default for MapType {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for MapType {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("MapType")?;
        state.encode_field(0, "key", &self.key)?;
        state.encode_field(1, "value", &self.value)?;
        state.encode_field(2, "bound", &self.bound)?;
        state.encode_field(3, "span", &self.span)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for MapType {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("MapType")?;
        state.decode_field(0, "key", &mut self.key)?;
        state.decode_field(1, "value", &mut self.value)?;
        state.decode_field(2, "bound", &mut self.bound)?;
        state.decode_field(3, "span", &mut self.span)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Fixed {
    pub total: crate::ast::Expr,
    pub fractional: crate::ast::Expr,
}

impl Fixed {
    #[must_use]
    pub fn new() -> Self {
        Self {
            total: <crate::ast::Expr>::default(),
            fractional: <crate::ast::Expr>::default(),
        }
    }
}

impl ::std::default::Default for Fixed {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Fixed {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Fixed")?;
        state.encode_field(0, "total", &self.total)?;
        state.encode_field(1, "fractional", &self.fractional)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Fixed {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Fixed")?;
        state.decode_field(0, "total", &mut self.total)?;
        state.decode_field(1, "fractional", &mut self.fractional)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FixedType {
    pub span: crate::ast::Span,
    pub bounds: Option<crate::ast::Fixed>,
}

impl FixedType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            bounds: None,
        }
    }
}

impl ::std::default::Default for FixedType {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for FixedType {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("FixedType")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "bounds", &self.bounds)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for FixedType {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("FixedType")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "bounds", &mut self.bounds)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum TypeKind {
    TypeAny,
    TypeSequence,
    TypeString,
    TypeMap,
    TypeFixed,
    TypePath,
}

impl TypeKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::TypeKind::TypeAny
    }
}

impl ::std::str::FromStr for TypeKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "TYPE_ANY" => Ok(Self::TypeAny),
            "TYPE_SEQUENCE" => Ok(Self::TypeSequence),
            "TYPE_STRING" => Ok(Self::TypeString),
            "TYPE_MAP" => Ok(Self::TypeMap),
            "TYPE_FIXED" => Ok(Self::TypeFixed),
            "TYPE_PATH" => Ok(Self::TypePath),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for TypeKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::TypeAny => f.write_str("TYPE_ANY"),
            Self::TypeSequence => f.write_str("TYPE_SEQUENCE"),
            Self::TypeString => f.write_str("TYPE_STRING"),
            Self::TypeMap => f.write_str("TYPE_MAP"),
            Self::TypeFixed => f.write_str("TYPE_FIXED"),
            Self::TypePath => f.write_str("TYPE_PATH"),
        }
    }
}

impl ::std::default::Default for TypeKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for TypeKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("TypeKind")?;
        match self {
            Self::TypeAny => state.encode_variant::<i32>("TYPE_ANY", 0),
            Self::TypeSequence => state.encode_variant::<i32>("TYPE_SEQUENCE", 1),
            Self::TypeString => state.encode_variant::<i32>("TYPE_STRING", 2),
            Self::TypeMap => state.encode_variant::<i32>("TYPE_MAP", 3),
            Self::TypeFixed => state.encode_variant::<i32>("TYPE_FIXED", 4),
            Self::TypePath => state.encode_variant::<i32>("TYPE_PATH", 5),
        }
    }
}

impl ::intercom_cts::Unmarshal for TypeKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("TypeKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for TypeKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::TypeAny,
            1 => Self::TypeSequence,
            2 => Self::TypeString,
            3 => Self::TypeMap,
            4 => Self::TypeFixed,
            5 => Self::TypePath,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "TYPE_ANY" => Self::TypeAny,
            "TYPE_SEQUENCE" => Self::TypeSequence,
            "TYPE_STRING" => Self::TypeString,
            "TYPE_MAP" => Self::TypeMap,
            "TYPE_FIXED" => Self::TypeFixed,
            "TYPE_PATH" => Self::TypePath,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Type {
    /// The `any` type.
    Any(crate::ast::AnyType),
    /// Sequence of another type, e.g. `sequence<string>`.
    Sequence(crate::ast::SequenceType),
    /// A possibly bounded string.
    String(crate::ast::StringType),
    /// (key, value) pair of types, e.g. `map<string, string>`.
    Map(crate::ast::MapType),
    /// Fixed-point type, e.g. `fixed` or `fixed<4, 2>`.
    Fixed(crate::ast::FixedType),
    /// A possibly qualified identifier of a type, e.g. `foo::Bar`.
    Path(crate::ast::Path),
}

impl Type {
    #[must_use]
    pub fn new() -> Self {
        Self::Any(<crate::ast::AnyType>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::TypeKind {
        match self {
            Self::Any(_) => crate::ast::TypeKind::TypeAny,
            Self::Sequence(_) => crate::ast::TypeKind::TypeSequence,
            Self::String(_) => crate::ast::TypeKind::TypeString,
            Self::Map(_) => crate::ast::TypeKind::TypeMap,
            Self::Fixed(_) => crate::ast::TypeKind::TypeFixed,
            Self::Path(_) => crate::ast::TypeKind::TypePath,
        }
    }
}

impl From<crate::ast::TypeKind> for Type {
    fn from(disc: crate::ast::TypeKind) -> Self {
        match disc {
            crate::ast::TypeKind::TypeAny => Self::Any(<crate::ast::AnyType>::default()),
            crate::ast::TypeKind::TypeSequence => {
                Self::Sequence(<crate::ast::SequenceType>::default())
            }
            crate::ast::TypeKind::TypeString => Self::String(<crate::ast::StringType>::default()),
            crate::ast::TypeKind::TypeMap => Self::Map(<crate::ast::MapType>::default()),
            crate::ast::TypeKind::TypeFixed => Self::Fixed(<crate::ast::FixedType>::default()),
            crate::ast::TypeKind::TypePath => Self::Path(<crate::ast::Path>::default()),
        }
    }
}

impl ::std::default::Default for Type {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Type {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("Type")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Any(v) => state.encode_variant(0, "any", v),
            Self::Sequence(v) => state.encode_variant(1, "sequence", v),
            Self::String(v) => state.encode_variant(2, "string", v),
            Self::Map(v) => state.encode_variant(3, "map", v),
            Self::Fixed(v) => state.encode_variant(4, "fixed", v),
            Self::Path(v) => state.encode_variant(5, "path", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for Type {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("Type")?;
        let mut disc = crate::ast::TypeKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::TypeKind::TypeAny => {
                let mut value = <crate::ast::AnyType>::default();
                state.decode_variant(0, "any", &mut value)?;
                Self::Any(value)
            }
            crate::ast::TypeKind::TypeSequence => {
                let mut value = <crate::ast::SequenceType>::default();
                state.decode_variant(1, "sequence", &mut value)?;
                Self::Sequence(value)
            }
            crate::ast::TypeKind::TypeString => {
                let mut value = <crate::ast::StringType>::default();
                state.decode_variant(2, "string", &mut value)?;
                Self::String(value)
            }
            crate::ast::TypeKind::TypeMap => {
                let mut value = <crate::ast::MapType>::default();
                state.decode_variant(3, "map", &mut value)?;
                Self::Map(value)
            }
            crate::ast::TypeKind::TypeFixed => {
                let mut value = <crate::ast::FixedType>::default();
                state.decode_variant(4, "fixed", &mut value)?;
                Self::Fixed(value)
            }
            crate::ast::TypeKind::TypePath => {
                let mut value = <crate::ast::Path>::default();
                state.decode_variant(5, "path", &mut value)?;
                Self::Path(value)
            }
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum DeclaratorKind {
    /// A single, non-qualified identifier.
    DeclaratorSimple,
    /// An array declarator, e.g. `value[3][4][5]`.
    DeclaratorArray,
}

impl DeclaratorKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::DeclaratorKind::DeclaratorSimple
    }
}

impl ::std::str::FromStr for DeclaratorKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "DECLARATOR_SIMPLE" => Ok(Self::DeclaratorSimple),
            "DECLARATOR_ARRAY" => Ok(Self::DeclaratorArray),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for DeclaratorKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::DeclaratorSimple => f.write_str("DECLARATOR_SIMPLE"),
            Self::DeclaratorArray => f.write_str("DECLARATOR_ARRAY"),
        }
    }
}

impl ::std::default::Default for DeclaratorKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for DeclaratorKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("DeclaratorKind")?;
        match self {
            Self::DeclaratorSimple => state.encode_variant::<i32>("DECLARATOR_SIMPLE", 0),
            Self::DeclaratorArray => state.encode_variant::<i32>("DECLARATOR_ARRAY", 1),
        }
    }
}

impl ::intercom_cts::Unmarshal for DeclaratorKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("DeclaratorKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for DeclaratorKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::DeclaratorSimple,
            1 => Self::DeclaratorArray,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "DECLARATOR_SIMPLE" => Self::DeclaratorSimple,
            "DECLARATOR_ARRAY" => Self::DeclaratorArray,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ArrayDeclarator {
    pub ident: crate::ast::Ident,
    pub bounds: Vec<crate::ast::Expr>,
}

impl ArrayDeclarator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: <crate::ast::Ident>::default(),
            bounds: <Vec<crate::ast::Expr>>::default(),
        }
    }
}

impl ::std::default::Default for ArrayDeclarator {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ArrayDeclarator {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("ArrayDeclarator")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "bounds", &self.bounds)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for ArrayDeclarator {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("ArrayDeclarator")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "bounds", &mut self.bounds)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Declarator {
    Simple(crate::ast::Ident),
    Array(crate::ast::ArrayDeclarator),
}

impl Declarator {
    #[must_use]
    pub fn new() -> Self {
        Self::Simple(<crate::ast::Ident>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::DeclaratorKind {
        match self {
            Self::Simple(_) => crate::ast::DeclaratorKind::DeclaratorSimple,
            Self::Array(_) => crate::ast::DeclaratorKind::DeclaratorArray,
        }
    }
}

impl From<crate::ast::DeclaratorKind> for Declarator {
    fn from(disc: crate::ast::DeclaratorKind) -> Self {
        match disc {
            crate::ast::DeclaratorKind::DeclaratorSimple => {
                Self::Simple(<crate::ast::Ident>::default())
            }
            crate::ast::DeclaratorKind::DeclaratorArray => {
                Self::Array(<crate::ast::ArrayDeclarator>::default())
            }
        }
    }
}

impl ::std::default::Default for Declarator {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Declarator {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("Declarator")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Simple(v) => state.encode_variant(0, "simple", v),
            Self::Array(v) => state.encode_variant(1, "array", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for Declarator {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("Declarator")?;
        let mut disc = crate::ast::DeclaratorKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::DeclaratorKind::DeclaratorSimple => {
                let mut value = <crate::ast::Ident>::default();
                state.decode_variant(0, "simple", &mut value)?;
                Self::Simple(value)
            }
            crate::ast::DeclaratorKind::DeclaratorArray => {
                let mut value = <crate::ast::ArrayDeclarator>::default();
                state.decode_variant(1, "array", &mut value)?;
                Self::Array(value)
            }
        };
        Ok(())
    }
}

/// A parameter inside an applied annotation, e.g. `value=true` in
/// `@optional(value=true)`.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AnnotationArg {
    /// Name of the parameter if one was specified.
    /// May be omitted for annotations with only a single, non-default member,
    /// but this is not enforced by the parser.
    pub ident: Option<crate::ast::Ident>,
    /// Span of the entire parameter.
    pub span: crate::ast::Span,
    /// The specified value of the parameter.
    pub value: crate::ast::Expr,
}

impl AnnotationArg {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: None,
            span: <crate::ast::Span>::default(),
            value: <crate::ast::Expr>::default(),
        }
    }
}

impl ::std::default::Default for AnnotationArg {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AnnotationArg {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("AnnotationArg")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "span", &self.span)?;
        state.encode_field(2, "value", &self.value)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for AnnotationArg {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("AnnotationArg")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "span", &mut self.span)?;
        state.decode_field(2, "value", &mut self.value)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AnnotationAppl {
    pub ty: crate::ast::Path,
    pub span: crate::ast::Span,
    pub args: Vec<crate::ast::AnnotationArg>,
}

impl AnnotationAppl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ty: <crate::ast::Path>::default(),
            span: <crate::ast::Span>::default(),
            args: <Vec<crate::ast::AnnotationArg>>::default(),
        }
    }
}

impl ::std::default::Default for AnnotationAppl {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AnnotationAppl {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("AnnotationAppl")?;
        state.encode_field(0, "ty", &self.ty)?;
        state.encode_field(1, "span", &self.span)?;
        state.encode_field(2, "args", &self.args)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for AnnotationAppl {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("AnnotationAppl")?;
        state.decode_field(0, "ty", &mut self.ty)?;
        state.decode_field(1, "span", &mut self.span)?;
        state.decode_field(2, "args", &mut self.args)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum AnnotationFieldKind {
    FieldDefinition,
    FieldMember,
}

impl AnnotationFieldKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::AnnotationFieldKind::FieldDefinition
    }
}

impl ::std::str::FromStr for AnnotationFieldKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "FIELD_DEFINITION" => Ok(Self::FieldDefinition),
            "FIELD_MEMBER" => Ok(Self::FieldMember),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for AnnotationFieldKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::FieldDefinition => f.write_str("FIELD_DEFINITION"),
            Self::FieldMember => f.write_str("FIELD_MEMBER"),
        }
    }
}

impl ::std::default::Default for AnnotationFieldKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AnnotationFieldKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("AnnotationFieldKind")?;
        match self {
            Self::FieldDefinition => state.encode_variant::<i32>("FIELD_DEFINITION", 0),
            Self::FieldMember => state.encode_variant::<i32>("FIELD_MEMBER", 1),
        }
    }
}

impl ::intercom_cts::Unmarshal for AnnotationFieldKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("AnnotationFieldKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for AnnotationFieldKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::FieldDefinition,
            1 => Self::FieldMember,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "FIELD_DEFINITION" => Self::FieldDefinition,
            "FIELD_MEMBER" => Self::FieldMember,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum AnnotationField {
    Item(Box<crate::ast::Item>),
    Member(Box<crate::ast::Field>),
}

impl AnnotationField {
    #[must_use]
    pub fn new() -> Self {
        Self::Item(Box::new(<crate::ast::Item>::default()))
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::AnnotationFieldKind {
        match self {
            Self::Item(_) => crate::ast::AnnotationFieldKind::FieldDefinition,
            Self::Member(_) => crate::ast::AnnotationFieldKind::FieldMember,
        }
    }
}

impl From<crate::ast::AnnotationFieldKind> for AnnotationField {
    fn from(disc: crate::ast::AnnotationFieldKind) -> Self {
        match disc {
            crate::ast::AnnotationFieldKind::FieldDefinition => {
                Self::Item(Box::new(<crate::ast::Item>::default()))
            }
            crate::ast::AnnotationFieldKind::FieldMember => {
                Self::Member(Box::new(<crate::ast::Field>::default()))
            }
        }
    }
}

impl ::std::default::Default for AnnotationField {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AnnotationField {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("AnnotationField")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Item(v) => state.encode_variant(0, "item", v),
            Self::Member(v) => state.encode_variant(1, "member", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for AnnotationField {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("AnnotationField")?;
        let mut disc = crate::ast::AnnotationFieldKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::AnnotationFieldKind::FieldDefinition => {
                let mut value = Box::new(<crate::ast::Item>::default());
                state.decode_variant(0, "item", &mut value)?;
                Self::Item(value)
            }
            crate::ast::AnnotationFieldKind::FieldMember => {
                let mut value = Box::new(<crate::ast::Field>::default());
                state.decode_variant(1, "member", &mut value)?;
                Self::Member(value)
            }
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AnnotationDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub params: Vec<crate::ast::AnnotationField>,
}

impl AnnotationDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            params: <Vec<crate::ast::AnnotationField>>::default(),
        }
    }
}

impl ::std::default::Default for AnnotationDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AnnotationDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("AnnotationDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "params", &self.params)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for AnnotationDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("AnnotationDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "params", &mut self.params)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ModuleDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub definitions: Vec<crate::ast::Item>,
}

impl ModuleDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            definitions: <Vec<crate::ast::Item>>::default(),
        }
    }
}

impl ::std::default::Default for ModuleDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ModuleDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("ModuleDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "definitions", &self.definitions)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for ModuleDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("ModuleDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "definitions", &mut self.definitions)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Field {
    pub names: Vec<crate::ast::Declarator>,
    pub ty: crate::ast::Type,
}

impl Field {
    #[must_use]
    pub fn new() -> Self {
        Self {
            names: <Vec<crate::ast::Declarator>>::default(),
            ty: <crate::ast::Type>::default(),
        }
    }
}

impl ::std::default::Default for Field {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Field {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Field")?;
        state.encode_field(0, "names", &self.names)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Field {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Field")?;
        state.decode_field(0, "names", &mut self.names)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct StructDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub members: Vec<crate::ast::Field>,
    pub parent: Option<crate::ast::Path>,
}

impl StructDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            members: <Vec<crate::ast::Field>>::default(),
            parent: None,
        }
    }
}

impl ::std::default::Default for StructDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for StructDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("StructDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "members", &self.members)?;
        state.encode_field(1, "parent", &self.parent)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for StructDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("StructDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "members", &mut self.members)?;
        state.decode_field(1, "parent", &mut self.parent)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Discriminator {
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    pub ty: crate::ast::Type,
}

impl Discriminator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ty: <crate::ast::Type>::default(),
        }
    }
}

impl ::std::default::Default for Discriminator {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Discriminator {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Discriminator")?;
        state.encode_field(0, "annotations", &self.annotations)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Discriminator {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Discriminator")?;
        state.decode_field(0, "annotations", &mut self.annotations)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Empty {}

impl Empty {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Empty {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let state = ar.encode_struct("Empty")?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Empty {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        ar.decode_struct("Empty")?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum LabelKind {
    LabelCase,
    LabelDefault,
}

impl LabelKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::LabelKind::LabelCase
    }
}

impl ::std::str::FromStr for LabelKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "LABEL_CASE" => Ok(Self::LabelCase),
            "LABEL_DEFAULT" => Ok(Self::LabelDefault),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for LabelKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::LabelCase => f.write_str("LABEL_CASE"),
            Self::LabelDefault => f.write_str("LABEL_DEFAULT"),
        }
    }
}

impl ::std::default::Default for LabelKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for LabelKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("LabelKind")?;
        match self {
            Self::LabelCase => state.encode_variant::<i32>("LABEL_CASE", 0),
            Self::LabelDefault => state.encode_variant::<i32>("LABEL_DEFAULT", 1),
        }
    }
}

impl ::intercom_cts::Unmarshal for LabelKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("LabelKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for LabelKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::LabelCase,
            1 => Self::LabelDefault,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "LABEL_CASE" => Self::LabelCase,
            "LABEL_DEFAULT" => Self::LabelDefault,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Label {
    Case(crate::ast::Expr),
    Default(crate::ast::Empty),
}

impl Label {
    #[must_use]
    pub fn new() -> Self {
        Self::Case(<crate::ast::Expr>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::LabelKind {
        match self {
            Self::Case(_) => crate::ast::LabelKind::LabelCase,
            Self::Default(_) => crate::ast::LabelKind::LabelDefault,
        }
    }
}

impl From<crate::ast::LabelKind> for Label {
    fn from(disc: crate::ast::LabelKind) -> Self {
        match disc {
            crate::ast::LabelKind::LabelCase => Self::Case(<crate::ast::Expr>::default()),
            crate::ast::LabelKind::LabelDefault => Self::Default(<crate::ast::Empty>::default()),
        }
    }
}

impl ::std::default::Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Label {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("Label")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Case(v) => state.encode_variant(0, "case", v),
            Self::Default(v) => state.encode_variant(1, "default", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for Label {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("Label")?;
        let mut disc = crate::ast::LabelKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::LabelKind::LabelCase => {
                let mut value = <crate::ast::Expr>::default();
                state.decode_variant(0, "case", &mut value)?;
                Self::Case(value)
            }
            crate::ast::LabelKind::LabelDefault => {
                let mut value = <crate::ast::Empty>::default();
                state.decode_variant(1, "default", &mut value)?;
                Self::Default(value)
            }
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum UnionElementKind {
    ElementMember,
    ElementNull,
}

impl UnionElementKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::UnionElementKind::ElementMember
    }
}

impl ::std::str::FromStr for UnionElementKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "ELEMENT_MEMBER" => Ok(Self::ElementMember),
            "ELEMENT_NULL" => Ok(Self::ElementNull),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for UnionElementKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::ElementMember => f.write_str("ELEMENT_MEMBER"),
            Self::ElementNull => f.write_str("ELEMENT_NULL"),
        }
    }
}

impl ::std::default::Default for UnionElementKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for UnionElementKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("UnionElementKind")?;
        match self {
            Self::ElementMember => state.encode_variant::<i32>("ELEMENT_MEMBER", 0),
            Self::ElementNull => state.encode_variant::<i32>("ELEMENT_NULL", 1),
        }
    }
}

impl ::intercom_cts::Unmarshal for UnionElementKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("UnionElementKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for UnionElementKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::ElementMember,
            1 => Self::ElementNull,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "ELEMENT_MEMBER" => Self::ElementMember,
            "ELEMENT_NULL" => Self::ElementNull,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct UnionMember {
    pub ty: Box<crate::ast::Type>,
    pub decl: crate::ast::Declarator,
}

impl UnionMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ty: Box::new(<crate::ast::Type>::default()),
            decl: <crate::ast::Declarator>::default(),
        }
    }
}

impl ::std::default::Default for UnionMember {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for UnionMember {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("UnionMember")?;
        state.encode_field(0, "ty", &self.ty)?;
        state.encode_field(1, "decl", &self.decl)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for UnionMember {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("UnionMember")?;
        state.decode_field(0, "ty", &mut self.ty)?;
        state.decode_field(1, "decl", &mut self.decl)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct UnionNull {
    pub span: crate::ast::Span,
}

impl UnionNull {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
        }
    }
}

impl ::std::default::Default for UnionNull {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for UnionNull {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("UnionNull")?;
        state.encode_field(0, "span", &self.span)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for UnionNull {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("UnionNull")?;
        state.decode_field(0, "span", &mut self.span)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum UnionElement {
    Member(crate::ast::UnionMember),
    Null(crate::ast::UnionNull),
}

impl UnionElement {
    #[must_use]
    pub fn new() -> Self {
        Self::Member(<crate::ast::UnionMember>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::UnionElementKind {
        match self {
            Self::Member(_) => crate::ast::UnionElementKind::ElementMember,
            Self::Null(_) => crate::ast::UnionElementKind::ElementNull,
        }
    }
}

impl From<crate::ast::UnionElementKind> for UnionElement {
    fn from(disc: crate::ast::UnionElementKind) -> Self {
        match disc {
            crate::ast::UnionElementKind::ElementMember => {
                Self::Member(<crate::ast::UnionMember>::default())
            }
            crate::ast::UnionElementKind::ElementNull => {
                Self::Null(<crate::ast::UnionNull>::default())
            }
        }
    }
}

impl ::std::default::Default for UnionElement {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for UnionElement {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("UnionElement")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Member(v) => state.encode_variant(0, "member", v),
            Self::Null(v) => state.encode_variant(1, "null", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for UnionElement {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("UnionElement")?;
        let mut disc = crate::ast::UnionElementKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::UnionElementKind::ElementMember => {
                let mut value = <crate::ast::UnionMember>::default();
                state.decode_variant(0, "member", &mut value)?;
                Self::Member(value)
            }
            crate::ast::UnionElementKind::ElementNull => {
                let mut value = <crate::ast::UnionNull>::default();
                state.decode_variant(1, "null", &mut value)?;
                Self::Null(value)
            }
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct UnionField {
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Case labels that map to this variant.
    pub labels: Vec<crate::ast::Label>,
    pub field: crate::ast::UnionElement,
}

impl UnionField {
    #[must_use]
    pub fn new() -> Self {
        Self {
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            labels: <Vec<crate::ast::Label>>::default(),
            field: <crate::ast::UnionElement>::default(),
        }
    }
}

impl ::std::default::Default for UnionField {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for UnionField {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("UnionField")?;
        state.encode_field(0, "annotations", &self.annotations)?;
        state.encode_field(1, "labels", &self.labels)?;
        state.encode_field(2, "field", &self.field)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for UnionField {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("UnionField")?;
        state.decode_field(0, "annotations", &mut self.annotations)?;
        state.decode_field(1, "labels", &mut self.labels)?;
        state.decode_field(2, "field", &mut self.field)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct UnionDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    /// The discriminator component of the union.
    pub disc: crate::ast::Discriminator,
    /// All variants of the union. The case labels that map to each variant can
    /// be found in `UnionField`.
    pub fields: Vec<crate::ast::UnionField>,
}

impl UnionDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            disc: <crate::ast::Discriminator>::default(),
            fields: <Vec<crate::ast::UnionField>>::default(),
        }
    }
}

impl ::std::default::Default for UnionDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for UnionDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("UnionDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "disc", &self.disc)?;
        state.encode_field(1, "fields", &self.fields)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for UnionDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("UnionDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "disc", &mut self.disc)?;
        state.decode_field(1, "fields", &mut self.fields)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ConstDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    pub decl: crate::ast::Declarator,
    pub ty: crate::ast::Type,
    pub value: crate::ast::Expr,
}

impl ConstDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            decl: <crate::ast::Declarator>::default(),
            ty: <crate::ast::Type>::default(),
            value: <crate::ast::Expr>::default(),
        }
    }
}

impl ::std::default::Default for ConstDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ConstDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("ConstDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "decl", &self.decl)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.encode_field(2, "value", &self.value)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for ConstDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("ConstDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "decl", &mut self.decl)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        state.decode_field(2, "value", &mut self.value)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Enumerator {
    pub ident: crate::ast::Ident,
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// An explicit value, e.g. `enum Foo { VALUE = 1 };`
    /// The `@value` annotation will *not* populate this field.
    pub value: Option<crate::ast::Expr>,
}

impl Enumerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: <crate::ast::Ident>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            value: None,
        }
    }
}

impl ::std::default::Default for Enumerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Enumerator {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Enumerator")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(2, "value", &self.value)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Enumerator {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Enumerator")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(2, "value", &mut self.value)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct EnumDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub fields: Vec<crate::ast::Enumerator>,
}

impl EnumDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            fields: <Vec<crate::ast::Enumerator>>::default(),
        }
    }
}

impl ::std::default::Default for EnumDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for EnumDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("EnumDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "fields", &self.fields)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for EnumDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("EnumDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "fields", &mut self.fields)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ExceptDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub members: Vec<crate::ast::Field>,
}

impl ExceptDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            members: <Vec<crate::ast::Field>>::default(),
        }
    }
}

impl ::std::default::Default for ExceptDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ExceptDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("ExceptDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "members", &self.members)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for ExceptDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("ExceptDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "members", &mut self.members)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AliasDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// List of all declarators. Always contains at least one declarator.
    pub decl: Vec<crate::ast::Declarator>,
    /// The underlying type of the typedef.
    pub ty: crate::ast::Type,
}

impl AliasDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            decl: <Vec<crate::ast::Declarator>>::default(),
            ty: <crate::ast::Type>::default(),
        }
    }
}

impl ::std::default::Default for AliasDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for AliasDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("AliasDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "decl", &self.decl)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for AliasDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("AliasDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "decl", &mut self.decl)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Bit {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub value: Option<crate::ast::Expr>,
}

impl Bit {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            value: None,
        }
    }
}

impl ::std::default::Default for Bit {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Bit {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Bit")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "value", &self.value)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Bit {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Bit")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "value", &mut self.value)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct BitmaskDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub bits: Vec<crate::ast::Bit>,
}

impl BitmaskDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            bits: <Vec<crate::ast::Bit>>::default(),
        }
    }
}

impl ::std::default::Default for BitmaskDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for BitmaskDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("BitmaskDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "bits", &self.bits)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for BitmaskDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("BitmaskDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "bits", &mut self.bits)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Bitfield {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub size: crate::ast::Expr,
    pub ty: Option<crate::ast::Type>,
}

impl Bitfield {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            size: <crate::ast::Expr>::default(),
            ty: None,
        }
    }
}

impl ::std::default::Default for Bitfield {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Bitfield {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Bitfield")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "size", &self.size)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Bitfield {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Bitfield")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "size", &mut self.size)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct BitsetDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub parent: Option<crate::ast::Path>,
    pub fields: Vec<crate::ast::Bitfield>,
}

impl BitsetDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            parent: None,
            fields: <Vec<crate::ast::Bitfield>>::default(),
        }
    }
}

impl ::std::default::Default for BitsetDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for BitsetDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("BitsetDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "parent", &self.parent)?;
        state.encode_field(1, "fields", &self.fields)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for BitsetDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("BitsetDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "parent", &mut self.parent)?;
        state.decode_field(1, "fields", &mut self.fields)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Attribute {
    /// Name of the attribute.
    pub ident: crate::ast::Ident,
    /// The type of the attribute.
    pub ty: crate::ast::Type,
    /// Indicates whether this attribute was marked as `readonly`, and if
    /// so, the span of the keyword.
    pub readonly: Option<crate::ast::Span>,
}

impl Attribute {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: <crate::ast::Ident>::default(),
            ty: <crate::ast::Type>::default(),
            readonly: None,
        }
    }
}

impl ::std::default::Default for Attribute {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Attribute {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Attribute")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.encode_field(2, "readonly", &self.readonly)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Attribute {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Attribute")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        state.decode_field(2, "readonly", &mut self.readonly)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum ParamKind {
    /// Explicitly marked as `in`
    ParamIn,
    /// Explicitly marked as `out`
    ParamOut,
    /// Explicitly marked as `inout`
    ParamInout,
}

impl ParamKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::ParamKind::ParamIn
    }
}

impl ::std::str::FromStr for ParamKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "PARAM_IN" => Ok(Self::ParamIn),
            "PARAM_OUT" => Ok(Self::ParamOut),
            "PARAM_INOUT" => Ok(Self::ParamInout),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for ParamKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::ParamIn => f.write_str("PARAM_IN"),
            Self::ParamOut => f.write_str("PARAM_OUT"),
            Self::ParamInout => f.write_str("PARAM_INOUT"),
        }
    }
}

impl ::std::default::Default for ParamKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ParamKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("ParamKind")?;
        match self {
            Self::ParamIn => state.encode_variant::<i32>("PARAM_IN", 0),
            Self::ParamOut => state.encode_variant::<i32>("PARAM_OUT", 1),
            Self::ParamInout => state.encode_variant::<i32>("PARAM_INOUT", 2),
        }
    }
}

impl ::intercom_cts::Unmarshal for ParamKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("ParamKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for ParamKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::ParamIn,
            1 => Self::ParamOut,
            2 => Self::ParamInout,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "PARAM_IN" => Self::ParamIn,
            "PARAM_OUT" => Self::ParamOut,
            "PARAM_INOUT" => Self::ParamInout,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Param {
    /// Name of the parameter.
    pub ident: crate::ast::Ident,
    /// Type of the parameter.
    pub ty: crate::ast::Type,
    /// Specifies whether this is an `in`, `out`, or `inout` parameter.
    pub kind: Option<crate::ast::ParamKind>,
}

impl Param {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: <crate::ast::Ident>::default(),
            ty: <crate::ast::Type>::default(),
            kind: None,
        }
    }
}

impl ::std::default::Default for Param {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Param {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Param")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.encode_field(2, "kind", &self.kind)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Param {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Param")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        state.decode_field(2, "kind", &mut self.kind)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Prototype {
    /// Name of the prototype.
    pub ident: crate::ast::Ident,
    /// Return type.
    pub ret: crate::ast::Type,
    pub params: Vec<crate::ast::Param>,
    pub raises: Vec<crate::ast::Path>,
    /// Indicates whether this function was prefixed with the `oneway` keyword.
    /// Does not account for the `@oneway` annotation.
    pub oneway: Option<crate::ast::Span>,
}

impl Prototype {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: <crate::ast::Ident>::default(),
            ret: <crate::ast::Type>::default(),
            params: <Vec<crate::ast::Param>>::default(),
            raises: <Vec<crate::ast::Path>>::default(),
            oneway: None,
        }
    }
}

impl ::std::default::Default for Prototype {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Prototype {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Prototype")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "ret", &self.ret)?;
        state.encode_field(2, "params", &self.params)?;
        state.encode_field(3, "raises", &self.raises)?;
        state.encode_field(4, "oneway", &self.oneway)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Prototype {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Prototype")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "ret", &mut self.ret)?;
        state.decode_field(2, "params", &mut self.params)?;
        state.decode_field(3, "raises", &mut self.raises)?;
        state.decode_field(4, "oneway", &mut self.oneway)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct InterfaceDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub members: Vec<crate::ast::InterfaceMember>,
    pub inherits: Vec<crate::ast::Path>,
    pub local: Option<crate::ast::Span>,
}

impl InterfaceDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            members: <Vec<crate::ast::InterfaceMember>>::default(),
            inherits: <Vec<crate::ast::Path>>::default(),
            local: None,
        }
    }
}

impl ::std::default::Default for InterfaceDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for InterfaceDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("InterfaceDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "members", &self.members)?;
        state.encode_field(1, "inherits", &self.inherits)?;
        state.encode_field(2, "local", &self.local)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for InterfaceDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("InterfaceDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "members", &mut self.members)?;
        state.decode_field(1, "inherits", &mut self.inherits)?;
        state.decode_field(2, "local", &mut self.local)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ValueMember {
    pub ident: crate::ast::Ident,
    pub ty: crate::ast::Type,
    pub public: Option<crate::ast::Span>,
}

impl ValueMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ident: <crate::ast::Ident>::default(),
            ty: <crate::ast::Type>::default(),
            public: None,
        }
    }
}

impl ::std::default::Default for ValueMember {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ValueMember {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("ValueMember")?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(1, "ty", &self.ty)?;
        state.encode_field(2, "public", &self.public)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for ValueMember {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("ValueMember")?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(1, "ty", &mut self.ty)?;
        state.decode_field(2, "public", &mut self.public)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ValuetypeDef {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub members: Vec<crate::ast::ValueMember>,
    pub prototypes: Vec<crate::ast::Prototype>,
    pub inherits: Option<crate::ast::Path>,
    pub supports: Option<crate::ast::Path>,
}

impl ValuetypeDef {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            members: <Vec<crate::ast::ValueMember>>::default(),
            prototypes: <Vec<crate::ast::Prototype>>::default(),
            inherits: None,
            supports: None,
        }
    }
}

impl ::std::default::Default for ValuetypeDef {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ValuetypeDef {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("ValuetypeDef")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "members", &self.members)?;
        state.encode_field(1, "prototypes", &self.prototypes)?;
        state.encode_field(2, "inherits", &self.inherits)?;
        state.encode_field(3, "supports", &self.supports)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for ValuetypeDef {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("ValuetypeDef")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "members", &mut self.members)?;
        state.decode_field(1, "prototypes", &mut self.prototypes)?;
        state.decode_field(2, "inherits", &mut self.inherits)?;
        state.decode_field(3, "supports", &mut self.supports)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum DeclKind {
    DeclStruct,
    DeclUnion,
    DeclNative,
    DeclInterface,
    DeclValuetype,
}

impl DeclKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::DeclKind::DeclStruct
    }
}

impl ::std::str::FromStr for DeclKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "DECL_STRUCT" => Ok(Self::DeclStruct),
            "DECL_UNION" => Ok(Self::DeclUnion),
            "DECL_NATIVE" => Ok(Self::DeclNative),
            "DECL_INTERFACE" => Ok(Self::DeclInterface),
            "DECL_VALUETYPE" => Ok(Self::DeclValuetype),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for DeclKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::DeclStruct => f.write_str("DECL_STRUCT"),
            Self::DeclUnion => f.write_str("DECL_UNION"),
            Self::DeclNative => f.write_str("DECL_NATIVE"),
            Self::DeclInterface => f.write_str("DECL_INTERFACE"),
            Self::DeclValuetype => f.write_str("DECL_VALUETYPE"),
        }
    }
}

impl ::std::default::Default for DeclKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for DeclKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("DeclKind")?;
        match self {
            Self::DeclStruct => state.encode_variant::<i32>("DECL_STRUCT", 0),
            Self::DeclUnion => state.encode_variant::<i32>("DECL_UNION", 1),
            Self::DeclNative => state.encode_variant::<i32>("DECL_NATIVE", 2),
            Self::DeclInterface => state.encode_variant::<i32>("DECL_INTERFACE", 3),
            Self::DeclValuetype => state.encode_variant::<i32>("DECL_VALUETYPE", 4),
        }
    }
}

impl ::intercom_cts::Unmarshal for DeclKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("DeclKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for DeclKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::DeclStruct,
            1 => Self::DeclUnion,
            2 => Self::DeclNative,
            3 => Self::DeclInterface,
            4 => Self::DeclValuetype,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "DECL_STRUCT" => Self::DeclStruct,
            "DECL_UNION" => Self::DeclUnion,
            "DECL_NATIVE" => Self::DeclNative,
            "DECL_INTERFACE" => Self::DeclInterface,
            "DECL_VALUETYPE" => Self::DeclValuetype,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Decl {
    /// Span of the entire item, from start to end. For example, given the
    /// following IDL:
    ///
    /// ```idl
    /// module foo { ... };
    /// ````
    ///
    /// The span of the above module will start at 'm' and end at '}'.
    pub span: crate::ast::Span,
    /// Annotations that were applied to this item.
    pub annotations: Vec<crate::ast::AnnotationAppl>,
    /// Name of the item.
    pub ident: crate::ast::Ident,
    pub kind: crate::ast::DeclKind,
}

impl Decl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            span: <crate::ast::Span>::default(),
            annotations: <Vec<crate::ast::AnnotationAppl>>::default(),
            ident: <crate::ast::Ident>::default(),
            kind: crate::ast::DeclKind::DeclStruct,
        }
    }
}

impl ::std::default::Default for Decl {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Decl {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::FieldSerializer as _;

        let mut state = ar.encode_struct("Decl")?;
        state.encode_field(0, "span", &self.span)?;
        state.encode_field(1, "annotations", &self.annotations)?;
        state.encode_field(0, "ident", &self.ident)?;
        state.encode_field(0, "kind", &self.kind)?;
        state.end()
    }
}

impl ::intercom_cts::Unmarshal for Decl {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::FieldDeserializer as _;

        let mut state = ar.decode_struct("Decl")?;
        state.decode_field(0, "span", &mut self.span)?;
        state.decode_field(1, "annotations", &mut self.annotations)?;
        state.decode_field(0, "ident", &mut self.ident)?;
        state.decode_field(0, "kind", &mut self.kind)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum ItemKind {
    /// A definition of an annotation
    ItemAnnotation,
    /// Module declaration
    ItemModule,
    /// Struct definition
    ItemStruct,
    /// Union definition
    ItemUnion,
    /// Enum definition
    ItemEnum,
    /// Exception definition
    ItemException,
    /// Bitmask definition
    ItemBitmask,
    /// Bitset definition
    ItemBitset,
    /// Declaration of a `const`
    ItemConst,
    /// Typedef definition
    ItemTypedef,
    /// Interface definition
    ItemInterface,
    /// Valuetype definition
    ItemValuetype,
    /// A forward declaration
    ItemDecl,
}

impl ItemKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::ItemKind::ItemAnnotation
    }
}

impl ::std::str::FromStr for ItemKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "ITEM_ANNOTATION" => Ok(Self::ItemAnnotation),
            "ITEM_MODULE" => Ok(Self::ItemModule),
            "ITEM_STRUCT" => Ok(Self::ItemStruct),
            "ITEM_UNION" => Ok(Self::ItemUnion),
            "ITEM_ENUM" => Ok(Self::ItemEnum),
            "ITEM_EXCEPTION" => Ok(Self::ItemException),
            "ITEM_BITMASK" => Ok(Self::ItemBitmask),
            "ITEM_BITSET" => Ok(Self::ItemBitset),
            "ITEM_CONST" => Ok(Self::ItemConst),
            "ITEM_TYPEDEF" => Ok(Self::ItemTypedef),
            "ITEM_INTERFACE" => Ok(Self::ItemInterface),
            "ITEM_VALUETYPE" => Ok(Self::ItemValuetype),
            "ITEM_DECL" => Ok(Self::ItemDecl),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::ItemAnnotation => f.write_str("ITEM_ANNOTATION"),
            Self::ItemModule => f.write_str("ITEM_MODULE"),
            Self::ItemStruct => f.write_str("ITEM_STRUCT"),
            Self::ItemUnion => f.write_str("ITEM_UNION"),
            Self::ItemEnum => f.write_str("ITEM_ENUM"),
            Self::ItemException => f.write_str("ITEM_EXCEPTION"),
            Self::ItemBitmask => f.write_str("ITEM_BITMASK"),
            Self::ItemBitset => f.write_str("ITEM_BITSET"),
            Self::ItemConst => f.write_str("ITEM_CONST"),
            Self::ItemTypedef => f.write_str("ITEM_TYPEDEF"),
            Self::ItemInterface => f.write_str("ITEM_INTERFACE"),
            Self::ItemValuetype => f.write_str("ITEM_VALUETYPE"),
            Self::ItemDecl => f.write_str("ITEM_DECL"),
        }
    }
}

impl ::std::default::Default for ItemKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for ItemKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("ItemKind")?;
        match self {
            Self::ItemAnnotation => state.encode_variant::<i32>("ITEM_ANNOTATION", 0),
            Self::ItemModule => state.encode_variant::<i32>("ITEM_MODULE", 1),
            Self::ItemStruct => state.encode_variant::<i32>("ITEM_STRUCT", 2),
            Self::ItemUnion => state.encode_variant::<i32>("ITEM_UNION", 3),
            Self::ItemEnum => state.encode_variant::<i32>("ITEM_ENUM", 4),
            Self::ItemException => state.encode_variant::<i32>("ITEM_EXCEPTION", 5),
            Self::ItemBitmask => state.encode_variant::<i32>("ITEM_BITMASK", 6),
            Self::ItemBitset => state.encode_variant::<i32>("ITEM_BITSET", 7),
            Self::ItemConst => state.encode_variant::<i32>("ITEM_CONST", 8),
            Self::ItemTypedef => state.encode_variant::<i32>("ITEM_TYPEDEF", 9),
            Self::ItemInterface => state.encode_variant::<i32>("ITEM_INTERFACE", 10),
            Self::ItemValuetype => state.encode_variant::<i32>("ITEM_VALUETYPE", 11),
            Self::ItemDecl => state.encode_variant::<i32>("ITEM_DECL", 12),
        }
    }
}

impl ::intercom_cts::Unmarshal for ItemKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("ItemKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for ItemKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::ItemAnnotation,
            1 => Self::ItemModule,
            2 => Self::ItemStruct,
            3 => Self::ItemUnion,
            4 => Self::ItemEnum,
            5 => Self::ItemException,
            6 => Self::ItemBitmask,
            7 => Self::ItemBitset,
            8 => Self::ItemConst,
            9 => Self::ItemTypedef,
            10 => Self::ItemInterface,
            11 => Self::ItemValuetype,
            12 => Self::ItemDecl,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "ITEM_ANNOTATION" => Self::ItemAnnotation,
            "ITEM_MODULE" => Self::ItemModule,
            "ITEM_STRUCT" => Self::ItemStruct,
            "ITEM_UNION" => Self::ItemUnion,
            "ITEM_ENUM" => Self::ItemEnum,
            "ITEM_EXCEPTION" => Self::ItemException,
            "ITEM_BITMASK" => Self::ItemBitmask,
            "ITEM_BITSET" => Self::ItemBitset,
            "ITEM_CONST" => Self::ItemConst,
            "ITEM_TYPEDEF" => Self::ItemTypedef,
            "ITEM_INTERFACE" => Self::ItemInterface,
            "ITEM_VALUETYPE" => Self::ItemValuetype,
            "ITEM_DECL" => Self::ItemDecl,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Item {
    /// A definition of an annotation
    AnnotationValue(crate::ast::AnnotationDef),
    /// Module declaration
    ModuleValue(crate::ast::ModuleDef),
    /// Struct definition
    StructValue(crate::ast::StructDef),
    /// Union definition
    UnionValue(crate::ast::UnionDef),
    /// Enum definition
    EnumValue(crate::ast::EnumDef),
    /// Exception definition
    ExceptionValue(crate::ast::ExceptDef),
    /// Bitmask definition
    BitmaskValue(crate::ast::BitmaskDef),
    /// Bitset definition
    BitsetValue(crate::ast::BitsetDef),
    /// Declaration of a `const`
    ConstValue(crate::ast::ConstDef),
    /// Typedef definition
    AliasValue(crate::ast::AliasDef),
    /// Interface definition
    InterfaceValue(crate::ast::InterfaceDef),
    /// Valuetype definition
    ValuetypeValue(crate::ast::ValuetypeDef),
    /// A forward declaration
    DeclValue(crate::ast::Decl),
}

impl Item {
    #[must_use]
    pub fn new() -> Self {
        Self::AnnotationValue(<crate::ast::AnnotationDef>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::ItemKind {
        match self {
            Self::AnnotationValue(_) => crate::ast::ItemKind::ItemAnnotation,
            Self::ModuleValue(_) => crate::ast::ItemKind::ItemModule,
            Self::StructValue(_) => crate::ast::ItemKind::ItemStruct,
            Self::UnionValue(_) => crate::ast::ItemKind::ItemUnion,
            Self::EnumValue(_) => crate::ast::ItemKind::ItemEnum,
            Self::ExceptionValue(_) => crate::ast::ItemKind::ItemException,
            Self::BitmaskValue(_) => crate::ast::ItemKind::ItemBitmask,
            Self::BitsetValue(_) => crate::ast::ItemKind::ItemBitset,
            Self::ConstValue(_) => crate::ast::ItemKind::ItemConst,
            Self::AliasValue(_) => crate::ast::ItemKind::ItemTypedef,
            Self::InterfaceValue(_) => crate::ast::ItemKind::ItemInterface,
            Self::ValuetypeValue(_) => crate::ast::ItemKind::ItemValuetype,
            Self::DeclValue(_) => crate::ast::ItemKind::ItemDecl,
        }
    }
}

impl From<crate::ast::ItemKind> for Item {
    fn from(disc: crate::ast::ItemKind) -> Self {
        match disc {
            crate::ast::ItemKind::ItemAnnotation => {
                Self::AnnotationValue(<crate::ast::AnnotationDef>::default())
            }
            crate::ast::ItemKind::ItemModule => {
                Self::ModuleValue(<crate::ast::ModuleDef>::default())
            }
            crate::ast::ItemKind::ItemStruct => {
                Self::StructValue(<crate::ast::StructDef>::default())
            }
            crate::ast::ItemKind::ItemUnion => Self::UnionValue(<crate::ast::UnionDef>::default()),
            crate::ast::ItemKind::ItemEnum => Self::EnumValue(<crate::ast::EnumDef>::default()),
            crate::ast::ItemKind::ItemException => {
                Self::ExceptionValue(<crate::ast::ExceptDef>::default())
            }
            crate::ast::ItemKind::ItemBitmask => {
                Self::BitmaskValue(<crate::ast::BitmaskDef>::default())
            }
            crate::ast::ItemKind::ItemBitset => {
                Self::BitsetValue(<crate::ast::BitsetDef>::default())
            }
            crate::ast::ItemKind::ItemConst => Self::ConstValue(<crate::ast::ConstDef>::default()),
            crate::ast::ItemKind::ItemTypedef => {
                Self::AliasValue(<crate::ast::AliasDef>::default())
            }
            crate::ast::ItemKind::ItemInterface => {
                Self::InterfaceValue(<crate::ast::InterfaceDef>::default())
            }
            crate::ast::ItemKind::ItemValuetype => {
                Self::ValuetypeValue(<crate::ast::ValuetypeDef>::default())
            }
            crate::ast::ItemKind::ItemDecl => Self::DeclValue(<crate::ast::Decl>::default()),
        }
    }
}

impl ::std::default::Default for Item {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for Item {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("Item")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::AnnotationValue(v) => state.encode_variant(0, "annotation_value", v),
            Self::ModuleValue(v) => state.encode_variant(1, "module_value", v),
            Self::StructValue(v) => state.encode_variant(2, "struct_value", v),
            Self::UnionValue(v) => state.encode_variant(3, "union_value", v),
            Self::EnumValue(v) => state.encode_variant(4, "enum_value", v),
            Self::ExceptionValue(v) => state.encode_variant(5, "exception_value", v),
            Self::BitmaskValue(v) => state.encode_variant(6, "bitmask_value", v),
            Self::BitsetValue(v) => state.encode_variant(7, "bitset_value", v),
            Self::ConstValue(v) => state.encode_variant(8, "const_value", v),
            Self::AliasValue(v) => state.encode_variant(9, "alias_value", v),
            Self::InterfaceValue(v) => state.encode_variant(10, "interface_value", v),
            Self::ValuetypeValue(v) => state.encode_variant(11, "valuetype_value", v),
            Self::DeclValue(v) => state.encode_variant(12, "decl_value", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for Item {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("Item")?;
        let mut disc = crate::ast::ItemKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::ItemKind::ItemAnnotation => {
                let mut value = <crate::ast::AnnotationDef>::default();
                state.decode_variant(0, "annotation_value", &mut value)?;
                Self::AnnotationValue(value)
            }
            crate::ast::ItemKind::ItemModule => {
                let mut value = <crate::ast::ModuleDef>::default();
                state.decode_variant(1, "module_value", &mut value)?;
                Self::ModuleValue(value)
            }
            crate::ast::ItemKind::ItemStruct => {
                let mut value = <crate::ast::StructDef>::default();
                state.decode_variant(2, "struct_value", &mut value)?;
                Self::StructValue(value)
            }
            crate::ast::ItemKind::ItemUnion => {
                let mut value = <crate::ast::UnionDef>::default();
                state.decode_variant(3, "union_value", &mut value)?;
                Self::UnionValue(value)
            }
            crate::ast::ItemKind::ItemEnum => {
                let mut value = <crate::ast::EnumDef>::default();
                state.decode_variant(4, "enum_value", &mut value)?;
                Self::EnumValue(value)
            }
            crate::ast::ItemKind::ItemException => {
                let mut value = <crate::ast::ExceptDef>::default();
                state.decode_variant(5, "exception_value", &mut value)?;
                Self::ExceptionValue(value)
            }
            crate::ast::ItemKind::ItemBitmask => {
                let mut value = <crate::ast::BitmaskDef>::default();
                state.decode_variant(6, "bitmask_value", &mut value)?;
                Self::BitmaskValue(value)
            }
            crate::ast::ItemKind::ItemBitset => {
                let mut value = <crate::ast::BitsetDef>::default();
                state.decode_variant(7, "bitset_value", &mut value)?;
                Self::BitsetValue(value)
            }
            crate::ast::ItemKind::ItemConst => {
                let mut value = <crate::ast::ConstDef>::default();
                state.decode_variant(8, "const_value", &mut value)?;
                Self::ConstValue(value)
            }
            crate::ast::ItemKind::ItemTypedef => {
                let mut value = <crate::ast::AliasDef>::default();
                state.decode_variant(9, "alias_value", &mut value)?;
                Self::AliasValue(value)
            }
            crate::ast::ItemKind::ItemInterface => {
                let mut value = <crate::ast::InterfaceDef>::default();
                state.decode_variant(10, "interface_value", &mut value)?;
                Self::InterfaceValue(value)
            }
            crate::ast::ItemKind::ItemValuetype => {
                let mut value = <crate::ast::ValuetypeDef>::default();
                state.decode_variant(11, "valuetype_value", &mut value)?;
                Self::ValuetypeValue(value)
            }
            crate::ast::ItemKind::ItemDecl => {
                let mut value = <crate::ast::Decl>::default();
                state.decode_variant(12, "decl_value", &mut value)?;
                Self::DeclValue(value)
            }
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum InterfaceMemberKind {
    /// An interface attribute.
    InterfaceAttribute,
    /// Function definition.
    InterfacePrototype,
    /// Type definition nested inside the interface.
    InterfaceItem,
}

impl InterfaceMemberKind {
    #[must_use]
    pub const fn new() -> Self {
        crate::ast::InterfaceMemberKind::InterfaceAttribute
    }
}

impl ::std::str::FromStr for InterfaceMemberKind {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "INTERFACE_ATTRIBUTE" => Ok(Self::InterfaceAttribute),
            "INTERFACE_PROTOTYPE" => Ok(Self::InterfacePrototype),
            "INTERFACE_ITEM" => Ok(Self::InterfaceItem),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for InterfaceMemberKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::InterfaceAttribute => f.write_str("INTERFACE_ATTRIBUTE"),
            Self::InterfacePrototype => f.write_str("INTERFACE_PROTOTYPE"),
            Self::InterfaceItem => f.write_str("INTERFACE_ITEM"),
        }
    }
}

impl ::std::default::Default for InterfaceMemberKind {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for InterfaceMemberKind {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::EnumSerializer as _;

        let state = ar.encode_enum("InterfaceMemberKind")?;
        match self {
            Self::InterfaceAttribute => state.encode_variant::<i32>("INTERFACE_ATTRIBUTE", 0),
            Self::InterfacePrototype => state.encode_variant::<i32>("INTERFACE_PROTOTYPE", 1),
            Self::InterfaceItem => state.encode_variant::<i32>("INTERFACE_ITEM", 2),
        }
    }
}

impl ::intercom_cts::Unmarshal for InterfaceMemberKind {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::EnumDeserializer as _;

        let state = ar.decode_enum("InterfaceMemberKind")?;
        *self = state.decode_enumerator(*self)?;
        Ok(())
    }
}

impl ::intercom_cts::decode::EnumVisitor for InterfaceMemberKind {
    fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match de.decode_i32()? {
            0 => Self::InterfaceAttribute,
            1 => Self::InterfacePrototype,
            2 => Self::InterfaceItem,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }

    fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::error::Error as _;

        let value = match name {
            "INTERFACE_ATTRIBUTE" => Self::InterfaceAttribute,
            "INTERFACE_PROTOTYPE" => Self::InterfacePrototype,
            "INTERFACE_ITEM" => Self::InterfaceItem,
            _ => return Err(D::Error::custom("Invalid enum value")),
        };
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum InterfaceMember {
    /// An interface attribute.
    Attr(crate::ast::Attribute),
    /// Function definition.
    Proto(crate::ast::Prototype),
    /// Type definition nested inside the interface.
    Item(crate::ast::Item),
}

impl InterfaceMember {
    #[must_use]
    pub fn new() -> Self {
        Self::Attr(<crate::ast::Attribute>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> crate::ast::InterfaceMemberKind {
        match self {
            Self::Attr(_) => crate::ast::InterfaceMemberKind::InterfaceAttribute,
            Self::Proto(_) => crate::ast::InterfaceMemberKind::InterfacePrototype,
            Self::Item(_) => crate::ast::InterfaceMemberKind::InterfaceItem,
        }
    }
}

impl From<crate::ast::InterfaceMemberKind> for InterfaceMember {
    fn from(disc: crate::ast::InterfaceMemberKind) -> Self {
        match disc {
            crate::ast::InterfaceMemberKind::InterfaceAttribute => {
                Self::Attr(<crate::ast::Attribute>::default())
            }
            crate::ast::InterfaceMemberKind::InterfacePrototype => {
                Self::Proto(<crate::ast::Prototype>::default())
            }
            crate::ast::InterfaceMemberKind::InterfaceItem => {
                Self::Item(<crate::ast::Item>::default())
            }
        }
    }
}

impl ::std::default::Default for InterfaceMember {
    fn default() -> Self {
        Self::new()
    }
}

impl ::intercom_cts::Marshal for InterfaceMember {
    fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::intercom_cts::encode::Serializer,
    {
        use ::intercom_cts::encode::UnionSerializer as _;

        let mut state = ar.encode_union("InterfaceMember")?;
        state.encode_discriminant(&self.disc())?;
        match self {
            Self::Attr(v) => state.encode_variant(0, "attr", v),
            Self::Proto(v) => state.encode_variant(1, "proto", v),
            Self::Item(v) => state.encode_variant(2, "item", v),
        }
    }
}

impl ::intercom_cts::Unmarshal for InterfaceMember {
    fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
    where
        D: ::intercom_cts::decode::Deserializer,
    {
        use ::intercom_cts::decode::UnionDeserializer as _;

        let mut state = ar.decode_union("InterfaceMember")?;
        let mut disc = crate::ast::InterfaceMemberKind::default();
        state.decode_discriminant(&mut disc)?;
        *self = match disc {
            crate::ast::InterfaceMemberKind::InterfaceAttribute => {
                let mut value = <crate::ast::Attribute>::default();
                state.decode_variant(0, "attr", &mut value)?;
                Self::Attr(value)
            }
            crate::ast::InterfaceMemberKind::InterfacePrototype => {
                let mut value = <crate::ast::Prototype>::default();
                state.decode_variant(1, "proto", &mut value)?;
                Self::Proto(value)
            }
            crate::ast::InterfaceMemberKind::InterfaceItem => {
                let mut value = <crate::ast::Item>::default();
                state.decode_variant(2, "item", &mut value)?;
                Self::Item(value)
            }
        };
        Ok(())
    }
}
