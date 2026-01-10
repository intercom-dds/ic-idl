// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

use std::fmt::{Debug, Display};

pub trait Error: Debug {
    fn custom<T>(msg: T) -> Self
    where
        T: Display;
}

/// Error produced by the [`FromStr`] implementation for enum types.
///
/// [`FromStr`]: std::str::FromStr
pub struct UnknownVariant;
