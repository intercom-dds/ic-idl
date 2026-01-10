// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

//! Serialization framework for InterCOM DDS, built around X-Types type system.

mod bitmask;
mod bound;
pub mod buf;
pub mod cdr;
pub mod cdr1;
pub mod cdr2;
pub mod decode;
pub mod encode;
pub mod error;
pub mod infallible;
pub mod json;
pub mod key;
pub mod type_info;

pub use cdr1::{MemberFlag, TypeFlag};
pub use decode::Unmarshal;
pub use encode::Marshal;
pub use type_info::{DISC_INFO, MemberInfo, TypeDescriptor, TypeInfo, TypeKind, type_info};

/// Wrapper for handling UTF16 characters.
#[doc(hidden)]
pub struct WChar<T>(pub T);

/// Wrapper for handling UTF16-encoded strings.
#[doc(hidden)]
pub struct WString<T>(pub T);

/// Applies a bound to the given collection.
#[doc(hidden)]
pub const fn bound<T, const N: usize>(value: T) -> bound::Bound<T, N> {
    bound::Bound(value)
}

/// Applies a `min` bound to the given type.
#[doc(hidden)]
pub const fn min<T, N>(value: T, bound: N) -> bound::Min<T, N> {
    bound::Min(value, bound)
}

/// Applies a `max` bound to the given type.
#[doc(hidden)]
pub const fn max<T, N>(value: T, bound: N) -> bound::Max<T, N> {
    bound::Max(value, bound)
}

/// Applies a `range` bound to the given type.
#[doc(hidden)]
pub const fn range<T, N, M>(value: T, min: N, max: M) -> bound::Range<T, N, M> {
    bound::Range { value, min, max }
}
