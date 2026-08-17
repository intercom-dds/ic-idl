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

#include <doctest/doctest.h>
#include <ic_cts/member_info.h>
#include <ic_cts/omg_types.h>

#include "bitmasks.h"

TEST_CASE("bitmask_is_flag_type" * doctest::test_suite("bitmasks")) {
    CHECK(ic_cts::TypeTraits<bitmask_types::Permissions>::is_bitmask);
}

TEST_CASE("bitmask_members_exist" * doctest::test_suite("bitmasks")) {
    bitmask_types::Permissions read = bitmask_types::Permissions::READ;
    bitmask_types::Permissions write = bitmask_types::Permissions::WRITE;
    bitmask_types::Permissions execute = bitmask_types::Permissions::EXECUTE;
    bitmask_types::Permissions del = bitmask_types::Permissions::DELETE;
    (void)read;
    (void)write;
    (void)execute;
    (void)del;
}

TEST_CASE("bitmask_auto_values" * doctest::test_suite("bitmasks")) {
    CHECK(static_cast<uint32_t>(bitmask_types::Permissions::READ) == 1);
    CHECK(static_cast<uint32_t>(bitmask_types::Permissions::WRITE) == 2);
    CHECK(static_cast<uint32_t>(bitmask_types::Permissions::EXECUTE) == 4);
    CHECK(static_cast<uint32_t>(bitmask_types::Permissions::DELETE) == 8);
}

TEST_CASE("bitmask_explicit_values" * doctest::test_suite("bitmasks")) {
    CHECK(static_cast<uint32_t>(bitmask_types::ExplicitFlags::FLAG_A) == 2);
    CHECK(static_cast<uint32_t>(bitmask_types::ExplicitFlags::FLAG_B) == 4);
    CHECK(static_cast<uint32_t>(bitmask_types::ExplicitFlags::FLAG_C) == 16);
    CHECK(static_cast<uint32_t>(bitmask_types::ExplicitFlags::FLAG_D) == 256);
}

TEST_CASE("bitmask_or_operation" * doctest::test_suite("bitmasks")) {
    bitmask_types::Permissions combined = bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE;
    CHECK(combined == 3);
    CHECK((combined & bitmask_types::Permissions::READ) != 0);
    CHECK((combined & bitmask_types::Permissions::WRITE) != 0);
}

TEST_CASE("bitmask_and_operation" * doctest::test_suite("bitmasks")) {
    bitmask_types::Permissions combined =
        bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE | bitmask_types::Permissions::EXECUTE;
    bitmask_types::Permissions result = combined & bitmask_types::Permissions::READ;
    CHECK(result == static_cast<uint32_t>(bitmask_types::Permissions::READ));
}

TEST_CASE("bitmask_in_struct" * doctest::test_suite("bitmasks")) {
    bitmask_types::FileInfo file_info("test.txt", bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE);
    CHECK(file_info.path == "test.txt");
    CHECK((file_info.perms & bitmask_types::Permissions::READ) != 0);
    CHECK((file_info.perms & bitmask_types::Permissions::WRITE) != 0);
}

TEST_CASE("bitmask_none_value" * doctest::test_suite("bitmasks")) {
    bitmask_types::Permissions none = 0;
    CHECK(none == 0);
    CHECK_FALSE((none & bitmask_types::Permissions::READ) != 0);
    CHECK_FALSE((none & bitmask_types::Permissions::WRITE) != 0);
}

TEST_CASE("bitmask_all_combined" * doctest::test_suite("bitmasks")) {
    bitmask_types::Permissions all =
        bitmask_types::Permissions::READ | bitmask_types::Permissions::WRITE | bitmask_types::Permissions::EXECUTE | bitmask_types::Permissions::DELETE;
    CHECK(all == 15);
    CHECK((all & bitmask_types::Permissions::READ) != 0);
    CHECK((all & bitmask_types::Permissions::WRITE) != 0);
    CHECK((all & bitmask_types::Permissions::EXECUTE) != 0);
    CHECK((all & bitmask_types::Permissions::DELETE) != 0);
}

TEST_CASE("bitmask_gapped_positions" * doctest::test_suite("bitmasks")) {
    CHECK(static_cast<uint32_t>(bitmask_types::GappedFlags::LOW) == 1);
    CHECK(static_cast<uint32_t>(bitmask_types::GappedFlags::HIGH) == 128);
}

TEST_CASE("bitmask_single_flag" * doctest::test_suite("bitmasks")) {
    CHECK(static_cast<uint32_t>(bitmask_types::SingleFlag::ONLY) == 1);
}

TEST_CASE("bitmask_mixed_explicit_auto" * doctest::test_suite("bitmasks")) {
    CHECK(static_cast<uint32_t>(bitmask_types::MixedFlags::AUTO_FIRST) == 1);
    CHECK(static_cast<uint32_t>(bitmask_types::MixedFlags::EXPLICIT_FOUR) == 16);
    CHECK(static_cast<uint32_t>(bitmask_types::MixedFlags::AUTO_FIVE) == 32);
    CHECK(static_cast<uint32_t>(bitmask_types::MixedFlags::AUTO_SIX) == 64);
}

TEST_CASE("bitmask_xor_operation" * doctest::test_suite("bitmasks")) {
    bitmask_types::Permissions xor_result = bitmask_types::Permissions::READ ^ bitmask_types::Permissions::WRITE;
    CHECK(xor_result == 3);

    bitmask_types::Permissions same = bitmask_types::Permissions::READ ^ bitmask_types::Permissions::READ;
    CHECK(same == 0);
}

TEST_CASE("bitmask_bit_bound" * doctest::test_suite("bitmasks")) {
    CHECK(omg::types::bit_bound_v<bitmask_types::BitBoundFlags>() == 7);
    CHECK(std::is_same_v<omg::types::underlying_type_t<bitmask_types::BitBoundFlags>, uint8_t>);
}
