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
#include <ic_cts/json_serializer.h>

#include <algorithm>
#include <vector>

#include "serialization.h"

template <typename T>
std::string json(T& value, ic_cts::SerializerFlags flags = {}) {
    std::stringstream ss;
    ic_cts::marshal_json(ss, value, flags);
    return ss.str();
}

TEST_CASE("serialization_json_numbers_large_integers" * doctest::test_suite("interfaces")) {
    constexpr int64_t safe = ic_cts::detail::json::MAX_SAFE_INTEGER;
    constexpr int64_t unsafe = ic_cts::detail::json::MAX_SAFE_INTEGER + 1;

    CHECK(json(safe).find('"') == std::string::npos);
    CHECK(json(unsafe).find('"') == std::string::npos);

    CHECK(json(safe, ic_cts::SERIALIZER_LARGE_INTEGERS_AS_STRINGS).find('"') == std::string::npos);
    CHECK(
        json(unsafe, ic_cts::SERIALIZER_LARGE_INTEGERS_AS_STRINGS).find('"') != std::string::npos
    );
}

TEST_CASE(
    "serialization_json_numbers_large_integers_structure" * doctest::test_suite("interfaces")
) {
    const serialization_types::JsonSafeNumbers safe{
        ic_cts::detail::json::MAX_SAFE_INTEGER,
        ic_cts::detail::json::MAX_SAFE_INTEGER,
        -ic_cts::detail::json::MAX_SAFE_INTEGER,
        {{ic_cts::detail::json::MAX_SAFE_INTEGER, -ic_cts::detail::json::MAX_SAFE_INTEGER}}
    };
    const serialization_types::JsonSafeNumbers unsafe{
        ic_cts::detail::json::MAX_SAFE_INTEGER + 1,
        ic_cts::detail::json::MAX_SAFE_INTEGER + 1,
        -ic_cts::detail::json::MAX_SAFE_INTEGER - 1,
        {{ic_cts::detail::json::MAX_SAFE_INTEGER + 1, -ic_cts::detail::json::MAX_SAFE_INTEGER - 1}}
    };

    CHECK(
        json(safe) ==
        R"({"a":9007199254740991,"b":9007199254740991,"c":-9007199254740991,"d":{"9007199254740991":-9007199254740991}})"
    );
    CHECK(
        json(unsafe) ==
        R"({"a":9007199254740992,"b":9007199254740992,"c":-9007199254740992,"d":{"9007199254740992":-9007199254740992}})"
    );

    CHECK(
        json(safe, ic_cts::SERIALIZER_LARGE_INTEGERS_AS_STRINGS) ==
        R"({"a":9007199254740991,"b":9007199254740991,"c":-9007199254740991,"d":{"9007199254740991":-9007199254740991}})"
    );
    CHECK(
        json(unsafe, ic_cts::SERIALIZER_LARGE_INTEGERS_AS_STRINGS) ==
        R"({"a":"9007199254740992","b":"9007199254740992","c":"-9007199254740992","d":{"9007199254740992":"-9007199254740992"}})"
    );
}

TEST_CASE("map_member_type_info" * doctest::test_suite("serialization")) {
    const auto& info = ic_cts::TypeTraits<serialization_types::MapMembers>::type_info;
    REQUIRE(info.member_count == 8);

    for (uint32_t i = 0; i < 6; ++i) {
        CAPTURE(i);
        const auto* map = info.members[i].type;
        REQUIRE(map != nullptr);
        CHECK(map->kind == ic_cts::dcps::xtypes::TK_MAP);
        CHECK(map->bit_size == 32);
        CHECK(map->max_length == ((i == 2 || i == 4) ? 5U : 0U));
        REQUIRE(map->key_type != nullptr);
        REQUIRE(map->element_type != nullptr);
        CHECK(
            ((info.members[i].flags & ic_cts::dcps::xtypes::IS_OPTIONAL) != 0) == (i == 3 || i == 4)
        );
    }

    const auto* primitive = info.members[0].type;
    CHECK(primitive->key_type == &ic_cts::LONG_TYPE_INFO);
    CHECK(primitive->element_type == &ic_cts::LONG_TYPE_INFO);

    const auto* unbounded = info.members[1].type;
    CHECK(unbounded->key_type->kind == ic_cts::dcps::xtypes::TK_STRING8);
    CHECK(unbounded->key_type->max_length == 0);
    CHECK(unbounded->key_type->element_type == &ic_cts::CHAR_TYPE_INFO);

    const auto* bounded = info.members[2].type;
    CHECK(bounded->key_type->kind == ic_cts::dcps::xtypes::TK_STRING8);
    CHECK(bounded->key_type->max_length == 8);

    for (uint32_t i = 1; i < 5; ++i) {
        CHECK(
            info.members[i].type->element_type ==
            &ic_cts::TypeTraits<serialization_types::MapValue>::type_info
        );
    }

    const auto* nested = info.members[5].type->element_type;
    REQUIRE(nested != nullptr);
    CHECK(nested->kind == ic_cts::dcps::xtypes::TK_SEQUENCE);
    REQUIRE(nested->element_type != nullptr);
    CHECK(nested->element_type->kind == ic_cts::dcps::xtypes::TK_MAP);
    CHECK(nested->element_type->key_type == &ic_cts::LONG_TYPE_INFO);
    CHECK(
        nested->element_type->element_type ==
        &ic_cts::TypeTraits<serialization_types::MapValue>::type_info
    );

    for (uint32_t i = 6; i < 8; ++i) {
        const auto* collection = info.members[i].type;
        REQUIRE(collection != nullptr);
        CHECK(
            collection->kind ==
            (i == 6 ? ic_cts::dcps::xtypes::TK_SEQUENCE : ic_cts::dcps::xtypes::TK_ARRAY)
        );
        const auto* map = collection->element_type;
        REQUIRE(map != nullptr);
        CHECK(map->kind == ic_cts::dcps::xtypes::TK_MAP);
        CHECK(map->max_length == 5);
        CHECK(map->key_type == &ic_cts::SHORT_TYPE_INFO);
        REQUIRE(map->element_type != nullptr);
        CHECK(map->element_type->kind == ic_cts::dcps::xtypes::TK_STRING8);
    }
}

TEST_CASE("map_union_type_info" * doctest::test_suite("serialization")) {
    const auto& info = ic_cts::TypeTraits<serialization_types::MapUnion>::type_info;
    REQUIRE(info.member_count == 3);
    const auto* map = info.members[1].type;
    REQUIRE(map != nullptr);
    CHECK(map->kind == ic_cts::dcps::xtypes::TK_MAP);
    CHECK(map->key_type == &ic_cts::LONG_TYPE_INFO);
    CHECK(map->element_type == &ic_cts::TypeTraits<serialization_types::MapValue>::type_info);
}

TEST_CASE("bounded_map_cdr_dheader" * doctest::test_suite("serialization")) {
    serialization_types::MyMap5 source;
    source.value[1] = "abc";

    {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_LITTLE_ENDIAN;
        ic_cts::Buffer buffer;
        ic_cts::CdrWriter writer(buffer, encoding);
        ic_cts::CdrMarshal(writer).io(source);

        const std::vector<uint8_t> expected = {
            1, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 'a', 'b', 'c', 0
        };
        CHECK(std::vector<uint8_t>(buffer.begin(), buffer.end()) == expected);

        ic_cts::Buffer input(expected.data(), expected.size());
        serialization_types::MyMap5 decoded;
        ic_cts::CdrReader reader(input, encoding);
        ic_cts::CdrUnmarshal(reader).io(decoded);
        CHECK(decoded == source);
    }

    {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN;
        ic_cts::Buffer buffer;
        ic_cts::CdrWriter writer(buffer, encoding);
        ic_cts::CdrMarshal(writer).io(source);

        const std::vector<uint8_t> expected = {20, 0, 0, 0, 16, 0, 0, 0, 1,   0,   0,   0,
                                               1,  0, 0, 0, 4,  0, 0, 0, 'a', 'b', 'c', 0};
        CHECK(std::vector<uint8_t>(buffer.begin(), buffer.end()) == expected);

        ic_cts::Buffer input(expected.data(), expected.size());
        serialization_types::MyMap5 decoded;
        ic_cts::CdrReader reader(input, encoding);
        ic_cts::CdrUnmarshal(reader).io(decoded);
        CHECK(decoded == source);
    }
}
