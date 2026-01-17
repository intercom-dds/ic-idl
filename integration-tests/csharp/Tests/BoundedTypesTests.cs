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
using BoundedTypes;

namespace IntegrationTests;

public class BoundedTypesTests
{
    [Fact]
    public void BoundedFields_Defaults()
    {
        var b = new BoundedFields();
        Assert.Equal("", b.Name);
        Assert.Equal("", b.Description);
        Assert.Empty(b.Values);
        Assert.Empty(b.Tags);
    }

    [Fact]
    public void BoundedFields_CanSetFields()
    {
        var b = new BoundedFields();
        b.Name = "test name";
        b.Description = "a longer description";
        b.Values.Add(1);
        b.Values.Add(2);
        b.Tags.Add("tag1");

        Assert.Equal("test name", b.Name);
        Assert.Equal("a longer description", b.Description);
        Assert.Equal(new[] { 1, 2 }, b.Values);
        Assert.Single(b.Tags);
    }

    [Fact]
    public void BoundedFields_FieldTypes()
    {
        Assert.Equal(typeof(string), typeof(BoundedFields).GetProperty("Name")!.PropertyType);
        Assert.Equal(typeof(string), typeof(BoundedFields).GetProperty("Description")!.PropertyType);
        Assert.True(typeof(IList<int>).IsAssignableFrom(typeof(BoundedFields).GetProperty("Values")!.PropertyType));
        Assert.True(typeof(IList<string>).IsAssignableFrom(typeof(BoundedFields).GetProperty("Tags")!.PropertyType));
    }

    [Fact]
    public void NestedBounded_Defaults()
    {
        var n = new NestedBounded();
        Assert.Empty(n.Matrix);
        Assert.Empty(n.IndexedLists);
    }

    [Fact]
    public void NestedBounded_CanAddNestedData()
    {
        var n = new NestedBounded();
        n.Matrix.Add(new List<int> { 1, 2, 3 });
        n.IndexedLists["key"] = new List<int> { 4, 5 };

        Assert.Single(n.Matrix);
        Assert.Single(n.IndexedLists);
    }

    [Fact]
    public void NestedBounded_FieldTypes()
    {
        Assert.True(typeof(IList<IList<int>>).IsAssignableFrom(typeof(NestedBounded).GetProperty("Matrix")!.PropertyType));
        Assert.True(typeof(IDictionary<string, IList<int>>).IsAssignableFrom(typeof(NestedBounded).GetProperty("IndexedLists")!.PropertyType));
    }

    [Fact]
    public void MixedBounds_Defaults()
    {
        var m = new MixedBounds();
        Assert.Equal("", m.BoundedString);
        Assert.Equal("", m.UnboundedString);
        Assert.Empty(m.BoundedSeq);
        Assert.Empty(m.UnboundedSeq);
    }

    [Fact]
    public void MixedBounds_CanSetFields()
    {
        var m = new MixedBounds();
        m.BoundedString = "short";
        m.UnboundedString = "this can be any length";
        m.BoundedSeq.Add(1);
        m.UnboundedSeq.Add(2);

        Assert.Equal("short", m.BoundedString);
        Assert.Equal("this can be any length", m.UnboundedString);
        Assert.Single(m.BoundedSeq);
        Assert.Single(m.UnboundedSeq);
    }

    [Fact]
    public void MixedBounds_FieldTypes()
    {
        Assert.Equal(typeof(string), typeof(MixedBounds).GetProperty("BoundedString")!.PropertyType);
        Assert.Equal(typeof(string), typeof(MixedBounds).GetProperty("UnboundedString")!.PropertyType);
        Assert.True(typeof(IList<int>).IsAssignableFrom(typeof(MixedBounds).GetProperty("BoundedSeq")!.PropertyType));
        Assert.True(typeof(IList<int>).IsAssignableFrom(typeof(MixedBounds).GetProperty("UnboundedSeq")!.PropertyType));
    }

    [Fact]
    public void BoundsNotEnforcedAtRuntime()
    {
        var b = new BoundedFields();
        b.Name = new string('x', 1000);
        for (int i = 0; i < 1000; i++)
        {
            b.Values.Add(i);
        }

        Assert.Equal(1000, b.Name.Length);
        Assert.Equal(1000, b.Values.Count);
    }
}
