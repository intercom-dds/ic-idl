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

use std::fmt::Display;
use std::iter::Enumerate;

pub trait IterExt: Iterator + Sized {
    /// Joins all elements of an interator into a string with the specified
    /// separator.
    fn join(mut self, sep: &str) -> String
    where
        Self::Item: Display,
    {
        self.map(|v| v.to_string()).collect::<Vec<_>>().join("::")
    }

    /// Skips the `N`-th elment of an iterator. Elements before and after the
    /// `N`-th elements will be yielded as usual.
    fn skip_nth(self, nth: usize) -> SkipNth<Self> {
        SkipNth {
            nth,
            iter: self.enumerate(),
        }
    }
}

pub struct SkipNth<I> {
    nth: usize,
    iter: Enumerate<I>,
}

impl<I: Iterator> Iterator for SkipNth<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, next) = self.iter.next()?;
            if i != self.nth {
                break Some(next);
            }
        }
    }
}

impl<T: Iterator> IterExt for T {}
