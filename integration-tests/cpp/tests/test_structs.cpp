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

#include "structs.h"

TEST_CASE("point_instantiation" * doctest::test_suite("structs")) {
    struct_types::Point p(10, 20);
    CHECK(p.x == 10);
    CHECK(p.y == 20);
}

TEST_CASE("point_defaults" * doctest::test_suite("structs")) {
    struct_types::Point p;
    CHECK(p.x == 0);
    CHECK(p.y == 0);
}

TEST_CASE("point_field_modification" * doctest::test_suite("structs")) {
    struct_types::Point p(5, 10);
    p.x = 100;
    p.y = 200;
    CHECK(p.x == 100);
    CHECK(p.y == 200);
}

TEST_CASE("point3d_inheritance" * doctest::test_suite("structs")) {
    struct_types::Point3D p3d(1, 2, 3);
    CHECK(p3d.x == 1);
    CHECK(p3d.y == 2);
    CHECK(p3d.z == 3);

    struct_types::Point* p = &p3d;
    CHECK(p->x == 1);
    CHECK(p->y == 2);
}

TEST_CASE("nested_struct" * doctest::test_suite("structs")) {
    struct_types::Point tl(0, 0);
    struct_types::Point br(100, 100);
    struct_types::Rectangle rect(tl, br);
    CHECK(rect.top_left.x == 0);
    CHECK(rect.top_left.y == 0);
    CHECK(rect.bottom_right.x == 100);
    CHECK(rect.bottom_right.y == 100);
}

TEST_CASE("all_primitives" * doctest::test_suite("structs")) {
    struct_types::AllPrimitives p(
        true, 255, -100, 1000, -50000, 100000, -9999999999LL, 9999999999ULL, 3.14f, 2.71828, "hello"
    );
    CHECK(p.bool_val == true);
    CHECK(p.byte_val == 255);
    CHECK(p.short_val == -100);
    CHECK(p.ushort_val == 1000);
    CHECK(p.long_val == -50000);
    CHECK(p.ulong_val == 100000U);
    CHECK(p.longlong_val == -9999999999LL);
    CHECK(p.ulonglong_val == 9999999999ULL);
    CHECK(p.float_val == doctest::Approx(3.14f));
    CHECK(p.double_val == doctest::Approx(2.71828));
    CHECK(p.string_val == "hello");
}

TEST_CASE("struct_with_sequence" * doctest::test_suite("structs")) {
    struct_types::WithSequence s(
        ::std::vector<int32_t>{1, 2, 3}, ::std::vector<::std::string>{"a", "b"}
    );
    CHECK(s.numbers.size() == 3U);
    CHECK(s.numbers[0] == 1);
    CHECK(s.numbers[1] == 2);
    CHECK(s.numbers[2] == 3);
    CHECK(s.names.size() == 2U);
    CHECK(s.names[0] == "a");
    CHECK(s.names[1] == "b");
}

TEST_CASE("struct_with_array" * doctest::test_suite("structs")) {
    struct_types::WithArray w(::std::array<int32_t, 5>{1, 2, 3, 4, 5});
    CHECK(w.fixed_numbers.size() == 5U);
    CHECK(w.fixed_numbers[0] == 1);
    CHECK(w.fixed_numbers[4] == 5);
}

TEST_CASE("struct_with_map" * doctest::test_suite("structs")) {
    ::std::map<::std::string, int32_t> map;
    map["one"] = 1;
    map["two"] = 2;

    struct_types::WithMap m(map);
    CHECK(m.string_to_int.size() == 2U);
    CHECK(m.string_to_int.at("one") == 1);
    CHECK(m.string_to_int.at("two") == 2);
}

TEST_CASE("multi_level_inheritance" * doctest::test_suite("structs")) {
    struct_types::Point4D p4d(1, 2, 3, 4);
    CHECK(p4d.x == 1);
    CHECK(p4d.y == 2);
    CHECK(p4d.z == 3);
    CHECK(p4d.w == 4);

    struct_types::Point3D* p3d = &p4d;
    CHECK(p3d->x == 1);
    CHECK(p3d->y == 2);
    CHECK(p3d->z == 3);

    struct_types::Point* p = &p4d;
    CHECK(p->x == 1);
    CHECK(p->y == 2);
}

TEST_CASE("empty_struct" * doctest::test_suite("structs")) {
    struct_types::Empty e;
    (void)e;
}

TEST_CASE("all_primitives_defaults" * doctest::test_suite("structs")) {
    struct_types::AllPrimitives p;
    CHECK(p.bool_val == false);
    CHECK(p.byte_val == 0);
    CHECK(p.short_val == 0);
    CHECK(p.ushort_val == 0);
    CHECK(p.long_val == 0);
    CHECK(p.ulong_val == 0U);
    CHECK(p.longlong_val == 0);
    CHECK(p.ulonglong_val == 0ULL);
    CHECK(p.float_val == doctest::Approx(0.0f));
    CHECK(p.double_val == doctest::Approx(0.0));
    CHECK(p.string_val == "");
}

TEST_CASE("point_copy_constructor" * doctest::test_suite("structs")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(p1);
    CHECK(p2.x == 10);
    CHECK(p2.y == 20);

    p1.x = 30;
    CHECK(p1.x == 30);
    CHECK(p2.x == 10);
}

TEST_CASE("point_assignment" * doctest::test_suite("structs")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2;
    p2 = p1;
    CHECK(p2.x == 10);
    CHECK(p2.y == 20);

    p1.x = 30;
    CHECK(p1.x == 30);
    CHECK(p2.x == 10);
}

TEST_CASE("point_equality" * doctest::test_suite("structs")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);
    struct_types::Point p3(10, 30);

    CHECK(p1 == p2);
    CHECK_FALSE(p1 == p3);
}

TEST_CASE("point_inequality" * doctest::test_suite("structs")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);
    struct_types::Point p3(10, 30);

    CHECK_FALSE(p1 != p2);
    CHECK(p1 != p3);
}

TEST_CASE("point_less_than" * doctest::test_suite("structs")) {
    struct_types::Point p1(5, 10);
    struct_types::Point p2(5, 20);
    struct_types::Point p3(10, 5);
    struct_types::Point p4(5, 10);

    CHECK(p1 < p2);
    CHECK(p1 < p3);
    CHECK_FALSE(p2 < p1);
    CHECK_FALSE(p1 < p4);
}

TEST_CASE("struct_move_semantics" * doctest::test_suite("structs")) {
    struct_types::Rectangle r1(struct_types::Point(0, 0), struct_types::Point(100, 100));

    struct_types::Rectangle r2(::std::move(r1));
    CHECK(r2.top_left.x == 0);
    CHECK(r2.bottom_right.x == 100);

    struct_types::Rectangle r3;
    r3 = ::std::move(r2);
    CHECK(r3.top_left.x == 0);
    CHECK(r3.bottom_right.x == 100);
}

TEST_CASE("nested_struct_deep_copy" * doctest::test_suite("structs")) {
    struct_types::Point tl(0, 0);
    struct_types::Point br(100, 100);
    struct_types::Rectangle r1(tl, br);

    struct_types::Rectangle r2(r1);

    r1.top_left.x = 50;
    CHECK(r1.top_left.x == 50);
    CHECK(r2.top_left.x == 0);
}

TEST_CASE("sequence_field_operations" * doctest::test_suite("structs")) {
    struct_types::WithSequence s;

    s.numbers.push_back(1);
    s.numbers.push_back(2);
    s.numbers.push_back(3);
    CHECK(s.numbers.size() == 3U);
    CHECK(s.numbers[0] == 1);
    CHECK(s.numbers[2] == 3);

    s.numbers.erase(s.numbers.begin() + 1);
    CHECK(s.numbers.size() == 2U);
    CHECK(s.numbers[0] == 1);
    CHECK(s.numbers[1] == 3);

    s.names.push_back("hello");
    s.names.push_back("world");
    CHECK(s.names.size() == 2U);
    CHECK(s.names[0] == "hello");
    CHECK(s.names[1] == "world");
}

TEST_CASE("map_field_operations" * doctest::test_suite("structs")) {
    struct_types::WithMap m;

    m.string_to_int["one"] = 1;
    m.string_to_int["two"] = 2;
    m.string_to_int["three"] = 3;

    CHECK(m.string_to_int.size() == 3U);
    CHECK(m.string_to_int["one"] == 1);
    CHECK(m.string_to_int["three"] == 3);

    m.string_to_int.erase("two");
    CHECK(m.string_to_int.size() == 2U);
    CHECK(m.string_to_int.count("two") == 0U);
}

TEST_CASE("constructor_with_all_fields" * doctest::test_suite("structs")) {
    struct_types::Point p(42, 84);
    CHECK(p.x == 42);
    CHECK(p.y == 84);

    struct_types::Point3D p3d(1, 2, 3);
    CHECK(p3d.x == 1);
    CHECK(p3d.y == 2);
    CHECK(p3d.z == 3);

    struct_types::Point4D p4d(10, 20, 30, 40);
    CHECK(p4d.x == 10);
    CHECK(p4d.y == 20);
    CHECK(p4d.z == 30);
    CHECK(p4d.w == 40);
}

TEST_CASE("struct_with_defaults" * doctest::test_suite("structs")) {
    struct_types::WithDefaults w;
    CHECK(w.count == 0);
    CHECK(w.name == "");
    CHECK(w.value == doctest::Approx(0.0));

    struct_types::WithDefaults w2(42, "test", 3.14);
    CHECK(w2.count == 42);
    CHECK(w2.name == "test");
    CHECK(w2.value == doctest::Approx(3.14));
}

TEST_CASE("struct_swap" * doctest::test_suite("structs")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(30, 40);

    using std::swap;
    swap(p1, p2);

    CHECK(p1.x == 30);
    CHECK(p1.y == 40);
    CHECK(p2.x == 10);
    CHECK(p2.y == 20);
}

TEST_CASE("struct_swap_nested" * doctest::test_suite("structs")) {
    struct_types::Rectangle r1(struct_types::Point(0, 0), struct_types::Point(10, 10));
    struct_types::Rectangle r2(struct_types::Point(20, 20), struct_types::Point(30, 30));

    using std::swap;
    swap(r1, r2);

    CHECK(r1.top_left.x == 20);
    CHECK(r1.top_left.y == 20);
    CHECK(r1.bottom_right.x == 30);
    CHECK(r1.bottom_right.y == 30);

    CHECK(r2.top_left.x == 0);
    CHECK(r2.top_left.y == 0);
    CHECK(r2.bottom_right.x == 10);
    CHECK(r2.bottom_right.y == 10);
}
