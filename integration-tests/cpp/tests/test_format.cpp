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

#include <format>
#include <string>

#include "use_fmt/bitmasks.h"
#include "use_fmt/enums.h"
#include "use_fmt/structs.h"
#include "use_fmt/unions.h"
#include "use_fmt/valuetypes.h"

TEST_CASE("format_struct" * doctest::test_suite("format")) {
    struct_types::Point p(10, 20);
    CHECK(std::format("{}", p) == "{\"x\":10,\"y\":20}");
}

TEST_CASE("format_nested_struct" * doctest::test_suite("format")) {
    struct_types::Rectangle rect(struct_types::Point(0, 1), struct_types::Point(2, 3));
    CHECK(
        std::format("{}", rect) ==
        "{\"top_left\":{\"x\":0,\"y\":1},\"bottom_right\":{\"x\":2,\"y\":3}}"
    );
}

TEST_CASE("format_inherited_struct" * doctest::test_suite("format")) {
    struct_types::Point3D p(1, 2, 3);
    CHECK(std::format("{}", p) == "{\"x\":1,\"y\":2,\"z\":3}");
}

TEST_CASE("format_empty_struct" * doctest::test_suite("format")) {
    struct_types::Empty empty;
    CHECK(std::format("{}", empty) == "{}");
}

TEST_CASE("format_sequence_member" * doctest::test_suite("format")) {
    struct_types::WithSequence value;
    value.numbers = {1, 2, 3};
    value.names = {"a", "b"};
    CHECK(std::format("{}", value) == "{\"numbers\":[1,2,3],\"names\":[\"a\",\"b\"]}");
}

TEST_CASE("format_enum" * doctest::test_suite("format")) {
    CHECK(std::format("{}", enum_types::Color::BLUE) == "\"BLUE\"");
    CHECK(std::format("{}", enum_types::Status::WARNING) == "\"WARNING\"");
}

TEST_CASE("format_bitmask" * doctest::test_suite("format")) {
    bitmask_types::Permissions perms;
    perms |= bitmask_types::Permissions::READ;
    perms |= bitmask_types::Permissions::WRITE;
    CHECK(std::format("{}", perms) == "\"READ|WRITE\"");
}

TEST_CASE("format_union" * doctest::test_suite("format")) {
    union_types::IntOrString value;
    value.str_val("hello");
    CHECK(std::format("{}", value) == "{\"str_val\":\"hello\"}");
}

TEST_CASE("format_valuetype" * doctest::test_suite("format")) {
    valuetype_types::SimpleValue value(42, "answer");
    CHECK(std::format("{}", value) == "{\"id\":42,\"name\":\"answer\"}");
}

TEST_CASE("format_pretty" * doctest::test_suite("format")) {
    struct_types::Point p(10, 20);
    CHECK(std::format("{:#}", p) == "{\n  \"x\": 10,\n  \"y\": 20\n}");
    CHECK(std::format("{:#}|", p).back() == '|');
}

TEST_CASE("format_string_specification" * doctest::test_suite("format")) {
    struct_types::Empty empty;
    CHECK(std::format("[{:>6}]", empty) == "[    {}]");
    CHECK(std::format("[{:*<6}]", empty) == "[{}****]");
}

TEST_CASE("format_escapes_strings" * doctest::test_suite("format")) {
    struct_types::WithDefaults value;
    value.name = "quote\" backslash\\";
    CHECK(std::format("{}", value).find("\\\"") != std::string::npos);
    CHECK(std::format("{}", value).find("\\\\") != std::string::npos);
}
