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

#include <type_traits>

#include "generated/enums.h"

namespace {

TEST(EnumsTest, test_enum_members_exist) {
    enum_types::Color red = enum_types::RED;
    enum_types::Color green = enum_types::GREEN;
    enum_types::Color blue = enum_types::BLUE;
    (void)red;
    (void)green;
    (void)blue;
}

TEST(EnumsTest, test_enum_is_enum_type) {
    EXPECT_TRUE(std::is_enum<enum_types::Color>::value);
    EXPECT_TRUE(std::is_enum<enum_types::Status>::value);
    EXPECT_TRUE(std::is_enum<enum_types::GappedEnum>::value);
    EXPECT_TRUE(std::is_enum<enum_types::NegativeEnum>::value);
    EXPECT_TRUE(std::is_enum<enum_types::MixedEnum>::value);
}

TEST(EnumsTest, test_enum_auto_values) {
    EXPECT_EQ(static_cast<int32_t>(enum_types::RED), 0);
    EXPECT_EQ(static_cast<int32_t>(enum_types::GREEN), 1);
    EXPECT_EQ(static_cast<int32_t>(enum_types::BLUE), 2);
}

TEST(EnumsTest, test_enum_explicit_values) {
    EXPECT_EQ(static_cast<int32_t>(enum_types::OK), 0);
    EXPECT_EQ(static_cast<int32_t>(enum_types::WARNING), 100);
    EXPECT_EQ(static_cast<int32_t>(enum_types::ERROR), 200);
}

TEST(EnumsTest, test_enum_iteration) {
    GTEST_SKIP() << "C++ does not support enum iteration without additional reflection libraries";
}

TEST(EnumsTest, test_enum_comparison) {
    EXPECT_TRUE(enum_types::RED == enum_types::RED);
    EXPECT_FALSE(enum_types::RED == enum_types::BLUE);
    EXPECT_TRUE(enum_types::RED != enum_types::BLUE);
    EXPECT_FALSE(enum_types::RED != enum_types::RED);

    EXPECT_TRUE(enum_types::WARNING == enum_types::WARNING);
    EXPECT_TRUE(enum_types::OK != enum_types::ERROR);
}

TEST(EnumsTest, test_enum_by_value) {
    enum_types::Color c0 = static_cast<enum_types::Color>(0);
    enum_types::Color c1 = static_cast<enum_types::Color>(1);
    enum_types::Color c2 = static_cast<enum_types::Color>(2);

    EXPECT_EQ(c0, enum_types::RED);
    EXPECT_EQ(c1, enum_types::GREEN);
    EXPECT_EQ(c2, enum_types::BLUE);

    enum_types::Status s100 = static_cast<enum_types::Status>(100);
    EXPECT_EQ(s100, enum_types::WARNING);
}

TEST(EnumsTest, test_enum_by_name) {
    GTEST_SKIP(
    ) << "C++ does not support enum lookup by name without additional reflection libraries";
}

TEST(EnumsTest, test_enum_name_property) {
    GTEST_SKIP() << "C++ enums do not have a name property without additional reflection libraries";
}

TEST(EnumsTest, test_enum_gapped_values) {
    EXPECT_EQ(static_cast<int32_t>(enum_types::FIRST), 0);
    EXPECT_EQ(static_cast<int32_t>(enum_types::SECOND), 5);
    EXPECT_EQ(static_cast<int32_t>(enum_types::THIRD), 10);
    EXPECT_EQ(static_cast<int32_t>(enum_types::FOURTH), 100);

    enum_types::GappedEnum g5 = static_cast<enum_types::GappedEnum>(5);
    EXPECT_EQ(g5, enum_types::SECOND);

    enum_types::GappedEnum g100 = static_cast<enum_types::GappedEnum>(100);
    EXPECT_EQ(g100, enum_types::FOURTH);
}

TEST(EnumsTest, test_enum_negative_values) {
    EXPECT_EQ(static_cast<int32_t>(enum_types::NEG_TWO), -2);
    EXPECT_EQ(static_cast<int32_t>(enum_types::NEG_ONE), -1);
    EXPECT_EQ(static_cast<int32_t>(enum_types::ZERO), 0);
    EXPECT_EQ(static_cast<int32_t>(enum_types::POS_ONE), 1);

    enum_types::NegativeEnum neg = static_cast<enum_types::NegativeEnum>(-2);
    EXPECT_EQ(neg, enum_types::NEG_TWO);

    enum_types::NegativeEnum zero = static_cast<enum_types::NegativeEnum>(0);
    EXPECT_EQ(zero, enum_types::ZERO);
}

TEST(EnumsTest, test_enum_const_from_enum_value) {
    EXPECT_EQ(enum_types::ENUM_CONST, 100);
    EXPECT_EQ(enum_types::ENUM_CONST, static_cast<int32_t>(enum_types::WARNING));
}

TEST(EnumsTest, test_enum_mixed_explicit_auto) {
    EXPECT_EQ(static_cast<int32_t>(enum_types::AUTO_FIRST), 0);
    EXPECT_EQ(static_cast<int32_t>(enum_types::EXPLICIT_TEN), 10);
    EXPECT_EQ(static_cast<int32_t>(enum_types::AUTO_ELEVEN), 11);
    EXPECT_EQ(static_cast<int32_t>(enum_types::EXPLICIT_HUNDRED), 100);
    EXPECT_EQ(static_cast<int32_t>(enum_types::AUTO_HUNDRED_ONE), 101);

    enum_types::MixedEnum m11 = static_cast<enum_types::MixedEnum>(11);
    EXPECT_EQ(m11, enum_types::AUTO_ELEVEN);

    enum_types::MixedEnum m101 = static_cast<enum_types::MixedEnum>(101);
    EXPECT_EQ(m101, enum_types::AUTO_HUNDRED_ONE);
}

} // namespace
