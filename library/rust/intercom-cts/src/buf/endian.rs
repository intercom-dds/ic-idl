// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

#![allow(clippy::inline_always)]

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

impl Little {
    #[must_use]
    pub fn is_le(&self) -> bool {
        true
    }

    #[must_use]
    pub fn is_be(&self) -> bool {
        false
    }
}

impl Big {
    #[must_use]
    pub fn is_le(&self) -> bool {
        false
    }

    #[must_use]
    pub fn is_be(&self) -> bool {
        true
    }
}

macro_rules! read_order {
    ($slice:expr, $type:ty, $order:ident) => {{
        const SIZE: usize = std::mem::size_of::<$type>();
        let mut bytes = [0; SIZE];
        bytes.copy_from_slice(&$slice[0..SIZE]);
        <$type>::$order(bytes)
    }};
}

pub trait Endian: 'static {
    #[inline(always)]
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

    #[inline(always)]
    fn write_u8(value: u8, buf: &mut [u8]) {
        buf[0] = value;
    }

    fn write_u16(value: u16, buf: &mut [u8]);

    fn write_u32(value: u32, buf: &mut [u8]);

    fn write_u64(value: u64, buf: &mut [u8]);

    /// Reads a u8 from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 1 byte
    /// - `ptr` is properly aligned (though u8 has no alignment requirements)
    /// - The memory at `ptr` is initialized
    #[inline(always)]
    #[must_use]
    unsafe fn read_u8_raw(ptr: *const u8) -> u8 {
        // SAFETY: Caller guarantees ptr is valid and readable
        unsafe { *ptr }
    }

    /// Reads a u16 from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 2 bytes
    /// - The memory at `ptr` through `ptr.add(1)` is initialized
    #[must_use]
    unsafe fn read_u16_raw(ptr: *const u8) -> u16;

    /// Reads a u32 from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 4 bytes
    /// - The memory at `ptr` through `ptr.add(3)` is initialized
    #[must_use]
    unsafe fn read_u32_raw(ptr: *const u8) -> u32;

    /// Reads a u64 from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 8 bytes
    /// - The memory at `ptr` through `ptr.add(7)` is initialized
    #[must_use]
    unsafe fn read_u64_raw(ptr: *const u8) -> u64;

    /// Reads two u16 values from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 4 bytes
    /// - The memory at `ptr` through `ptr.add(3)` is initialized
    #[inline(always)]
    #[must_use]
    unsafe fn read_u16x2_raw(ptr: *const u8) -> [u16; 2] {
        let val = unsafe { ptr.cast::<u32>().read_unaligned() };
        Self::u32_to_u16x2(val)
    }

    /// Reads four u16 values from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 8 bytes
    /// - The memory at `ptr` through `ptr.add(7)` is initialized
    #[inline(always)]
    #[must_use]
    unsafe fn read_u16x4_raw(ptr: *const u8) -> [u16; 4] {
        let val = unsafe { ptr.cast::<u64>().read_unaligned() };
        Self::u64_to_u16x4(val)
    }

    /// Reads two u32 values from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 8 bytes
    /// - The memory at `ptr` through `ptr.add(7)` is initialized
    #[inline(always)]
    #[must_use]
    unsafe fn read_u32x2_raw(ptr: *const u8) -> [u32; 2] {
        let val = unsafe { ptr.cast::<u64>().read_unaligned() };
        Self::u64_to_u32x2(val)
    }

    /// Reads four u32 values from a raw pointer in the correct endianness.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` is valid for reads of 16 bytes
    /// - The memory at `ptr` through `ptr.add(15)` is initialized
    #[inline(always)]
    #[must_use]
    unsafe fn read_u32x4_raw(ptr: *const u8) -> [u32; 4] {
        let val = unsafe { ptr.cast::<u128>().read_unaligned() };
        Self::u128_to_u32x4(val)
    }

    #[must_use]
    fn u32_to_u16x2(val: u32) -> [u16; 2];

    #[must_use]
    fn u64_to_u16x4(val: u64) -> [u16; 4];

    #[must_use]
    fn u64_to_u32x2(val: u64) -> [u32; 2];

    #[must_use]
    fn u128_to_u32x4(val: u128) -> [u32; 4];

    #[must_use]
    fn u16x2_to_u32(arr: [u16; 2]) -> u32;

    #[must_use]
    fn u16x4_to_u64(arr: [u16; 4]) -> u64;

    #[must_use]
    fn u32x2_to_u64(arr: [u32; 2]) -> u64;

    #[must_use]
    fn u32x4_to_u128(arr: [u32; 4]) -> u128;
}

impl Endian for Big {
    #[inline(always)]
    fn read_u16(slice: &[u8]) -> u16 {
        read_order!(slice, u16, from_be_bytes)
    }

    #[inline(always)]
    fn read_u32(slice: &[u8]) -> u32 {
        read_order!(slice, u32, from_be_bytes)
    }

    #[inline(always)]
    fn read_u64(slice: &[u8]) -> u64 {
        read_order!(slice, u64, from_be_bytes)
    }

    #[inline(always)]
    fn write_u16(value: u16, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    fn write_u32(value: u32, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    fn write_u64(value: u64, buf: &mut [u8]) {
        buf.copy_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    unsafe fn read_u16_raw(ptr: *const u8) -> u16 {
        unsafe { ptr.cast::<u16>().read_unaligned().to_be() }
    }

    #[inline(always)]
    unsafe fn read_u32_raw(ptr: *const u8) -> u32 {
        unsafe { ptr.cast::<u32>().read_unaligned().to_be() }
    }

    #[inline(always)]
    unsafe fn read_u64_raw(ptr: *const u8) -> u64 {
        unsafe { ptr.cast::<u64>().read_unaligned().to_be() }
    }

    #[inline(always)]
    fn u32_to_u16x2(val: u32) -> [u16; 2] {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from u32 to [u16; 2] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "little")]
        [(val >> 16) as u16, (val & 0xFFFF) as u16]
    }

    #[inline(always)]
    fn u64_to_u16x4(val: u64) -> [u16; 4] {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from u64 to [u16; 4] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "little")]
        [
            (val >> 48) as u16,
            ((val >> 32) & 0xFFFF) as u16,
            ((val >> 16) & 0xFFFF) as u16,
            (val & 0xFFFF) as u16,
        ]
    }

    #[inline(always)]
    fn u64_to_u32x2(val: u64) -> [u32; 2] {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from u64 to [u32; 2] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "little")]
        [(val >> 32) as u32, (val & 0xFFFF_FFFF) as u32]
    }

    #[inline(always)]
    fn u128_to_u32x4(val: u128) -> [u32; 4] {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from u128 to [u32; 4] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "little")]
        [
            (val >> 96) as u32,
            ((val >> 64) & 0xFFFF_FFFF) as u32,
            ((val >> 32) & 0xFFFF_FFFF) as u32,
            (val & 0xFFFF_FFFF) as u32,
        ]
    }

    #[inline(always)]
    fn u16x2_to_u32(arr: [u16; 2]) -> u32 {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from [u16; 2] to u32 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "big")]
        {
            ((arr[1] as u32) << 16) | (arr[0] as u32)
        }
    }

    #[inline(always)]
    fn u16x4_to_u64(arr: [u16; 4]) -> u64 {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from [u16; 4] to u64 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "big")]
        {
            ((arr[3] as u64) << 48)
                | ((arr[2] as u64) << 32)
                | ((arr[1] as u64) << 16)
                | (arr[0] as u64)
        }
    }

    #[inline(always)]
    fn u32x2_to_u64(arr: [u32; 2]) -> u64 {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from [u32; 2] to u64 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "big")]
        {
            ((arr[1] as u64) << 32) | (arr[0] as u64)
        }
    }

    #[inline(always)]
    fn u32x4_to_u128(arr: [u32; 4]) -> u128 {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from [u32; 4] to u128 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "little")]
        {
            (u128::from(arr[0]) << 96)
                | (u128::from(arr[1]) << 64)
                | (u128::from(arr[2]) << 32)
                | u128::from(arr[3])
        }
    }
}

impl Endian for Little {
    #[inline(always)]
    fn read_u16(slice: &[u8]) -> u16 {
        read_order!(slice, u16, from_le_bytes)
    }

    #[inline(always)]
    fn read_u32(slice: &[u8]) -> u32 {
        read_order!(slice, u32, from_le_bytes)
    }

    #[inline(always)]
    fn read_u64(slice: &[u8]) -> u64 {
        read_order!(slice, u64, from_le_bytes)
    }

    #[inline(always)]
    fn write_u16(value: u16, buf: &mut [u8]) {
        buf[..std::mem::size_of::<u16>()].copy_from_slice(&value.to_le_bytes());
    }

    #[inline(always)]
    fn write_u32(value: u32, buf: &mut [u8]) {
        buf[..std::mem::size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    }

    #[inline(always)]
    fn write_u64(value: u64, buf: &mut [u8]) {
        buf[..std::mem::size_of::<u64>()].copy_from_slice(&value.to_le_bytes());
    }

    #[inline(always)]
    unsafe fn read_u16_raw(ptr: *const u8) -> u16 {
        // SAFETY: Caller guarantees ptr is valid for 2 bytes
        unsafe { ptr.cast::<u16>().read_unaligned().to_le() }
    }

    #[inline(always)]
    unsafe fn read_u32_raw(ptr: *const u8) -> u32 {
        // SAFETY: Caller guarantees ptr is valid for 4 bytes
        unsafe { ptr.cast::<u32>().read_unaligned().to_le() }
    }

    #[inline(always)]
    unsafe fn read_u64_raw(ptr: *const u8) -> u64 {
        // SAFETY: Caller guarantees ptr is valid for 8 bytes
        unsafe { ptr.cast::<u64>().read_unaligned().to_le() }
    }

    #[inline(always)]
    fn u32_to_u16x2(val: u32) -> [u16; 2] {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from u32 to [u16; 2] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "big")]
        [(val & 0xFFFF) as u16, (val >> 16) as u16]
    }

    #[inline(always)]
    fn u64_to_u16x4(val: u64) -> [u16; 4] {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from u64 to [u16; 4] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "big")]
        [
            (val & 0xFFFF) as u16,
            ((val >> 16) & 0xFFFF) as u16,
            ((val >> 32) & 0xFFFF) as u16,
            (val >> 48) as u16,
        ]
    }

    #[inline(always)]
    fn u64_to_u32x2(val: u64) -> [u32; 2] {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from u64 to [u32; 2] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "big")]
        [(val & 0xFFFF_FFFF) as u32, (val >> 32) as u32]
    }

    #[inline(always)]
    fn u128_to_u32x4(val: u128) -> [u32; 4] {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from u128 to [u32; 4] is safe as they have the same size
            unsafe { std::mem::transmute(val) }
        }

        #[cfg(target_endian = "big")]
        [
            (val & 0xFFFF_FFFF) as u32,
            ((val >> 32) & 0xFFFF_FFFF) as u32,
            ((val >> 64) & 0xFFFF_FFFF) as u32,
            (val >> 96) as u32,
        ]
    }

    #[inline(always)]
    fn u16x2_to_u32(arr: [u16; 2]) -> u32 {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from [u16; 2] to u32 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "little")]
        {
            (u32::from(arr[0]) << 16) | u32::from(arr[1])
        }
    }

    #[inline(always)]
    fn u16x4_to_u64(arr: [u16; 4]) -> u64 {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from [u16; 4] to u64 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "little")]
        {
            (u64::from(arr[0]) << 48)
                | (u64::from(arr[1]) << 32)
                | (u64::from(arr[2]) << 16)
                | u64::from(arr[3])
        }
    }

    #[inline(always)]
    fn u32x2_to_u64(arr: [u32; 2]) -> u64 {
        #[cfg(target_endian = "big")]
        {
            // SAFETY: transmute from [u32; 2] to u64 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "little")]
        {
            (u64::from(arr[0]) << 32) | u64::from(arr[1])
        }
    }

    #[inline(always)]
    fn u32x4_to_u128(arr: [u32; 4]) -> u128 {
        #[cfg(target_endian = "little")]
        {
            // SAFETY: transmute from [u32; 4] to u128 is safe as they have the same size
            unsafe { std::mem::transmute(arr) }
        }

        #[cfg(target_endian = "big")]
        {
            ((arr[3] as u128) << 96)
                | ((arr[2] as u128) << 64)
                | ((arr[1] as u128) << 32)
                | (arr[0] as u128)
        }
    }
}
