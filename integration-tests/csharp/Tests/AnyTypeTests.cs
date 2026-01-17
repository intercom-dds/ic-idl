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
using AnyTypes;
using StructTypes;

namespace IntegrationTests;

public class AnyTypeTests
{
    [Fact]
    public void ContainsAny_DefaultIsNull()
    {
        var c = new ContainsAny();
        Assert.Null(c.Value);
    }

    [Fact]
    public void ContainsAny_AcceptsInt()
    {
        var c = new ContainsAny();
        c.Value = 42;
        Assert.Equal(42, c.Value);
        Assert.IsType<int>(c.Value);
    }

    [Fact]
    public void ContainsAny_AcceptsString()
    {
        var c = new ContainsAny();
        c.Value = "hello";
        Assert.Equal("hello", c.Value);
        Assert.IsType<string>(c.Value);
    }

    [Fact]
    public void ContainsAny_AcceptsList()
    {
        var c = new ContainsAny();
        c.Value = new List<int> { 1, 2, 3 };
        Assert.IsType<List<int>>(c.Value);
        Assert.Equal(new[] { 1, 2, 3 }, (List<int>)c.Value);
    }

    [Fact]
    public void ContainsAny_AcceptsDict()
    {
        var c = new ContainsAny();
        c.Value = new Dictionary<string, int> { { "one", 1 }, { "two", 2 } };
        Assert.IsType<Dictionary<string, int>>(c.Value);
        var dict = (Dictionary<string, int>)c.Value;
        Assert.Equal(1, dict["one"]);
        Assert.Equal(2, dict["two"]);
    }

    [Fact]
    public void ContainsAny_AcceptsNestedStruct()
    {
        var c = new ContainsAny();
        var point = new Point(10, 20);
        c.Value = point;
        Assert.IsType<Point>(c.Value);
        var p = (Point)c.Value;
        Assert.Equal(10, p.X);
        Assert.Equal(20, p.Y);
    }

    [Fact]
    public void MultipleAny_HasThreeFields()
    {
        var m = new MultipleAny();
        m.First = 1;
        m.Second = "two";
        m.Third = 3.0;

        Assert.Equal(1, m.First);
        Assert.Equal("two", m.Second);
        Assert.Equal(3.0, m.Third);
    }

    [Fact]
    public void MultipleAny_FieldTypes()
    {
        var firstProp = typeof(MultipleAny).GetProperty("First");
        var secondProp = typeof(MultipleAny).GetProperty("Second");
        var thirdProp = typeof(MultipleAny).GetProperty("Third");

        Assert.Equal(typeof(object), firstProp!.PropertyType);
        Assert.Equal(typeof(object), secondProp!.PropertyType);
        Assert.Equal(typeof(object), thirdProp!.PropertyType);
    }

    [Fact]
    public void AnyWithOtherFields_HasMixedFields()
    {
        var a = new AnyWithOtherFields();
        a.Id = 42;
        a.Name = "test";
        a.Payload = new[] { 1, 2, 3 };

        Assert.Equal(42, a.Id);
        Assert.Equal("test", a.Name);
        Assert.IsType<int[]>(a.Payload);
    }

    [Fact]
    public void AnyWithOtherFields_FieldTypes()
    {
        Assert.Equal(typeof(int), typeof(AnyWithOtherFields).GetProperty("Id")!.PropertyType);
        Assert.Equal(typeof(string), typeof(AnyWithOtherFields).GetProperty("Name")!.PropertyType);
        Assert.Equal(typeof(object), typeof(AnyWithOtherFields).GetProperty("Payload")!.PropertyType);
    }

    [Fact]
    public void SequenceOfAny_DefaultEmpty()
    {
        var s = new SequenceOfAny();
        Assert.Empty(s.Items);
    }

    [Fact]
    public void SequenceOfAny_CanAddMixedTypes()
    {
        var s = new SequenceOfAny();
        s.Items.Add(1);
        s.Items.Add("two");
        s.Items.Add(3.0);
        s.Items.Add(new Point(0, 0));

        Assert.Equal(4, s.Items.Count);
        Assert.IsType<int>(s.Items[0]);
        Assert.IsType<string>(s.Items[1]);
        Assert.IsType<double>(s.Items[2]);
        Assert.IsType<Point>(s.Items[3]);
    }

    [Fact]
    public void MapWithAny_DefaultEmpty()
    {
        var m = new MapWithAny();
        Assert.Empty(m.Properties);
    }

    [Fact]
    public void MapWithAny_CanAddMixedTypes()
    {
        var m = new MapWithAny();
        m.Properties["int"] = 42;
        m.Properties["string"] = "hello";
        m.Properties["list"] = new List<int> { 1, 2, 3 };

        Assert.Equal(3, m.Properties.Count);
        Assert.Equal(42, m.Properties["int"]);
        Assert.Equal("hello", m.Properties["string"]);
        Assert.IsType<List<int>>(m.Properties["list"]);
    }

    [Fact]
    public void OptionalAny_DefaultIsNull()
    {
        var o = new OptionalAny();
        Assert.Null(o.MaybeValue);
    }

    [Fact]
    public void OptionalAny_CanSetValue()
    {
        var o = new OptionalAny();
        o.MaybeValue = "test value";
        Assert.Equal("test value", o.MaybeValue);
    }

    [Fact]
    public void UsingAnyAlias_HasDataField()
    {
        var u = new UsingAnyAlias();
        Assert.Null(u.Data);

        u.Data = 123;
        Assert.Equal(123, u.Data);
    }

    [Fact]
    public void ContainsAny_CanReassignDifferentTypes()
    {
        var c = new ContainsAny();
        c.Value = 42;
        Assert.IsType<int>(c.Value);

        c.Value = "now a string";
        Assert.IsType<string>(c.Value);

        c.Value = new List<double> { 1.1, 2.2 };
        Assert.IsType<List<double>>(c.Value);
    }

    [Fact]
    public void AnyFieldType_IsObject()
    {
        var valueProp = typeof(ContainsAny).GetProperty("Value");
        Assert.Equal(typeof(object), valueProp!.PropertyType);
    }
}
