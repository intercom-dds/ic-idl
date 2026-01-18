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

#include "generated/unions.h"

namespace {

TEST(UnionsTest, test_union_int_variant) {
    union_types::IntOrString u;
    u.int_val(42);
    EXPECT_EQ(u._d(), 1);
    EXPECT_EQ(u.int_val(), 42);
}

TEST(UnionsTest, test_union_string_variant) {
    union_types::IntOrString u;
    u.str_val("hello");
    EXPECT_EQ(u._d(), 2);
    EXPECT_EQ(u.str_val(), "hello");
}

TEST(UnionsTest, test_union_wrong_variant_raises) {
    union_types::IntOrString u;
    u.int_val(42);
    EXPECT_THROW(std::ignore = u.str_val(), std::logic_error);

    u.str_val("test");
    EXPECT_THROW(std::ignore = u.int_val(), std::logic_error);
}

TEST(UnionsTest, test_union_enum_discriminator) {
    union_types::TypedValue tv;
    EXPECT_TRUE((std::is_same_v<decltype(tv._d()), union_types::ValueKind>));
}

TEST(UnionsTest, test_union_enum_string_variant) {
    union_types::TypedValue tv;
    tv.string_value("test string");
    EXPECT_EQ(tv._d(), union_types::STRING_KIND);
    EXPECT_EQ(tv.string_value(), "test string");
}

TEST(UnionsTest, test_union_bool_discriminator) {
    union_types::BoolSwitch bs;
    bs.true_val(100);
    EXPECT_TRUE(bs._d());
    EXPECT_EQ(bs.true_val(), 100);

    bs.false_val("false branch");
    EXPECT_FALSE(bs._d());
    EXPECT_EQ(bs.false_val(), "false branch");
}

TEST(UnionsTest, test_union_multi_case) {
    union_types::MultiCase mc;
    mc.small_val(5);
    EXPECT_TRUE(mc._d() == 1);

    mc._d(2);
    EXPECT_EQ(mc._d(), 2);
    EXPECT_EQ(mc.small_val(), 5);
    mc.small_val(10);
    EXPECT_EQ(mc._d(), 2);
    EXPECT_EQ(mc.small_val(), 10);

    mc._d(3);
    EXPECT_EQ(mc._d(), 3);
    EXPECT_EQ(mc.small_val(), 10);

    mc._d(10);
    mc.text_val("test");
    EXPECT_TRUE(mc._d() == 10 || mc._d() == 20);
    EXPECT_EQ(mc.text_val(), "test");
}

TEST(UnionsTest, test_union_default_method) {
    union_types::IntOrString u;
    u.str_val("hello");
    u.default_val(true);
    EXPECT_NE(u._d(), 1);
    EXPECT_NE(u._d(), 2);
    EXPECT_TRUE(u.default_val());
}

TEST(UnionsTest, test_union_discriminator_property) {
    union_types::IntOrString u;
    u.int_val(42);
    EXPECT_EQ(u._d(), 1);

    u.str_val("test");
    EXPECT_EQ(u._d(), 2);

    union_types::TypedValue tv;
    tv.int_value(100);
    EXPECT_EQ(tv._d(), union_types::INT_KIND);
}

TEST(UnionsTest, test_union_equality) {
    union_types::IntOrString u1;
    u1.int_val(42);

    union_types::IntOrString u2;
    u2.int_val(42);

    union_types::IntOrString u3;
    u3.int_val(100);

    EXPECT_TRUE(u1 == u2);
    EXPECT_FALSE(u1 == u3);
    EXPECT_FALSE(u1 != u2);
    EXPECT_TRUE(u1 != u3);
}

TEST(UnionsTest, test_union_default_constructor_with_default_case) {
    union_types::IntOrString u;
    EXPECT_EQ(u._d(), 0);
    EXPECT_FALSE(u.default_val());
}

TEST(UnionsTest, test_union_default_constructor_without_default_case) {
    union_types::MultiCase mc;
    EXPECT_EQ(mc._d(), 0);
    EXPECT_FALSE(mc.flag());
}

TEST(UnionsTest, test_union_default_constructor_enum_discriminator) {
    union_types::TypedValue tv;
    EXPECT_EQ(tv._d(), union_types::INT_KIND);
    EXPECT_EQ(tv.int_value(), 0);
}

TEST(UnionsTest, test_union_default_variant_sets_discriminator) {
    union_types::IntOrString u;
    u.int_val(42);
    EXPECT_EQ(u._d(), 1);

    u.default_val(true);
    EXPECT_EQ(u._d(), 0);
    EXPECT_TRUE(u.default_val());
}

} // namespace
