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

use crate::token::Kind;

/// ASCII whitespace lookup table for faster checking
pub const ASCII_WHITESPACE: [bool; 128] = {
    let mut table = [false; 128];
    table[b' ' as usize] = true;
    table[b'\t' as usize] = true;
    table[b'\n' as usize] = true;
    table[b'\r' as usize] = true;
    table[0x0B] = true;
    table[0x0C] = true;
    table
};

/// Lookup table for single-character tokens
pub const SINGLE_CHAR_TOKENS: [Option<Kind>; 128] = {
    let mut table = [None; 128];
    table[b'#' as usize] = Some(Kind::Hash);
    table[b',' as usize] = Some(Kind::Comma);
    table[b'.' as usize] = Some(Kind::Period);
    table[b';' as usize] = Some(Kind::Semi);
    table[b'{' as usize] = Some(Kind::LBrace);
    table[b'}' as usize] = Some(Kind::RBrace);
    table[b'[' as usize] = Some(Kind::LBracket);
    table[b']' as usize] = Some(Kind::RBracket);
    table[b'(' as usize] = Some(Kind::LParen);
    table[b')' as usize] = Some(Kind::RParen);
    table[b'+' as usize] = Some(Kind::Plus);
    table[b'-' as usize] = Some(Kind::Minus);
    table[b'*' as usize] = Some(Kind::Star);
    table[b'%' as usize] = Some(Kind::Modulo);
    table[b'?' as usize] = Some(Kind::Question);
    table[b'\n' as usize] = Some(Kind::Newline);
    table[b'\\' as usize] = Some(Kind::Backslash);
    table[b'~' as usize] = Some(Kind::BitNot);
    table[b'^' as usize] = Some(Kind::BitXor);
    table
};

/// Fast lookup for ASCII characters to determine if they need special handling
pub const SPECIAL_CHARS: [bool; 128] = {
    let mut table = [false; 128];
    table[b'&' as usize] = true;
    table[b'|' as usize] = true;
    table[b'=' as usize] = true;
    table[b':' as usize] = true;
    table[b'!' as usize] = true;
    table[b'>' as usize] = true;
    table[b'<' as usize] = true;
    table[b'"' as usize] = true;
    table[b'\'' as usize] = true;
    table[b'@' as usize] = true;
    table[b'/' as usize] = true;
    table
};

#[inline]
pub fn get_single_char_token(c: char) -> Option<Kind> {
    if (c as u32) < 128 {
        SINGLE_CHAR_TOKENS[c as usize]
    } else {
        None
    }
}

#[inline]
pub fn is_special_char(c: char) -> bool {
    (c as u32) < 128 && SPECIAL_CHARS[c as usize]
}

#[inline]
pub fn is_ascii_whitespace(c: char) -> bool {
    (c as u32) < 128 && ASCII_WHITESPACE[c as usize]
}
