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

namespace {
template <bool Reading>
struct DelimitedBooleanArchive {
    static constexpr bool IS_READER = Reading;
    static constexpr bool IS_WRITER = !Reading;

    std::stringstream& stream;

    template <typename T>
    struct VectorValue {
        DelimitedBooleanArchive& archive;

        VectorValue(DelimitedBooleanArchive& a, const ic_cts::TypeInfo*) : archive(a) {}

        template <typename Vector>
        void io(Vector& value) {
            if constexpr (Reading) {
                std::vector<T> elements;
                for (char element; archive.stream.get(element) && element != ';';) {
                    elements.push_back(element == '1');
                }

                value.resize(elements.size());
                for (size_t i = 0; i < elements.size(); ++i) {
                    value[i] = elements[i];
                }
            } else {
                for (size_t i = 0; i < value.size(); ++i) {
                    archive.stream.put(value[i] ? '1' : '0');
                }

                archive.stream.put(';');
            }
        }
    };
};
}

template <typename T>
static std::string json(T& value, ic_cts::SerializerFlags flags = {}) {
    std::stringstream ss;
    ic_cts::marshal_json(ss, value, flags);
    return ss.str();
}

template <typename T>
static void decode_json(const char* text, T& value) {
    std::stringstream stream(text);
    ic_cts::unmarshal_json(stream, value);
}

template <typename T>
static std::vector<uint8_t>
encode_cdr(const T& value, ic_cts::SerializerFlags encoding, const ic_cts::TypeInfo* type_info) {
    ic_cts::Buffer buffer;
    ic_cts::CdrWriter writer(buffer, encoding);
    ic_cts::CdrMarshal(writer).io(value, type_info);

    return {buffer.begin(), buffer.end()};
}

template <typename T>
static void decode_cdr(
    const std::vector<uint8_t>& bytes,
    T& value,
    ic_cts::SerializerFlags encoding,
    const ic_cts::TypeInfo* type_info
) {
    ic_cts::Buffer buffer(bytes.data(), bytes.size());
    ic_cts::CdrReader reader(buffer, encoding);
    ic_cts::CdrUnmarshal(reader).io(value, type_info);
}

static std::string encode_delimited(std::vector<bool>& value) {
    std::stringstream stream;
    DelimitedBooleanArchive<false> writer{stream};
    ic_cts::Serializer<DelimitedBooleanArchive<false>, std::vector<bool>>{}(writer, value, nullptr);

    return stream.str();
}

static void decode_delimited(const char* text, std::vector<bool>& value) {
    std::stringstream stream(text);
    DelimitedBooleanArchive<true> reader{stream};
    ic_cts::Serializer<DelimitedBooleanArchive<true>, std::vector<bool>>{}(reader, value, nullptr);
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

TEST_CASE("boolean_sequence_delimited_archive" * doctest::test_suite("serialization")) {
    SUBCASE("empty") {
        std::vector<bool> source;
        CHECK(encode_delimited(source) == ";");

        std::vector<bool> decoded = {false, true};
        decode_delimited(";", decoded);
        CHECK(decoded == source);
    }

    SUBCASE("true") {
        std::vector<bool> source = {true};
        CHECK(encode_delimited(source) == "1;");

        std::vector<bool> decoded = {false, true};
        decode_delimited("1;", decoded);
        CHECK(decoded == source);
    }

    SUBCASE("false") {
        std::vector<bool> source = {false};
        CHECK(encode_delimited(source) == "0;");

        std::vector<bool> decoded = {false, true};
        decode_delimited("0;", decoded);
        CHECK(decoded == source);
    }

    SUBCASE("mixed") {
        std::vector<bool> source = {true, false, true, true, false};
        CHECK(encode_delimited(source) == "10110;");

        std::vector<bool> decoded = {false, true};
        decode_delimited("10110;", decoded);
        CHECK(decoded == source);
    }
}

TEST_CASE("boolean_sequence_json" * doctest::test_suite("serialization")) {
    SUBCASE("empty") {
        const std::vector<bool> source;
        CHECK(json(source) == "[]");

        std::vector<bool> decoded = {false, true};
        decode_json("[]", decoded);
        CHECK(decoded == source);
    }

    SUBCASE("true") {
        const std::vector<bool> source = {true};
        CHECK(json(source) == "[true]");

        std::vector<bool> decoded = {false, true};
        decode_json("[true]", decoded);
        CHECK(decoded == source);
    }

    SUBCASE("false") {
        const std::vector<bool> source = {false};
        CHECK(json(source) == "[false]");

        std::vector<bool> decoded = {false, true};
        decode_json("[false]", decoded);
        CHECK(decoded == source);
    }

    SUBCASE("mixed") {
        const std::vector<bool> source = {true, false, true, true, false};
        CHECK(json(source) == "[true,false,true,true,false]");

        std::vector<bool> decoded = {false, true};
        decode_json("[true,false,true,true,false]", decoded);
        CHECK(decoded == source);
    }
}

TEST_CASE("boolean_sequence_cdr" * doctest::test_suite("serialization")) {
    auto type_info = ic_cts::NULL_SEQ_TYPE_INFO;
    type_info.element_type = &ic_cts::BOOLEAN_TYPE_INFO;

    SUBCASE("xcdr1 little endian empty") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source;
        const std::vector<uint8_t> expected = {0, 0, 0, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 little endian true") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source = {true};
        const std::vector<uint8_t> expected = {1, 0, 0, 0, 1};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 little endian false") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source = {false};
        const std::vector<uint8_t> expected = {1, 0, 0, 0, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 little endian mixed") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source = {true, false, true, true, false};
        const std::vector<uint8_t> expected = {5, 0, 0, 0, 1, 0, 1, 1, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 big endian empty") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source;
        const std::vector<uint8_t> expected = {0, 0, 0, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 big endian true") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source = {true};
        const std::vector<uint8_t> expected = {0, 0, 0, 1, 1};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 big endian false") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source = {false};
        const std::vector<uint8_t> expected = {0, 0, 0, 1, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr1 big endian mixed") {
        const auto encoding = ic_cts::CDR_XCDR1 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source = {true, false, true, true, false};
        const std::vector<uint8_t> expected = {0, 0, 0, 5, 1, 0, 1, 1, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 little endian empty") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source;
        const std::vector<uint8_t> expected = {0, 0, 0, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 little endian true") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source = {true};
        const std::vector<uint8_t> expected = {1, 0, 0, 0, 1};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 little endian false") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source = {false};
        const std::vector<uint8_t> expected = {1, 0, 0, 0, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 little endian mixed") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_LITTLE_ENDIAN;
        const std::vector<bool> source = {true, false, true, true, false};
        const std::vector<uint8_t> expected = {5, 0, 0, 0, 1, 0, 1, 1, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 big endian empty") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source;
        const std::vector<uint8_t> expected = {0, 0, 0, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 big endian true") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source = {true};
        const std::vector<uint8_t> expected = {0, 0, 0, 1, 1};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 big endian false") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source = {false};
        const std::vector<uint8_t> expected = {0, 0, 0, 1, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }

    SUBCASE("xcdr2 big endian mixed") {
        const auto encoding = ic_cts::CDR_XCDR2 | ic_cts::CDR_BIG_ENDIAN;
        const std::vector<bool> source = {true, false, true, true, false};
        const std::vector<uint8_t> expected = {0, 0, 0, 5, 1, 0, 1, 1, 0};
        CHECK(encode_cdr(source, encoding, &type_info) == expected);

        std::vector<bool> decoded = {false, true};
        decode_cdr(expected, decoded, encoding, &type_info);
        CHECK(decoded == source);
    }
}

TEST_CASE("boolean_sequence_bounds" * doctest::test_suite("serialization")) {
    auto type_info = ic_cts::NULL_SEQ_TYPE_INFO;
    type_info.element_type = &ic_cts::BOOLEAN_TYPE_INFO;
    type_info.max_length = 2;

    SUBCASE("trim") {
        type_info.flags = ic_cts::dcps::xtypes::IS_TRIM_TRY_CONSTRUCT;
        std::stringstream stream("[true,false,true]");
        ic_cts::JsonReader reader(stream, {});
        ic_cts::bounded_vector<bool, 2> decoded;
        ic_cts::JsonUnmarshal(reader).io(decoded, &type_info);
        CHECK(decoded == std::vector<bool>{true, false});
    }

    SUBCASE("use default") {
        type_info.flags = ic_cts::dcps::xtypes::IS_USE_DEFAULT_TRY_CONSTRUCT;
        std::stringstream stream("[true,false,true]");
        ic_cts::JsonReader reader(stream, {});
        ic_cts::bounded_vector<bool, 2> decoded;
        ic_cts::JsonUnmarshal(reader).io(decoded, &type_info);
        CHECK(decoded.empty());
    }

    SUBCASE("discard") {
        type_info.flags = ic_cts::dcps::xtypes::IS_DISCARD_TRY_CONSTRUCT;
        std::stringstream stream("[true,false,true]");
        ic_cts::JsonReader reader(stream, {});
        ic_cts::bounded_vector<bool, 2> decoded;
        CHECK_THROWS_WITH(
            ic_cts::JsonUnmarshal(reader).io(decoded, &type_info),
            "Sequence length out of bounds in vector serializer"
        );
    }
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
