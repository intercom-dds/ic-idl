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

use std::ops::Range;

use intercom_cts::{Marshal, Unmarshal};

use crate::FileId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Marshal, Unmarshal)]
pub struct Position {
    pub offset: u32,
    pub file_id: FileId,
}

impl Position {
    #[must_use]
    pub fn new(offset: u32, file_id: FileId) -> Self {
        Self { offset, file_id }
    }
}

// This really shouldn't be default constructible, but all generated code
// relies on it. We have a sanity lint that verifies all `Position`s in the AST
// has a valid `FileId`, so that should hopefully catch such cases.
impl Default for Position {
    fn default() -> Self {
        Self::new(0, FileId::_do_not_use())
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Marshal, Unmarshal)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

// This doesn't really belong here, but since we can't implement the trait in
// `ic-parse` because of orphan rules, we have to do it here instead. Using a
// newtype wrapper in `ic-parse` is not ideal because it's used _everywhere_.
impl chumsky::Span for Span {
    type Context = ();
    type Offset = Position;

    #[inline]
    fn new(_: Self::Context, range: Range<Self::Offset>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    #[inline]
    fn context(&self) -> Self::Context {}

    #[inline]
    fn start(&self) -> Self::Offset {
        self.start
    }

    #[inline]
    fn end(&self) -> Self::Offset {
        self.end
    }
}

impl Span {
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        debug_assert_eq!(
            self.start.file_id, self.end.file_id,
            "attempted to derive Range<> from Span where start.file_id != end.file_id",
        );
        self.start.offset as usize..self.end.offset as usize
    }
}

impl From<Span> for Range<usize> {
    #[inline]
    fn from(val: Span) -> Self {
        val.range()
    }
}
