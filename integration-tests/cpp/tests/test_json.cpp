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
#include <ic_cts/json.h>

#include <cstdint>
#include <map>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

using ic_cts::Json;
using ic_cts::JsonValue;

template <typename T, typename = void>
struct HasDiscriminator : std::false_type {};

template <typename T>
struct HasDiscriminator<T, std::void_t<decltype(std::declval<T&>()._d())>> : std::true_type {};

template <typename T, typename = void>
struct HasJsonType : std::false_type {};

template <typename T>
struct HasJsonType<T, std::void_t<decltype(std::declval<T&>().json_type())>> : std::true_type {};

TEST_CASE("str_root_null" * doctest::test_suite("json")) {
    CHECK(Json().str() == "null");
}

TEST_CASE("str_root_bool" * doctest::test_suite("json")) {
    CHECK(Json(true).str() == "true");
    CHECK(Json(false).str() == "false");
}

TEST_CASE("str_root_integer" * doctest::test_suite("json")) {
    CHECK(Json(42).str() == "42");
    CHECK(Json(-7).str() == "-7");
}

TEST_CASE("str_root_float" * doctest::test_suite("json")) {
    CHECK(Json(1.5).str() == "1.5000000000000000e+00");
}

TEST_CASE("str_root_string" * doctest::test_suite("json")) {
    CHECK(Json("hi").str() == "\"hi\"");
}

TEST_CASE("str_nested" * doctest::test_suite("json")) {
    Json json = Json::object("a", 7, "b", Json::array("hi", 7, nullptr), "c", nullptr);

    CHECK(json.str() == R"({"a":7,"b":["hi",7,null],"c":null})");
}

TEST_CASE("str_empty_containers" * doctest::test_suite("json")) {
    CHECK(Json::array().str() == "[]");
    CHECK(Json::object().str() == "{}");
    CHECK(Json("").str() == "\"\"");
}

TEST_CASE("stream_insert" * doctest::test_suite("json")) {
    std::ostringstream out;

    out << Json(7);

    CHECK(out.str() == "7");
}

TEST_CASE("stream_extract" * doctest::test_suite("json")) {
    std::istringstream in(R"({"k":[true]})");
    Json json;

    in >> json;

    CHECK(json["k"][0].get<bool>());
}

TEST_CASE("value_stream_insert" * doctest::test_suite("json")) {
    JsonValue value;
    value.integer_value(5);
    std::ostringstream out;

    out << value;

    CHECK(out.str() == "5");
}

TEST_CASE("value_stream_extract" * doctest::test_suite("json")) {
    std::istringstream in("\"text\"");
    JsonValue value;

    in >> value;

    CHECK(value.string_value() == "text");
}

TEST_CASE("parse_root_scalars" * doctest::test_suite("json")) {
    CHECK(Json::parse("null").is_null());
    CHECK(Json::parse("true").get<bool>());
    CHECK(Json::parse("42").get<int64_t>() == 42);
    CHECK(Json::parse("1.5").get<double>() == 1.5);
    CHECK(Json::parse("\"hi\"").get<std::string>() == "hi");
}

TEST_CASE("parse_nested" * doctest::test_suite("json")) {
    Json json = Json::parse(R"({"a":[1,{"b":null}],"s":"x\ny"})");

    CHECK(json.is_object());
    CHECK(json["a"].is_array());
    CHECK(json["a"][0].get<int64_t>() == 1);
    CHECK(json["a"][1]["b"].is_null());
    CHECK(json["s"].get<std::string>() == "x\ny");
}

TEST_CASE("parse_stream" * doctest::test_suite("json")) {
    std::istringstream in("[1,2]");

    Json json = Json::parse(in);

    CHECK(json.is_array());
    CHECK(json.size() == 2U);
}

TEST_CASE("parse_invalid" * doctest::test_suite("json")) {
    CHECK_THROWS_WITH_AS(
        Json::parse("{"), "Invalid JSON data supplied to JsonReader", std::runtime_error
    );
}

TEST_CASE("parse_integer_out_of_range" * doctest::test_suite("json")) {
    CHECK_THROWS_WITH_AS(
        Json::parse("18446744073709551616"), "Failed to read integer value", std::runtime_error
    );
}

TEST_CASE("parse_integer_limits" * doctest::test_suite("json")) {
    CHECK(Json::parse("9223372036854775807").get<int64_t>() == INT64_MAX);
    CHECK(Json::parse("-9223372036854775808").get<int64_t>() == INT64_MIN);
}

TEST_CASE("extract_over_existing" * doctest::test_suite("json")) {
    std::istringstream in("[3]");
    Json json = Json::array(1, 2);

    in >> json;

    CHECK(json.str() == "[3]");
}

TEST_CASE("round_trip" * doctest::test_suite("json")) {
    const std::string text = R"({"a\"b":[1,-2,true,false,null,"x\ny"],"e":[],"o":{}})";

    CHECK(Json::parse(text).str() == text);
}

TEST_CASE("clear_array" * doctest::test_suite("json")) {
    Json json = Json::array(1, 2);

    json.clear();

    CHECK(json.is_array());
    CHECK(json.empty());
}

TEST_CASE("clear_object" * doctest::test_suite("json")) {
    Json json = Json::object("k", 1);

    json.clear();

    CHECK(json.is_object());
    CHECK(json.empty());
}

TEST_CASE("clear_string" * doctest::test_suite("json")) {
    Json json("text");

    json.clear();

    CHECK(json.is_string());
    CHECK(json.empty());
}

TEST_CASE("clear_scalar" * doctest::test_suite("json")) {
    Json json(5);

    json.clear();

    CHECK(json.is_null());
}

TEST_CASE("value_default_null" * doctest::test_suite("json")) {
    std::ostringstream out;

    out << JsonValue();

    CHECK(out.str() == "null");
}

TEST_CASE("value_inactive_member" * doctest::test_suite("json")) {
    JsonValue value;

    CHECK_THROWS_WITH_AS(
        value.array_value(), "Union JsonValue not set to value array_value", std::logic_error
    );

    value.integer_value(1);

    CHECK_THROWS_AS(value.string_value(), std::logic_error);
    CHECK_THROWS_AS(value.float_value(), std::logic_error);
    CHECK_THROWS_AS(value.bool_value(), std::logic_error);
    CHECK_THROWS_AS(value.object_value(), std::logic_error);
}

TEST_CASE("json_value_accessor" * doctest::test_suite("json")) {
    Json json = Json::object("name", "x");

    CHECK(json["name"].json_value().string_value() == "x");
}

TEST_CASE("discriminator_hidden" * doctest::test_suite("json")) {
    CHECK_FALSE(HasDiscriminator<JsonValue>::value);
    CHECK_FALSE(HasJsonType<Json>::value);
}

TEST_CASE("value_setters" * doctest::test_suite("json")) {
    JsonValue value;

    value.float_value(2.5);
    CHECK(value.float_value() == 2.5);

    value.integer_value(3);
    CHECK(value.integer_value() == 3);

    value.string_value("s");
    CHECK(value.string_value() == "s");

    value.bool_value(true);
    CHECK(value.bool_value());
}

TEST_CASE("value_comparison" * doctest::test_suite("json")) {
    JsonValue null;
    JsonValue one;
    one.integer_value(1);
    JsonValue two;
    two.integer_value(2);
    JsonValue text;
    text.string_value("a");

    CHECK(one == one);
    CHECK(one != two);
    CHECK(one < two);
    CHECK(two > one);
    CHECK(one <= one);
    CHECK(two >= one);
    CHECK(null < one);
    CHECK(one < text);
}

TEST_CASE("ref_path" * doctest::test_suite("json")) {
    Json json = Json::parse(R"({"a":{"b":[10,20]}})");

    CHECK(json.ref() == "#");
    CHECK(json["a"]["b"].ref() == "#/a/b");
    CHECK(json["a"]["b"][1].ref() == "#/a/b/1");
}

TEST_CASE("index_by_path" * doctest::test_suite("json")) {
    Json json = Json::parse(R"({"a":{"b":[10,20]}})");

    CHECK(json["a/b/1"].get<int64_t>() == 20);
    CHECK(json["#/a/b/0"].get<int64_t>() == 10);
    CHECK_THROWS_AS(json["missing/b"], std::runtime_error);
}

TEST_CASE("index_on_null" * doctest::test_suite("json")) {
    Json json;

    json["k"] = 1;

    CHECK(json.str() == R"({"k":1})");
}

TEST_CASE("find_key" * doctest::test_suite("json")) {
    Json json = Json::object("k", 1);

    CHECK((json.find("k") != json.end()));
    CHECK((json.find("other") == json.end()));
}

TEST_CASE("at_out_of_bounds" * doctest::test_suite("json")) {
    Json json = Json::array(1, 2);

    CHECK((*json.at(1)).get<int64_t>() == 2);
    CHECK_THROWS_AS(json.at(3), std::runtime_error);
}

TEST_CASE("begin_on_scalar" * doctest::test_suite("json")) {
    Json json(5);

    CHECK_THROWS_WITH_AS(json.begin(), "Invalid value type for json iterator", std::runtime_error);
    CHECK_THROWS_WITH_AS(json.end(), "Invalid value type for json iterator", std::runtime_error);
}

TEST_CASE("iterate_object" * doctest::test_suite("json")) {
    Json json = Json::parse(R"({"a":1,"b":2})");
    std::string keys;
    int64_t sum = 0;

    for (auto it = json.begin(); it != json.end(); ++it) {
        keys += it.key();
        sum += (*it).get<int64_t>();
    }

    CHECK(keys == "ab");
    CHECK(sum == 3);
}

TEST_CASE("iterate_array" * doctest::test_suite("json")) {
    Json json = Json::array(1, 2, 3);
    std::string refs;
    int64_t sum = 0;

    for (auto&& child : json) {
        refs += child.ref();
        sum += child.get<int64_t>();
    }

    CHECK(refs == "#/0#/1#/2");
    CHECK(sum == 6);
}

TEST_CASE("iterate_const_object" * doctest::test_suite("json")) {
    const Json json = Json::parse(R"({"a":1,"b":2})");
    std::string keys;
    int64_t sum = 0;

    for (auto it = json.begin(); it != json.end(); ++it) {
        keys += it.key();
        sum += (*it).get<int64_t>();
    }

    CHECK(keys == "ab");
    CHECK(sum == 3);
}

TEST_CASE("iterate_const_array" * doctest::test_suite("json")) {
    const Json json = Json::array(1, 2, 3);
    std::string refs;
    int64_t sum = 0;

    for (auto&& child : json) {
        refs += child.ref();
        sum += child.get<int64_t>();
    }

    CHECK(refs == "#/0#/1#/2");
    CHECK(sum == 6);
}

TEST_CASE("size_and_empty" * doctest::test_suite("json")) {
    CHECK(Json().empty());
    CHECK(Json().size() == 0U);
    CHECK(Json::array(1, 2).size() == 2U);
    CHECK(Json::object("k", 1).size() == 1U);
    CHECK(Json("abc").size() == 3U);
    CHECK_FALSE(Json(5).empty());
}

TEST_CASE("factories" * doctest::test_suite("json")) {
    CHECK(Json::array().is_array());
    CHECK(Json::object().is_object());
    CHECK(Json::null().is_null());
    CHECK(Json::number(2.5).get<double>() == 2.5);
    CHECK(Json::integer(3).get<int64_t>() == 3);
    CHECK(Json::string("s").get<std::string>() == "s");
    CHECK(Json::boolean(true).get<bool>());
}

TEST_CASE("kind_predicates" * doctest::test_suite("json")) {
    CHECK(Json().is_null());
    CHECK(Json(1.5).is_float());
    CHECK(Json(1).is_integer());
    CHECK(Json("s").is_string());
    CHECK(Json::array().is_array());
    CHECK(Json::object().is_object());
    CHECK(Json(true).is_bool());
}

TEST_CASE("push_back" * doctest::test_suite("json")) {
    Json json;

    json.push_back(1);
    json.push_back("two");

    CHECK(json.str() == R"([1,"two"])");
}

TEST_CASE("assign_vector" * doctest::test_suite("json")) {
    Json json;

    json = std::vector<int32_t>{1, 2, 3};

    CHECK(json.str() == "[1,2,3]");
}

TEST_CASE("assign_map" * doctest::test_suite("json")) {
    Json json;

    json = std::map<std::string, std::string>{{"a", "x"}, {"b", "y"}};

    CHECK(json.str() == R"({"a":"x","b":"y"})");
}

TEST_CASE("assign_u16string" * doctest::test_suite("json")) {
    Json json;

    json = std::u16string_view(u"hé\U0001F600");

    CHECK(json.get<std::string>() == "h\xC3\xA9\xF0\x9F\x98\x80");
}

TEST_CASE("assign_ref" * doctest::test_suite("json")) {
    Json json = Json::object("a", 1, "b", 2);

    json["a"] = json["b"];

    CHECK(json["a"].get<int64_t>() == 2);
}

TEST_CASE("get_reference" * doctest::test_suite("json")) {
    Json json(1);

    json.get<int64_t>() = 5;

    CHECK(json.str() == "5");
}

TEST_CASE("deep_copy" * doctest::test_suite("json")) {
    Json original = Json::array(1);

    Json copy = original;
    copy.push_back(2);

    CHECK(original.size() == 1U);
    CHECK(copy.size() == 2U);
}

TEST_CASE("equality" * doctest::test_suite("json")) {
    CHECK(Json::parse("[1,2]") == Json::array(1, 2));
    CHECK(Json(1) != Json(2));
    CHECK(Json(1) < Json(2));
}

TEST_CASE("set_reference" * doctest::test_suite("json")) {
    Json json = Json::object("target", 1, "link", nullptr);

    json["link"].ref(json["target"]);

    CHECK(json["link"]["$ref"].get<std::string>() == "#/target");
}

TEST_CASE("update_references" * doctest::test_suite("json")) {
    Json json = Json::parse(
        R"({"defs":{"x":{"$ref":"#/old/y"}},"list":[{"$ref":"#/old/z"},{"$ref":"#/other"}]})"
    );

    json.update_references("#/old", "#/new");

    CHECK(json["defs"]["x"]["$ref"].get<std::string>() == "#/new/y");
    CHECK(json["list"][0]["$ref"].get<std::string>() == "#/new/z");
    CHECK(json["list"][1]["$ref"].get<std::string>() == "#/other");
}
