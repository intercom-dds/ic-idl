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
#include <ic_cts/debug_serializer.h>
#include <ic_cts/json_serializer.h>

#include <array>
#include <limits>
#include <map>
#include <sstream>
#include <vector>

#include "bitmasks.h"
#include "circular_types.h"
#include "enums.h"
#include "exceptions.h"
#include "serialization.h"
#include "string_types.h"
#include "structs.h"
#include "unions.h"
#include "valuetypes.h"

template <typename T>
static std::string json(T& value, ic_cts::SerializerFlags flags = {}) {
    std::stringstream ss;
    ic_cts::marshal_json(ss, value, flags);
    return ss.str();
}

template <typename T>
static std::string debug(const T& value, bool pretty = false) {
    std::ostringstream stream;
    ic_cts::marshal_debug(stream, value, pretty);
    return stream.str();
}

TEST_CASE("debug_structs" * doctest::test_suite("serialization")) {
    CHECK(debug(struct_types::Point(10, 20)) == "Point { x: 10, y: 20 }");
    CHECK(debug(struct_types::Point3D(1, 2, 3)) == "Point3D { x: 1, y: 2, z: 3 }");
    CHECK(debug(struct_types::Empty{}) == "Empty {}");
    CHECK(
        debug(struct_types::Rectangle(struct_types::Point(0, 1), struct_types::Point(2, 3))) ==
        "Rectangle { top_left: Point { x: 0, y: 1 }, bottom_right: Point { x: 2, y: 3 } }"
    );
    CHECK(
        debug(valuetype_types::SimpleValue(42, "answer")) ==
        R"(SimpleValue { id: 42, name: "answer" })"
    );
    CHECK(
        debug(exception_types::SimpleError(7, "broken")) ==
        R"(SimpleError { error_code: 7, message: "broken" })"
    );
    CHECK(debug(exception_types::EmptyError{}) == "EmptyError {}");
    CHECK(debug(valuetype_types::WithNestedAlias::Nested(42)) == "Nested { value: 42 }");
}

TEST_CASE("debug_unions" * doctest::test_suite("serialization")) {
    union_types::IntOrString value;
    value.str_val("string");
    CHECK(debug(value) == R"(IntOrString { _d: 2, str_val: "string" })");

    value.default_val(true);
    value._d(123);
    CHECK(debug(value) == "IntOrString { _d: 123, default_val: true }");

    union_types::MultiCase multi;
    multi.small_val(42);
    multi._d(3);
    CHECK(debug(multi) == "MultiCase { _d: 3, small_val: 42 }");

    union_types::ByteSwitch byte;
    byte.number(7);
    byte._d(128);
    CHECK(debug(byte) == "ByteSwitch { _d: 128, number: 7 }");

    union_types::TypedValue typed;
    typed.string_value("hello");
    CHECK(debug(typed) == R"(TypedValue { _d: ValueKind::STRING_KIND, string_value: "hello" })");

    union_types::BoolSwitch boolean;
    boolean.true_val(7);
    CHECK(debug(boolean) == "BoolSwitch { _d: true, true_val: 7 }");

    union_types::ImplicitPrimitiveDisc inactive;
    inactive._d(42);
    CHECK(debug(inactive) == "ImplicitPrimitiveDisc { _d: 42 }");
}

TEST_CASE("debug_collections" * doctest::test_suite("serialization")) {
    CHECK(debug(std::vector<int32_t>{}) == "[]");
    CHECK(debug(std::vector<int32_t>{1, 2, 3}) == "[1, 2, 3]");
    CHECK(debug(std::vector<bool>{true, false}) == "[true, false]");
    CHECK(debug(std::array<int32_t, 2>{1, 2}) == "[1, 2]");
    const std::vector<std::vector<int32_t>> nested = {{}, {1, 2}};
    CHECK(debug(nested) == "[[], [1, 2]]");
    CHECK(json(nested) == "[[],[1,2]]");
    CHECK(debug(std::map<int32_t, int32_t>{}) == "{}");
    CHECK(debug(std::map<int32_t, int32_t>{{1, 2}, {3, 4}}) == "{1: 2, 3: 4}");

    const std::vector<std::map<std::string, int32_t>> maps = {{{"key", 42}}, {}};
    CHECK(debug(maps) == R"([{"key": 42}, {}])");
    CHECK(json(maps) == R"([{"key":42},{}])");
    CHECK(
        debug(
            std::map<std::string, std::vector<struct_types::Point>>{
                {"points", {struct_types::Point(1, 2), struct_types::Point(3, 4)}}
            }
        ) == R"({"points": [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]})"
    );

    struct_types::WithSequence value;
    value.numbers = {1, 2};
    value.names = {"a", "b"};
    CHECK(debug(value) == R"(WithSequence { numbers: [1, 2], names: ["a", "b"] })");
}

TEST_CASE("debug_optional_and_external_members" * doctest::test_suite("serialization")) {
    union_types::StructDefaults value;
    CHECK(
        debug(value) ==
        "StructDefaults { number: 42, external_data: ExternalData { number: 0 }, optional_number: None }"
    );

    value.external_data->number = 7;
    value.optional_number = 9;
    CHECK(
        debug(value) ==
        "StructDefaults { number: 42, external_data: ExternalData { number: 7 }, optional_number: Some(9) }"
    );
}

TEST_CASE("debug_optional_containers" * doctest::test_suite("serialization")) {
    serialization_types::MapMembers value;
    value.optional_unbounded.emplace();
    (*value.optional_unbounded)[1] = serialization_types::MapValue(42);
    CHECK(
        debug(value).find("optional_unbounded: Some({1: MapValue { number: 42 }})") !=
        std::string::npos
    );
    CHECK(debug(value).find("optional_bounded: None") != std::string::npos);
}

TEST_CASE("debug_nesting_limit" * doctest::test_suite("serialization")) {
    circular_types::TreeNode tree;
    auto* node = &tree;
    for (uint32_t i = 0; i < ic_cts::MAX_NESTED_DEPTH; ++i) {
        node->children.emplace_back();
        node = &node->children.back();
    }

    CHECK_THROWS_WITH_AS(
        debug(tree), "Cannot serialize type, nesting too deep", std::runtime_error
    );
}

TEST_CASE("debug_scalars" * doctest::test_suite("serialization")) {
    CHECK(debug(true) == "true");
    CHECK(debug(int8_t{-128}) == "-128");
    CHECK(debug(uint8_t{255}) == "255");
    CHECK(debug(std::numeric_limits<int64_t>::min()) == "-9223372036854775808");
    CHECK(debug(std::numeric_limits<uint64_t>::max()) == "18446744073709551615");
    CHECK(debug(1.25) == "1.25");
    CHECK(debug(std::numeric_limits<double>::infinity()) == "inf");
    CHECK(debug(enum_types::Color::BLUE) == "Color::BLUE");
    CHECK(debug(static_cast<enum_types::Color>(99)) == "Color(99)");

    bitmask_types::Permissions permissions;
    CHECK(debug(permissions) == "Permissions(0)");

    permissions |= bitmask_types::Permissions::READ;
    permissions |= bitmask_types::Permissions::WRITE;
    CHECK(debug(permissions) == "Permissions(READ | WRITE)");
}

TEST_CASE("debug_strings_and_characters" * doctest::test_suite("serialization")) {
    CHECK(debug(std::string("quote\" backslash\\\n\r\t")) == R"("quote\" backslash\\\n\r\t")");
    CHECK(debug(std::string("a\0b", 3)) == R"("a\0b")");
    CHECK(debug(std::string("\x01\x7f")) == R"("\u{1}\u{7f}")");
    CHECK(debug(std::u16string(u"\u03bb\U0001f600")) == u8"\"\u03bb\U0001f600\"");
    CHECK(debug(std::wstring(L"wide")) == R"("wide")");
    CHECK(debug('x') == "'x'");
    CHECK(debug('\n') == R"('\n')");
    CHECK(debug('\'') == R"('\'')");
    CHECK(debug(u'\u03bb') == u8"'\u03bb'");
}

TEST_CASE("debug_narrow_character_bytes" * doctest::test_suite("serialization")) {
    CHECK(debug(static_cast<char>(0x80)) == R"('\x80')");
    CHECK(debug(static_cast<char>(0xe9)) == R"('\xe9')");
    CHECK(debug(static_cast<char>(0xff)) == R"('\xff')");
    CHECK(debug('\0') == R"('\0')");
    CHECK(debug('\x7f') == R"('\u{7f}')");
    CHECK(debug(std::string("\xc3\xa9")) == u8"\"\u00e9\"");

    const char_wstring_types::CharFields fields(static_cast<char>(0xe9), u'A');
    CHECK(debug(fields) == R"(CharFields { single_char: '\xe9', wide_char: 'A' })");

    std::ostringstream stream;
    stream << fields;
    CHECK(stream.str() == R"(CharFields { single_char: '\xe9', wide_char: 'A' })");

    const char_wstring_types::CharSequences sequences(
        {static_cast<char>(0x80), static_cast<char>(0xe9), static_cast<char>(0xff)}, {}
    );
    CHECK(
        debug(sequences) == R"(CharSequences { char_seq: ['\x80', '\xe9', '\xff'], wchar_seq: [] })"
    );
}

TEST_CASE("debug_pretty_records" * doctest::test_suite("serialization")) {
    CHECK(debug(struct_types::Point(10, 20), true) == "Point {\n    x: 10,\n    y: 20,\n}");
    CHECK(debug(struct_types::Empty{}, true) == "Empty {}");
    CHECK(
        debug(
            struct_types::Rectangle(struct_types::Point(0, 1), struct_types::Point(2, 3)), true
        ) ==
        "Rectangle {\n"
        "    top_left: Point {\n"
        "        x: 0,\n"
        "        y: 1,\n"
        "    },\n"
        "    bottom_right: Point {\n"
        "        x: 2,\n"
        "        y: 3,\n"
        "    },\n"
        "}"
    );

    union_types::MultiCase value;
    value.small_val(42);
    value._d(3);
    CHECK(debug(value, true) == "MultiCase {\n    _d: 3,\n    small_val: 42,\n}");
}

TEST_CASE("debug_pretty_collections" * doctest::test_suite("serialization")) {
    CHECK(debug(std::vector<int32_t>{}, true) == "[]");
    CHECK(debug(std::map<int32_t, int32_t>{}, true) == "{}");
    CHECK(debug(std::array<int32_t, 2>{1, 2}, true) == "[\n    1,\n    2,\n]");
    CHECK(
        debug(std::map<std::string, std::vector<int32_t>>{{"values", {1, 2}}}, true) ==
        "{\n    \"values\": [\n        1,\n        2,\n    ],\n}"
    );
}

TEST_CASE("debug_pretty_optional_members" * doctest::test_suite("serialization")) {
    union_types::StructDefaults value;
    value.optional_number = 9;
    CHECK(
        debug(value, true) ==
        "StructDefaults {\n"
        "    number: 42,\n"
        "    external_data: ExternalData {\n"
        "        number: 0,\n"
        "    },\n"
        "    optional_number: Some(\n"
        "        9,\n"
        "    ),\n"
        "}"
    );

    value.optional_number.reset();
    CHECK(debug(value, true).find("    optional_number: None,\n}") != std::string::npos);
}

TEST_CASE("debug_stream_chaining" * doctest::test_suite("serialization")) {
    const struct_types::Point point(10, 20);
    std::ostringstream stream;
    stream << std::hex << std::showpos << std::boolalpha;
    std::ostream& result = ic_cts::marshal_debug(stream, point);
    CHECK(&result == &stream);

    stream << '|';
    ic_cts::marshal_debug(stream, true) << '|';
    ic_cts::marshal_debug(stream, point);
    CHECK(stream.str() == "Point { x: 10, y: 20 }|true|Point { x: 10, y: 20 }");
    CHECK((stream.flags() & std::ios::basefield) == std::ios::hex);
}
