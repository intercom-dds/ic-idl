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

#include <type_traits>

#include "valuetypes.h"

TEST_CASE("valuetype state" * doctest::test_suite("valuetypes")) {
    valuetype_types_SimpleValue value{};
    value.id = 42;
    value.name = "value";

    CHECK(value.id == 42);
    CHECK(value.name[0] == 'v');
}

TEST_CASE("private state is public" * doctest::test_suite("valuetypes")) {
    valuetype_types_ValueWithPrivate value{};
    value.internal_id = 7;

    CHECK(value.internal_id == 7);
}

TEST_CASE("valuetype inheritance" * doctest::test_suite("valuetypes")) {
    valuetype_types_DerivedValue value{};
    value.valuetype_types_SimpleValue_base.id = 10;
    value.description = "derived";

    CHECK(value.valuetype_types_SimpleValue_base.id == 10);
    CHECK(value.description[0] == 'd');
}

TEST_CASE("valuetype collection members" * doctest::test_suite("valuetypes")) {
    static_assert(std::is_same_v<decltype(valuetype_types_WithSequence::numbers), idl_sequence_t*>);
    static_assert(std::is_same_v<decltype(valuetype_types_WithSequence::names), idl_sequence_t*>);
}

TEST_CASE("valuetype supported interface" * doctest::test_suite("valuetypes")) {
    static_assert(std::is_same_v<
                  decltype(valuetype_types_IdentifiableValue::valuetype_types_Identifiable_base),
                  valuetype_types_Identifiable>);
}
