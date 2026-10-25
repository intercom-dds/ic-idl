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
#include <ic_cts/cdr_serializer.h>

#include <type_traits>
#include <vector>

#include "any_type.h"

static ic_cts::Buffer empty_any_sequence_map_cdr() {
    ic_cts::Buffer buffer;
    ic_cts::CdrWriter writer(buffer, ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN);
    ic_cts::CdrMarshal marshal(writer);

    const uint32_t sequence_length = 0;
    const uint32_t map_length = 1;
    const int32_t key = 7;
    const uint32_t map_data_size = sizeof(map_length) + sizeof(key) + sizeof(sequence_length);
    const uint32_t struct_data_size = sizeof(map_data_size) + map_data_size;
    marshal.io(struct_data_size);
    marshal.io(map_data_size);
    marshal.io(map_length);
    marshal.io(key);
    marshal.io(sequence_length);

    return buffer;
}

TEST_CASE("any_alias_typedef" * doctest::test_suite("any_type")) {
    CHECK((std::is_same<any_types::AnyAlias, ic_cts::Any>::value));
}

TEST_CASE("using_any_alias" * doctest::test_suite("any_type")) {
    CHECK((std::is_same<decltype(any_types::UsingAnyAlias().data), any_types::AnyAlias>::value));
}

TEST_CASE("map_with_any_type_info" * doctest::test_suite("any_type")) {
    const auto& info = ic_cts::TypeTraits<any_types::MapWithAny>::type_info;
    REQUIRE(info.member_count == 1);
    const auto* map = info.members[0].type;
    REQUIRE(map != nullptr);
    CHECK(map->kind == ic_cts::dcps::xtypes::TK_MAP);
    REQUIRE(map->key_type != nullptr);
    CHECK(map->key_type->kind == ic_cts::dcps::xtypes::TK_STRING8);
    CHECK(map->key_type->element_type == &ic_cts::CHAR_TYPE_INFO);
    CHECK(map->element_type == &ic_cts::NULL_TYPE_INFO);
}

TEST_CASE("map_with_any_sequence_type_info" * doctest::test_suite("any_type")) {
    const auto& info = ic_cts::TypeTraits<any_types::MapWithAnySequence>::type_info;
    REQUIRE(info.member_count == 1);
    const auto* map = info.members[0].type;
    REQUIRE(map != nullptr);
    CHECK(map->kind == ic_cts::dcps::xtypes::TK_MAP);
    CHECK(map->key_type == &ic_cts::LONG_TYPE_INFO);
    const auto* sequence = map->element_type;
    REQUIRE(sequence != nullptr);
    CHECK(sequence->kind == ic_cts::dcps::xtypes::TK_SEQUENCE);
    CHECK(sequence->max_length == 0);
    CHECK(sequence->element_type == &ic_cts::NULL_TYPE_INFO);
}

TEST_CASE("map_with_any_array_type_info" * doctest::test_suite("any_type")) {
    const auto& info = ic_cts::TypeTraits<any_types::MapWithAnyArray>::type_info;
    REQUIRE(info.member_count == 1);
    const auto* map = info.members[0].type;
    REQUIRE(map != nullptr);
    CHECK(map->kind == ic_cts::dcps::xtypes::TK_MAP);
    CHECK(map->key_type == &ic_cts::LONG_TYPE_INFO);
    const auto* array = map->element_type;
    REQUIRE(array != nullptr);
    CHECK(array->kind == ic_cts::dcps::xtypes::TK_ARRAY);
    CHECK(array->max_length == 2);
    CHECK(array->element_type == &ic_cts::NULL_TYPE_INFO);
}

TEST_CASE("map_with_empty_any_sequence_xcdr2_write" * doctest::test_suite("any_type")) {
    any_types::MapWithAnySequence source;
    source.properties[7] = {};
    ic_cts::Buffer buffer;
    ic_cts::CdrWriter writer(buffer, ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN);
    ic_cts::CdrMarshal(writer).io(source);

    const auto expected = empty_any_sequence_map_cdr();
    CHECK(buffer == expected);
}

TEST_CASE("map_with_empty_any_sequence_xcdr2_read" * doctest::test_suite("any_type")) {
    auto buffer = empty_any_sequence_map_cdr();
    any_types::MapWithAnySequence decoded;
    ic_cts::CdrReader reader(buffer, ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN);
    ic_cts::CdrUnmarshal(reader).io(decoded);

    REQUIRE(decoded.properties.size() == 1);
    CHECK(decoded.properties.at(7).empty());
    CHECK(buffer.unread_bytes() == 0);
}

TEST_CASE("empty_map_with_any_xcdr2_write" * doctest::test_suite("any_type")) {
    const any_types::MapWithAny source;
    ic_cts::Buffer buffer;
    ic_cts::CdrWriter writer(buffer, ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN);
    ic_cts::CdrMarshal(writer).io(source);

    const std::vector<uint8_t> expected = {8, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0};
    CHECK(std::vector<uint8_t>(buffer.begin(), buffer.end()) == expected);
}

TEST_CASE("empty_map_with_any_xcdr2_read" * doctest::test_suite("any_type")) {
    const std::vector<uint8_t> encoded = {8, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0};
    ic_cts::Buffer buffer(encoded.data(), encoded.size());
    any_types::MapWithAny decoded;
    ic_cts::CdrReader reader(buffer, ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN);
    ic_cts::CdrUnmarshal(reader).io(decoded);

    CHECK(decoded.properties.empty());
    CHECK(buffer.unread_bytes() == 0);
}
