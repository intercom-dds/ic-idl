// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2024 KONGSBERG - All rights reserved

//! XCDR2 serialization module with context-aware type handling.

mod de;
mod ser;

pub use de::{from_be_bytes, from_bytes, from_bytes_mut, from_le_bytes};
pub use ser::{to_be_bytes, to_buffer, to_bytes, to_le_bytes};
