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
        let bytes = bits.div_ceil(8); // Round up to nearest byte
        Self(vec![0u8; bytes].into_boxed_slice())
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.iter().filter(|&bit| bit).count()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len() * 8
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_size() {
        let bitset = Bitset::with_size(16);
        assert_eq!(bitset.size(), 16); // Returns size in bits

        let bitset = Bitset::with_size(1);
        assert_eq!(bitset.size(), 8); // 1 bit allocated = 1 byte = 8 bits

        let bitset = Bitset::with_size(17);
        assert_eq!(bitset.size(), 24); // 17 bits = 3 bytes = 24 bits
    }

    #[test]
    #[should_panic(expected = "size cannot be zero")]
    fn test_with_size_zero_panics() {
        let _ = Bitset::with_size(0);
    }

    #[test]
    fn test_set_basic() {
        let mut bitset = Bitset::with_size(8);
        bitset.set(0);
        bitset.set(7);

        assert!(bitset.is_set(0));
        assert!(bitset.is_set(7));
        assert!(!bitset.is_set(1));
    }

    #[test]
    fn test_set_and_is_set() {
        let mut bitset = Bitset::with_size(16);

        assert!(!bitset.is_set(0));
        assert!(!bitset.is_set(7));
        assert!(!bitset.is_set(8));
        assert!(!bitset.is_set(15));

        bitset.set(0);
        bitset.set(7);
        bitset.set(8);
        bitset.set(15);

        assert!(bitset.is_set(0));
        assert!(bitset.is_set(7));
        assert!(bitset.is_set(8));
        assert!(bitset.is_set(15));

        assert!(!bitset.is_set(1));
        assert!(!bitset.is_set(6));
        assert!(!bitset.is_set(9));
        assert!(!bitset.is_set(14));
    }

    #[test]
    fn test_count() {
        let mut bitset = Bitset::with_size(32);
        assert_eq!(bitset.count(), 0);

        bitset.set(0);
        bitset.set(5);
        bitset.set(10);
        bitset.set(15);
        bitset.set(20);

        assert_eq!(bitset.count(), 5);
    }

    #[test]
    fn test_size() {
        let bitset = Bitset::with_size(16);
        assert_eq!(bitset.size(), 16); // Returns size in bits

        let bitset = Bitset::with_size(17);
        assert_eq!(bitset.size(), 24); // 17 bits = 3 bytes = 24 bits
    }

    #[test]
    fn test_bitand() {
        let mut bitset = Bitset::with_size(16);
        bitset.set(5);
        bitset.set(10);

        assert!(bitset.clone() & 5);
        assert!(bitset.clone() & 10);
        assert!(!(bitset.clone() & 3));
        assert!(!(bitset.clone() & 11));
    }

    #[test]
    fn test_bitand_assign() {
        let mut bitset = Bitset::with_size(16);

        bitset &= 3;
        bitset &= 7;
        bitset &= 15;

        assert!(bitset.is_set(3));
        assert!(bitset.is_set(7));
        assert!(bitset.is_set(15));
    }

    #[test]
    fn test_iter() {
        let mut bitset = Bitset::with_size(8);
        bitset.set(1);
        bitset.set(3);
        bitset.set(5);
        bitset.set(7);

        let bits: Vec<bool> = bitset.iter().collect();
        assert_eq!(
            bits,
            vec![false, true, false, true, false, true, false, true]
        );
    }

    #[test]
    fn test_into_iter() {
        let mut bitset = Bitset::with_size(4);
        bitset.set(0);
        bitset.set(2);

        let bits: Vec<bool> = (&bitset).into_iter().collect();
        // Bitset::with_size(4) allocates 1 byte = 8 bits
        assert_eq!(
            bits,
            vec![true, false, true, false, false, false, false, false]
        );
    }

    #[test]
    fn test_clone() {
        let mut bitset = Bitset::with_size(16);
        bitset.set(5);
        bitset.set(10);

        let cloned = bitset.clone();
        assert!(cloned.is_set(5));
        assert!(cloned.is_set(10));
        assert!(!cloned.is_set(3));
    }

    #[test]
    fn test_debug() {
        let bitset = Bitset::with_size(8);
        let debug_str = format!("{bitset:?}");
        assert!(debug_str.contains("Bitset"));
    }

    #[test]
    fn test_large_bitset() {
        let mut bitset = Bitset::with_size(1024);

        bitset.set(0);
        bitset.set(511);
        bitset.set(1023);

        assert!(bitset.is_set(0));
        assert!(bitset.is_set(511));
        assert!(bitset.is_set(1023));

        assert!(!bitset.is_set(1));
        assert!(!bitset.is_set(510));
        assert!(!bitset.is_set(1022));
    }

    #[test]
    fn test_all_bits_in_byte() {
        let mut bitset = Bitset::with_size(8);

        // Set all bits in the byte
        for i in 0..8 {
            bitset.set(i);
        }

        // Check all bits are set
        for i in 0..8 {
            assert!(bitset.is_set(i));
        }

        assert_eq!(bitset.count(), 8);
    }

    #[test]
    fn test_cross_byte_boundary() {
        let mut bitset = Bitset::with_size(16);

        // Set bits around byte boundary
        bitset.set(6);
        bitset.set(7);
        bitset.set(8);
        bitset.set(9);

        assert!(bitset.is_set(6));
        assert!(bitset.is_set(7));
        assert!(bitset.is_set(8));
        assert!(bitset.is_set(9));
    }
}
