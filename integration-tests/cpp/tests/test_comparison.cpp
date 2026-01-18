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

#include <algorithm>
#include <unordered_set>
#include <vector>

#include "generated/circular_types.h"
#include "generated/exceptions.h"
#include "generated/structs.h"
#include "generated/unions.h"

namespace {

TEST(ComparisonTest, test_struct_equality) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);
    struct_types::Point p3(5, 10);

    EXPECT_EQ(p1, p2);
    EXPECT_NE(p1, p3);
    EXPECT_TRUE(p1 == p2);
    EXPECT_FALSE(p1 == p3);
}

TEST(ComparisonTest, test_struct_ordering) {
    struct_types::Point p1(1, 2);
    struct_types::Point p2(2, 1);
    struct_types::Point p3(1, 3);

    EXPECT_TRUE(p1 < p2);
    EXPECT_TRUE(p1 < p3);
    EXPECT_FALSE(p2 < p1);
    EXPECT_TRUE(p2 > p1);
    EXPECT_TRUE(p3 > p1);
    EXPECT_TRUE(p1 <= p2);
    EXPECT_TRUE(p1 >= p1);
}

TEST(ComparisonTest, test_struct_sorting) {
    std::vector<struct_types::Point> points;
    points.push_back(struct_types::Point(2, 1));
    points.push_back(struct_types::Point(1, 3));
    points.push_back(struct_types::Point(1, 2));

    std::sort(points.begin(), points.end());

    EXPECT_EQ(points[0], struct_types::Point(1, 2));
    EXPECT_EQ(points[1], struct_types::Point(1, 3));
    EXPECT_EQ(points[2], struct_types::Point(2, 1));
}

TEST(ComparisonTest, test_struct_hashable) {
    struct_types::Point p1(10, 20);
    struct_types::Point p2(10, 20);

    std::hash<struct_types::Point> hasher;
    EXPECT_EQ(hasher(p1), hasher(p2));

    std::unordered_set<struct_types::Point> set;
    set.insert(p1);
    EXPECT_TRUE(set.find(p2) != set.end());
}

TEST(ComparisonTest, test_union_equality) {
    union_types::IntOrString u1;
    u1.int_val(42);
    union_types::IntOrString u2;
    u2.int_val(42);
    union_types::IntOrString u3;
    u3.int_val(99);

    EXPECT_EQ(u1, u2);
    EXPECT_NE(u1, u3);

    union_types::IntOrString u4;
    u4.str_val("hello");
    union_types::IntOrString u5;
    u5.str_val("hello");

    EXPECT_EQ(u4, u5);
    EXPECT_NE(u1, u4);
}

TEST(ComparisonTest, test_exception_equality) {
    exception_types::SimpleError e1(100, "error");
    exception_types::SimpleError e2(100, "error");
    exception_types::SimpleError e3(200, "different");

    EXPECT_EQ(e1, e2);
    EXPECT_NE(e1, e3);

    EXPECT_EQ(e1.error_code, e2.error_code);
    EXPECT_EQ(e1.message, e2.message);
    EXPECT_NE(e1.error_code, e3.error_code);
    EXPECT_NE(e1.message, e3.message);
}

TEST(ComparisonTest, test_circular_type_hash) {
    circular_types::TreeNode node;
    node.value = 42;

    std::hash<circular_types::TreeNode> hasher;
    std::size_t hash1 = hasher(node);
    EXPECT_NE(hash1, 0ULL);

    circular_types::TreeNode node2;
    node2.value = 42;
    std::size_t hash2 = hasher(node2);
    EXPECT_EQ(hash1, hash2);
}

} // namespace
