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

#include "generated/multi_module.h"

namespace {

TEST(MultiModuleTest, test_module_a_exists) {
    module_a::StructA1 s1(42);
    EXPECT_EQ(s1.value, 42);
}

TEST(MultiModuleTest, test_module_b_exists) {
    module_b::StructB1 s1("test");
    EXPECT_EQ(s1.name, "test");
}

TEST(MultiModuleTest, test_module_a_first_opening) {
    module_a::StructA1 s1(10);
    EXPECT_EQ(s1.value, 10);
    EXPECT_EQ(module_a::CONST_A1, 100);
    EXPECT_EQ(module_a::EnumA::X, 0);
    EXPECT_EQ(module_a::EnumA::Y, 1);
}

TEST(MultiModuleTest, test_module_a_second_opening) {
    module_a::StructA1 a1(5);
    module_a::StructA2 s2(3.14, a1);
    EXPECT_EQ(s2.data, 3.14);
    EXPECT_EQ(s2.ref_to_a1.value, 5);
    EXPECT_EQ(module_a::CONST_A2, 101);
    EXPECT_EQ(module_a::EnumA2::P, 0);
    EXPECT_EQ(module_a::EnumA2::Q, 1);
    EXPECT_EQ(module_a::EnumA2::R, 2);
}

TEST(MultiModuleTest, test_module_a_third_opening) {
    module_a::StructA1 a1(1);
    module_a::StructA2 a2(2.0, a1);
    module_a::StructA3 s3(true, a1, a2);
    EXPECT_EQ(s3.flag, true);
    EXPECT_EQ(s3.a1.value, 1);
    EXPECT_EQ(s3.a2.data, 2.0);
    EXPECT_EQ(module_a::CONST_A3, 102);
}

TEST(MultiModuleTest, test_module_b_both_openings) {
    module_b::StructB1 b1("first");
    EXPECT_EQ(b1.name, "first");
    EXPECT_EQ(module_b::CONST_B1, 200);

    module_b::StructB2 b2(42, b1);
    EXPECT_EQ(b2.id, 42);
    EXPECT_EQ(b2.ref_to_b1.name, "first");
    EXPECT_EQ(module_b::CONST_B2, 201);
}

TEST(MultiModuleTest, test_reopened_module_types_can_reference_earlier) {
    module_a::StructA1 a1(100);
    module_a::StructA2 a2(99.5, a1);
    EXPECT_EQ(a2.ref_to_a1.value, 100);
    EXPECT_EQ(a2.data, 99.5);
}

TEST(MultiModuleTest, test_reopened_module_chain) {
    module_a::StructA1 a1(10);
    module_a::StructA2 a2(20.0, a1);
    module_a::StructA3 a3(false, a1, a2);
    EXPECT_EQ(a3.a1.value, 10);
    EXPECT_EQ(a3.a2.data, 20.0);
    EXPECT_EQ(a3.a2.ref_to_a1.value, 10);
    EXPECT_EQ(a3.flag, false);
}

TEST(MultiModuleTest, test_constants_only_module) {
    EXPECT_EQ(constants_only::C1, 1);
    EXPECT_EQ(constants_only::C2, 2);
    EXPECT_EQ(constants_only::C3, 3);
}

TEST(MultiModuleTest, test_enums_only_module) {
    EXPECT_EQ(enums_only::Color::RED, 0);
    EXPECT_EQ(enums_only::Color::GREEN, 1);
    EXPECT_EQ(enums_only::Color::BLUE, 2);
    EXPECT_EQ(enums_only::Size::SMALL, 0);
    EXPECT_EQ(enums_only::Size::MEDIUM, 1);
    EXPECT_EQ(enums_only::Size::LARGE, 2);
}

TEST(MultiModuleTest, test_cross_module_references) {
    module_a::StructA1 a1(50);
    module_b::StructB1 b1("cross");
    EXPECT_EQ(a1.value, 50);
    EXPECT_EQ(b1.name, "cross");
}

} // namespace
