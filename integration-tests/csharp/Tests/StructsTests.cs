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
using StructTypes;

namespace IntegrationTests;

public class StructsTests
{
    [Fact]
    public void Point_DefaultConstructor()
    {
        var p = new Point();
        Assert.Equal(0, p.X);
        Assert.Equal(0, p.Y);
    }

    [Fact]
    public void Point_ParameterizedConstructor()
    {
        var p = new Point(10, 20);
        Assert.Equal(10, p.X);
        Assert.Equal(20, p.Y);
    }

    [Fact]
    public void Point_CopyConstructor()
    {
        var p1 = new Point(5, 10);
        var p2 = new Point(p1);
        Assert.Equal(p1.X, p2.X);
        Assert.Equal(p1.Y, p2.Y);
        Assert.NotSame(p1, p2);
    }

    [Fact]
    public void Point_PropertyModification()
    {
        var p = new Point(5, 10);
        p.X = 100;
        p.Y = 200;
        Assert.Equal(100, p.X);
        Assert.Equal(200, p.Y);
    }

    [Fact]
    public void Point_Equality()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(10, 20);
        var p3 = new Point(5, 10);

        Assert.Equal(p1, p2);
        Assert.NotEqual(p1, p3);
        Assert.True(p1.Equals(p2));
        Assert.False(p1.Equals(p3));
    }

    [Fact]
    public void Point_GetHashCode()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(10, 20);
        Assert.Equal(p1.GetHashCode(), p2.GetHashCode());
    }

    [Fact]
    public void Point3D_Inheritance()
    {
        var p3d = new Point3D { X = 1, Y = 2, Z = 3 };
        Assert.Equal(1, p3d.X);
        Assert.Equal(2, p3d.Y);
        Assert.Equal(3, p3d.Z);
        Assert.IsAssignableFrom<Point>(p3d);
    }

    [Fact]
    public void Point4D_MultiLevelInheritance()
    {
        var p4d = new Point4D { X = 1, Y = 2, Z = 3, W = 4 };
        Assert.Equal(1, p4d.X);
        Assert.Equal(2, p4d.Y);
        Assert.Equal(3, p4d.Z);
        Assert.Equal(4, p4d.W);
        Assert.IsAssignableFrom<Point3D>(p4d);
        Assert.IsAssignableFrom<Point>(p4d);
    }

    [Fact]
    public void Rectangle_NestedStruct()
    {
        var tl = new Point(0, 0);
        var br = new Point(100, 100);
        var rect = new Rectangle(tl, br);
        Assert.Equal(0, rect.TopLeft.X);
        Assert.Equal(100, rect.BottomRight.Y);
    }

    [Fact]
    public void Empty_CanBeInstantiated()
    {
        var e = new Empty();
        Assert.NotNull(e);
    }

    [Fact]
    public void AllPrimitives_Defaults()
    {
        var p = new AllPrimitives();
        Assert.False(p.BoolVal);
        Assert.Equal(0, p.ByteVal);
        Assert.Equal(0, p.ShortVal);
        Assert.Equal(0u, p.UshortVal);
        Assert.Equal(0, p.LongVal);
        Assert.Equal(0u, p.UlongVal);
        Assert.Equal(0L, p.LonglongVal);
        Assert.Equal(0UL, p.UlonglongVal);
        Assert.Equal(0.0f, p.FloatVal);
        Assert.Equal(0.0, p.DoubleVal);
        Assert.Equal("", p.StringVal);
    }

    [Fact]
    public void AllPrimitives_AllFieldTypes()
    {
        var p = new AllPrimitives(
            BoolVal: true,
            ByteVal: 255,
            ShortVal: -32000,
            UshortVal: 65000,
            LongVal: -2000000000,
            UlongVal: 4000000000,
            LonglongVal: -9000000000000000000,
            UlonglongVal: 18000000000000000000,
            FloatVal: 3.14f,
            DoubleVal: 2.71828,
            StringVal: "hello"
        );
        Assert.True(p.BoolVal);
        Assert.Equal(255, p.ByteVal);
        Assert.Equal(-32000, p.ShortVal);
        Assert.Equal(65000, p.UshortVal);
        Assert.Equal(-2000000000, p.LongVal);
        Assert.Equal(4000000000u, p.UlongVal);
        Assert.Equal(-9000000000000000000, p.LonglongVal);
        Assert.Equal(18000000000000000000ul, p.UlonglongVal);
        Assert.Equal(3.14f, p.FloatVal);
        Assert.Equal(2.71828, p.DoubleVal);
        Assert.Equal("hello", p.StringVal);
    }

    [Fact]
    public void WithDefaults_DefaultConstructor()
    {
        var w = new WithDefaults();
        Assert.Equal(0, w.Count);
        Assert.Equal("", w.Name);
        Assert.Equal(0.0, w.Value);
    }

    [Fact]
    public void WithDefaults_ParameterizedConstructor()
    {
        var w = new WithDefaults(Count: 42, Name: "test", Value: 3.14);
        Assert.Equal(42, w.Count);
        Assert.Equal("test", w.Name);
        Assert.Equal(3.14, w.Value);
    }

    [Fact]
    public void WithSequence_EmptyByDefault()
    {
        var w = new WithSequence();
        Assert.Empty(w.Numbers);
        Assert.Empty(w.Names);
    }

    [Fact]
    public void WithSequence_CanAddElements()
    {
        var w = new WithSequence();
        w.Numbers.Add(1);
        w.Numbers.Add(2);
        w.Numbers.Add(3);
        w.Names.Add("alice");
        w.Names.Add("bob");
        Assert.Equal(3, w.Numbers.Count);
        Assert.Equal(2, w.Names.Count);
        Assert.Equal(new[] { 1, 2, 3 }, w.Numbers);
        Assert.Equal(new[] { "alice", "bob" }, w.Names);
    }

    [Fact]
    public void WithArray_FixedSize()
    {
        var w = new WithArray();
        Assert.Equal(5, w.FixedNumbers.Length);
    }

    [Fact]
    public void WithArray_RejectsWrongSize()
    {
        var w = new WithArray();
        Assert.Throws<ArgumentOutOfRangeException>(() => w.FixedNumbers = new int[3]);
        Assert.Throws<ArgumentOutOfRangeException>(() => w.FixedNumbers = new int[10]);
    }

    [Fact]
    public void WithArray_AcceptsCorrectSize()
    {
        var w = new WithArray(new int[] { 1, 2, 3, 4, 5 });
        Assert.Equal(new[] { 1, 2, 3, 4, 5 }, w.FixedNumbers);
    }

    [Fact]
    public void WithMap_EmptyByDefault()
    {
        var w = new WithMap();
        Assert.Empty(w.StringToInt);
    }

    [Fact]
    public void WithMap_CanAddEntries()
    {
        var w = new WithMap();
        w.StringToInt["one"] = 1;
        w.StringToInt["two"] = 2;
        Assert.Equal(2, w.StringToInt.Count);
        Assert.Equal(1, w.StringToInt["one"]);
        Assert.Equal(2, w.StringToInt["two"]);
    }

    [Fact]
    public void Point3D_ParameterizedConstructor()
    {
        var p = new Point3D(1, 2, 3);
        Assert.Equal(1, p.X);
        Assert.Equal(2, p.Y);
        Assert.Equal(3, p.Z);
    }

    [Fact]
    public void Point4D_ParameterizedConstructor()
    {
        var p = new Point4D(1, 2, 3, 4);
        Assert.Equal(1, p.X);
        Assert.Equal(2, p.Y);
        Assert.Equal(3, p.Z);
        Assert.Equal(4, p.W);
    }
}
