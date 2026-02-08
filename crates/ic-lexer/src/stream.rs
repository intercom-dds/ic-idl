// Copyright 2026 KONGSBERG
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

use ic_vfs::Span;

use crate::token::{Kind, Token};

#[must_use]
#[derive(Debug)]
pub struct Stream {
    tokens: Vec<Token>,
    pos: usize,
}

#[must_use]
#[derive(Clone, Copy)]
pub struct StreamCheckpoint {
    pos: usize,
}

impl Stream {
    #[inline]
    pub fn new(iter: impl IntoIterator<Item = Token>) -> Self {
        let tokens: Vec<_> = iter.into_iter().collect();
        Self { tokens, pos: 0 }
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Kind {
        self.tokens.get(self.pos).map_or(Kind::Eoi, |tok| tok.kind)
    }

    #[inline]
    #[must_use]
    pub fn current(&self) -> Token {
        self.tokens.get(self.pos).copied().unwrap_or(Token {
            kind: Kind::Eoi,
            span: Span::default(),
        })
    }

    #[inline]
    #[must_use]
    pub fn advance(&mut self) -> Token {
        let tok = self.current();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    #[inline]
    #[must_use]
    pub fn at(&self, kind: Kind) -> bool {
        self.peek() == kind
    }

    #[inline]
    #[must_use]
    pub fn peek_nth(&self, n: usize) -> Kind {
        self.tokens
            .get(self.pos + n)
            .map_or(Kind::Eoi, |tok| tok.kind)
    }

    #[inline]
    pub fn checkpoint(&self) -> StreamCheckpoint {
        StreamCheckpoint { pos: self.pos }
    }

    #[inline]
    pub fn rewind(&mut self, cp: StreamCheckpoint) {
        self.pos = cp.pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(kind: Kind) -> Token {
        Token {
            kind,
            span: Span::default(),
        }
    }

    fn sample_stream() -> Stream {
        Stream::new(vec![tok(Kind::Ident), tok(Kind::Semi), tok(Kind::Ident)])
    }

    #[test]
    fn empty_stream_is_eoi() {
        let s = Stream::new(std::iter::empty());
        assert_eq!(s.peek(), Kind::Eoi);
        assert_eq!(s.current().kind, Kind::Eoi);
    }

    #[test]
    fn advance_returns_current_and_moves_forward() {
        let mut s = sample_stream();
        let first = s.advance();
        assert_eq!(first.kind, Kind::Ident);
        assert_eq!(s.peek(), Kind::Semi);
    }

    #[test]
    fn advance_past_end_returns_eoi() {
        let mut s = Stream::new(vec![tok(Kind::Ident)]);
        _ = s.advance();
        let past = s.advance();
        assert_eq!(past.kind, Kind::Eoi);
        assert_eq!(s.peek(), Kind::Eoi);
    }

    #[test]
    fn peek_nth_looks_ahead() {
        let s = sample_stream();
        assert_eq!(s.peek_nth(0), Kind::Ident);
        assert_eq!(s.peek_nth(1), Kind::Semi);
        assert_eq!(s.peek_nth(2), Kind::Ident);
        assert_eq!(s.peek_nth(3), Kind::Eoi);
    }

    #[test]
    fn checkpoint_and_rewind() {
        let mut s = sample_stream();
        let cp = s.checkpoint();
        _ = s.advance();
        _ = s.advance();
        assert_eq!(s.peek(), Kind::Ident);
        s.rewind(cp);
        assert_eq!(s.peek(), Kind::Ident);
        assert_eq!(s.peek_nth(1), Kind::Semi);
    }

    #[test]
    fn at_matches_current_kind() {
        let s = sample_stream();
        assert!(s.at(Kind::Ident));
        assert!(!s.at(Kind::Semi));
    }
}
