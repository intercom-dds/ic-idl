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

use std::fmt;

use ic_vfs::Span;

/// Represents a lexical token with its kind and source location.
#[derive(Copy, Clone, Debug)]
pub struct Token {
    /// The type of token.
    pub kind: Kind,

    /// The source location of the token.
    pub span: Span,
}

/// Numeric base for integer literals.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Base {
    /// Base 8 (octal) numbers, e.g., 0777
    Octal = 8,

    /// Base 10 (decimal) numbers, e.g., 123
    Decimal = 10,

    /// Base 16 (hexadecimal) numbers, e.g., 0xFF
    Hexadecimal = 16,
}

/// IDL keywords.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Kw {
    Annotation,
    Module,
    Struct,
    Const,
    Bitmask,
    Bitset,
    Bitfield,
    Enum,
    Exception,
    Typedef,
    Native,
    Fixed,
    Union,
    Switch,
    Case,
    Default,
    Null,
    Valuetype,
    Public,
    Private,
    Supports,
    Factory,
    Local,
    Interface,
    Raises,
    GetRaises,
    SetRaises,
    Attribute,
    ReadOnly,
    Oneway,
    In,
    Out,
    InOut,
    Map,
    Sequence,
    String,
    WString,
    Unsigned,
    Short,
    Long,
    Float,
    Double,
    True,
    False,
}

/// Token kinds representing different lexical elements.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Kind {
    /// An IDL keyword
    Keyword(Kw),

    /// Any valid UTF-8 identifier
    Ident,

    /// A documentation-style comment
    /// If `trailing` is true, this comment appears on the same line as code
    /// If `terminated` is false, this is an unterminated block comment
    Comment { trailing: bool, terminated: bool },

    /// Octal, decimal or hexadecimal number
    Number { base: Base },

    /// Floating-point literal
    Float,

    /// String literal
    String { terminated: bool },

    /// Wide string literal
    WString { terminated: bool },

    /// Single UTF-8 character literal
    Char,

    /// Wide character literal
    WChar,

    /// `@`
    At,

    /// `#`
    Hash,

    /// `,`
    Comma,

    /// `.`
    Period,

    /// `:`
    Colon,

    /// `::`
    DColon,

    /// `:`
    Semi,

    /// `=`
    Eq,

    /// `==`
    EqEq,

    /// `!=`
    NotEq,

    /// `{`
    LBrace,

    /// `}`
    RBrace,

    /// `(`
    LParen,

    /// `)`
    RParen,

    /// `[`
    LBracket,

    /// `]`
    RBracket,

    /// `<`
    Lt,

    /// `>`
    Gt,

    /// `<=`
    LtEq,

    /// `>=`
    GtEq,

    /// `~`
    BitNot,

    /// `&`
    BitAnd,

    /// `|`
    BitOr,

    /// `^`
    BitXor,

    /// `!`
    Not,

    /// `&&`
    And,

    /// `||`
    Or,

    /// `+`
    Plus,

    /// `-`
    Minus,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `%`
    Modulo,

    /// `?`
    Question,

    /// `\n`
    Newline,

    /// `\`
    Backslash,

    /// Fallback for invalid tokens
    Unknown,

    /// End of input. This token is necessary to be able to propagate the span
    /// of the last token that was yielded, so we can properly pinpoint where
    /// an error occurred.
    Eoi,
}

impl Kind {
    /// Returns the string representation of the token.
    #[must_use]
    pub fn as_str(&self) -> Option<&'static str> {
        Some(match self {
            Kind::Keyword(v) => v.as_str(),
            Kind::Colon => ":",
            Kind::DColon => "::",
            Kind::Eq => "=",
            Kind::Semi => ";",
            Kind::Comma => ",",
            Kind::Period => ".",
            Kind::Lt => "<",
            Kind::Gt => ">",
            Kind::LtEq => "<=",
            Kind::GtEq => ">=",
            Kind::LBrace => "{",
            Kind::RBrace => "}",
            Kind::LParen => "(",
            Kind::RParen => ")",
            Kind::LBracket => "[",
            Kind::RBracket => "]",
            Kind::BitNot => "~",
            Kind::BitAnd => "&",
            Kind::BitOr => "|",
            Kind::BitXor => "^",
            Kind::Plus => "+",
            Kind::Minus => "-",
            Kind::Star => "*",
            Kind::Slash => "/",
            Kind::Modulo => "%",
            Kind::Hash => "#",
            Kind::EqEq => "==",
            Kind::NotEq => "!=",
            Kind::And => "&&",
            Kind::Not => "!",
            Kind::Or => "||",
            Kind::Question => "?",
            Kind::Backslash => "\\",
            Kind::Newline => "\n",
            Kind::At => "@",
            Kind::Ident
            | Kind::Comment { .. }
            | Kind::Number { .. }
            | Kind::String { .. }
            | Kind::WString { .. }
            | Kind::Float
            | Kind::Char
            | Kind::WChar
            | Kind::Unknown
            | Kind::Eoi => return None,
        })
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Keyword(v) => write!(f, "{v}"),
            Kind::Colon => write!(f, "`:`"),
            Kind::DColon => write!(f, "`::`"),
            Kind::Eq => write!(f, "`=`"),
            Kind::Semi => write!(f, "`;`"),
            Kind::Comma => write!(f, "`,`"),
            Kind::Period => write!(f, "`.`"),
            Kind::Lt => write!(f, "`<`"),
            Kind::Gt => write!(f, "`>`"),
            Kind::LtEq => write!(f, "`<=`"),
            Kind::GtEq => write!(f, "`>=`"),
            Kind::LBrace => write!(f, "`{{`"),
            Kind::RBrace => write!(f, "`}}`"),
            Kind::LParen => write!(f, "`(`"),
            Kind::RParen => write!(f, "`)`"),
            Kind::LBracket => write!(f, "`[`"),
            Kind::RBracket => write!(f, "`]`"),
            Kind::BitNot => write!(f, "`~`"),
            Kind::BitAnd => write!(f, "`&`"),
            Kind::BitOr => write!(f, "`|`"),
            Kind::BitXor => write!(f, "`^`"),
            Kind::Plus => write!(f, "`+`"),
            Kind::Minus => write!(f, "`-`"),
            Kind::Star => write!(f, "`*`"),
            Kind::Slash => write!(f, "`/`"),
            Kind::Modulo => write!(f, "`%`"),
            Kind::Hash => write!(f, "`#`"),
            Kind::EqEq => write!(f, "`==`"),
            Kind::NotEq => write!(f, "`!=`"),
            Kind::And => write!(f, "`&&`"),
            Kind::Not => write!(f, "`!`"),
            Kind::Or => write!(f, "`||`"),
            Kind::Question => write!(f, "`?`"),
            Kind::Backslash => write!(f, "`\\`"),
            Kind::Newline => write!(f, "newline"),
            Kind::Char => write!(f, "char"),
            Kind::WChar => write!(f, "wide char"),
            Kind::Number { .. } => write!(f, "number"),
            Kind::Ident => write!(f, "identifier"),
            Kind::At => write!(f, "annotation"),
            Kind::Float => write!(f, "floating-point number"),
            Kind::String { .. } => write!(f, "string"),
            Kind::WString { .. } => write!(f, "wide string"),
            Kind::Comment { .. } => write!(f, "comment"),
            Kind::Eoi => write!(f, "end of input"),
            Kind::Unknown => write!(f, "unknown"),
        }
    }
}

impl Kw {
    /// Returns the string representation of the keyword.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Kw::Annotation => "@annotation",
            Kw::Module => "module",
            Kw::Struct => "struct",
            Kw::Const => "const",
            Kw::Bitmask => "bitmask",
            Kw::Bitset => "bitset",
            Kw::Bitfield => "bitfield",
            Kw::Enum => "enum",
            Kw::Exception => "exception",
            Kw::Typedef => "typedef",
            Kw::Native => "native",
            Kw::Fixed => "fixed",
            Kw::Union => "union",
            Kw::Switch => "switch",
            Kw::Case => "case",
            Kw::Default => "default",
            Kw::Null => "null",
            Kw::Valuetype => "valuetype",
            Kw::Public => "public",
            Kw::Private => "private",
            Kw::Supports => "supports",
            Kw::Factory => "factory",
            Kw::Local => "local",
            Kw::Interface => "interface",
            Kw::Raises => "raises",
            Kw::GetRaises => "getraises",
            Kw::SetRaises => "setraises",
            Kw::Attribute => "attribute",
            Kw::ReadOnly => "readonly",
            Kw::Oneway => "oneway",
            Kw::In => "in",
            Kw::Out => "out",
            Kw::InOut => "inout",
            Kw::Map => "map",
            Kw::Sequence => "sequence",
            Kw::String => "string",
            Kw::WString => "wstring",
            Kw::Unsigned => "unsigned",
            Kw::Short => "short",
            Kw::Long => "long",
            Kw::Float => "float",
            Kw::Double => "double",
            Kw::True => "TRUE",
            Kw::False => "FALSE",
        }
    }

    /// Converts a string slice to a keyword if it matches one.
    ///
    /// Returns `None` if the string is not a recognized keyword.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn from_str(str: &str) -> Option<Self> {
        Some(match str {
            "@annotation" => Kw::Annotation,
            "module" => Kw::Module,
            "struct" => Kw::Struct,
            "const" => Kw::Const,
            "bitmask" => Kw::Bitmask,
            "bitset" => Kw::Bitset,
            "bitfield" => Kw::Bitfield,
            "enum" => Kw::Enum,
            "exception" => Kw::Exception,
            "typedef" => Kw::Typedef,
            "native" => Kw::Native,
            "fixed" => Kw::Fixed,
            "union" => Kw::Union,
            "switch" => Kw::Switch,
            "case" => Kw::Case,
            "default" => Kw::Default,
            "null" => Kw::Null,
            "valuetype" => Kw::Valuetype,
            "public" => Kw::Public,
            "private" => Kw::Private,
            "supports" => Kw::Supports,
            "factory" => Kw::Factory,
            "local" => Kw::Local,
            "interface" => Kw::Interface,
            "raises" => Kw::Raises,
            "getraises" => Kw::GetRaises,
            "setraises" => Kw::SetRaises,
            "attribute" => Kw::Attribute,
            "readonly" => Kw::ReadOnly,
            "oneway" => Kw::Oneway,
            "in" => Kw::In,
            "out" => Kw::Out,
            "inout" => Kw::InOut,
            "map" => Kw::Map,
            "sequence" => Kw::Sequence,
            "string" => Kw::String,
            "wstring" => Kw::WString,
            "unsigned" => Kw::Unsigned,
            "short" => Kw::Short,
            "long" => Kw::Long,
            "float" => Kw::Float,
            "double" => Kw::Double,
            "TRUE" | "true" => Kw::True,
            "FALSE" | "false" => Kw::False,
            _ => return None,
        })
    }
}

impl fmt::Display for Kw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
