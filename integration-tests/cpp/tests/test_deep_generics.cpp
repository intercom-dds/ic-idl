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

#include "generated/deep_generics.h"

TEST_CASE("two_level_seq" * doctest::test_suite("deep_generics")) {
    std::vector<std::vector<int32_t>> matrix = {{1, 2, 3}, {4, 5, 6}};
    deep_generic_types::TwoLevelSeq seq(matrix);
    CHECK(seq.matrix.size() == 2);
    CHECK(seq.matrix[0].size() == 3);
    CHECK(seq.matrix[0][0] == 1);
    CHECK(seq.matrix[1][2] == 6);
}

TEST_CASE("three_level_seq" * doctest::test_suite("deep_generics")) {
    std::vector<std::vector<std::vector<int32_t>>> cube = {{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}};
    deep_generic_types::ThreeLevelSeq seq(cube);
    CHECK(seq.cube.size() == 2);
    CHECK(seq.cube[0].size() == 2);
    CHECK(seq.cube[0][0].size() == 2);
    CHECK(seq.cube[0][0][0] == 1);
    CHECK(seq.cube[1][1][1] == 8);
}

TEST_CASE("four_level_deep" * doctest::test_suite("deep_generics")) {
    std::vector<std::vector<std::vector<std::vector<int32_t>>>> hypercube = {{{{1, 2}}}};
    deep_generic_types::FourLevelDeep deep(hypercube);
    CHECK(deep.hypercube.size() == 1);
    CHECK(deep.hypercube[0][0][0][0] == 1);
    CHECK(deep.hypercube[0][0][0][1] == 2);
}

TEST_CASE("map_of_seq" * doctest::test_suite("deep_generics")) {
    std::map<std::string, std::vector<int32_t>> indexed = {
        {"first", {1, 2, 3}}, {"second", {4, 5}}
    };
    deep_generic_types::MapOfSeq seq(indexed);
    CHECK(seq.indexed_lists.size() == 2);
    CHECK(seq.indexed_lists["first"].size() == 3);
    CHECK(seq.indexed_lists["first"][0] == 1);
    CHECK(seq.indexed_lists["second"][1] == 5);
}

TEST_CASE("seq_of_map" * doctest::test_suite("deep_generics")) {
    std::vector<std::map<std::string, int32_t>> dicts = {{{"a", 1}, {"b", 2}}, {{"c", 3}}};
    deep_generic_types::SeqOfMap seq(dicts);
    CHECK(seq.list_of_dicts.size() == 2);
    CHECK(seq.list_of_dicts[0]["a"] == 1);
    CHECK(seq.list_of_dicts[1]["c"] == 3);
}

TEST_CASE("map_of_map" * doctest::test_suite("deep_generics")) {
    std::map<std::string, std::map<std::string, int32_t>> nested = {{"outer", {{"inner", 42}}}};
    deep_generic_types::MapOfMap map(nested);
    CHECK(map.nested_dict.size() == 1);
    CHECK(map.nested_dict["outer"]["inner"] == 42);
}

TEST_CASE("map_seq_map" * doctest::test_suite("deep_generics")) {
    std::map<std::string, std::vector<std::map<std::string, int32_t>>> complex = {
        {"key", {{{"a", 1}, {"b", 2}}}}
    };
    deep_generic_types::MapSeqMap seq(complex);
    CHECK(seq.complex_structure.size() == 1);
    CHECK(seq.complex_structure["key"][0]["a"] == 1);
}

TEST_CASE("seq_map_seq" * doctest::test_suite("deep_generics")) {
    std::vector<std::map<std::string, std::vector<int32_t>>> inverse = {{{"key", {1, 2, 3}}}};
    deep_generic_types::SeqMapSeq seq(inverse);
    CHECK(seq.inverse_structure.size() == 1);
    CHECK(seq.inverse_structure[0]["key"][1] == 2);
}

TEST_CASE("point_struct" * doctest::test_suite("deep_generics")) {
    deep_generic_types::Point p(10, 20);
    CHECK(p.x == 10);
    CHECK(p.y == 20);
}

TEST_CASE("seq_of_points" * doctest::test_suite("deep_generics")) {
    std::vector<deep_generic_types::Point> points = {
        deep_generic_types::Point(1, 2), deep_generic_types::Point(3, 4)
    };
    deep_generic_types::SeqOfPoints seq(points);
    CHECK(seq.points.size() == 2);
    CHECK(seq.points[0].x == 1);
    CHECK(seq.points[1].y == 4);
}

TEST_CASE("map_of_points" * doctest::test_suite("deep_generics")) {
    std::map<std::string, deep_generic_types::Point> named = {
        {"origin", deep_generic_types::Point(0, 0)}, {"unit", deep_generic_types::Point(1, 1)}
    };
    deep_generic_types::MapOfPoints map(named);
    CHECK(map.named_points.size() == 2);
    CHECK(map.named_points["origin"].x == 0);
    CHECK(map.named_points["unit"].y == 1);
}

TEST_CASE("seq_of_seq_of_points" * doctest::test_suite("deep_generics")) {
    std::vector<std::vector<deep_generic_types::Point>> matrix = {
        {deep_generic_types::Point(1, 2), deep_generic_types::Point(3, 4)},
        {deep_generic_types::Point(5, 6)}
    };
    deep_generic_types::SeqOfSeqOfPoints seq(matrix);
    CHECK(seq.point_matrix.size() == 2);
    CHECK(seq.point_matrix[0].size() == 2);
    CHECK(seq.point_matrix[0][0].x == 1);
    CHECK(seq.point_matrix[1][0].y == 6);
}

TEST_CASE("map_of_seq_of_points" * doctest::test_suite("deep_generics")) {
    std::map<std::string, std::vector<deep_generic_types::Point>> lists = {
        {"path1", {deep_generic_types::Point(0, 0), deep_generic_types::Point(1, 1)}},
        {"path2", {deep_generic_types::Point(2, 2)}}
    };
    deep_generic_types::MapOfSeqOfPoints map(lists);
    CHECK(map.point_lists.size() == 2);
    CHECK(map.point_lists["path1"].size() == 2);
    CHECK(map.point_lists["path1"][0].x == 0);
    CHECK(map.point_lists["path2"][0].y == 2);
}

TEST_CASE("typedef_aliases_exist" * doctest::test_suite("deep_generics")) {
    deep_generic_types::IntList list = {1, 2, 3};
    CHECK(list.size() == 3);

    deep_generic_types::IntMatrix matrix = {{1, 2}, {3, 4}};
    CHECK(matrix.size() == 2);

    deep_generic_types::NamedMatrices named = {{"mat1", {{1, 2}, {3, 4}}}};
    CHECK(named.size() == 1);
}

TEST_CASE("using_typedef_chain" * doctest::test_suite("deep_generics")) {
    deep_generic_types::NamedMatrices data = {
        {"matrix1", {{1, 2, 3}, {4, 5, 6}}}, {"matrix2", {{7, 8}}}
    };
    deep_generic_types::UsingTypedefChain chain(data);
    CHECK(chain.data.size() == 2);
    CHECK(chain.data["matrix1"].size() == 2);
    CHECK(chain.data["matrix1"][0][2] == 3);
}

TEST_CASE("array_of_seq" * doctest::test_suite("deep_generics")) {
    std::array<std::vector<int32_t>, 3> items = {
        std::vector<int32_t>{1, 2}, std::vector<int32_t>{3, 4, 5}, std::vector<int32_t>{6}
    };
    deep_generic_types::ArrayOfSeq seq(items);
    CHECK(seq.items.size() == 3);
    CHECK(seq.items[0].size() == 2);
    CHECK(seq.items[1].size() == 3);
    CHECK(seq.items[0][0] == 1);
    CHECK(seq.items[2][0] == 6);
}

TEST_CASE("three_ints_typedef" * doctest::test_suite("deep_generics")) {
    deep_generic_types::ThreeInts triple = {1, 2, 3};
    CHECK(triple.size() == 3);
    CHECK(triple[0] == 1);
    CHECK(triple[2] == 3);
}

TEST_CASE("seq_of_array" * doctest::test_suite("deep_generics")) {
    std::vector<deep_generic_types::ThreeInts> triples = {{1, 2, 3}, {4, 5, 6}};
    deep_generic_types::SeqOfArray seq(triples);
    CHECK(seq.fixed_triples.size() == 2);
    CHECK(seq.fixed_triples[0][0] == 1);
    CHECK(seq.fixed_triples[1][2] == 6);
}

TEST_CASE("map_of_array" * doctest::test_suite("deep_generics")) {
    std::map<std::string, deep_generic_types::ThreeInts> named = {
        {"rgb", {255, 128, 64}}, {"xyz", {1, 2, 3}}
    };
    deep_generic_types::MapOfArray map(named);
    CHECK(map.named_triples.size() == 2);
    CHECK(map.named_triples["rgb"][0] == 255);
    CHECK(map.named_triples["xyz"][2] == 3);
}

TEST_CASE("empty_nested_structures" * doctest::test_suite("deep_generics")) {
    std::vector<std::vector<int32_t>> empty;
    deep_generic_types::TwoLevelSeq seq(empty);
    CHECK(seq.matrix.size() == 0);
}

TEST_CASE("deeply_nested_point_lists" * doctest::test_suite("deep_generics")) {
    std::vector<std::vector<deep_generic_types::Point>> grid = {
        {deep_generic_types::Point(0, 0), deep_generic_types::Point(0, 1)},
        {deep_generic_types::Point(1, 0), deep_generic_types::Point(1, 1)}
    };
    deep_generic_types::SeqOfSeqOfPoints matrix(grid);
    CHECK(matrix.point_matrix.size() == 2);
    CHECK(matrix.point_matrix[1][1].x == 1);
    CHECK(matrix.point_matrix[1][1].y == 1);
}
