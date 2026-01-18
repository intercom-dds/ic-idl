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

#include "generated/structs.h"

namespace {

TEST(StructsTest, test_point_instantiation) {
    struct_types::Point p(10, 20);
    EXPECT_EQ(p.x, 10);
    EXPECT_EQ(p.y, 20);
}

TEST(StructsTest, test_point_defaults) {
    struct_types::Point p;
    EXPECT_EQ(p.x, 0);
    EXPECT_EQ(p.y, 0);
}

TEST(StructsTest, test_point_field_modification) {
    struct_types::Point p(5, 10);
    p.x = 100;
    p.y = 200;
    EXPECT_EQ(p.x, 100);
    EXPECT_EQ(p.y, 200);
}

TEST(StructsTest, test_point3d_inheritance) {
    struct_types::Point3D p3d(1, 2, 3);
    EXPECT_EQ(p3d.x, 1);
    EXPECT_EQ(p3d.y, 2);
    EXPECT_EQ(p3d.z, 3);

    struct_types::Point* p = &p3d;
    EXPECT_EQ(p->x, 1);
    EXPECT_EQ(p->y, 2);
}

TEST(StructsTest, test_nested_struct) {
    struct_types::Point tl(0, 0);
    struct_types::Point br(100, 100);
    struct_types::Rectangle rect(tl, br);
    EXPECT_EQ(rect.top_left.x, 0);
    EXPECT_EQ(rect.top_left.y, 0);
    EXPECT_EQ(rect.bottom_right.x, 100);
    EXPECT_EQ(rect.bottom_right.y, 100);
}

TEST(StructsTest, test_all_primitives) {
    struct_types::AllPrimitives p(
        true, 255, -100, 1000, -50000, 100000, -9999999999LL, 9999999999ULL, 3.14f, 2.71828, "hello"
    );
    EXPECT_EQ(p.bool_val, true);
    EXPECT_EQ(p.byte_val, 255);
    EXPECT_EQ(p.short_val, -100);
    EXPECT_EQ(p.ushort_val, 1000);
    EXPECT_EQ(p.long_val, -50000);
    EXPECT_EQ(p.ulong_val, 100000U);
    EXPECT_EQ(p.longlong_val, -9999999999LL);
    EXPECT_EQ(p.ulonglong_val, 9999999999ULL);
    EXPECT_FLOAT_EQ(p.float_val, 3.14f);
    EXPECT_DOUBLE_EQ(p.double_val, 2.71828);
    EXPECT_EQ(p.string_val, "hello");
}

TEST(StructsTest, test_struct_with_sequence) {
    struct_types::WithSequence s(
        ::std::vector<int32_t>{1, 2, 3}, ::std::vector<::std::string>{"a", "b"}
    );
    EXPECT_EQ(s.numbers.size(), 3U);
    EXPECT_EQ(s.numbers[0], 1);
    EXPECT_EQ(s.numbers[1], 2);
    EXPECT_EQ(s.numbers[2], 3);
    EXPECT_EQ(s.names.size(), 2U);
    EXPECT_EQ(s.names[0], "a");
    EXPECT_EQ(s.names[1], "b");
}

TEST(StructsTest, test_struct_with_array) {
    struct_types::WithArray w(::std::array<int32_t, 5>{1, 2, 3, 4, 5});
    EXPECT_EQ(w.fixed_numbers.size(), 5U);
    EXPECT_EQ(w.fixed_numbers[0], 1);
    EXPECT_EQ(w.fixed_numbers[4], 5);
}

TEST(StructsTest, test_struct_with_map) {
    ::std::map<::std::string, int32_t> map;
    map["one"] = 1;
    map["two"] = 2;

    struct_types::WithMap m(map);
    EXPECT_EQ(m.string_to_int.size(), 2U);
    EXPECT_EQ(m.string_to_int.at("one"), 1);
    EXPECT_EQ(m.string_to_int.at("two"), 2);
}

TEST(StructsTest, test_multi_level_inheritance) {
    struct_types::Point4D p4d(1, 2, 3, 4);
    EXPECT_EQ(p4d.x, 1);
    EXPECT_EQ(p4d.y, 2);
    EXPECT_EQ(p4d.z, 3);
    EXPECT_EQ(p4d.w, 4);

    struct_types::Point3D* p3d = &p4d;
    EXPECT_EQ(p3d->x, 1);
    EXPECT_EQ(p3d->y, 2);
    EXPECT_EQ(p3d->z, 3);

    struct_types::Point* p = &p4d;
    EXPECT_EQ(p->x, 1);
    EXPECT_EQ(p->y, 2);
}

TEST(StructsTest, test_empty_struct) {
    struct_types::Empty e;
    (void)e;
}

TEST(StructsTest, test_all_primitives_defaults) {
    struct_types::AllPrimitives p;
    EXPECT_EQ(p.bool_val, false);
    EXPECT_EQ(p.byte_val, 0);
    EXPECT_EQ(p.short_val, 0);
    EXPECT_EQ(p.ushort_val, 0);
    EXPECT_EQ(p.long_val, 0);
    EXPECT_EQ(p.ulong_val, 0U);
    EXPECT_EQ(p.longlong_val, 0);
    EXPECT_EQ(p.ulonglong_val, 0ULL);
    EXPECT_FLOAT_EQ(p.float_val, 0.0f);
    EXPECT_DOUBLE_EQ(p.double_val, 0.0);
    EXPECT_EQ(p.string_val, "");
}

TEST(StructsTest, test_point_copy_constructor) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(p1);
    EXPECT_EQ(p2.x, 10);
    EXPECT_EQ(p2.y, 20);

    p1.x = 30;
    EXPECT_EQ(p1.x, 30);
    EXPECT_EQ(p2.x, 10);
}

TEST(StructsTest, test_point_assignment) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2;
    p2 = p1;
    EXPECT_EQ(p2.x, 10);
    EXPECT_EQ(p2.y, 20);

    p1.x = 30;
    EXPECT_EQ(p1.x, 30);
    EXPECT_EQ(p2.x, 10);
}

TEST(StructsTest, test_point_equality) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);
    struct_types::Point p3(10, 30);

    EXPECT_TRUE(p1 == p2);
    EXPECT_FALSE(p1 == p3);
}

TEST(StructsTest, test_point_inequality) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);
    struct_types::Point p3(10, 30);

    EXPECT_FALSE(p1 != p2);
    EXPECT_TRUE(p1 != p3);
}

TEST(StructsTest, test_point_less_than) {
    struct_types::Point p1(5, 10);
    struct_types::Point p2(5, 20);
    struct_types::Point p3(10, 5);
    struct_types::Point p4(5, 10);

    EXPECT_TRUE(p1 < p2);
    EXPECT_TRUE(p1 < p3);
    EXPECT_FALSE(p2 < p1);
    EXPECT_FALSE(p1 < p4);
}

TEST(StructsTest, test_struct_move_semantics) {
    struct_types::Rectangle r1(struct_types::Point(0, 0), struct_types::Point(100, 100));

    struct_types::Rectangle r2(::std::move(r1));
    EXPECT_EQ(r2.top_left.x, 0);
    EXPECT_EQ(r2.bottom_right.x, 100);

    struct_types::Rectangle r3;
    r3 = ::std::move(r2);
    EXPECT_EQ(r3.top_left.x, 0);
    EXPECT_EQ(r3.bottom_right.x, 100);
}

TEST(StructsTest, test_nested_struct_deep_copy) {
    struct_types::Point tl(0, 0);
    struct_types::Point br(100, 100);
    struct_types::Rectangle r1(tl, br);

    struct_types::Rectangle r2(r1);

    r1.top_left.x = 50;
    EXPECT_EQ(r1.top_left.x, 50);
    EXPECT_EQ(r2.top_left.x, 0);
}

TEST(StructsTest, test_sequence_field_operations) {
    struct_types::WithSequence s;

    s.numbers.push_back(1);
    s.numbers.push_back(2);
    s.numbers.push_back(3);
    EXPECT_EQ(s.numbers.size(), 3U);
    EXPECT_EQ(s.numbers[0], 1);
    EXPECT_EQ(s.numbers[2], 3);

    s.numbers.erase(s.numbers.begin() + 1);
    EXPECT_EQ(s.numbers.size(), 2U);
    EXPECT_EQ(s.numbers[0], 1);
    EXPECT_EQ(s.numbers[1], 3);

    s.names.push_back("hello");
    s.names.push_back("world");
    EXPECT_EQ(s.names.size(), 2U);
    EXPECT_EQ(s.names[0], "hello");
    EXPECT_EQ(s.names[1], "world");
}

TEST(StructsTest, test_map_field_operations) {
    struct_types::WithMap m;

    m.string_to_int["one"] = 1;
    m.string_to_int["two"] = 2;
    m.string_to_int["three"] = 3;

    EXPECT_EQ(m.string_to_int.size(), 3U);
    EXPECT_EQ(m.string_to_int["one"], 1);
    EXPECT_EQ(m.string_to_int["three"], 3);

    m.string_to_int.erase("two");
    EXPECT_EQ(m.string_to_int.size(), 2U);
    EXPECT_EQ(m.string_to_int.count("two"), 0U);
}

TEST(StructsTest, test_constructor_with_all_fields) {
    struct_types::Point p(42, 84);
    EXPECT_EQ(p.x, 42);
    EXPECT_EQ(p.y, 84);

    struct_types::Point3D p3d(1, 2, 3);
    EXPECT_EQ(p3d.x, 1);
    EXPECT_EQ(p3d.y, 2);
    EXPECT_EQ(p3d.z, 3);

    struct_types::Point4D p4d(10, 20, 30, 40);
    EXPECT_EQ(p4d.x, 10);
    EXPECT_EQ(p4d.y, 20);
    EXPECT_EQ(p4d.z, 30);
    EXPECT_EQ(p4d.w, 40);
}

TEST(StructsTest, test_struct_with_defaults) {
    struct_types::WithDefaults w;
    EXPECT_EQ(w.count, 0);
    EXPECT_EQ(w.name, "");
    EXPECT_DOUBLE_EQ(w.value, 0.0);

    struct_types::WithDefaults w2(42, "test", 3.14);
    EXPECT_EQ(w2.count, 42);
    EXPECT_EQ(w2.name, "test");
    EXPECT_DOUBLE_EQ(w2.value, 3.14);
}

} // namespace
