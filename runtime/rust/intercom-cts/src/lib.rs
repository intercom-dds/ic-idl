// Copyright 2023 KONGSBERG
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
#[cfg(feature = "derive")]
pub use intercom_derive::Marshal;
pub use type_info::{
    DISC_INFO, MemberInfo, TypeDescriptor, TypeInfo, TypeKind, member_info, type_info,
};

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
