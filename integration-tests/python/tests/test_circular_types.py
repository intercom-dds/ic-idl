# Copyright 2026 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

from types import ModuleType


def test_tree_node_instantiation(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    leaf = ct.TreeNode(value=1, children=[])
    assert leaf.value == 1
    assert leaf.children == []


def test_tree_node_with_children(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    leaf1 = ct.TreeNode(value=1, children=[])
    leaf2 = ct.TreeNode(value=2, children=[])
    parent = ct.TreeNode(value=0, children=[leaf1, leaf2])
    assert parent.value == 0
    assert len(parent.children) == 2
    assert parent.children[0].value == 1
    assert parent.children[1].value == 2


def test_tree_node_deep_nesting(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    deep = ct.TreeNode(value=3, children=[])
    for i in range(2, -1, -1):
        deep = ct.TreeNode(value=i, children=[deep])
    assert deep.value == 0
    assert deep.children[0].value == 1
    assert deep.children[0].children[0].value == 2
    assert deep.children[0].children[0].children[0].value == 3


def test_list_node_single(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    node = ct.ListNode(data=42, next=[])
    assert node.data == 42
    assert node.next == []


def test_list_node_chain(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    tail = ct.ListNode(data=3, next=[])
    mid = ct.ListNode(data=2, next=[tail])
    head = ct.ListNode(data=1, next=[mid])
    assert head.data == 1
    assert head.next[0].data == 2
    assert head.next[0].next[0].data == 3
    assert head.next[0].next[0].next == []


def test_graph_node_single(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    node = ct.GraphNode(label="A", neighbors=[], parents=[])
    assert node.label == "A"


def test_graph_node_with_neighbors(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    a = ct.GraphNode(label="A", neighbors=[], parents=[])
    b = ct.GraphNode(label="B", neighbors=[], parents=[])
    c = ct.GraphNode(label="C", neighbors=[a, b], parents=[])
    assert c.label == "C"
    assert len(c.neighbors) == 2
    assert c.neighbors[0].label == "A"
    assert c.neighbors[1].label == "B"


def test_map_self_ref(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    leaf = ct.MapSelfRef(id="leaf", children_by_name={})
    parent = ct.MapSelfRef(id="parent", children_by_name={"child": leaf})
    assert parent.id == "parent"
    assert parent.children_by_name["child"].id == "leaf"


def test_map_self_ref_multiple_children(
    generated_modules: dict[str, ModuleType],
) -> None:
    ct = generated_modules["circular_types"]
    a = ct.MapSelfRef(id="a", children_by_name={})
    b = ct.MapSelfRef(id="b", children_by_name={})
    root = ct.MapSelfRef(id="root", children_by_name={"a": a, "b": b})
    assert root.children_by_name["a"].id == "a"
    assert root.children_by_name["b"].id == "b"


def test_complex_self_ref(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    inner = ct.ComplexSelfRef(id=1, levels=[])
    outer = ct.ComplexSelfRef(id=0, levels=[{"inner": inner}])
    assert outer.id == 0
    assert outer.levels[0]["inner"].id == 1


def test_nested_self_ref(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    cell = ct.NestedSelfRef(name="cell", grid=[])
    row = ct.NestedSelfRef(name="row", grid=[[cell]])
    assert row.name == "row"
    assert row.grid[0][0].name == "cell"


def test_tree_node_type_annotations(generated_modules: dict[str, ModuleType]) -> None:
    ct = generated_modules["circular_types"]
    annotations = ct.TreeNode.__annotations__
    assert annotations["value"] == "int"
    assert annotations["children"] == "list[TreeNode]"


def test_map_self_ref_type_annotations(
    generated_modules: dict[str, ModuleType],
) -> None:
    ct = generated_modules["circular_types"]
    annotations = ct.MapSelfRef.__annotations__
    assert annotations["id"] == "str"
    assert annotations["children_by_name"] == "dict[str, MapSelfRef]"
