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

use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

pub const EOF: char = '\0';

/// Extension trait for iterators.
pub trait IteratorExt<'a, T>
where
    T: Iterator,
{
    /// Advances the iterator until the predicate yields `false` for the next
    /// (peeked) element. This will not consume the element for which the
    /// predicate yielded `false`.
    fn take_while_peek<P>(self, pred: P) -> TakeWhilePeek<'a, T, P>
    where
        P: FnMut(&T::Item) -> bool;
}

impl<'a, T> IteratorExt<'a, T> for &'a mut Peekable<T>
where
    T: Iterator,
{
    fn take_while_peek<P>(self, pred: P) -> TakeWhilePeek<'a, T, P>
    where
        P: FnMut(&T::Item) -> bool,
    {
        TakeWhilePeek { iter: self, pred }
    }
}

/// See [`IteratorExt::take_while_peek`] for details.
#[must_use]
pub struct TakeWhilePeek<'a, I, P>
where
    I: Iterator,
{
    iter: &'a mut Peekable<I>,
    pred: P,
}

impl<I, P> Iterator for TakeWhilePeek<'_, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let peek = self.iter.peek()?;
        if (self.pred)(peek) {
            self.iter.next()
        } else {
            None
        }
    }
}

/// An indexed, self-referential version of `std::str::Chars` that owns the
/// data it iterates over, which lets us bypass the lifetime bound.
#[must_use]
#[derive(Clone, Debug)]
pub struct OwnedChars {
    chars: Peekable<Chars<'static>>,
    inner: Rc<str>,
    index: u32,
    line: u32,
}

impl OwnedChars {
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.inner.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn index(&self) -> u32 {
        self.index
    }

    #[inline]
    #[must_use]
    pub fn line(&self) -> u32 {
        self.line
    }

    #[inline(always)]
    #[must_use]
    pub fn peek(&mut self) -> char {
        match self.chars.peek() {
            Some(&c) => c,
            None => EOF,
        }
    }
}

impl From<Rc<str>> for OwnedChars {
    fn from(inner: Rc<str>) -> Self {
        // SAFETY: The pointed-to buffer is already heap allocated and is
        // guaranteed to not move. Since we also hold ownership over the
        // buffer, it will never go out of scope for the lifetime of `Self`.
        let iter = unsafe { std::mem::transmute::<Chars<'_>, Chars<'_>>(inner.chars()) };
        Self {
            chars: iter.peekable(),
            inner,
            index: 0,
            line: 1,
        }
    }
}

impl Iterator for OwnedChars {
    type Item = char;

    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.chars.next()?;
        // Most characters are ASCII (1 byte), optimize for that case
        if c.is_ascii() {
            self.index += 1;
            if c == '\n' {
                self.line += 1;
            }
        } else {
            self.index += c.len_utf8() as u32;
        }
        Some(c)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.chars.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.chars.last()
    }
}
