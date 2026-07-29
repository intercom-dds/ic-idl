// Copyright 2025 KONGSBERG
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

use std::ops::{Deref, Range};
use std::sync::Arc;

use super::Cursor;

/// An owned, cloneable, zero-copy-sliceable byte cursor backed by an `Arc<[u8]>`.
///
/// `ArcSlice` holds a reference-counted byte buffer and tracks a visible
/// window into it via an offset and length. Cloning an `ArcSlice` is cheap,
/// and slicing produces a new cursor that shares the same underlying
/// allocation.
#[must_use]
#[derive(Clone, Default)]
pub struct ArcSlice {
    data: Arc<[u8]>,
    offset: usize,
    len: usize,
}

impl ArcSlice {
    pub fn new(data: Arc<[u8]>) -> Self {
        let len = data.len();
        Self {
            data,
            offset: 0,
            len,
        }
    }

    pub fn slice(&self, range: Range<usize>) -> ArcSlice {
        assert!(
            range.start <= range.end,
            "slice range start ({}) must not exceed end ({})",
            range.start,
            range.end,
        );
        assert!(
            range.end <= self.len,
            "slice range end ({}) out of bounds for length {}",
            range.end,
            self.len,
        );

        ArcSlice {
            data: Arc::clone(&self.data),
            offset: self.offset + range.start,
            len: range.end - range.start,
        }
    }

    pub fn as_cursor(&self) -> Cursor<'_> {
        Cursor::new(self.as_ref())
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl std::fmt::Debug for ArcSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcSlice")
            .field("len", &self.len)
            .field("bytes", &&self.data[self.offset..self.offset + self.len])
            .finish()
    }
}

impl AsRef<[u8]> for ArcSlice {
    fn as_ref(&self) -> &[u8] {
        &self.data[self.offset..self.offset + self.len]
    }
}

impl Deref for ArcSlice {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data[self.offset..self.offset + self.len]
    }
}

impl PartialEq for ArcSlice {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for ArcSlice {}

impl From<Arc<[u8]>> for ArcSlice {
    fn from(data: Arc<[u8]>) -> Self {
        Self::new(data)
    }
}

impl From<Vec<u8>> for ArcSlice {
    fn from(data: Vec<u8>) -> Self {
        Self::new(Arc::from(data))
    }
}

impl From<&[u8]> for ArcSlice {
    fn from(data: &[u8]) -> Self {
        Self::new(Arc::from(data))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ArcSlice;
    use crate::buf::endian::Native;

    #[test]
    fn from_arc() {
        let data: Arc<[u8]> = Arc::from([1u8, 2, 3].as_slice());
        let cursor = ArcSlice::from(data);
        assert_eq!(cursor.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn from_vec() {
        let cursor = ArcSlice::from(vec![4u8, 5, 6]);
        assert_eq!(cursor.as_ref(), &[4, 5, 6]);
    }

    #[test]
    fn from_slice() {
        let cursor = ArcSlice::from([7u8, 8, 9].as_slice());
        assert_eq!(cursor.as_ref(), &[7, 8, 9]);
    }

    #[test]
    fn empty_cursor() {
        let cursor = ArcSlice::from(Vec::new());
        assert_eq!(cursor.len(), 0);
        assert!(cursor.is_empty());
    }

    #[test]
    fn slice_subrange() {
        let cursor = ArcSlice::from(vec![10, 20, 30, 40, 50]);
        let sub = cursor.slice(1..4);
        assert_eq!(sub.as_ref(), &[20, 30, 40]);
        assert_eq!(sub.len(), 3);
        assert!(!sub.is_empty());
    }

    #[test]
    fn slice_full_range() {
        let cursor = ArcSlice::from(vec![1, 2, 3]);
        let sub = cursor.slice(0..3);
        assert_eq!(sub.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn slice_empty_range() {
        let cursor = ArcSlice::from(vec![1, 2, 3]);
        let sub = cursor.slice(2..2);
        assert!(sub.is_empty());
        assert_eq!(sub.as_ref(), &[]);
    }

    #[test]
    fn slice_of_slice() {
        let cursor = ArcSlice::from(vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let first = cursor.slice(2..6);
        assert_eq!(first.as_ref(), &[2, 3, 4, 5]);

        let second = first.slice(1..3);
        assert_eq!(second.as_ref(), &[3, 4]);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn slice_oob_panics() {
        let cursor = ArcSlice::from(vec![1, 2, 3]);
        let _ = cursor.slice(0..4);
    }

    #[test]
    #[should_panic(expected = "must not exceed")]
    #[allow(clippy::reversed_empty_ranges)]
    fn slice_inverted_range_panics() {
        let cursor = ArcSlice::from(vec![1, 2, 3]);
        let _ = cursor.slice(2..1);
    }

    #[test]
    fn clone_independence() {
        let cursor = ArcSlice::from(vec![1, 2, 3, 4]);
        let sliced = cursor.slice(1..3);
        let cloned = sliced.clone();

        assert_eq!(sliced, cloned);
        assert_eq!(cursor.as_ref(), &[1, 2, 3, 4]);
        assert_eq!(sliced.as_ref(), &[2, 3]);
        assert_eq!(cloned.as_ref(), &[2, 3]);
    }

    #[test]
    fn partial_eq_across_arcs() {
        let a = ArcSlice::from(vec![1, 2, 3]);
        let b = ArcSlice::from(vec![1, 2, 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn as_cursor_reads_correct_bytes() {
        let cursor = ArcSlice::from(vec![0, 0, 0, 42]);
        let mut read = cursor.as_cursor();
        assert_eq!(
            read.read_u32::<Native>().unwrap(),
            u32::from_ne_bytes([0, 0, 0, 42])
        );
    }

    #[test]
    fn as_cursor_on_slice() {
        let cursor = ArcSlice::from(vec![10, 20, 30, 40, 50]);
        let sliced = cursor.slice(1..4);
        let read = sliced.as_cursor();
        assert_eq!(read.as_ref(), &[20, 30, 40]);
    }
}
