// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2025 KONGSBERG - All rights reserved

#![no_main]

use intercom_cts::cdr2::from_le_bytes;
use intercom_cts::cdr2::mutable::SeqType;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    _ = from_le_bytes::<SeqType>(data);
});
