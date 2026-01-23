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

import { describe, expect, test } from "bun:test";
import type {
  TreeNode,
  ListNode,
  GraphNode,
  MapSelfRef,
  ComplexSelfRef,
  NestedSelfRef,
} from "../generated/circular_types";

describe("circular types", () => {
  describe("TreeNode", () => {
    test("instantiation", () => {
      const leaf: TreeNode = { value: 1, children: [] };
      expect(leaf.value).toBe(1);
      expect(leaf.children).toEqual([]);
    });

    test("with children", () => {
      const leaf1: TreeNode = { value: 1, children: [] };
      const leaf2: TreeNode = { value: 2, children: [] };
      const parent: TreeNode = { value: 0, children: [leaf1, leaf2] };
      expect(parent.value).toBe(0);
      expect(parent.children.length).toBe(2);
      expect(parent.children[0].value).toBe(1);
      expect(parent.children[1].value).toBe(2);
    });

    test("deep nesting", () => {
      let deep: TreeNode = { value: 3, children: [] };
      for (let i = 2; i >= 0; i--) {
        deep = { value: i, children: [deep] };
      }
      expect(deep.value).toBe(0);
      expect(deep.children[0].value).toBe(1);
      expect(deep.children[0].children[0].value).toBe(2);
      expect(deep.children[0].children[0].children[0].value).toBe(3);
    });
  });

  describe("ListNode", () => {
    test("single node", () => {
      const node: ListNode = { data: 42, next: [] };
      expect(node.data).toBe(42);
      expect(node.next).toEqual([]);
    });

    test("chain", () => {
      const tail: ListNode = { data: 3, next: [] };
      const mid: ListNode = { data: 2, next: [tail] };
      const head: ListNode = { data: 1, next: [mid] };
      expect(head.data).toBe(1);
      expect(head.next[0].data).toBe(2);
      expect(head.next[0].next[0].data).toBe(3);
      expect(head.next[0].next[0].next).toEqual([]);
    });
  });

  describe("GraphNode", () => {
    test("single node", () => {
      const node: GraphNode = { label: "A", neighbors: [], parents: [] };
      expect(node.label).toBe("A");
    });

    test("with neighbors", () => {
      const a: GraphNode = { label: "A", neighbors: [], parents: [] };
      const b: GraphNode = { label: "B", neighbors: [], parents: [] };
      const c: GraphNode = { label: "C", neighbors: [a, b], parents: [] };
      expect(c.label).toBe("C");
      expect(c.neighbors.length).toBe(2);
      expect(c.neighbors[0].label).toBe("A");
      expect(c.neighbors[1].label).toBe("B");
    });

    test("with neighbors and parents", () => {
      const a: GraphNode = { label: "A", neighbors: [], parents: [] };
      const b: GraphNode = { label: "B", neighbors: [a], parents: [] };
      a.parents.push(b);

      expect(b.neighbors.length).toBe(1);
      expect(b.neighbors[0].label).toBe("A");
      expect(a.parents.length).toBe(1);
      expect(a.parents[0].label).toBe("B");
    });
  });

  describe("MapSelfRef", () => {
    test("with child", () => {
      const leaf: MapSelfRef = { id: "leaf", children_by_name: {} };
      const parent: MapSelfRef = {
        id: "parent",
        children_by_name: { child: leaf },
      };
      expect(parent.id).toBe("parent");
      expect(parent.children_by_name.child.id).toBe("leaf");
    });

    test("multiple children", () => {
      const a: MapSelfRef = { id: "a", children_by_name: {} };
      const b: MapSelfRef = { id: "b", children_by_name: {} };
      const root: MapSelfRef = {
        id: "root",
        children_by_name: { a, b },
      };
      expect(root.children_by_name.a.id).toBe("a");
      expect(root.children_by_name.b.id).toBe("b");
    });
  });

  describe("ComplexSelfRef", () => {
    test("nested structure", () => {
      const inner: ComplexSelfRef = { id: 1, levels: [] };
      const outer: ComplexSelfRef = {
        id: 0,
        levels: [{ inner }],
      };
      expect(outer.id).toBe(0);
      expect(outer.levels[0].inner.id).toBe(1);
    });
  });

  describe("NestedSelfRef", () => {
    test("grid structure", () => {
      const cell: NestedSelfRef = { name: "cell", grid: [] };
      const row: NestedSelfRef = { name: "row", grid: [[cell]] };
      expect(row.name).toBe("row");
      expect(row.grid[0][0].name).toBe("cell");
    });
  });
});
