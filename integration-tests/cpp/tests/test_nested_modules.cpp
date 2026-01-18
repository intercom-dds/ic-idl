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

#include <gtest/gtest.h>

#include "generated/nested_modules.h"

namespace {

TEST(NestedModulesTest, test_top_level_types_exist) {
    nested_module_types::TopLevelStruct s(42);
    EXPECT_EQ(s.value, 42);
    EXPECT_EQ(nested_module_types::TopLevelEnum::FIRST, 0);
    EXPECT_EQ(nested_module_types::TopLevelEnum::SECOND, 1);
}

TEST(NestedModulesTest, test_nested_module_level1_exists) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct s(20, top);
    EXPECT_EQ(s.data, 20);
    EXPECT_EQ(s.parent_ref.value, 10);
    EXPECT_EQ(nested_module_types::level1::Level1Enum::A, 0);
    EXPECT_EQ(nested_module_types::level1::Level1Enum::B, 1);
    EXPECT_EQ(nested_module_types::level1::Level1Enum::C, 2);
}

TEST(NestedModulesTest, test_nested_module_level2_exists) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct s("test", l1, top);
    EXPECT_EQ(s.name, "test");
    EXPECT_EQ(s.level1_ref.data, 2);
    EXPECT_EQ(s.top_ref.value, 1);
}

TEST(NestedModulesTest, test_nested_module_level3_exists) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct l2("level2", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct s(99, l2, l1, top);
    EXPECT_EQ(s.id, 99);
    EXPECT_EQ(s.level2_ref.name, "level2");
    EXPECT_EQ(s.level1_ref.data, 2);
    EXPECT_EQ(s.top_ref.value, 1);
    EXPECT_EQ(nested_module_types::level1::level2::level3::DEEP_CONST, 42);
}

TEST(NestedModulesTest, test_sibling_module_exists) {
    nested_module_types::sibling::SiblingStruct s(123);
    EXPECT_EQ(s.id, 123);
}

TEST(NestedModulesTest, test_top_level_struct_instantiation) {
    nested_module_types::TopLevelStruct s(100);
    EXPECT_EQ(s.value, 100);
}

TEST(NestedModulesTest, test_level1_struct_with_parent_ref) {
    nested_module_types::TopLevelStruct parent(50);
    nested_module_types::level1::Level1Struct s(75, parent);
    EXPECT_EQ(s.data, 75);
    EXPECT_EQ(s.parent_ref.value, 50);
}

TEST(NestedModulesTest, test_level2_struct_with_refs) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    nested_module_types::level1::level2::Level2Struct s("hello", l1, top);
    EXPECT_EQ(s.name, "hello");
    EXPECT_EQ(s.level1_ref.data, 20);
    EXPECT_EQ(s.level1_ref.parent_ref.value, 10);
    EXPECT_EQ(s.top_ref.value, 10);
}

TEST(NestedModulesTest, test_level3_struct_with_all_refs) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct l2("level2", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(3, l2, l1, top);
    EXPECT_EQ(l3.id, 3);
    EXPECT_EQ(l3.level2_ref.name, "level2");
    EXPECT_EQ(l3.level1_ref.data, 2);
    EXPECT_EQ(l3.top_ref.value, 1);
}

TEST(NestedModulesTest, test_deep_constant) {
    EXPECT_EQ(nested_module_types::level1::level2::level3::DEEP_CONST, 42);
}

TEST(NestedModulesTest, test_sibling_cross_ref_struct) {
    nested_module_types::TopLevelStruct top(100);
    nested_module_types::level1::Level1Struct l1(200, top);
    nested_module_types::level1::level2::Level2Struct l2("cross", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(300, l2, l1, top);
    nested_module_types::sibling::CrossRef cr(l1, l2, l3);
    EXPECT_EQ(cr.from_level1.data, 200);
    EXPECT_EQ(cr.from_level2.name, "cross");
    EXPECT_EQ(cr.from_level3.id, 300);
}

TEST(NestedModulesTest, test_top_using_nested_struct) {
    nested_module_types::TopLevelStruct top(1);
    nested_module_types::level1::Level1Struct l1(2, top);
    nested_module_types::level1::level2::Level2Struct l2("test", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(3, l2, l1, top);
    nested_module_types::sibling::SiblingStruct sib(4);
    nested_module_types::TopUsingNested tun(l1, l2, l3, sib);
    EXPECT_EQ(tun.l1.data, 2);
    EXPECT_EQ(tun.l2.name, "test");
    EXPECT_EQ(tun.l3.id, 3);
    EXPECT_EQ(tun.sib.id, 4);
}

TEST(NestedModulesTest, test_level1_enum) {
    EXPECT_EQ(nested_module_types::level1::Level1Enum::A, 0);
    EXPECT_EQ(nested_module_types::level1::Level1Enum::B, 1);
    EXPECT_EQ(nested_module_types::level1::Level1Enum::C, 2);
}

TEST(NestedModulesTest, test_namespace_hierarchy_level1) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    EXPECT_EQ(l1.data, 20);
}

TEST(NestedModulesTest, test_namespace_hierarchy_level2) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    nested_module_types::level1::level2::Level2Struct l2("nested", l1, top);
    EXPECT_EQ(l2.name, "nested");
}

TEST(NestedModulesTest, test_namespace_hierarchy_level3) {
    nested_module_types::TopLevelStruct top(10);
    nested_module_types::level1::Level1Struct l1(20, top);
    nested_module_types::level1::level2::Level2Struct l2("nested", l1, top);
    nested_module_types::level1::level2::level3::Level3Struct l3(30, l2, l1, top);
    EXPECT_EQ(l3.id, 30);
}

} // namespace
