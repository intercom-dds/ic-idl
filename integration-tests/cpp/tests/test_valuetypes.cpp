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

#include "generated/valuetypes.h"

namespace {

TEST(ValuetypesTest, test_valuetype_instantiation) {
    valuetype_types::SimpleValue sv(42, "test");
    EXPECT_EQ(sv.id, 42);
    EXPECT_EQ(sv.name, "test");
}

TEST(ValuetypesTest, test_valuetype_defaults) {
    valuetype_types::SimpleValue sv;
    EXPECT_EQ(sv.id, 0);
    EXPECT_EQ(sv.name, "");
}

TEST(ValuetypesTest, test_valuetype_inheritance) {
    valuetype_types::DerivedValue dv(1, "base", "derived");
    EXPECT_EQ(dv.id, 1);
    EXPECT_EQ(dv.name, "base");
    EXPECT_EQ(dv.description, "derived");

    valuetype_types::SimpleValue* sv = &dv;
    EXPECT_EQ(sv->id, 1);
    EXPECT_EQ(sv->name, "base");
}

TEST(ValuetypesTest, test_valuetype_empty) {
    valuetype_types::Empty e;
    EXPECT_EQ(e, e);
}

TEST(ValuetypesTest, test_valuetype_with_sequence) {
    std::vector<int32_t> nums = {1, 2, 3, 4, 5};
    std::vector<std::string> names = {"a", "b", "c"};
    valuetype_types::WithSequence ws(nums, names);

    EXPECT_EQ(ws.numbers.size(), 5);
    EXPECT_EQ(ws.names.size(), 3);
    EXPECT_EQ(ws.numbers[0], 1);
    EXPECT_EQ(ws.names[1], "b");
}

TEST(ValuetypesTest, test_valuetype_equality) {
    valuetype_types::SimpleValue v1(10, "test");
    valuetype_types::SimpleValue v2(10, "test");
    valuetype_types::SimpleValue v3(20, "other");

    EXPECT_EQ(v1, v2);
    EXPECT_NE(v1, v3);
}

TEST(ValuetypesTest, test_valuetype_supports_interface) {
    valuetype_types::IdentifiableValue iv(123, "data");
    EXPECT_EQ(iv.id, 123);
    EXPECT_EQ(iv.data, "data");
}

TEST(ValuetypesTest, test_valuetype_supports_named) {
    valuetype_types::NamedValue nv("test_name", 456);
    EXPECT_EQ(nv.name, "test_name");
    EXPECT_EQ(nv.value, 456);
}

TEST(ValuetypesTest, test_valuetype_inheritance_and_supports) {
    valuetype_types::FullValue fv(1, "name", "extra");
    EXPECT_EQ(fv.id, 1);
    EXPECT_EQ(fv.name, "name");
    EXPECT_EQ(fv.extra, "extra");

    valuetype_types::SimpleValue* sv = &fv;
    EXPECT_EQ(sv->id, 1);
    EXPECT_EQ(sv->name, "name");
}

TEST(ValuetypesTest, test_valuetype_field_types) {
    static_assert(
        std::is_same<decltype(valuetype_types::SimpleValue::id), int32_t>::value,
        "id should be int32_t"
    );
    static_assert(
        std::is_same<decltype(valuetype_types::SimpleValue::name), std::string>::value,
        "name should be std::string"
    );
}

TEST(ValuetypesTest, test_valuetype_sequence_field_types) {
    static_assert(
        std::is_same<decltype(valuetype_types::WithSequence::numbers), std::vector<int32_t>>::value,
        "numbers should be std::vector<int32_t>"
    );
    static_assert(
        std::is_same<decltype(valuetype_types::WithSequence::names), std::vector<std::string>>::
            value,
        "names should be std::vector<std::string>"
    );
}

TEST(ValuetypesTest, test_valuetype_derived_field_types) {
    static_assert(
        std::is_same<decltype(valuetype_types::DerivedValue::description), std::string>::value,
        "description should be std::string"
    );
    static_assert(
        std::is_base_of<valuetype_types::SimpleValue, valuetype_types::DerivedValue>::value,
        "DerivedValue should inherit from SimpleValue"
    );
}

TEST(ValuetypesTest, test_valuetype_comparison_operators) {
    valuetype_types::SimpleValue v1(10, "test");
    valuetype_types::SimpleValue v2(10, "test");
    valuetype_types::SimpleValue v3(5, "other");
    valuetype_types::SimpleValue v4(10, "zzz");

    EXPECT_TRUE(v1 == v2);
    EXPECT_FALSE(v1 == v3);
    EXPECT_TRUE(v1 != v3);
    EXPECT_TRUE(v3 < v1);
    EXPECT_TRUE(v1 > v3);
    EXPECT_TRUE(v1 < v4);
    EXPECT_TRUE(v1 <= v2);
    EXPECT_TRUE(v1 >= v2);
}

} // namespace
