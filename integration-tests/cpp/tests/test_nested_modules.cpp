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

#include "nested_modules.h"

TEST_CASE("top_level_types_exist" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct s(42);
    CHECK(s.value == 42);
    CHECK(nested_module_types::TopLevelEnum::FIRST == 0);
    CHECK(nested_module_types::TopLevelEnum::SECOND == 1);
}

TEST_CASE("nested_module_level1_exists" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct s(20, top);
    CHECK(s.data == 20);
    CHECK(s.parent_ref.value == 10);
    CHECK(nested_module_types::level1::Level1Enum::A == 0);
    CHECK(nested_module_types::level1::Level1Enum::B == 1);
    CHECK(nested_module_types::level1::Level1Enum::C == 2);
}

TEST_CASE("nested_module_level2_exists" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct s("test", l1, top);
    CHECK(s.name == "test");
    CHECK(s.level1_ref.data == 2);
    CHECK(s.top_ref.value == 1);
}

TEST_CASE("nested_module_level3_exists" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct l2("level2", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct s(99, l2, l1, top);
    CHECK(s.id == 99);
    CHECK(s.level2_ref.name == "level2");
    CHECK(s.level1_ref.data == 2);
    CHECK(s.top_ref.value == 1);
    CHECK(nested_module_types::level1::level2::level3::DEEP_CONST == 42);
}

TEST_CASE("sibling_module_exists" * doctest::test_suite("nested_modules")) {
    nested_module_types::sibling::SiblingStruct s(123);
    CHECK(s.id == 123);
}

TEST_CASE("top_level_struct_instantiation" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct s(100);
    CHECK(s.value == 100);
}

TEST_CASE("level1_struct_with_parent_ref" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct parent(50);
    nested_module_types::level1::Level1Struct s(75, parent);
    CHECK(s.data == 75);
    CHECK(s.parent_ref.value == 50);
}

TEST_CASE("level2_struct_with_refs" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    nested_module_types::level1::level2::Level2Struct s("hello", l1, top);
    CHECK(s.name == "hello");
    CHECK(s.level1_ref.data == 20);
    CHECK(s.level1_ref.parent_ref.value == 10);
    CHECK(s.top_ref.value == 10);
}

TEST_CASE("level3_struct_with_all_refs" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct l2("level2", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(3, l2, l1, top);
    CHECK(l3.id == 3);
    CHECK(l3.level2_ref.name == "level2");
    CHECK(l3.level1_ref.data == 2);
    CHECK(l3.top_ref.value == 1);
}

TEST_CASE("deep_constant" * doctest::test_suite("nested_modules")) {
    CHECK(nested_module_types::level1::level2::level3::DEEP_CONST == 42);
}

TEST_CASE("sibling_cross_ref_struct" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(100);
    nested_module_types::level1::Level1Struct l1(200, top);
    nested_module_types::level1::level2::Level2Struct l2("cross", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(300, l2, l1, top);
    nested_module_types::sibling::CrossRef cr(l1, l2, l3);
    CHECK(cr.from_level1.data == 200);
    CHECK(cr.from_level2.name == "cross");
    CHECK(cr.from_level3.id == 300);
}

TEST_CASE("top_using_nested_struct" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct l2("test", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(3, l2, l1, top);
    nested_module_types::sibling::SiblingStruct sib(4);
    nested_module_types::TopUsingNested tun(l1, l2, l3, sib);
    CHECK(tun.l1.data == 2);
    CHECK(tun.l2.name == "test");
    CHECK(tun.l3.id == 3);
    CHECK(tun.sib.id == 4);
}

TEST_CASE("level1_enum" * doctest::test_suite("nested_modules")) {
    CHECK(nested_module_types::level1::Level1Enum::A == 0);
    CHECK(nested_module_types::level1::Level1Enum::B == 1);
    CHECK(nested_module_types::level1::Level1Enum::C == 2);
}

TEST_CASE("namespace_hierarchy_level1" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    CHECK(l1.data == 20);
}

TEST_CASE("namespace_hierarchy_level2" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    nested_module_types::level1::level2::Level2Struct l2("nested", l1, top);
    CHECK(l2.name == "nested");
}

TEST_CASE("namespace_hierarchy_level3" * doctest::test_suite("nested_modules")) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    nested_module_types::level1::level2::Level2Struct l2("nested", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(30, l2, l1, top);
    CHECK(l3.id == 30);
}
