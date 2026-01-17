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

using Xunit;
using CircularTypes;

namespace IntegrationTests;

public class CircularTypesTests
{
    [Fact]
    public void TreeNode_Instantiation()
    {
        var node = new TreeNode();
        node.Value = 42;
        Assert.Equal(42, node.Value);
        Assert.Empty(node.Children);
    }

    [Fact]
    public void TreeNode_WithChildren()
    {
        var root = new TreeNode { Value = 1 };
        var child1 = new TreeNode { Value = 2 };
        var child2 = new TreeNode { Value = 3 };
        root.Children.Add(child1);
        root.Children.Add(child2);

        Assert.Equal(2, root.Children.Count);
        Assert.Equal(2, root.Children[0].Value);
        Assert.Equal(3, root.Children[1].Value);
    }

    [Fact]
    public void TreeNode_DeepNesting()
    {
        var level1 = new TreeNode { Value = 1 };
        var level2 = new TreeNode { Value = 2 };
        var level3 = new TreeNode { Value = 3 };
        var level4 = new TreeNode { Value = 4 };

        level1.Children.Add(level2);
        level2.Children.Add(level3);
        level3.Children.Add(level4);

        Assert.Equal(4, level1.Children[0].Children[0].Children[0].Value);
    }

    [Fact]
    public void ListNode_Single()
    {
        var node = new ListNode { Data = 100 };
        Assert.Equal(100, node.Data);
        Assert.Empty(node.Next);
    }

    [Fact]
    public void ListNode_Chain()
    {
        var node1 = new ListNode { Data = 1 };
        var node2 = new ListNode { Data = 2 };
        var node3 = new ListNode { Data = 3 };

        node1.Next.Add(node2);
        node2.Next.Add(node3);

        Assert.Equal(2, node1.Next[0].Data);
        Assert.Equal(3, node1.Next[0].Next[0].Data);
    }

    [Fact]
    public void GraphNode_Single()
    {
        var node = new GraphNode { Label = "root" };
        Assert.Equal("root", node.Label);
        Assert.Empty(node.Neighbors);
        Assert.Empty(node.Parents);
    }

    [Fact]
    public void GraphNode_WithNeighbors()
    {
        var nodeA = new GraphNode { Label = "A" };
        var nodeB = new GraphNode { Label = "B" };
        var nodeC = new GraphNode { Label = "C" };

        nodeA.Neighbors.Add(nodeB);
        nodeA.Neighbors.Add(nodeC);
        nodeB.Parents.Add(nodeA);
        nodeC.Parents.Add(nodeA);

        Assert.Equal(2, nodeA.Neighbors.Count);
        Assert.Equal("B", nodeA.Neighbors[0].Label);
        Assert.Equal("A", nodeB.Parents[0].Label);
    }

    [Fact]
    public void MapSelfRef_Single()
    {
        var node = new MapSelfRef { Id = "root" };
        Assert.Equal("root", node.Id);
        Assert.Empty(node.ChildrenByName);
    }

    [Fact]
    public void MapSelfRef_MultipleChildren()
    {
        var root = new MapSelfRef { Id = "root" };
        var child1 = new MapSelfRef { Id = "child1" };
        var child2 = new MapSelfRef { Id = "child2" };

        root.ChildrenByName["first"] = child1;
        root.ChildrenByName["second"] = child2;

        Assert.Equal(2, root.ChildrenByName.Count);
        Assert.Equal("child1", root.ChildrenByName["first"].Id);
        Assert.Equal("child2", root.ChildrenByName["second"].Id);
    }

    [Fact]
    public void ComplexSelfRef_Instantiation()
    {
        var c = new ComplexSelfRef { Id = 1 };
        Assert.Equal(1, c.Id);
        Assert.Empty(c.Levels);
    }

    [Fact]
    public void ComplexSelfRef_CanAddLevels()
    {
        var root = new ComplexSelfRef { Id = 1 };
        var child = new ComplexSelfRef { Id = 2 };
        var level = new Dictionary<string, ComplexSelfRef> { { "child", child } };
        root.Levels.Add(level);

        Assert.Single(root.Levels);
        Assert.Equal(2, root.Levels[0]["child"].Id);
    }

    [Fact]
    public void NestedSelfRef_Instantiation()
    {
        var n = new NestedSelfRef { Name = "root" };
        Assert.Equal("root", n.Name);
        Assert.Empty(n.Grid);
    }

    [Fact]
    public void NestedSelfRef_CanAddGrid()
    {
        var root = new NestedSelfRef { Name = "root" };
        var child = new NestedSelfRef { Name = "child" };
        root.Grid.Add(new List<NestedSelfRef> { child });

        Assert.Single(root.Grid);
        Assert.Equal("child", root.Grid[0][0].Name);
    }

    [Fact]
    public void TreeNode_FieldTypes()
    {
        Assert.Equal(typeof(int), typeof(TreeNode).GetProperty("Value")!.PropertyType);
        Assert.True(typeof(IList<TreeNode>).IsAssignableFrom(typeof(TreeNode).GetProperty("Children")!.PropertyType));
    }

    [Fact]
    public void MapSelfRef_FieldTypes()
    {
        Assert.Equal(typeof(string), typeof(MapSelfRef).GetProperty("Id")!.PropertyType);
        Assert.True(typeof(IDictionary<string, MapSelfRef>).IsAssignableFrom(typeof(MapSelfRef).GetProperty("ChildrenByName")!.PropertyType));
    }
}
