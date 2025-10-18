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

#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]

use std::io::Read;
use std::marker::PhantomData;
use std::ops::Range;

use super::Error;
use super::endian::{Endian, Native};

type Result<T> = std::result::Result<T, Error>;

#[must_use]
#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    start: *const u8,
    end: *const u8,
    read: *const u8,
    marker: PhantomData<&'a ()>,
}

impl<'a> Cursor<'a> {
    #[inline]
    pub const fn new(input: &'a [u8]) -> Self {
        let start = input.as_ptr();
        let end = unsafe { start.add(input.len()) };
        Self {
            start,
            end,
            read: start,
            marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub const fn pos(&self) -> usize {
        unsafe { self.read.byte_offset_from(self.start) as usize }
    }

    /// # Safety
    ///
    /// Caller must ensure the position is within bounds of the buffer.
    #[inline]
    pub const unsafe fn set_pos(&mut self, pos: usize) {
        self.read = unsafe { self.start.add(pos) };

        #[cfg(debug_assertions)]
        self.check_bounds();
    }

    /// Total length of the underlying buffer, starting from the very beginning
    /// to the end.
    #[inline]
    #[must_use]
    pub const fn total_len(&self) -> usize {
        unsafe { self.end.offset_from(self.start) as usize }
    }

    #[inline]
    #[must_use]
    pub const fn unread_bytes(&self) -> usize {
        unsafe { self.end.offset_from(self.read) as usize }
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.unread_bytes() == 0
    }

    /// # Safety
    ///
    /// Caller must ensure the position is within bounds of the buffer.
    #[inline]
    pub const unsafe fn advance(&mut self, n: usize) {
        unsafe {
            self.set_pos(self.pos() + n);
        }
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        self.read_advance(Native::read_u8)
    }

    #[inline]
    pub fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().map(|v| v as i8)
    }

    #[inline]
    pub fn read_u16<E: Endian>(&mut self) -> Result<u16> {
        self.read_advance(E::read_u16)
    }

    #[inline]
    pub fn read_i16<E: Endian>(&mut self) -> Result<i16> {
        self.read_u16::<E>().map(|v| v as i16)
    }

    #[inline]
    pub fn read_u32<E: Endian>(&mut self) -> Result<u32> {
        self.read_advance(E::read_u32)
    }

    #[inline]
    pub fn read_i32<E: Endian>(&mut self) -> Result<i32> {
        self.read_u32::<E>().map(|v| v as i32)
    }

    #[inline]
    pub fn read_u64<E: Endian>(&mut self) -> Result<u64> {
        self.read_advance(E::read_u64)
    }

    #[inline]
    pub fn read_i64<E: Endian>(&mut self) -> Result<i64> {
        self.read_u64::<E>().map(|v| {
            // Safe reinterpretation of u64 bits as i64 for decoding
            i64::from_ne_bytes(v.to_ne_bytes())
        })
    }

    #[inline]
    pub const fn align(&mut self, align: usize) {
        debug_assert!(
            align != 0 && align.is_power_of_two(),
            "alignment must be a power of 2",
        );

        let dt = (align - (self.pos() & (align - 1))) & (align - 1);
        if dt >= self.unread_bytes() {
            self.read = self.end;
        } else if dt > 0 {
            // SAFETY: bounds checked
            unsafe {
                self.advance(dt);
            }
        }
    }

    #[inline]
    pub const fn align_to<T>(&mut self) {
        self.align(size_of::<T>());
    }

    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get(&self, index: Range<usize>) -> Result<&'a [u8]> {
        assert!(index.end >= index.start);

        let dt = index.end - index.start;
        if self.unread_bytes() >= dt {
            // SAFETY: bounds checked
            Ok(unsafe { self.slice(dt) })
        } else {
            Err(Error::InvalidLen)
        }
    }

    /// Creates a slice from the cursor's current position with the specified
    /// length.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the specified length exceeds the end of the
    /// buffer.
    #[inline]
    #[must_use]
    pub const unsafe fn slice(&self, len: usize) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.read, len) }
    }

    #[inline]
    pub const fn reset(&mut self) {
        self.read = self.start;
    }

    #[inline]
    #[must_use]
    pub const fn start_ptr(&self) -> *const u8 {
        self.start
    }

    #[inline]
    #[must_use]
    pub const fn end_ptr(&self) -> *const u8 {
        self.end
    }

    #[inline]
    #[must_use]
    pub const fn read_ptr(&self) -> *const u8 {
        self.read
    }

    #[inline]
    fn read_advance<F, T>(&mut self, f: F) -> Result<T>
    where
        F: Fn(&[u8]) -> T,
    {
        if self.unread_bytes() >= size_of::<T>() {
            let val = f(self.as_ref());
            // SAFETY: bounds checked
            unsafe {
                self.advance(size_of::<T>());
            }
            Ok(val)
        } else {
            Err(Error::InvalidLen)
        }
    }

    #[inline]
    #[track_caller]
    const fn check_bounds(&self) {
        assert!(
            unsafe { self.read.byte_offset_from(self.end) } <= 0,
            "attempted to advance past end of buffer",
        );
        assert!(
            unsafe { self.read.byte_offset_from(self.start) } >= 0,
            "attempted to move before the beginning of buffer",
        );
    }
}

impl<'a> AsRef<[u8]> for Cursor<'a> {
    #[inline]
    fn as_ref(&self) -> &'a [u8] {
        let len = self.unread_bytes();
        unsafe { std::slice::from_raw_parts::<'a, _>(self.read, len) }
    }
}

impl Iterator for Cursor<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let byte = self.read.read();
                self.advance(1);
                Some(byte)
            }
        }
    }
}

impl Read for Cursor<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.unread_bytes().min(buf.len());
        unsafe {
            std::ptr::copy_nonoverlapping(self.read_ptr(), buf.as_mut_ptr(), len);
            self.advance(len);
        }
        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::buf::Cursor;
    use crate::buf::endian::Little;

    #[test]
    fn cursor_iter() {
        let mut buf = Cursor::new(&[0, 1, 2]);
        assert_eq!(buf.next(), Some(0));
        assert_eq!(buf.next(), Some(1));
        assert_eq!(buf.next(), Some(2));
        assert_eq!(buf.next(), None);

        buf.reset();
        assert_eq!(buf.next(), Some(0));
    }

    #[test]
    fn cursor_len() {
        let mut buf = Cursor::new(&[0, 1]);
        assert_eq!(buf.total_len(), 2);
        assert_eq!(buf.unread_bytes(), 2);
        buf.next();
        assert_eq!(buf.total_len(), 2);
        assert_eq!(buf.unread_bytes(), 1);
        buf.next();
        assert_eq!(buf.total_len(), 2);
        assert_eq!(buf.unread_bytes(), 0);
    }

    #[test]
    fn cursor_set_pos() {
        let mut buf = Cursor::new(&[0, 1, 2]);
        unsafe {
            buf.set_pos(0);
            assert_eq!(buf.unread_bytes(), 3);

            buf.set_pos(1);
            assert_eq!(buf.unread_bytes(), 2);

            buf.set_pos(2);
            assert_eq!(buf.unread_bytes(), 1);

            buf.set_pos(3);
            assert_eq!(buf.unread_bytes(), 0);
        }
    }

    #[test]
    #[should_panic(expected = "end of buffer")]
    #[cfg(debug_assertions)]
    fn cursor_advance_past_end() {
        let mut buf = Cursor::new(&[0]);
        unsafe { buf.advance(2) };
    }

    #[test]
    #[should_panic(expected = "end of buffer")]
    #[cfg(debug_assertions)]
    fn cursor_set_pos_oob() {
        let mut buf = Cursor::new(&[]);
        unsafe { buf.set_pos(1) };
    }

    #[test]
    fn cursor_as_ref() {
        let slice = &[0, 1, 2];
        let mut buf = Cursor::new(slice);
        unsafe {
            assert_eq!(buf.as_ref(), &slice[0..]);
            buf.advance(1);

            assert_eq!(buf.as_ref(), &slice[1..]);
            buf.advance(1);

            assert_eq!(buf.as_ref(), &slice[2..]);
            buf.advance(1);

            assert_eq!(buf.as_ref(), &slice[3..]);
        }
    }

    #[test]
    fn cursor_align() {
        let mut buf = Cursor::new(&[0, 1, 2, 3, 4]);
        assert_eq!(buf.pos(), 0);

        buf.align(1);
        assert_eq!(buf.pos(), 0);

        buf.next();
        buf.align(1);
        assert_eq!(buf.pos(), 1);

        buf.align(2);
        assert_eq!(buf.pos(), 2);

        buf.align(4);
        assert_eq!(buf.pos(), 4);

        // Saturated to buffer's len
        buf.align(8);
        assert_eq!(buf.pos(), 5);
    }

    #[test]
    fn cursor_raw_ptr() {
        let slice = &[0];
        let mut buf = Cursor::new(slice);
        assert_eq!(slice.as_ptr(), buf.start_ptr());
        assert_eq!(slice.as_ptr(), buf.read_ptr());
        buf.next();
        assert_ne!(buf.read_ptr(), buf.start_ptr());
        assert_eq!(buf.read_ptr(), buf.end_ptr());

        buf.reset();
        assert_eq!(buf.read_ptr(), buf.start_ptr());
    }

    #[test]
    fn cursor_get() {
        let slice = &[3];
        let buf = Cursor::new(slice);
        assert_eq!(buf.get(0..1).unwrap(), slice);
        assert!(buf.get(0..2).is_err());
    }

    #[test]
    fn error_fmt() {
        let slice = &[3];
        let mut buf = Cursor::new(slice);
        let res = buf.read_u16::<Little>().err().unwrap();
        assert_eq!(res.to_string(), "invalid length");
    }

    #[test]
    fn cursor_read() {
        {
            let value = u8::MAX.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_u8().unwrap(), u8::MAX);
            assert_eq!(buf.pos(), 1);
            assert!(buf.read_u8().is_err());
        }
        {
            let value = i8::MIN.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_i8().unwrap(), i8::MIN);
            assert_eq!(buf.pos(), 1);
            assert!(buf.read_i8().is_err());
        }
        {
            let value = u16::MAX.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_u16::<Little>().unwrap(), u16::MAX);
            assert_eq!(buf.pos(), 2);
            assert!(buf.read_u16::<Little>().is_err());
        }
        {
            let value = i16::MIN.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_i16::<Little>().unwrap(), i16::MIN);
            assert_eq!(buf.pos(), 2);
            assert!(buf.read_i16::<Little>().is_err());
        }
        {
            let value = u32::MAX.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_u32::<Little>().unwrap(), u32::MAX);
            assert_eq!(buf.pos(), 4);
            assert!(buf.read_u32::<Little>().is_err());
        }
        {
            let value = i32::MIN.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_i32::<Little>().unwrap(), i32::MIN);
            assert_eq!(buf.pos(), 4);
            assert!(buf.read_i32::<Little>().is_err());
        }
        {
            let value = u64::MAX.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_u64::<Little>().unwrap(), u64::MAX);
            assert_eq!(buf.pos(), 8);
            assert!(buf.read_u64::<Little>().is_err());
        }
        {
            let value = i64::MIN.to_le_bytes();
            let mut buf = Cursor::new(&value);
            assert_eq!(buf.pos(), 0);
            assert_eq!(buf.read_i64::<Little>().unwrap(), i64::MIN);
            assert_eq!(buf.pos(), 8);
            assert!(buf.read_i64::<Little>().is_err());
        }
    }

    #[test]
    fn cursor_io_read() {
        let value = u32::MAX.to_le_bytes();
        let mut buf = Cursor::new(&value);

        let mut first = [0u8; 1];
        buf.read_exact(&mut first).unwrap();
        assert_eq!(buf.pos(), 1);

        let mut too_many = [0u8; 16];
        let len = buf.read(&mut too_many).unwrap();
        assert_eq!(len, 3);

        let mut oob = [0u8; 1];
        assert!(buf.read_exact(&mut oob).is_err());
    }
}
