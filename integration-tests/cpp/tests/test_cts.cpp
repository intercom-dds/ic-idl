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

#include <any>
#include <clocale>
#include <cstdint>
#include <limits>
#include <locale>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>

#include "ic_cts/any.h"
#include "ic_cts/buffer.h"
#include "ic_cts/json_parser.h"

using ic_cts::Any;
using ic_cts::Buffer;
using ic_cts::JSON_FLOAT;
using ic_cts::JSON_STRING;
using ic_cts::JsonNode;
using ic_cts::JsonStream;
using std::any_cast;

TEST_CASE("any_copy_from_non_const_lvalue") {
    Any source{42};
    Any copy{source};
    CHECK(any_cast<int>(copy) == 42);

    Any assigned;
    assigned = source;
    CHECK(any_cast<int>(assigned) == 42);
    CHECK(any_cast<int>(source) == 42);
}

TEST_CASE("json_stream_string_survives_closing_quote") {
    for (size_t length = 0; length < 200; ++length) {
        std::string text(length, 'x');
        std::istringstream in('"' + text + '"');
        JsonStream stream(in);

        JsonNode node = JsonNode::from_data(stream);
        REQUIRE(node.get_type() == JSON_STRING);

        std::string value;
        CHECK(node.get_string(value));
        CHECK(value == text);
    }
}

TEST_CASE("json_number_uses_supplied_length") {
    std::unique_ptr<char[]> text(new char[3]{'1', '.', '5'});
    JsonNode node = JsonNode::from_data(std::string_view(text.get(), 3));
    REQUIRE(node.get_type() == JSON_FLOAT);

    double value = 0.0;
    CHECK(node.get_number(value));
    CHECK(value == 1.5);
}

TEST_CASE("buffer_read_checks_length_before_copying") {
    std::unique_ptr<uint8_t[]> bytes(new uint8_t[1]{7});
    Buffer buffer(static_cast<const uint8_t*>(bytes.get()), 1);

    uint8_t destination[2] = {0, 0};
    CHECK_THROWS_AS(buffer.read(destination, 2), std::runtime_error);
    CHECK(destination[0] == 0);
    CHECK(destination[1] == 0);
    CHECK(buffer.unread_bytes() == 1);

    buffer.read(destination, 1);
    CHECK(destination[0] == 7);
    CHECK(buffer.unread_bytes() == 0);
}

TEST_CASE("buffer_span_covers_unread_bytes") {
    std::unique_ptr<uint8_t[]> bytes(new uint8_t[4]{1, 2, 3, 4});
    Buffer buffer(static_cast<const uint8_t*>(bytes.get()), 4);
    buffer.advance_read(2);

    auto unread = buffer.span();
    REQUIRE(unread.size() == buffer.unread_bytes());
    CHECK(unread.size() == 2);

    uint8_t sum = 0;
    for (uint8_t byte : unread) {
        sum += byte;
    }
    CHECK(sum == 7);
}
