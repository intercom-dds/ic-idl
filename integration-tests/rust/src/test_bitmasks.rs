// Copyright 2026 KONGSBERG
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

use crate::bitmask_types;

#[test]
fn bitmask_is_flag_type() {
    assert_eq!(
        intercom_cts::type_info::<bitmask_types::Permissions>().kind,
        intercom_cts::TypeKind::Bitmask
    );
}

#[test]
fn bitmask_members_exist() {
    let _read = bitmask_types::Permissions::READ;
    let _write = bitmask_types::Permissions::WRITE;
    let _execute = bitmask_types::Permissions::EXECUTE;
    let _del = bitmask_types::Permissions::DELETE;
}

#[test]
fn bitmask_auto_values() {
    assert_eq!(bitmask_types::Permissions::READ.0, 1);
    assert_eq!(bitmask_types::Permissions::WRITE.0, 2);
    assert_eq!(bitmask_types::Permissions::EXECUTE.0, 4);
    assert_eq!(bitmask_types::Permissions::DELETE.0, 8);
}

#[test]
fn bitmask_explicit_values() {
    assert_eq!(bitmask_types::ExplicitFlags::FLAG_A.0, 2);
    assert_eq!(bitmask_types::ExplicitFlags::FLAG_B.0, 4);
    assert_eq!(bitmask_types::ExplicitFlags::FLAG_C.0, 16);
    assert_eq!(bitmask_types::ExplicitFlags::FLAG_D.0, 256);
}

#[test]
fn bitmask_or_operation() {
    let combined = bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE;

    assert_eq!(combined.0, 3);
    assert!(combined.contains(bitmask_types::Permissions::READ));
    assert!(combined.contains(bitmask_types::Permissions::WRITE));
}

#[test]
fn bitmask_and_operation() {
    let combined = bitmask_types::Permissions::READ
        | bitmask_types::Permissions::WRITE
        | bitmask_types::Permissions::EXECUTE;
    let result = combined & bitmask_types::Permissions::READ;

    assert_eq!(result, bitmask_types::Permissions::READ);
}

#[test]
fn bitmask_in_struct() {
    let file_info = bitmask_types::FileInfo {
        path: "test.txt".into(),
        perms: bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE,
    };

    assert_eq!(file_info.path, "test.txt");
    assert!(file_info.perms.contains(bitmask_types::Permissions::READ));
    assert!(file_info.perms.contains(bitmask_types::Permissions::WRITE));
}

#[test]
fn bitmask_none_value() {
    let none = bitmask_types::Permissions::nil();

    assert_eq!(none.0, 0);
    assert!(!none.contains(bitmask_types::Permissions::READ));
    assert!(!none.contains(bitmask_types::Permissions::WRITE));
}

#[test]
fn bitmask_all_combined() {
    let all = bitmask_types::Permissions::READ
        | bitmask_types::Permissions::WRITE
        | bitmask_types::Permissions::EXECUTE
        | bitmask_types::Permissions::DELETE;

    assert_eq!(all.0, 15);
    assert!(all.contains(bitmask_types::Permissions::READ));
    assert!(all.contains(bitmask_types::Permissions::WRITE));
    assert!(all.contains(bitmask_types::Permissions::EXECUTE));
    assert!(all.contains(bitmask_types::Permissions::DELETE));
}

#[test]
fn bitmask_gapped_positions() {
    assert_eq!(bitmask_types::GappedFlags::LOW.0, 1);
    assert_eq!(bitmask_types::GappedFlags::HIGH.0, 128);
}

#[test]
fn bitmask_single_flag() {
    assert_eq!(bitmask_types::SingleFlag::ONLY.0, 1);
}

#[test]
fn bitmask_mixed_explicit_auto() {
    assert_eq!(bitmask_types::MixedFlags::AUTO_FIRST.0, 1);
    assert_eq!(bitmask_types::MixedFlags::EXPLICIT_FOUR.0, 16);
    assert_eq!(bitmask_types::MixedFlags::AUTO_FIVE.0, 32);
    assert_eq!(bitmask_types::MixedFlags::AUTO_SIX.0, 64);
}

#[test]
fn bitmask_xor_operation() {
    let xor_result = bitmask_types::Permissions::READ ^ bitmask_types::Permissions::WRITE;
    assert_eq!(xor_result.0, 3);

    let same = bitmask_types::Permissions::READ ^ bitmask_types::Permissions::READ;
    assert_eq!(same.0, 0);
}
