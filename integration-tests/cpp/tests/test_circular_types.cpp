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

#include "generated/circular_types.h"

namespace {

TEST(CircularTypesTest, test_tree_node_instantiation) {
    circular_types::TreeNode node(42, {});
    EXPECT_EQ(node.value, 42);
    EXPECT_EQ(node.children.size(), 0);
}

TEST(CircularTypesTest, test_tree_node_with_children) {
    circular_types::TreeNode child1(10, {});
    circular_types::TreeNode child2(20, {});
    circular_types::TreeNode parent(5, {child1, child2});
    EXPECT_EQ(parent.value, 5);
    EXPECT_EQ(parent.children.size(), 2);
    EXPECT_EQ(parent.children[0].value, 10);
    EXPECT_EQ(parent.children[1].value, 20);
}

TEST(CircularTypesTest, test_tree_node_deep_nesting) {
    circular_types::TreeNode leaf(100, {});
    circular_types::TreeNode level3(30, {leaf});
    circular_types::TreeNode level2(20, {level3});
    circular_types::TreeNode root(10, {level2});
    EXPECT_EQ(root.value, 10);
    EXPECT_EQ(root.children[0].value, 20);
    EXPECT_EQ(root.children[0].children[0].value, 30);
    EXPECT_EQ(root.children[0].children[0].children[0].value, 100);
}

TEST(CircularTypesTest, test_list_node_single) {
    circular_types::ListNode node(42, {});
    EXPECT_EQ(node.data, 42);
    EXPECT_EQ(node.next.size(), 0);
}

TEST(CircularTypesTest, test_list_node_chain) {
    circular_types::ListNode third(3, {});
    circular_types::ListNode second(2, {third});
    circular_types::ListNode first(1, {second});
    EXPECT_EQ(first.data, 1);
    EXPECT_EQ(first.next.size(), 1);
    EXPECT_EQ(first.next[0].data, 2);
    EXPECT_EQ(first.next[0].next[0].data, 3);
}

TEST(CircularTypesTest, test_graph_node_single) {
    circular_types::GraphNode node("A", {}, {});
    EXPECT_EQ(node.label, "A");
    EXPECT_EQ(node.neighbors.size(), 0);
    EXPECT_EQ(node.parents.size(), 0);
}

TEST(CircularTypesTest, test_graph_node_with_neighbors) {
    circular_types::GraphNode nodeB("B", {}, {});
    circular_types::GraphNode nodeC("C", {}, {});
    circular_types::GraphNode nodeA("A", {nodeB, nodeC}, {});
    EXPECT_EQ(nodeA.label, "A");
    EXPECT_EQ(nodeA.neighbors.size(), 2);
    EXPECT_EQ(nodeA.neighbors[0].label, "B");
    EXPECT_EQ(nodeA.neighbors[1].label, "C");
}

TEST(CircularTypesTest, test_map_self_ref) {
    circular_types::MapSelfRef node("root", {});
    EXPECT_EQ(node.id, "root");
    EXPECT_EQ(node.children_by_name.size(), 0);
}

TEST(CircularTypesTest, test_map_self_ref_multiple_children) {
    circular_types::MapSelfRef child1("child1", {});
    circular_types::MapSelfRef child2("child2", {});
    std::map<std::string, circular_types::MapSelfRef> children = {
        {"first", child1}, {"second", child2}
    };
    circular_types::MapSelfRef parent("parent", children);
    EXPECT_EQ(parent.id, "parent");
    EXPECT_EQ(parent.children_by_name.size(), 2);
    EXPECT_EQ(parent.children_by_name["first"].id, "child1");
    EXPECT_EQ(parent.children_by_name["second"].id, "child2");
}

TEST(CircularTypesTest, test_complex_self_ref) {
    circular_types::ComplexSelfRef node(1, {});
    EXPECT_EQ(node.id, 1);
    EXPECT_EQ(node.levels.size(), 0);
}

TEST(CircularTypesTest, test_complex_self_ref_with_levels) {
    circular_types::ComplexSelfRef leaf(100, {});
    std::map<std::string, circular_types::ComplexSelfRef> level_map = {{"leaf", leaf}};
    std::vector<std::map<std::string, circular_types::ComplexSelfRef>> levels = {level_map};
    circular_types::ComplexSelfRef root(1, levels);
    EXPECT_EQ(root.id, 1);
    EXPECT_EQ(root.levels.size(), 1);
    EXPECT_EQ(root.levels[0]["leaf"].id, 100);
}

TEST(CircularTypesTest, test_nested_self_ref) {
    circular_types::NestedSelfRef node("node", {});
    EXPECT_EQ(node.name, "node");
    EXPECT_EQ(node.grid.size(), 0);
}

TEST(CircularTypesTest, test_nested_self_ref_with_grid) {
    circular_types::NestedSelfRef cell1("cell1", {});
    circular_types::NestedSelfRef cell2("cell2", {});
    std::vector<std::vector<circular_types::NestedSelfRef>> grid = {{cell1, cell2}};
    circular_types::NestedSelfRef container("container", grid);
    EXPECT_EQ(container.name, "container");
    EXPECT_EQ(container.grid.size(), 1);
    EXPECT_EQ(container.grid[0].size(), 2);
    EXPECT_EQ(container.grid[0][0].name, "cell1");
    EXPECT_EQ(container.grid[0][1].name, "cell2");
}

TEST(CircularTypesTest, test_tree_node_type_annotations) {
    EXPECT_TRUE((std::is_same<decltype(circular_types::TreeNode::value), int32_t>::value));
    EXPECT_TRUE((std::is_same<
                 decltype(circular_types::TreeNode::children),
                 std::vector<circular_types::TreeNode>>::value));
}

TEST(CircularTypesTest, test_map_self_ref_type_annotations) {
    EXPECT_TRUE((std::is_same<decltype(circular_types::MapSelfRef::id), std::string>::value));
    EXPECT_TRUE((std::is_same<
                 decltype(circular_types::MapSelfRef::children_by_name),
                 std::map<std::string, circular_types::MapSelfRef>>::value));
}

} // namespace
