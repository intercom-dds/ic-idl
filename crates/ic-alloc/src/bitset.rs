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

use std::ops::{BitAnd, BitAndAssign};

#[must_use]
#[derive(Clone, Debug)]
pub struct Bitset(Box<[u8]>);

impl Bitset {
    /// # Panics
    ///
    /// Panics if if `bits == 0`.
    pub fn with_size(bits: usize) -> Self {
        assert!(bits > 0, "size cannot be zero");
        Self(Vec::with_capacity(bits).into_boxed_slice())
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.iter().count()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn set(&mut self, pos: usize) {
        let bit = pos % 8;
        let nth = pos / 8;
        self.0[nth] |= 1 << bit;
    }

    #[must_use]
    pub fn is_set(&self, pos: usize) -> bool {
        let bit = pos % 8;
        let nth = pos / 8;
        (self.0[nth] & (1 << bit)) != 0
    }

    pub fn iter(&self) -> BitIterator<'_> {
        BitIterator {
            index: 0,
            bitset: self,
        }
    }
}

impl BitAnd<usize> for Bitset {
    type Output = bool;

    fn bitand(self, rhs: usize) -> Self::Output {
        self.is_set(rhs)
    }
}

impl BitAndAssign<usize> for Bitset {
    fn bitand_assign(&mut self, rhs: usize) {
        self.set(rhs);
    }
}

#[must_use]
pub struct BitIterator<'a> {
    index: usize,
    bitset: &'a Bitset,
}

impl Iterator for BitIterator<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.bitset.size() {
            let value = self.bitset.is_set(self.index);
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a Bitset {
    type IntoIter = BitIterator<'a>;
    type Item = bool;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
