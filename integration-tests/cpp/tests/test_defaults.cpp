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

#include "generated/defaults.h"

namespace {

TEST(DefaultsTest, test_const_string_values) {
    EXPECT_STREQ(default_types::DEFAULT_NAME, "unnamed");
    EXPECT_EQ(default_types::DEFAULT_COUNT, 100);
    EXPECT_DOUBLE_EQ(default_types::DEFAULT_RATE, 0.5);
}

TEST(DefaultsTest, test_struct_const_initializer) {
    EXPECT_EQ(default_types::DEFAULT_INNER.x, 10);
    EXPECT_EQ(default_types::DEFAULT_INNER.y, "default");
    EXPECT_EQ(default_types::NESTED_INNER.x, 99);
    EXPECT_EQ(default_types::NESTED_INNER.y, "nested");
}

TEST(DefaultsTest, test_optional_fields_are_none) {
    default_types::OptionalFields opt;
    EXPECT_FALSE(opt.maybe_int.has_value());
    EXPECT_FALSE(opt.maybe_string.has_value());
    EXPECT_FALSE(opt.maybe_struct.has_value());
}

TEST(DefaultsTest, test_optional_fields_type_annotations) {
    default_types::OptionalFields opt;
    static_assert(
        std::is_same<decltype(opt.maybe_int), std::optional<int32_t>>::value,
        "maybe_int should be std::optional<int32_t>"
    );
    static_assert(
        std::is_same<decltype(opt.maybe_string), std::optional<std::string>>::value,
        "maybe_string should be std::optional<std::string>"
    );
    static_assert(
        std::is_same<decltype(opt.maybe_struct), std::optional<default_types::Inner>>::value,
        "maybe_struct should be std::optional<Inner>"
    );
}

TEST(DefaultsTest, test_optional_fields_can_be_set) {
    default_types::OptionalFields opt;
    opt.maybe_int = 42;
    opt.maybe_string = "test";
    opt.maybe_struct = default_types::Inner(10, "hello");
    EXPECT_TRUE(opt.maybe_int.has_value());
    EXPECT_EQ(opt.maybe_int.value(), 42);
    EXPECT_TRUE(opt.maybe_string.has_value());
    EXPECT_EQ(opt.maybe_string.value(), "test");
    EXPECT_TRUE(opt.maybe_struct.has_value());
    EXPECT_EQ(opt.maybe_struct.value().x, 10);
    EXPECT_EQ(opt.maybe_struct.value().y, "hello");
}

TEST(DefaultsTest, test_enum_default_literal_exists) {
    EXPECT_EQ(default_types::Priority::LOW, 0);
    EXPECT_EQ(default_types::Priority::MEDIUM, 1);
    EXPECT_EQ(default_types::Priority::HIGH, 2);
}

TEST(DefaultsTest, test_primitive_bool_default) {
    default_types::PrimitiveDefaults p;
    EXPECT_EQ(p.bool_empty, false);
    EXPECT_EQ(p.bool_true, true);
    EXPECT_EQ(p.bool_false, false);
}

TEST(DefaultsTest, test_primitive_int_default) {
    default_types::PrimitiveDefaults p;
    EXPECT_EQ(p.int_empty, 0);
    EXPECT_EQ(p.int_value, 42);
    EXPECT_EQ(p.int_negative, -100);
}

TEST(DefaultsTest, test_primitive_float_default) {
    default_types::PrimitiveDefaults p;
    EXPECT_DOUBLE_EQ(p.float_empty, 0.0);
    EXPECT_NEAR(p.float_value, 3.14159, 0.00001);
    EXPECT_DOUBLE_EQ(p.float_negative, -0.5);
}

TEST(DefaultsTest, test_primitive_string_default) {
    default_types::PrimitiveDefaults p;
    EXPECT_EQ(p.string_empty, "");
    EXPECT_EQ(p.string_value, "hello");
    EXPECT_EQ(p.string_from_const, "unnamed");
}

TEST(DefaultsTest, test_array_default_values) {
    default_types::ArrayDefaults a;
    EXPECT_EQ(a.array_empty.size(), 3U);
    EXPECT_EQ(a.array_empty[0], 0);
    EXPECT_EQ(a.array_empty[1], 0);
    EXPECT_EQ(a.array_empty[2], 0);
    EXPECT_EQ(a.array_values.size(), 3U);
    EXPECT_EQ(a.array_values[0], 1);
    EXPECT_EQ(a.array_values[1], 2);
    EXPECT_EQ(a.array_values[2], 3);
    EXPECT_EQ(a.array_partial.size(), 2U);
    EXPECT_EQ(a.array_partial[0], 10);
    EXPECT_EQ(a.array_partial[1], 20);
    EXPECT_EQ(a.string_array_empty.size(), 2U);
    EXPECT_EQ(a.string_array_empty[0], "");
    EXPECT_EQ(a.string_array_empty[1], "");
    EXPECT_EQ(a.string_array_values.size(), 2U);
    EXPECT_EQ(a.string_array_values[0], "foo");
    EXPECT_EQ(a.string_array_values[1], "bar");
}

TEST(DefaultsTest, test_sequence_default_values) {
    default_types::SequenceDefaults s;
    EXPECT_EQ(s.seq_empty.size(), 0U);
    EXPECT_EQ(s.seq_values.size(), 5U);
    EXPECT_EQ(s.seq_values[0], 1);
    EXPECT_EQ(s.seq_values[1], 2);
    EXPECT_EQ(s.seq_values[2], 3);
    EXPECT_EQ(s.seq_values[3], 4);
    EXPECT_EQ(s.seq_values[4], 5);
    EXPECT_EQ(s.string_seq_empty.size(), 0U);
    EXPECT_EQ(s.string_seq_values.size(), 3U);
    EXPECT_EQ(s.string_seq_values[0], "a");
    EXPECT_EQ(s.string_seq_values[1], "b");
    EXPECT_EQ(s.string_seq_values[2], "c");
}

TEST(DefaultsTest, test_map_default_values) {
    default_types::MapDefaults m;
    EXPECT_EQ(m.map_empty.size(), 0U);
    EXPECT_EQ(m.map_values.size(), 2U);
    EXPECT_EQ(m.map_values.at("one"), 1);
    EXPECT_EQ(m.map_values.at("two"), 2);
    EXPECT_EQ(m.reverse_map_empty.size(), 0U);
    EXPECT_EQ(m.reverse_map_values.size(), 2U);
    EXPECT_EQ(m.reverse_map_values.at(1), "one");
    EXPECT_EQ(m.reverse_map_values.at(2), "two");
}

TEST(DefaultsTest, test_enum_field_default) {
    default_types::EnumDefaults e;
    EXPECT_EQ(e.priority_high, default_types::Priority::HIGH);
}

} // namespace
