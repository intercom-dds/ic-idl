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

#include "nested_modules.h"
#include "structs.h"

TEST_CASE("nested module structs" * doctest::test_suite("structs")) {
    nested_module_types_TopLevelStruct top{};
    top.value = 42;

    nested_module_types_level1_Level1Struct nested{};
    nested.data = 84;
    nested.parent_ref = top;

    CHECK(nested.data == 84);
    CHECK(nested.parent_ref.value == 42);
}

TEST_CASE("sequence members" * doctest::test_suite("structs")) {
    static_assert(std::is_same_v<decltype(struct_types_WithSequence::numbers), idl_sequence_t*>);
    static_assert(std::is_same_v<decltype(struct_types_WithSequence::names), idl_sequence_t*>);

    struct_types_WithSequence value{};

    CHECK(value.numbers == nullptr);
    CHECK(value.names == nullptr);
}

TEST_CASE("map members" * doctest::test_suite("structs")) {
    static_assert(std::is_same_v<decltype(struct_types_WithMap::string_to_int), idl_map_t*>);

    struct_types_WithMap value{};

    CHECK(value.string_to_int == nullptr);
}
