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
#include <ic_cts/json_serializer.h>

#include <algorithm>

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
