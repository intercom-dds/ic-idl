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
use std::sync::OnceLock;

use ic_expr::Op;
use ic_vfs::Span;
use rustc_hash::{FxBuildHasher, FxHashMap};

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

    /// An arithmetic or bitwise operator
    Op(Op),

    /// Any valid UTF-8 identifier
    Ident,

    /// A documentation-style comment
    /// If `trailing` is true, this comment appears on the same line as code
    Comment { trailing: bool },

    /// Octal, decimal or hexadecimal number
    Number { base: Base },

    /// Floating-point literal
    Float,

    /// String literal
    String { terminated: bool },

    /// Single UTF-8 character literal
    Char,

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

/// Static keyword map for efficient lookup.
static KEYWORD_MAP: OnceLock<FxHashMap<&'static str, Kw>> = OnceLock::new();

/// Returns the keyword map, initializing it on first use.
fn keyword_map() -> &'static FxHashMap<&'static str, Kw> {
    KEYWORD_MAP.get_or_init(|| {
        let mut map = FxHashMap::with_capacity_and_hasher(45, FxBuildHasher);
        map.insert("@annotation", Kw::Annotation);
        map.insert("module", Kw::Module);
        map.insert("struct", Kw::Struct);
        map.insert("const", Kw::Const);
        map.insert("bitmask", Kw::Bitmask);
        map.insert("bitset", Kw::Bitset);
        map.insert("bitfield", Kw::Bitfield);
        map.insert("enum", Kw::Enum);
        map.insert("exception", Kw::Exception);
        map.insert("typedef", Kw::Typedef);
        map.insert("native", Kw::Native);
        map.insert("fixed", Kw::Fixed);
        map.insert("union", Kw::Union);
        map.insert("switch", Kw::Switch);
        map.insert("case", Kw::Case);
        map.insert("default", Kw::Default);
        map.insert("null", Kw::Null);
        map.insert("valuetype", Kw::Valuetype);
        map.insert("public", Kw::Public);
        map.insert("private", Kw::Private);
        map.insert("supports", Kw::Supports);
        map.insert("factory", Kw::Factory);
        map.insert("local", Kw::Local);
        map.insert("interface", Kw::Interface);
        map.insert("raises", Kw::Raises);
        map.insert("getraises", Kw::GetRaises);
        map.insert("setraises", Kw::SetRaises);
        map.insert("attribute", Kw::Attribute);
        map.insert("readonly", Kw::ReadOnly);
        map.insert("oneway", Kw::Oneway);
        map.insert("in", Kw::In);
        map.insert("out", Kw::Out);
        map.insert("inout", Kw::InOut);
        map.insert("map", Kw::Map);
        map.insert("sequence", Kw::Sequence);
        map.insert("string", Kw::String);
        map.insert("wstring", Kw::WString);
        map.insert("unsigned", Kw::Unsigned);
        map.insert("short", Kw::Short);
        map.insert("long", Kw::Long);
        map.insert("float", Kw::Float);
        map.insert("double", Kw::Double);
        map.insert("TRUE", Kw::True);
        map.insert("true", Kw::True);
        map.insert("FALSE", Kw::False);
        map.insert("false", Kw::False);
        map
    })
}

impl Kw {
    /// Converts a string slice to a keyword if it matches one.
    ///
    /// Returns `None` if the string is not a recognized keyword.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn from_str(str: &str) -> Option<Self> {
        keyword_map().get(str).copied()
    }
}

impl fmt::Display for Kw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
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
        };
        write!(f, "{str}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_map_completeness() {
        let map = keyword_map();

        // Test all keywords are present
        assert_eq!(map.get("module"), Some(&Kw::Module));
        assert_eq!(map.get("struct"), Some(&Kw::Struct));

        // Test case sensitivity
        assert_eq!(map.get("TRUE"), Some(&Kw::True));
        assert_eq!(map.get("true"), Some(&Kw::True));
        assert_eq!(map.get("FALSE"), Some(&Kw::False));
        assert_eq!(map.get("false"), Some(&Kw::False));

        // Test non-keywords
        assert_eq!(map.get("foo"), None);
        assert_eq!(map.get(""), None);
    }
}
