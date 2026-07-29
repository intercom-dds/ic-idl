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

use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::sync::Arc;

use crate::buf::endian::Endian;
use crate::buf::{ArcSlice, Cursor, Native};

/// A write-oriented buffer for building byte sequences with a specific
/// endianness.
///
/// `Buffer` provides a convenient way to serialize primitive numeric types
/// (like `u16`, `i32`, etc.) and byte slices into a contiguous byte vector. It
/// maintains an internal write cursor, and all write operations advance this
/// cursor automatically.
///
/// The buffer will automatically resize itself as needed when write operations
/// would exceed its current capacity.
///
/// The generic parameter `E` specifies the endianness to use when writing
/// multi-byte integers. It defaults to using the target's native endianness
/// unless otherwise is specified.
#[must_use]
#[derive(Default)]
pub struct Buffer<E: Endian = Native> {
    buf: Vec<u8>,
    write_idx: usize,
    _marker: PhantomData<E>,
}

impl<E: Endian> Buffer<E> {
    /// Creates a new `Buffer` with a default initial capacity of 64 bytes.
    pub fn new() -> Self {
        Self::with_capacity(64)
    }

    /// Creates a new `Buffer` with a specified initial capacity.
    pub fn with_capacity(len: usize) -> Self {
        Self {
            buf: Vec::with_capacity(len),
            write_idx: 0,
            _marker: PhantomData,
        }
    }

    /// Writes a `u8` to the current position in the buffer, advancing the
    /// position.
    #[inline]
    pub fn write_u8(&mut self, val: u8) {
        self.reserve_n(size_of::<u8>());
        self.buf[self.write_idx] = val;
        self.write_idx += 1;
    }

    /// Writes an `i8` to the current position in the buffer, advancing the
    /// position.
    #[inline]
    pub fn write_i8(&mut self, val: i8) {
        self.write_u8(val.cast_unsigned());
    }

    /// Writes a `u16` to the buffer using the specified endianness and
    /// advances the position.
    #[inline]
    pub fn write_u16(&mut self, val: u16) {
        self.reserve_n(size_of::<u16>());
        E::write_u16(val, &mut self.buf[self.write_idx..self.write_idx + 2]);
        self.write_idx += 2;
    }

    /// Writes an `i16` to the buffer using the specified endianness and
    /// advances the position.
    #[inline]
    pub fn write_i16(&mut self, val: i16) {
        self.write_u16(val.cast_unsigned());
    }

    /// Writes a `u32` to the buffer using the specified endianness and
    /// advances the position.
    #[inline]
    pub fn write_u32(&mut self, val: u32) {
        self.reserve_n(size_of::<u32>());
        E::write_u32(val, &mut self.buf[self.write_idx..self.write_idx + 4]);
        self.write_idx += 4;
    }

    /// Writes an `i32` to the buffer using the specified endianness and
    /// advances the position.
    #[inline]
    pub fn write_i32(&mut self, val: i32) {
        self.write_u32(val.cast_unsigned());
    }

    /// Writes a `u64` to the buffer using the specified endianness and
    /// advances the position.
    #[inline]
    pub fn write_u64(&mut self, val: u64) {
        self.reserve_n(size_of::<u64>());
        E::write_u64(val, &mut self.buf[self.write_idx..self.write_idx + 8]);
        self.write_idx += 8;
    }

    /// Writes an `i64` to the buffer using the specified endianness and
    /// advances the position.
    #[inline]
    pub fn write_i64(&mut self, val: i64) {
        self.write_u64(val.cast_unsigned());
    }

    /// Appends a byte slice to the end of the buffer, resizing if necessary.
    #[inline]
    pub fn extend(&mut self, slice: &[u8]) {
        let end_idx = self.write_idx + slice.len();
        if end_idx >= self.remaining() {
            self.reserve_n(slice.len());
        }

        self.buf[self.write_idx..end_idx].copy_from_slice(slice);
        self.write_idx += slice.len();
    }

    /// Returns the number of bytes remaining in the buffer's current capacity.
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.buf.len() - self.write_idx
    }

    /// Advances the write position to ensure it meets the specified alignment.
    ///
    /// # Panics
    ///
    /// This function will panic in debug builds if `align` is not a power of
    /// two.
    #[inline]
    pub fn align(&mut self, align: usize) {
        debug_assert!(
            align != 0 && align.is_power_of_two(),
            "alignment must be a power of 2",
        );

        let dt = (align - (self.pos() & (align - 1))) & (align - 1);
        if dt > 0 {
            self.reserve_n(dt);
            self.write_idx += dt;
        }
    }

    /// Advances the write position to align it to the memory alignment of type
    /// `T`.
    #[inline]
    pub fn align_to<T>(&mut self) {
        self.align(size_of::<T>());
    }

    /// Resizes the buffer's underlying `Vec<u8>` to the specified new length.
    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.buf.resize(len, 0);
    }

    /// Ensures that the buffer has space for at least `n` more bytes, resizing
    /// the buffer if the remaining capacity is insufficient.
    #[inline]
    pub fn reserve_n(&mut self, n: usize) {
        if self.remaining() < n {
            self.resize(self.buf.len() + n);
        }
    }

    /// Unconditionally increases the buffer's capacity by `n` bytes.
    #[inline]
    pub fn really_reserve_n(&mut self, n: usize) {
        self.resize(self.buf.len() + n);
    }

    /// Returns the current write position (index) in the buffer.
    #[inline]
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.write_idx
    }

    /// Advances the write position by `n` bytes, reserving space if necessary.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.reserve_n(n);
        self.write_idx += n;
    }

    /// Sets the write position to a new index.
    ///
    /// # Panics
    ///
    /// Panics if the specified `index` is greater than the buffer's current
    /// length.
    #[inline]
    pub fn set_pos(&mut self, index: usize) {
        if self.buf.len() >= index {
            // SAFETY: bounds checked
            unsafe {
                self.unchecked_set_pos(index);
            }
        } else {
            panic!("index out of bounds");
        }
    }

    /// Copies a sequence of bytes from `src` range to the `dest` position
    /// within the buffer.
    ///
    /// # Panics
    ///
    /// This function will panic if the source range or destination index is
    /// out of bounds.
    #[inline]
    pub fn mem_move<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.buf.copy_within(src, dest);
    }

    /// Returns the total capacity of the buffer.
    #[inline]
    #[must_use]
    pub const fn len(&mut self) -> usize {
        self.buf.len()
    }

    /// Checks if the buffer's capacity is zero.
    #[inline]
    #[must_use]
    pub const fn is_empty(&mut self) -> bool {
        self.buf.is_empty()
    }

    /// Checks if the buffer's write position is at the end of its capacity.
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    /// Sets the write position to a new index without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `index` is not greater than the buffer's
    /// length. Behavior is undefined if the specified index is out of bounds.
    #[inline]
    pub const unsafe fn unchecked_set_pos(&mut self, index: usize) {
        debug_assert!(
            index <= self.buf.len(),
            "attempted to advance past end of buffer",
        );
        self.write_idx = index;
    }

    /// Consumes the buffer and returns the underlying `Vec<u8>`, truncated to the write position.
    #[must_use]
    pub fn to_vec(mut self) -> Vec<u8> {
        self.buf.truncate(self.write_idx);
        self.buf
    }

    /// Consumes the buffer and returns an [`ArcSlice`] over the written bytes.
    pub fn freeze(mut self) -> ArcSlice {
        self.buf.truncate(self.write_idx);
        ArcSlice::new(Arc::from(self.buf))
    }
}

impl<E: Endian> AsRef<[u8]> for Buffer<E> {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

impl<'a, E: Endian> From<&'a Buffer<E>> for Cursor<'a> {
    fn from(value: &'a Buffer<E>) -> Self {
        Self::new(&value.buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::buf::endian::{Big, Little, Native};
    use crate::buf::{ArcSlice, Buffer, Cursor};

    #[test]
    fn buf_little_endian() {
        let mut buf = Buffer::<Little>::with_capacity(32);
        buf.write_u16(576);
        assert_eq!(buf.as_ref(), &[64, 2]);

        let mut buf = Buffer::<Little>::with_capacity(32);
        buf.write_i16(576);
        assert_eq!(buf.as_ref(), &[64, 2]);

        let mut buf = Buffer::<Little>::with_capacity(32);
        buf.write_u32(901_829_528);
        assert_eq!(buf.as_ref(), &[152, 211, 192, 53]);

        let mut buf = Buffer::<Little>::with_capacity(32);
        buf.write_i32(901_829_528);
        assert_eq!(buf.as_ref(), &[152, 211, 192, 53]);

        let mut buf = Buffer::<Little>::with_capacity(32);
        buf.write_u64(9_223_372_936_854_775_948);
        assert_eq!(buf.as_ref(), &[140, 40, 46, 140, 209, 0, 0, 128]);

        let mut buf = Buffer::<Little>::with_capacity(32);
        buf.write_i64(i64::MIN);
        assert_eq!(buf.as_ref(), &i64::MIN.to_le_bytes());
    }

    #[test]
    fn buf_big_endian() {
        let mut buf = Buffer::<Big>::with_capacity(32);
        buf.write_u16(576);
        assert_eq!(buf.to_vec().as_slice(), &[2, 64]);

        let mut buf = Buffer::<Big>::with_capacity(32);
        buf.write_u32(901_829_528);
        assert_eq!(buf.as_ref(), &[53, 192, 211, 152]);

        let mut buf = Buffer::<Big>::with_capacity(32);
        buf.write_u64(9_223_372_936_854_775_948);
        assert_eq!(buf.as_ref(), &[128, 0, 0, 209, 140, 46, 40, 140]);
    }

    #[test]
    fn buf_advance() {
        let mut buf = Buffer::<Native>::new();
        assert_eq!(buf.pos(), 0);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert!(buf.is_full());

        buf.advance(1);
        assert_eq!(buf.pos(), 1);
        assert_eq!(buf.len(), 1);

        buf.advance(2);
        assert_eq!(buf.pos(), 3);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn buf_invalid_set_pos() {
        let mut buf = Buffer::<Native>::new();
        buf.set_pos(1);
    }

    #[test]
    fn buf_set_pos() {
        let mut buf = Buffer::<Native>::new();
        buf.reserve_n(1);
        buf.set_pos(1);
        assert_eq!(buf.remaining(), 0);
        buf.reserve_n(1);
        assert_eq!(buf.remaining(), 1);
        buf.set_pos(2);
        assert_eq!(buf.remaining(), 0);
    }

    #[test]
    fn buf_mem_move() {
        let mut buf = Buffer::<Native>::new();
        buf.write_u64(u64::MAX);
        buf.write_u64(0);
        assert_eq!(buf.len(), 2 * size_of::<u64>());

        buf.mem_move(0..8, 8);
        assert_eq!(buf.len(), 2 * size_of::<u64>());

        let mut cursor = Cursor::from(&buf);
        assert_eq!(cursor.read_u64::<Native>().unwrap(), u64::MAX);
        assert_eq!(cursor.read_u64::<Native>().unwrap(), u64::MAX);
        assert!(cursor.is_empty());
    }

    #[test]
    fn buf_freeze() {
        let mut buf = Buffer::<Native>::new();
        buf.write_u8(1);
        buf.write_u8(2);
        buf.write_u8(3);

        let frozen: ArcSlice = buf.freeze();
        assert_eq!(frozen.as_ref(), &[1, 2, 3]);
        assert_eq!(frozen.len(), 3);

        let sliced = frozen.slice(1..3);
        assert_eq!(sliced.as_ref(), &[2, 3]);
    }
}
