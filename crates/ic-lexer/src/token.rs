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

use ic_expr::Op;
use ic_vfs::Span;

#[derive(Copy, Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Base {
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Kw {
    Any,
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
}

// I think this may be the way to go. Maybe even have a generic K type for
// keywords. But this makes it actually doable to handle in the preprocessor.
// We can treat it as an identifier, and not have hundreds of branches.
//
// The cursor could identify keywords + preprocessor tokens. So we can then
// define directive here as well, and not have a Hash token etc.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Kind {
    /// An IDL keyword
    Keyword(Kw),

    /// An arithmetic or bitwise operator
    Op(Op),

    /// Any valid UAX#31 identifier
    Ident,

    /// A documentation-style comment
    Comment,

    /// Octal, decimal or hexadecimal number
    Number { base: Base },

    /// Floating-point literal
    Float,

    /// String literal
    String { terminated: bool },

    /// Single UTF-8 character literal
    Char,

    /// `#`
    Hash,

    /// `@`
    At,

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
