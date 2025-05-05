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

#![allow(clippy::cast_sign_loss)]

use std::marker::PhantomData;
use std::ops::RangeBounds;

use crate::buf::endian::Endian;
use crate::buf::Cursor;

#[must_use]
#[derive(Default)]
pub struct Buffer<E: Endian> {
    buf: Vec<u8>,
    write_idx: usize,
    _marker: PhantomData<E>,
}

impl<E: Endian> Buffer<E> {
    pub fn new() -> Self {
        Self::with_capacity(64)
    }

    pub fn with_capacity(len: usize) -> Self {
        Self {
            buf: Vec::with_capacity(len),
            write_idx: 0,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn write_u8(&mut self, val: u8) {
        self.reserve_n(1);
        self.buf[self.write_idx] = val;
        self.write_idx += 1;
    }

    #[inline]
    pub fn write_u16(&mut self, val: u16) {
        self.reserve_n(size_of::<u16>());
        E::write_u16(val, &mut self.buf[self.write_idx..]);
        self.write_idx += 2;
    }

    #[inline]
    pub fn write_i16(&mut self, val: i16) {
        self.write_u16(val as u16);
    }

    #[inline]
    pub fn write_u32(&mut self, val: u32) {
        self.reserve_n(size_of::<u32>());
        E::write_u32(val, &mut self.buf[self.write_idx..]);
        self.write_idx += 4;
    }

    #[inline]
    pub fn write_i32(&mut self, val: i32) {
        self.write_u32(val as u32);
    }

    #[inline]
    pub fn write_u64(&mut self, val: u64) {
        self.reserve_n(size_of::<u64>());
        E::write_u64(val, &mut self.buf[self.write_idx..]);
        self.write_idx += 8;
    }

    #[inline]
    pub fn write_i64(&mut self, val: i64) {
        self.write_u64(val as u64);
    }

    #[inline]
    pub fn extend(&mut self, slice: &[u8]) {
        let end_idx = self.write_idx + slice.len();
        if end_idx >= self.remaining() {
            self.reserve_n(slice.len());
        }

        self.buf[self.write_idx..end_idx].copy_from_slice(slice);
        self.write_idx += slice.len();
    }

    #[inline]
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.write_idx
    }

    #[inline]
    pub fn align(&mut self, align: usize) {
        let rem = self.write_idx % align;
        if rem > 0 {
            let dt = align - rem;
            self.reserve_n(dt);
            self.write_idx += dt;
        }
    }

    #[inline]
    pub fn align_to<T>(&mut self) {
        self.align(size_of::<T>());
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.buf.resize(len, 0);
    }

    #[inline]
    pub fn reserve_n(&mut self, n: usize) {
        if self.remaining() < n {
            self.resize(self.buf.len() + n);
        }
    }

    #[inline]
    pub fn really_reserve_n(&mut self, n: usize) {
        self.resize(self.buf.len() + n);
    }

    #[inline]
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.write_idx
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.reserve_n(n);
        self.write_idx += n;
    }

    /// # Panics
    ///
    /// Panics if the specified index is out of bounds.
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

    /// # Panics
    ///
    /// This function will panic if either range exceeds the end of the slice,
    /// or if the end of `src` is before the start.
    #[inline]
    pub fn mem_move<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.buf.copy_within(src, dest);
    }

    #[inline]
    #[must_use]
    pub fn len(&mut self) -> usize {
        self.buf.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&mut self) -> bool {
        self.buf.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.remaining() == 0
    }

    #[inline]
    #[must_use]
    pub fn bytes(self) -> Vec<u8> {
        self.buf
    }

    /// # Safety
    ///
    /// Behavior is undefined if the specified index exceeds the end of the
    /// buffer.
    #[inline]
    pub unsafe fn unchecked_set_pos(&mut self, index: usize) {
        self.write_idx = index;
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
    use crate::buf::{Buffer, Cursor};

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
        assert_eq!(buf.bytes().as_slice(), &[2, 64]);

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
}
