// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2025 KONGSBERG - All rights reserved

use std::io::Read;
use std::marker::PhantomData;
use std::ops::Range;
use std::ptr::NonNull;

use super::Error;
use super::endian::{Endian, Native};

type Result<T> = std::result::Result<T, Error>;

/// A read-oriented, zero-copy cursor over a byte slice.
///
/// `Cursor` provides a safe and efficient way to parse binary data from an
/// underlying byte slice without allocating new memory. It maintains an
/// internal read pointer that advances as data is read.
///
/// By using `NonNull<u8>` internally instead of raw pointers, it leverages the
/// niche-optimization capabilities of the compiler. The lifetime `'a` ensures
/// that the `Cursor` cannot outlive the slice it was created from.
#[must_use]
#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    start: NonNull<u8>,
    end: NonNull<u8>,
    read: NonNull<u8>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Cursor<'a> {
    /// Creates a new `Cursor` that wraps the given byte slice.
    ///
    /// The cursor's initial read position is at the start of the slice.
    #[inline]
    pub const fn new(input: &'a [u8]) -> Self {
        // SAFETY: The input slice's pointer is guaranteed to be non-null.
        let start = unsafe { NonNull::new_unchecked(input.as_ptr().cast_mut()) };
        // SAFETY: The offset is within the bounds of the original slice.
        let end = unsafe { start.add(input.len()) };

        Self {
            start,
            end,
            read: start,
            marker: PhantomData,
        }
    }

    /// Returns the current read position as an offset from the beginning of
    /// the slice.
    #[inline]
    #[must_use]
    pub const fn pos(&self) -> usize {
        // SAFETY: `read` and `start` are guaranteed to be valid and part of
        // the same allocation.
        unsafe { self.read.as_ptr().offset_from(self.start.as_ptr()) as usize }
    }

    /// Sets the current read position to a new offset from the beginning of
    /// the slice.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidLen` if the requested position is out of bounds.
    #[inline]
    pub const fn set_pos(&mut self, pos: usize) -> Result<()> {
        if pos <= self.total_len() {
            // SAFETY: bounds checked
            unsafe {
                self.set_pos_unchecked(pos);
            }
            Ok(())
        } else {
            Err(Error::InvalidLen)
        }
    }

    /// Sets the read position without performing bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `pos` is less than or equal to the total
    /// length of the underlying buffer.
    #[inline]
    #[track_caller]
    pub const unsafe fn set_pos_unchecked(&mut self, pos: usize) {
        debug_assert!(
            pos <= self.total_len(),
            "attempted to advance past end of buffer",
        );
        self.read = unsafe { self.start.add(pos) };
    }

    /// Returns the total length of the underlying buffer.
    #[inline]
    #[must_use]
    pub const fn total_len(&self) -> usize {
        // SAFETY: `end` and `start` are guaranteed to be valid and part of the
        // same allocation.
        unsafe { self.end.as_ptr().offset_from(self.start.as_ptr()) as usize }
    }

    /// Returns the number of bytes that have not yet been read.
    #[inline]
    #[must_use]
    pub const fn unread_bytes(&self) -> usize {
        // SAFETY: `end` and `read` are guaranteed to be valid and part of the
        // same allocation.
        unsafe { self.end.as_ptr().offset_from(self.read.as_ptr()) as usize }
    }

    /// Checks if all bytes in the cursor have been read.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.unread_bytes() == 0
    }

    /// Advances the read position by `n` bytes from the current position.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidLen` if `n` would advance the cursor out of
    /// bounds.
    #[inline]
    pub const fn advance(&mut self, n: usize) -> Result<()> {
        self.set_pos(self.pos() + n)
    }

    /// Advances the read position by `n` bytes without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that advancing by `n` bytes will not move the
    /// read position beyond the end of the buffer.
    #[inline]
    pub const unsafe fn advance_unchecked(&mut self, n: usize) {
        unsafe {
            self.set_pos_unchecked(self.pos() + n);
        }
    }

    /// Reads a `u8` from the current position and advances the cursor.
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        self.read_advance(Native::read_u8_raw)
    }

    /// Reads an `i8` from the current position and advances the cursor.
    #[inline]
    pub fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().map(|v| v as i8)
    }

    /// Reads a `u16` from the current position using the specified endianness
    /// and advances the cursor.
    #[inline]
    pub fn read_u16<E: Endian>(&mut self) -> Result<u16> {
        self.read_advance(E::read_u16_raw)
    }

    /// Reads an `i16` from the current position using the specified endianness
    /// and advances the cursor.
    #[inline]
    pub fn read_i16<E: Endian>(&mut self) -> Result<i16> {
        self.read_u16::<E>().map(|v| v as i16)
    }

    /// Reads a `u32` from the current position using the specified endianness
    /// and advances the cursor.
    #[inline]
    pub fn read_u32<E: Endian>(&mut self) -> Result<u32> {
        self.read_advance(E::read_u32_raw)
    }

    /// Reads an `i32` from the current position using the specified endianness
    /// and advances the cursor.
    #[inline]
    pub fn read_i32<E: Endian>(&mut self) -> Result<i32> {
        self.read_u32::<E>().map(|v| v as i32)
    }

    /// Reads a `u64` from the current position using the specified endianness
    /// and advances the cursor.
    #[inline]
    pub fn read_u64<E: Endian>(&mut self) -> Result<u64> {
        self.read_advance(E::read_u64_raw)
    }

    /// Reads an `i64` from the current position using the specified endianness
    /// and advances the cursor.
    #[inline]
    pub fn read_i64<E: Endian>(&mut self) -> Result<i64> {
        self.read_u64::<E>().map(|v| v as i64)
    }

    /// Advances the read position to ensure it meets the specified alignment.
    ///
    /// # Panics
    ///
    /// This function will panic in debug builds if `align` is not a power of two.
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
                self.advance_unchecked(dt);
            }
        }
    }

    /// Advances the read position to align it to the memory alignment of type `T`.
    #[inline]
    pub const fn align_to<T>(&mut self) {
        self.align(size_of::<T>());
    }

    /// Returns a slice of the underlying data within the specified range.
    ///
    /// # Panics
    ///
    /// Panics if the start of the range is greater than the end.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidLen` if the requested range is out of bounds.
    pub fn get(&self, index: Range<usize>) -> Result<&'a [u8]> {
        assert!(index.end >= index.start);

        let dt = index.end - index.start;
        if index.end <= self.total_len() {
            // SAFETY: bounds checked
            Ok(unsafe { std::slice::from_raw_parts(self.start.as_ptr().add(index.start), dt) })
        } else {
            Err(Error::InvalidLen)
        }
    }

    /// Creates a slice from the cursor's current position with the specified length.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if `len` is greater than the number of unread bytes.
    #[inline]
    #[must_use]
    pub const unsafe fn slice(&self, len: usize) -> &'a [u8] {
        debug_assert!(len <= self.unread_bytes());
        unsafe { std::slice::from_raw_parts(self.read.as_ptr(), len) }
    }

    /// Resets the read position to the beginning of the buffer.
    #[inline]
    pub const fn reset(&mut self) {
        self.read = self.start;
    }

    /// Returns a raw pointer to the start of the underlying buffer.
    #[inline]
    #[must_use]
    pub const fn start_ptr(&self) -> *const u8 {
        self.start.as_ptr()
    }

    /// Returns a raw pointer to the end of the underlying buffer.
    #[inline]
    #[must_use]
    pub const fn end_ptr(&self) -> *const u8 {
        self.end.as_ptr()
    }

    /// Returns a raw pointer to the current read position.
    #[inline]
    #[must_use]
    pub const fn read_ptr(&self) -> *const u8 {
        self.read.as_ptr()
    }

    /// Generic helper to read a value of type `T` and advance the cursor.
    #[inline]
    fn read_advance<T>(&mut self, f: unsafe fn(*const u8) -> T) -> Result<T> {
        if self.unread_bytes() >= size_of::<T>() {
            // SAFETY: The bounds check above guarantees that advancing is safe.
            unsafe {
                let val = f(self.read_ptr());
                self.advance_unchecked(size_of::<T>());
                Ok(val)
            }
        } else {
            Err(Error::InvalidLen)
        }
    }
}

impl AsRef<[u8]> for Cursor<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        // SAFETY: The slice is constructed from the read pointer to the end
        // pointer, which are guaranteed to be valid, aligned, and part of the
        // original slice.
        unsafe { std::slice::from_raw_parts(self.read.as_ptr(), self.unread_bytes()) }
    }
}

impl Iterator for Cursor<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_u8().ok()
    }
}

impl Read for Cursor<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.unread_bytes().min(buf.len());

        // SAFETY: The logic below is sound for several reasons:
        // 1. `len` is calculated as the minimum of the remaining bytes and the
        //    destination buffer's length. This guarantees we will not read past
        //    the end of the source cursor or write past the end of `buf`.
        // 2. The source pointer `self.read_ptr()` is valid for `len` bytes.
        // 3. The destination pointer `buf.as_mut_ptr()` is valid for `len` bytes.
        // 4. The source and destination do not overlap. `self` holds a shared
        //    reference to its data, while `buf` is an exclusive mutable reference.
        //    Rust's borrow checker guarantees they are distinct.
        // 5. `advance_unchecked` is safe because we are only advancing by `len`,
        //    which we have already proven is a valid number of bytes to read.
        unsafe {
            std::ptr::copy_nonoverlapping(self.read_ptr(), buf.as_mut_ptr(), len);
            self.advance_unchecked(len);
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
            buf.set_pos_unchecked(0);
            assert_eq!(buf.unread_bytes(), 3);

            buf.set_pos_unchecked(1);
            assert_eq!(buf.unread_bytes(), 2);

            buf.set_pos_unchecked(2);
            assert_eq!(buf.unread_bytes(), 1);

            buf.set_pos_unchecked(3);
            assert_eq!(buf.unread_bytes(), 0);
        }
    }

    #[test]
    fn cursor_checked_advance() {
        let mut buf = Cursor::new(&[0]);
        assert!(buf.advance(1).is_ok());
        assert!(buf.advance(1).is_err());
    }

    #[test]
    #[should_panic(expected = "end of buffer")]
    #[cfg(debug_assertions)]
    fn cursor_advance_past_end() {
        let mut buf = Cursor::new(&[0]);
        unsafe { buf.advance_unchecked(2) };
    }

    #[test]
    #[should_panic(expected = "end of buffer")]
    #[cfg(debug_assertions)]
    fn cursor_set_pos_oob() {
        let mut buf = Cursor::new(&[]);
        unsafe { buf.set_pos_unchecked(1) };
    }

    #[test]
    fn cursor_as_ref() {
        let slice = &[0, 1, 2];
        let mut buf = Cursor::new(slice);
        unsafe {
            assert_eq!(buf.as_ref(), &slice[0..]);
            buf.advance_unchecked(1);

            assert_eq!(buf.as_ref(), &slice[1..]);
            buf.advance_unchecked(1);

            assert_eq!(buf.as_ref(), &slice[2..]);
            buf.advance_unchecked(1);

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
