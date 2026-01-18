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

#include "generated/deep_generics.h"

namespace {

TEST(DeepGenericsTest, test_two_level_seq) {
    std::vector<std::vector<int32_t>> matrix = {{1, 2, 3}, {4, 5, 6}};
    deep_generic_types::TwoLevelSeq seq(matrix);
    EXPECT_EQ(seq.matrix.size(), 2);
    EXPECT_EQ(seq.matrix[0].size(), 3);
    EXPECT_EQ(seq.matrix[0][0], 1);
    EXPECT_EQ(seq.matrix[1][2], 6);
}

TEST(DeepGenericsTest, test_three_level_seq) {
    std::vector<std::vector<std::vector<int32_t>>> cube = {{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}};
    deep_generic_types::ThreeLevelSeq seq(cube);
    EXPECT_EQ(seq.cube.size(), 2);
    EXPECT_EQ(seq.cube[0].size(), 2);
    EXPECT_EQ(seq.cube[0][0].size(), 2);
    EXPECT_EQ(seq.cube[0][0][0], 1);
    EXPECT_EQ(seq.cube[1][1][1], 8);
}

TEST(DeepGenericsTest, test_four_level_deep) {
    std::vector<std::vector<std::vector<std::vector<int32_t>>>> hypercube = {{{{1, 2}}}};
    deep_generic_types::FourLevelDeep deep(hypercube);
    EXPECT_EQ(deep.hypercube.size(), 1);
    EXPECT_EQ(deep.hypercube[0][0][0][0], 1);
    EXPECT_EQ(deep.hypercube[0][0][0][1], 2);
}

TEST(DeepGenericsTest, test_map_of_seq) {
    std::map<std::string, std::vector<int32_t>> indexed = {
        {"first", {1, 2, 3}}, {"second", {4, 5}}
    };
    deep_generic_types::MapOfSeq seq(indexed);
    EXPECT_EQ(seq.indexed_lists.size(), 2);
    EXPECT_EQ(seq.indexed_lists["first"].size(), 3);
    EXPECT_EQ(seq.indexed_lists["first"][0], 1);
    EXPECT_EQ(seq.indexed_lists["second"][1], 5);
}

TEST(DeepGenericsTest, test_seq_of_map) {
    std::vector<std::map<std::string, int32_t>> dicts = {{{"a", 1}, {"b", 2}}, {{"c", 3}}};
    deep_generic_types::SeqOfMap seq(dicts);
    EXPECT_EQ(seq.list_of_dicts.size(), 2);
    EXPECT_EQ(seq.list_of_dicts[0]["a"], 1);
    EXPECT_EQ(seq.list_of_dicts[1]["c"], 3);
}

TEST(DeepGenericsTest, test_map_of_map) {
    std::map<std::string, std::map<std::string, int32_t>> nested = {{"outer", {{"inner", 42}}}};
    deep_generic_types::MapOfMap map(nested);
    EXPECT_EQ(map.nested_dict.size(), 1);
    EXPECT_EQ(map.nested_dict["outer"]["inner"], 42);
}

TEST(DeepGenericsTest, test_map_seq_map) {
    std::map<std::string, std::vector<std::map<std::string, int32_t>>> complex = {
        {"key", {{{"a", 1}, {"b", 2}}}}
    };
    deep_generic_types::MapSeqMap seq(complex);
    EXPECT_EQ(seq.complex_structure.size(), 1);
    EXPECT_EQ(seq.complex_structure["key"][0]["a"], 1);
}

TEST(DeepGenericsTest, test_seq_map_seq) {
    std::vector<std::map<std::string, std::vector<int32_t>>> inverse = {{{"key", {1, 2, 3}}}};
    deep_generic_types::SeqMapSeq seq(inverse);
    EXPECT_EQ(seq.inverse_structure.size(), 1);
    EXPECT_EQ(seq.inverse_structure[0]["key"][1], 2);
}

TEST(DeepGenericsTest, test_point_struct) {
    deep_generic_types::Point p(10, 20);
    EXPECT_EQ(p.x, 10);
    EXPECT_EQ(p.y, 20);
}

TEST(DeepGenericsTest, test_seq_of_points) {
    std::vector<deep_generic_types::Point> points = {
        deep_generic_types::Point(1, 2), deep_generic_types::Point(3, 4)
    };
    deep_generic_types::SeqOfPoints seq(points);
    EXPECT_EQ(seq.points.size(), 2);
    EXPECT_EQ(seq.points[0].x, 1);
    EXPECT_EQ(seq.points[1].y, 4);
}

TEST(DeepGenericsTest, test_map_of_points) {
    std::map<std::string, deep_generic_types::Point> named = {
        {"origin", deep_generic_types::Point(0, 0)}, {"unit", deep_generic_types::Point(1, 1)}
    };
    deep_generic_types::MapOfPoints map(named);
    EXPECT_EQ(map.named_points.size(), 2);
    EXPECT_EQ(map.named_points["origin"].x, 0);
    EXPECT_EQ(map.named_points["unit"].y, 1);
}

TEST(DeepGenericsTest, test_seq_of_seq_of_points) {
    std::vector<std::vector<deep_generic_types::Point>> matrix = {
        {deep_generic_types::Point(1, 2), deep_generic_types::Point(3, 4)},
        {deep_generic_types::Point(5, 6)}
    };
    deep_generic_types::SeqOfSeqOfPoints seq(matrix);
    EXPECT_EQ(seq.point_matrix.size(), 2);
    EXPECT_EQ(seq.point_matrix[0].size(), 2);
    EXPECT_EQ(seq.point_matrix[0][0].x, 1);
    EXPECT_EQ(seq.point_matrix[1][0].y, 6);
}

TEST(DeepGenericsTest, test_map_of_seq_of_points) {
    std::map<std::string, std::vector<deep_generic_types::Point>> lists = {
        {"path1", {deep_generic_types::Point(0, 0), deep_generic_types::Point(1, 1)}},
        {"path2", {deep_generic_types::Point(2, 2)}}
    };
    deep_generic_types::MapOfSeqOfPoints map(lists);
    EXPECT_EQ(map.point_lists.size(), 2);
    EXPECT_EQ(map.point_lists["path1"].size(), 2);
    EXPECT_EQ(map.point_lists["path1"][0].x, 0);
    EXPECT_EQ(map.point_lists["path2"][0].y, 2);
}

TEST(DeepGenericsTest, test_typedef_aliases_exist) {
    deep_generic_types::IntList list = {1, 2, 3};
    EXPECT_EQ(list.size(), 3);

    deep_generic_types::IntMatrix matrix = {{1, 2}, {3, 4}};
    EXPECT_EQ(matrix.size(), 2);

    deep_generic_types::NamedMatrices named = {{"mat1", {{1, 2}, {3, 4}}}};
    EXPECT_EQ(named.size(), 1);
}

TEST(DeepGenericsTest, test_using_typedef_chain) {
    deep_generic_types::NamedMatrices data = {
        {"matrix1", {{1, 2, 3}, {4, 5, 6}}}, {"matrix2", {{7, 8}}}
    };
    deep_generic_types::UsingTypedefChain chain(data);
    EXPECT_EQ(chain.data.size(), 2);
    EXPECT_EQ(chain.data["matrix1"].size(), 2);
    EXPECT_EQ(chain.data["matrix1"][0][2], 3);
}

TEST(DeepGenericsTest, test_array_of_seq) {
    std::array<std::vector<int32_t>, 3> items = {
        std::vector<int32_t>{1, 2}, std::vector<int32_t>{3, 4, 5}, std::vector<int32_t>{6}
    };
    deep_generic_types::ArrayOfSeq seq(items);
    EXPECT_EQ(seq.items.size(), 3);
    EXPECT_EQ(seq.items[0].size(), 2);
    EXPECT_EQ(seq.items[1].size(), 3);
    EXPECT_EQ(seq.items[0][0], 1);
    EXPECT_EQ(seq.items[2][0], 6);
}

TEST(DeepGenericsTest, test_three_ints_typedef) {
    deep_generic_types::ThreeInts triple = {1, 2, 3};
    EXPECT_EQ(triple.size(), 3);
    EXPECT_EQ(triple[0], 1);
    EXPECT_EQ(triple[2], 3);
}

TEST(DeepGenericsTest, test_seq_of_array) {
    std::vector<deep_generic_types::ThreeInts> triples = {{1, 2, 3}, {4, 5, 6}};
    deep_generic_types::SeqOfArray seq(triples);
    EXPECT_EQ(seq.fixed_triples.size(), 2);
    EXPECT_EQ(seq.fixed_triples[0][0], 1);
    EXPECT_EQ(seq.fixed_triples[1][2], 6);
}

TEST(DeepGenericsTest, test_map_of_array) {
    std::map<std::string, deep_generic_types::ThreeInts> named = {
        {"rgb", {255, 128, 64}}, {"xyz", {1, 2, 3}}
    };
    deep_generic_types::MapOfArray map(named);
    EXPECT_EQ(map.named_triples.size(), 2);
    EXPECT_EQ(map.named_triples["rgb"][0], 255);
    EXPECT_EQ(map.named_triples["xyz"][2], 3);
}

TEST(DeepGenericsTest, test_empty_nested_structures) {
    std::vector<std::vector<int32_t>> empty;
    deep_generic_types::TwoLevelSeq seq(empty);
    EXPECT_EQ(seq.matrix.size(), 0);
}

TEST(DeepGenericsTest, test_deeply_nested_point_lists) {
    std::vector<std::vector<deep_generic_types::Point>> grid = {
        {deep_generic_types::Point(0, 0), deep_generic_types::Point(0, 1)},
        {deep_generic_types::Point(1, 0), deep_generic_types::Point(1, 1)}
    };
    deep_generic_types::SeqOfSeqOfPoints matrix(grid);
    EXPECT_EQ(matrix.point_matrix.size(), 2);
    EXPECT_EQ(matrix.point_matrix[1][1].x, 1);
    EXPECT_EQ(matrix.point_matrix[1][1].y, 1);
}

} // namespace
