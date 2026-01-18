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

#include <gtest/gtest.h>
#include <ic_cts/member_info.h>

#include "generated/bitmasks.h"

namespace {

TEST(BitmasksTest, test_bitmask_is_flag_type) {
    EXPECT_TRUE(ic_cts::TypeTraits<bitmask_types::PermissionsBits>::is_bitmask);
}

TEST(BitmasksTest, test_bitmask_members_exist) {
    bitmask_types::PermissionsBits read = bitmask_types::READ;
    bitmask_types::PermissionsBits write = bitmask_types::WRITE;
    bitmask_types::PermissionsBits execute = bitmask_types::EXECUTE;
    bitmask_types::PermissionsBits del = bitmask_types::DELETE;
    (void)read;
    (void)write;
    (void)execute;
    (void)del;
}

TEST(BitmasksTest, test_bitmask_auto_values) {
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::READ), 1);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::WRITE), 2);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::EXECUTE), 4);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::DELETE), 8);
}

TEST(BitmasksTest, test_bitmask_explicit_values) {
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::FLAG_A), 2);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::FLAG_B), 4);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::FLAG_C), 16);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::FLAG_D), 256);
}

TEST(BitmasksTest, test_bitmask_or_operation) {
    bitmask_types::Permissions combined = bitmask_types::READ | bitmask_types::WRITE;
    EXPECT_EQ(combined, 3);
    EXPECT_TRUE((combined & bitmask_types::READ) != 0);
    EXPECT_TRUE((combined & bitmask_types::WRITE) != 0);
}

TEST(BitmasksTest, test_bitmask_and_operation) {
    bitmask_types::Permissions combined =
        bitmask_types::READ | bitmask_types::WRITE | bitmask_types::EXECUTE;
    bitmask_types::Permissions result = combined & bitmask_types::READ;
    EXPECT_EQ(result, static_cast<uint32_t>(bitmask_types::READ));
}

TEST(BitmasksTest, test_bitmask_in_struct) {
    bitmask_types::FileInfo file_info("test.txt", bitmask_types::READ | bitmask_types::WRITE);
    EXPECT_EQ(file_info.path, "test.txt");
    EXPECT_TRUE((file_info.perms & bitmask_types::READ) != 0);
    EXPECT_TRUE((file_info.perms & bitmask_types::WRITE) != 0);
}

TEST(BitmasksTest, test_bitmask_none_value) {
    bitmask_types::Permissions none = 0;
    EXPECT_EQ(none, 0);
    EXPECT_FALSE((none & bitmask_types::READ) != 0);
    EXPECT_FALSE((none & bitmask_types::WRITE) != 0);
}

TEST(BitmasksTest, test_bitmask_all_combined) {
    bitmask_types::Permissions all =
        bitmask_types::READ | bitmask_types::WRITE | bitmask_types::EXECUTE | bitmask_types::DELETE;
    EXPECT_EQ(all, 15);
    EXPECT_TRUE((all & bitmask_types::READ) != 0);
    EXPECT_TRUE((all & bitmask_types::WRITE) != 0);
    EXPECT_TRUE((all & bitmask_types::EXECUTE) != 0);
    EXPECT_TRUE((all & bitmask_types::DELETE) != 0);
}

TEST(BitmasksTest, test_bitmask_gapped_positions) {
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::LOW), 1);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::HIGH), 128);
}

TEST(BitmasksTest, test_bitmask_single_flag) {
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::ONLY), 1);
}

TEST(BitmasksTest, test_bitmask_mixed_explicit_auto) {
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::AUTO_FIRST), 1);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::EXPLICIT_FOUR), 16);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::AUTO_FIVE), 32);
    EXPECT_EQ(static_cast<uint32_t>(bitmask_types::AUTO_SIX), 64);
}

TEST(BitmasksTest, test_bitmask_xor_operation) {
    bitmask_types::Permissions xor_result = bitmask_types::READ ^ bitmask_types::WRITE;
    EXPECT_EQ(xor_result, 3);

    bitmask_types::Permissions same = bitmask_types::READ ^ bitmask_types::READ;
    EXPECT_EQ(same, 0);
}

} // namespace
