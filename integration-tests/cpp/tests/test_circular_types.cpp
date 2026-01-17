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

#include <type_traits>

#include "generated/circular_types.h"

TEST_CASE("tree_node_instantiation" * doctest::test_suite("circular_types")) {
    circular_types::TreeNode node(42, {});
    CHECK(node.value == 42);
    CHECK(node.children.size() == 0);
}

TEST_CASE("tree_node_with_children" * doctest::test_suite("circular_types")) {
    circular_types::TreeNode child1(10, {});
    circular_types::TreeNode child2(20, {});
    circular_types::TreeNode parent(5, {child1, child2});
    CHECK(parent.value == 5);
    CHECK(parent.children.size() == 2);
    CHECK(parent.children[0].value == 10);
    CHECK(parent.children[1].value == 20);
}

TEST_CASE("tree_node_deep_nesting" * doctest::test_suite("circular_types")) {
    circular_types::TreeNode leaf(100, {});
    circular_types::TreeNode level3(30, {leaf});
    circular_types::TreeNode level2(20, {level3});
    circular_types::TreeNode root(10, {level2});
    CHECK(root.value == 10);
    CHECK(root.children[0].value == 20);
    CHECK(root.children[0].children[0].value == 30);
    CHECK(root.children[0].children[0].children[0].value == 100);
}

TEST_CASE("list_node_single" * doctest::test_suite("circular_types")) {
    circular_types::ListNode node(42, {});
    CHECK(node.data == 42);
    CHECK(node.next.size() == 0);
}

TEST_CASE("list_node_chain" * doctest::test_suite("circular_types")) {
    circular_types::ListNode third(3, {});
    circular_types::ListNode second(2, {third});
    circular_types::ListNode first(1, {second});
    CHECK(first.data == 1);
    CHECK(first.next.size() == 1);
    CHECK(first.next[0].data == 2);
    CHECK(first.next[0].next[0].data == 3);
}

TEST_CASE("graph_node_single" * doctest::test_suite("circular_types")) {
    circular_types::GraphNode node("A", {}, {});
    CHECK(node.label == "A");
    CHECK(node.neighbors.size() == 0);
    CHECK(node.parents.size() == 0);
}

TEST_CASE("graph_node_with_neighbors" * doctest::test_suite("circular_types")) {
    circular_types::GraphNode nodeB("B", {}, {});
    circular_types::GraphNode nodeC("C", {}, {});
    circular_types::GraphNode nodeA("A", {nodeB, nodeC}, {});
    CHECK(nodeA.label == "A");
    CHECK(nodeA.neighbors.size() == 2);
    CHECK(nodeA.neighbors[0].label == "B");
    CHECK(nodeA.neighbors[1].label == "C");
}

TEST_CASE("map_self_ref" * doctest::test_suite("circular_types")) {
    circular_types::MapSelfRef node("root", {});
    CHECK(node.id == "root");
    CHECK(node.children_by_name.size() == 0);
}

TEST_CASE("map_self_ref_multiple_children" * doctest::test_suite("circular_types")) {
    circular_types::MapSelfRef child1("child1", {});
    circular_types::MapSelfRef child2("child2", {});
    std::map<std::string, circular_types::MapSelfRef> children = {
        {"first", child1}, {"second", child2}
    };
    circular_types::MapSelfRef parent("parent", children);
    CHECK(parent.id == "parent");
    CHECK(parent.children_by_name.size() == 2);
    CHECK(parent.children_by_name["first"].id == "child1");
    CHECK(parent.children_by_name["second"].id == "child2");
}

TEST_CASE("complex_self_ref" * doctest::test_suite("circular_types")) {
    circular_types::ComplexSelfRef node(1, {});
    CHECK(node.id == 1);
    CHECK(node.levels.size() == 0);
}

TEST_CASE("complex_self_ref_with_levels" * doctest::test_suite("circular_types")) {
    circular_types::ComplexSelfRef leaf(100, {});
    std::map<std::string, circular_types::ComplexSelfRef> level_map = {{"leaf", leaf}};
    std::vector<std::map<std::string, circular_types::ComplexSelfRef>> levels = {level_map};
    circular_types::ComplexSelfRef root(1, levels);
    CHECK(root.id == 1);
    CHECK(root.levels.size() == 1);
    CHECK(root.levels[0]["leaf"].id == 100);
}

TEST_CASE("nested_self_ref" * doctest::test_suite("circular_types")) {
    circular_types::NestedSelfRef node("node", {});
    CHECK(node.name == "node");
    CHECK(node.grid.size() == 0);
}

TEST_CASE("nested_self_ref_with_grid" * doctest::test_suite("circular_types")) {
    circular_types::NestedSelfRef cell1("cell1", {});
    circular_types::NestedSelfRef cell2("cell2", {});
    std::vector<std::vector<circular_types::NestedSelfRef>> grid = {{cell1, cell2}};
    circular_types::NestedSelfRef container("container", grid);
    CHECK(container.name == "container");
    CHECK(container.grid.size() == 1);
    CHECK(container.grid[0].size() == 2);
    CHECK(container.grid[0][0].name == "cell1");
    CHECK(container.grid[0][1].name == "cell2");
}

TEST_CASE("tree_node_type_annotations" * doctest::test_suite("circular_types")) {
    CHECK((std::is_same<decltype(circular_types::TreeNode::value), int32_t>::value));
    CHECK((std::is_same<
           decltype(circular_types::TreeNode::children),
           std::vector<circular_types::TreeNode>>::value));
}

TEST_CASE("map_self_ref_type_annotations" * doctest::test_suite("circular_types")) {
    CHECK((std::is_same<decltype(circular_types::MapSelfRef::id), std::string>::value));
    CHECK((std::is_same<
           decltype(circular_types::MapSelfRef::children_by_name),
           std::map<std::string, circular_types::MapSelfRef>>::value));
}
