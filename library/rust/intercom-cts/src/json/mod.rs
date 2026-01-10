// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

mod de;
mod error;
mod key;
mod parse;
mod ser;
mod value;

pub use de::{from_str, from_string_mut, from_value, from_value_mut};
pub use error::Error;
pub use ser::{to_bytes, to_string};
#[doc(inline)]
pub use value::value;
pub use value::{Array, Number, Object, Value, to_value};

/// Alias for a `Result` with the error type [`json::Error`].
///
/// [`json::Error`]: Error
pub type Result<T> = std::result::Result<T, Error>;
