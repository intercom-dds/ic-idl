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

package integrationtests;

import static org.junit.jupiter.api.Assertions.*;

import circular_types.*;
import org.junit.jupiter.api.Test;

class CircularTypesTests {

    @Test
    void listNode_defaults() {
        var node = new ListNode();
        assertEquals(0, node.getData());
        assertNotNull(node.getNext());
        assertTrue(node.getNext().isEmpty());
    }

    @Test
    void listNode_setData() {
        var node = new ListNode();
        node.setData(42);
        assertEquals(42, node.getData());
    }

    @Test
    void listNode_canAddNext() {
        var node1 = new ListNode();
        node1.setData(1);
        var node2 = new ListNode();
        node2.setData(2);
        node1.getNext().add(node2);
        assertEquals(1, node1.getNext().size());
        assertEquals(2, node1.getNext().get(0).getData());
    }

    @Test
    void listNode_selfReferential() {
        var head = new ListNode();
        head.setData(1);
        var second = new ListNode();
        second.setData(2);
        var third = new ListNode();
        third.setData(3);

        head.getNext().add(second);
        second.getNext().add(third);

        assertEquals(1, head.getData());
        assertEquals(2, head.getNext().get(0).getData());
        assertEquals(3, head.getNext().get(0).getNext().get(0).getData());
    }

    @Test
    void listNode_copyConstructor() {
        var node1 = new ListNode();
        node1.setData(10);
        var child = new ListNode();
        child.setData(20);
        node1.getNext().add(child);

        var node2 = new ListNode(node1);
        assertEquals(node1.getData(), node2.getData());
        assertEquals(node1.getNext().size(), node2.getNext().size());
        assertNotSame(node1, node2);
        assertNotSame(node1.getNext().get(0), node2.getNext().get(0));
    }

    @Test
    void listNode_equality() {
        var n1 = new ListNode();
        n1.setData(5);
        var n2 = new ListNode();
        n2.setData(5);
        var n3 = new ListNode();
        n3.setData(10);
        assertEquals(n1, n2);
        assertNotEquals(n1, n3);
    }

    @Test
    void treeNode_defaults() {
        var node = new TreeNode();
        assertEquals(0, node.getValue());
        assertNotNull(node.getChildren());
        assertTrue(node.getChildren().isEmpty());
    }

    @Test
    void treeNode_setValue() {
        var node = new TreeNode();
        node.setValue(100);
        assertEquals(100, node.getValue());
    }

    @Test
    void treeNode_canAddChildren() {
        var root = new TreeNode();
        root.setValue(1);
        var child1 = new TreeNode();
        child1.setValue(2);
        var child2 = new TreeNode();
        child2.setValue(3);
        root.getChildren().add(child1);
        root.getChildren().add(child2);
        assertEquals(2, root.getChildren().size());
        assertEquals(2, root.getChildren().get(0).getValue());
        assertEquals(3, root.getChildren().get(1).getValue());
    }

    @Test
    void treeNode_deepHierarchy() {
        var root = new TreeNode();
        root.setValue(1);
        var child = new TreeNode();
        child.setValue(2);
        var grandchild = new TreeNode();
        grandchild.setValue(3);

        root.getChildren().add(child);
        child.getChildren().add(grandchild);

        assertEquals(1, root.getValue());
        assertEquals(2, root.getChildren().get(0).getValue());
        assertEquals(3, root.getChildren().get(0).getChildren().get(0).getValue());
    }

    @Test
    void treeNode_copyConstructor() {
        var root = new TreeNode();
        root.setValue(1);
        var child = new TreeNode();
        child.setValue(2);
        root.getChildren().add(child);

        var copy = new TreeNode(root);
        assertEquals(root.getValue(), copy.getValue());
        assertEquals(root.getChildren().size(), copy.getChildren().size());
        assertNotSame(root, copy);
        assertNotSame(root.getChildren().get(0), copy.getChildren().get(0));
    }

    @Test
    void treeNode_clone() {
        var root = new TreeNode();
        root.setValue(42);
        var copy = root.clone();
        assertEquals(root, copy);
        assertNotSame(root, copy);
    }

    @Test
    void treeNode_equality() {
        var t1 = new TreeNode();
        t1.setValue(10);
        var t2 = new TreeNode();
        t2.setValue(10);
        var t3 = new TreeNode();
        t3.setValue(20);
        assertEquals(t1, t2);
        assertNotEquals(t1, t3);
    }
}
