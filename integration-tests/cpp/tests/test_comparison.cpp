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

#include <algorithm>
#include <unordered_set>
#include <vector>

#include "circular_types.h"
#include "exceptions.h"
#include "structs.h"
#include "unions.h"

TEST_CASE("struct_equality" * doctest::test_suite("comparison")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);
    struct_types::Point p3(5, 10);

    CHECK(p1 == p2);
    CHECK(p1 != p3);
    CHECK_FALSE(p1 == p3);
}

TEST_CASE("struct_ordering" * doctest::test_suite("comparison")) {
    struct_types::Point p1(1, 2);
    struct_types::Point p2(2, 1);
    struct_types::Point p3(1, 3);

    CHECK(p1 < p2);
    CHECK(p1 < p3);
    CHECK_FALSE(p2 < p1);
    CHECK(p2 > p1);
    CHECK(p3 > p1);
    CHECK(p1 <= p2);
    CHECK(p1 >= p1);
}

TEST_CASE("struct_sorting" * doctest::test_suite("comparison")) {
    std::vector<struct_types::Point> points;
    points.push_back(struct_types::Point(2, 1));
    points.push_back(struct_types::Point(1, 3));
    points.push_back(struct_types::Point(1, 2));

    std::sort(points.begin(), points.end());

    CHECK(points[0] == struct_types::Point(1, 2));
    CHECK(points[1] == struct_types::Point(1, 3));
    CHECK(points[2] == struct_types::Point(2, 1));
}

TEST_CASE("struct_hashable" * doctest::test_suite("comparison")) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);

    std::hash<struct_types::Point> hasher;
    CHECK(hasher(p1) == hasher(p2));

    std::unordered_set<struct_types::Point> set;
    set.insert(p1);
    CHECK(set.find(p2) != set.end());
}

TEST_CASE("union_equality" * doctest::test_suite("comparison")) {
    union_types::IntOrString u1;
    u1.int_val(42);
    union_types::IntOrString u2;
    u2.int_val(42);
    union_types::IntOrString u3;
    u3.int_val(99);

    CHECK(u1 == u2);
    CHECK(u1 != u3);

    union_types::IntOrString u4;
    u4.str_val("hello");
    union_types::IntOrString u5;
    u5.str_val("hello");

    CHECK(u4 == u5);
    CHECK(u1 != u4);
}

TEST_CASE("union_sorting" * doctest::test_suite("comparison")) {
    std::vector<union_types::IntOrString> unions;

    union_types::IntOrString u1;
    u1.int_val(50);
    unions.push_back(u1);

    union_types::IntOrString u2;
    u2.int_val(10);
    unions.push_back(u2);

    union_types::IntOrString u3;
    u3.int_val(30);
    unions.push_back(u3);

    std::sort(unions.begin(), unions.end());

    CHECK(unions[0].int_val() == 10);
    CHECK(unions[1].int_val() == 30);
    CHECK(unions[2].int_val() == 50);
}

TEST_CASE("union_hashable" * doctest::test_suite("comparison")) {
    union_types::IntOrString u1;
    u1.int_val(42);
    union_types::IntOrString u2;
    u2.int_val(42);

    std::hash<union_types::IntOrString> hasher;
    CHECK(hasher(u1) == hasher(u2));

    std::unordered_set<union_types::IntOrString> set;
    set.insert(u1);
    CHECK(set.find(u2) != set.end());

    union_types::IntOrString u3;
    u3.str_val("test");
    set.insert(u3);
    CHECK(set.size() == 2);
}

TEST_CASE("exception_equality" * doctest::test_suite("comparison")) {
    exception_types::SimpleError e1(100, "error");
    exception_types::SimpleError e2(100, "error");
    exception_types::SimpleError e3(200, "different");

    CHECK(e1 == e2);
    CHECK(e1 != e3);

    CHECK(e1.error_code == e2.error_code);
    CHECK(e1.message == e2.message);
    CHECK(e1.error_code != e3.error_code);
    CHECK(e1.message != e3.message);
}

TEST_CASE("exception_sorting" * doctest::test_suite("comparison")) {
    std::vector<exception_types::SimpleError> errors;
    errors.push_back(exception_types::SimpleError(500, "server error"));
    errors.push_back(exception_types::SimpleError(100, "continue"));
    errors.push_back(exception_types::SimpleError(404, "not found"));

    std::sort(errors.begin(), errors.end());

    CHECK(errors[0].error_code == 100);
    CHECK(errors[1].error_code == 404);
    CHECK(errors[2].error_code == 500);
}

TEST_CASE("exception_hashable" * doctest::test_suite("comparison")) {
    exception_types::SimpleError e1(404, "not found");
    exception_types::SimpleError e2(404, "not found");

    std::hash<exception_types::SimpleError> hasher;
    CHECK(hasher(e1) == hasher(e2));

    std::unordered_set<exception_types::SimpleError> set;
    set.insert(e1);
    CHECK(set.find(e2) != set.end());

    exception_types::SimpleError e3(500, "server error");
    set.insert(e3);
    CHECK(set.size() == 2);
}

TEST_CASE("circular_type_hash" * doctest::test_suite("comparison")) {
    circular_types::TreeNode node;
    node.value = 42;

    std::hash<circular_types::TreeNode> hasher;
    std::size_t hash1 = hasher(node);
    CHECK(hash1 != 0ULL);

    circular_types::TreeNode node2;
    node2.value = 42;
    std::size_t hash2 = hasher(node2);
    CHECK(hash1 == hash2);
}
