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

use std::collections::BTreeMap;

use crate::circular_types;

#[test]
fn tree_node_instantiation() {
    let node = circular_types::TreeNode {
        value: 42,
        children: vec![],
    };

    assert_eq!(node.value, 42);
    assert!(node.children.is_empty());
}

#[test]
fn tree_node_with_children() {
    let child1 = circular_types::TreeNode {
        value: 10,
        children: vec![],
    };
    let child2 = circular_types::TreeNode {
        value: 20,
        children: vec![],
    };
    let parent = circular_types::TreeNode {
        value: 5,
        children: vec![child1, child2],
    };

    assert_eq!(parent.value, 5);
    assert_eq!(parent.children.len(), 2);
    assert_eq!(parent.children[0].value, 10);
    assert_eq!(parent.children[1].value, 20);
}

#[test]
fn tree_node_deep_nesting() {
    let leaf = circular_types::TreeNode {
        value: 100,
        children: vec![],
    };
    let level3 = circular_types::TreeNode {
        value: 30,
        children: vec![leaf],
    };
    let level2 = circular_types::TreeNode {
        value: 20,
        children: vec![level3],
    };
    let root = circular_types::TreeNode {
        value: 10,
        children: vec![level2],
    };

    assert_eq!(root.value, 10);
    assert_eq!(root.children[0].value, 20);
    assert_eq!(root.children[0].children[0].value, 30);
    assert_eq!(root.children[0].children[0].children[0].value, 100);
}

#[test]
fn list_node_single() {
    let node = circular_types::ListNode {
        data: 42,
        next: vec![],
    };

    assert_eq!(node.data, 42);
    assert!(node.next.is_empty());
}

#[test]
fn list_node_chain() {
    let third = circular_types::ListNode {
        data: 3,
        next: vec![],
    };
    let second = circular_types::ListNode {
        data: 2,
        next: vec![third],
    };
    let first = circular_types::ListNode {
        data: 1,
        next: vec![second],
    };

    assert_eq!(first.data, 1);
    assert_eq!(first.next.len(), 1);
    assert_eq!(first.next[0].data, 2);
    assert_eq!(first.next[0].next.len(), 1);
    assert_eq!(first.next[0].next[0].data, 3);
}

#[test]
fn graph_node_single() {
    let node = circular_types::GraphNode {
        label: "A".into(),
        neighbors: vec![],
        parents: vec![],
    };

    assert_eq!(node.label, "A");
    assert!(node.neighbors.is_empty());
    assert!(node.parents.is_empty());
}

#[test]
fn graph_node_with_neighbors() {
    let node_b = circular_types::GraphNode {
        label: "B".into(),
        neighbors: vec![],
        parents: vec![],
    };
    let node_c = circular_types::GraphNode {
        label: "C".into(),
        neighbors: vec![],
        parents: vec![],
    };
    let node_a = circular_types::GraphNode {
        label: "A".into(),
        neighbors: vec![node_b, node_c],
        parents: vec![],
    };

    assert_eq!(node_a.label, "A");
    assert_eq!(node_a.neighbors.len(), 2);
    assert_eq!(node_a.neighbors[0].label, "B");
    assert_eq!(node_a.neighbors[1].label, "C");
}

#[test]
fn map_self_ref() {
    let node = circular_types::MapSelfRef {
        id: "root".into(),
        ..Default::default()
    };

    assert_eq!(node.id, "root");
    assert!(node.children_by_name.is_empty());
}

#[test]
fn map_self_ref_multiple_children() {
    let child1 = circular_types::MapSelfRef {
        id: "child1".into(),
        ..Default::default()
    };
    let child2 = circular_types::MapSelfRef {
        id: "child2".into(),
        ..Default::default()
    };
    let children_by_name = BTreeMap::<String, circular_types::MapSelfRef>::from([
        ("first".into(), child1),
        ("second".into(), child2),
    ]);

    let parent = circular_types::MapSelfRef {
        id: "parent".into(),
        children_by_name,
    };

    assert_eq!(parent.id, "parent");
    assert_eq!(parent.children_by_name.len(), 2);
    assert_eq!(parent.children_by_name["first"].id, "child1");
    assert_eq!(parent.children_by_name["second"].id, "child2");
}

#[test]
fn complex_self_ref() {
    let node = circular_types::ComplexSelfRef {
        id: 1,
        levels: vec![],
    };

    assert_eq!(node.id, 1);
    assert!(node.levels.is_empty());
}

#[test]
fn complex_self_ref_with_levels() {
    let leaf = circular_types::ComplexSelfRef {
        id: 100,
        levels: vec![],
    };

    let root = circular_types::ComplexSelfRef {
        id: 1,
        levels: vec![BTreeMap::<String, circular_types::ComplexSelfRef>::from([
            ("leaf".into(), leaf),
        ])],
    };

    assert_eq!(root.id, 1);
    assert_eq!(root.levels.len(), 1);
    assert_eq!(root.levels[0]["leaf"].id, 100);
}

#[test]
fn nested_self_ref() {
    let node = circular_types::NestedSelfRef {
        name: "node".into(),
        grid: vec![],
    };

    assert_eq!(node.name, "node");
    assert!(node.grid.is_empty());
}

#[test]
fn nested_self_ref_with_grid() {
    let cell1 = circular_types::NestedSelfRef {
        name: "cell1".into(),
        grid: vec![],
    };
    let cell2 = circular_types::NestedSelfRef {
        name: "cell2".into(),
        grid: vec![],
    };
    let container = circular_types::NestedSelfRef {
        name: "container".into(),
        grid: vec![vec![cell1, cell2]],
    };

    assert_eq!(container.name, "container");
    assert_eq!(container.grid.len(), 1);
    assert_eq!(container.grid[0].len(), 2);
    assert_eq!(container.grid[0][0].name, "cell1");
    assert_eq!(container.grid[0][1].name, "cell2");
}

#[test]
fn tree_node_type_annotations() {
    assert_eq!(
        std::any::type_name_of_val(&circular_types::TreeNode::new().value),
        std::any::type_name::<i32>()
    );
    assert_eq!(
        std::any::type_name_of_val(&circular_types::TreeNode::new().children),
        std::any::type_name::<Vec<circular_types::TreeNode>>()
    );
}

#[test]
fn map_self_ref_type_annotations() {
    assert_eq!(
        std::any::type_name_of_val(&circular_types::MapSelfRef::new().id),
        std::any::type_name::<String>()
    );
    assert_eq!(
        std::any::type_name_of_val(&circular_types::MapSelfRef::new().children_by_name),
        std::any::type_name::<BTreeMap<String, circular_types::MapSelfRef>>()
    );
}
