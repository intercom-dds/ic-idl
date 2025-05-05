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

/// Marker for endianness.
#[derive(Debug)]
pub enum Big {}

/// Marker for endianness.
#[derive(Debug)]
pub enum Little {}

/// Marker for the system's native endianness.
#[cfg(target_endian = "little")]
pub type Native = Little;

/// Marker for the system's native endianness.
#[cfg(target_endian = "big")]
pub type Native = Big;

macro_rules! read_order {
    ($slice:expr, $type:ty, $order:ident) => {{
        const SIZE: usize = std::mem::size_of::<$type>();
        let mut bytes = [0; SIZE];
        bytes.copy_from_slice(&$slice[0..SIZE]);
        <$type>::$order(bytes)
    }};
}

pub trait Endian: 'static {
    #[inline]
    #[must_use]
    fn read_u8(slice: &[u8]) -> u8 {
        slice[0]
    }

    #[must_use]
    fn read_u16(slice: &[u8]) -> u16;

    #[must_use]
    fn read_u32(slice: &[u8]) -> u32;

    #[must_use]
    fn read_u64(slice: &[u8]) -> u64;

    #[inline]
    fn write_u8(value: u8, buf: &mut [u8]) {
        buf[0] = value;
    }

    fn write_u16(value: u16, buf: &mut [u8]);

    fn write_u32(value: u32, buf: &mut [u8]);

    fn write_u64(value: u64, buf: &mut [u8]);
}

impl Endian for Big {
    #[inline]
    fn read_u16(slice: &[u8]) -> u16 {
        read_order!(slice, u16, from_be_bytes)
    }

    #[inline]
    fn read_u32(slice: &[u8]) -> u32 {
        read_order!(slice, u32, from_be_bytes)
    }

    #[inline]
    fn read_u64(slice: &[u8]) -> u64 {
        read_order!(slice, u64, from_be_bytes)
    }

    #[inline]
    fn write_u16(value: u16, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes()[..2]);
    }

    #[inline]
    fn write_u32(value: u32, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes()[..4]);
    }

    #[inline]
    fn write_u64(value: u64, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes()[..8]);
    }
}

impl Endian for Little {
    #[inline]
    fn read_u16(slice: &[u8]) -> u16 {
        read_order!(slice, u16, from_le_bytes)
    }

    #[inline]
    fn read_u32(slice: &[u8]) -> u32 {
        read_order!(slice, u32, from_le_bytes)
    }

    #[inline]
    fn read_u64(slice: &[u8]) -> u64 {
        read_order!(slice, u64, from_le_bytes)
    }

    #[inline]
    fn write_u16(value: u16, buf: &mut [u8]) {
        buf[..2].copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn write_u32(value: u32, buf: &mut [u8]) {
        buf[..4].copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    fn write_u64(value: u64, buf: &mut [u8]) {
        buf[..8].copy_from_slice(&value.to_le_bytes());
    }
}
