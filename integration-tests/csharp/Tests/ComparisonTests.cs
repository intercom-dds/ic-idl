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
using UnionTypes;
using ExceptionTypes;

namespace IntegrationTests;

public class ComparisonTests
{
    [Fact]
    public void Struct_Equality()
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
    public void Struct_GetHashCode_ConsistentWithEquals()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(10, 20);
        Assert.Equal(p1.GetHashCode(), p2.GetHashCode());
    }

    [Fact]
    public void Struct_GetHashCode_DifferentValuesUsuallyDifferent()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(30, 40);
        Assert.NotEqual(p1.GetHashCode(), p2.GetHashCode());
    }

    [Fact]
    public void Struct_CanBeUsedInHashSet()
    {
        var set = new HashSet<Point>();
        var p1 = new Point(10, 20);
        var p2 = new Point(10, 20);
        var p3 = new Point(30, 40);

        set.Add(p1);
        Assert.Contains(p2, set);
        Assert.DoesNotContain(p3, set);
    }

    [Fact]
    public void Struct_CanBeUsedAsDictionaryKey()
    {
        var dict = new Dictionary<Point, string>();
        var p1 = new Point(10, 20);
        dict[p1] = "test";

        var p2 = new Point(10, 20);
        Assert.Equal("test", dict[p2]);
    }

    [Fact]
    public void Union_Equality()
    {
        var u1 = new IntOrString();
        var u2 = new IntOrString();
        u1.IntVal = 42;
        u2.IntVal = 42;
        Assert.Equal(u1, u2);

        var u3 = new IntOrString();
        u3.StrVal = "hello";
        Assert.NotEqual(u1, u3);
    }

    [Fact]
    public void Union_GetHashCode_ConsistentWithEquals()
    {
        var u1 = new IntOrString();
        var u2 = new IntOrString();
        u1.IntVal = 42;
        u2.IntVal = 42;
        Assert.Equal(u1.GetHashCode(), u2.GetHashCode());
    }

    [Fact]
    public void Exception_FieldEquality()
    {
        var e1 = new SimpleError(100, "error");
        var e2 = new SimpleError(100, "error");
        var e3 = new SimpleError(200, "different");

        Assert.Equal(e1.ErrorCode, e2.ErrorCode);
        Assert.Equal(e1.Message, e2.Message);
        Assert.NotEqual(e1.ErrorCode, e3.ErrorCode);
        Assert.NotEqual(e1.Message, e3.Message);
    }

    [Fact]
    public void AllPrimitives_Equality()
    {
        var a1 = new AllPrimitives(
            BoolVal: true,
            ByteVal: 255,
            ShortVal: -100,
            UshortVal: 65000,
            LongVal: -2000000000,
            UlongVal: 4000000000,
            LonglongVal: -9000000000000000000,
            UlonglongVal: 18000000000000000000,
            FloatVal: 3.14f,
            DoubleVal: 2.71828,
            StringVal: "test"
        );
        var a2 = new AllPrimitives(
            BoolVal: true,
            ByteVal: 255,
            ShortVal: -100,
            UshortVal: 65000,
            LongVal: -2000000000,
            UlongVal: 4000000000,
            LonglongVal: -9000000000000000000,
            UlonglongVal: 18000000000000000000,
            FloatVal: 3.14f,
            DoubleVal: 2.71828,
            StringVal: "test"
        );
        Assert.Equal(a1, a2);
        Assert.Equal(a1.GetHashCode(), a2.GetHashCode());
    }

    [Fact]
    public void Struct_NullComparison()
    {
        var p1 = new Point(10, 20);
        Point? p2 = null;

        Assert.False(p1.Equals(p2));
    }

    [Fact]
    public void Struct_ReferenceEquality()
    {
        var p1 = new Point(10, 20);
        var p2 = p1;

        Assert.True(ReferenceEquals(p1, p2));
        Assert.True(p1.Equals(p2));
    }

    [Fact]
    public void Struct_CompareTo_Equal()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(10, 20);

        Assert.Equal(0, p1.CompareTo(p2));
    }

    [Fact]
    public void Struct_CompareTo_LessThan()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(10, 30);
        var p3 = new Point(20, 10);

        Assert.True(p1.CompareTo(p2) < 0);
        Assert.True(p1.CompareTo(p3) < 0);
    }

    [Fact]
    public void Struct_CompareTo_GreaterThan()
    {
        var p1 = new Point(10, 30);
        var p2 = new Point(10, 20);

        Assert.True(p1.CompareTo(p2) > 0);
    }

    [Fact]
    public void Struct_CompareTo_Null()
    {
        var p1 = new Point(10, 20);

        Assert.True(p1.CompareTo(null) > 0);
    }

    [Fact]
    public void Struct_CompareTo_ReferenceEquals()
    {
        var p1 = new Point(10, 20);

        Assert.Equal(0, p1.CompareTo(p1));
    }

    [Fact]
    public void Struct_CompareTo_Sorting()
    {
        var points = new List<Point>
        {
            new Point(2, 1),
            new Point(1, 3),
            new Point(1, 2)
        };

        points.Sort();

        Assert.Equal(new Point(1, 2), points[0]);
        Assert.Equal(new Point(1, 3), points[1]);
        Assert.Equal(new Point(2, 1), points[2]);
    }

    [Fact]
    public void Struct_CompareTo_AllPrimitives()
    {
        var a1 = new AllPrimitives(
            BoolVal: false,
            ByteVal: 100,
            ShortVal: -50,
            UshortVal: 30000,
            LongVal: -1000000000,
            UlongVal: 2000000000,
            LonglongVal: -5000000000000000000,
            UlonglongVal: 10000000000000000000,
            FloatVal: 2.5f,
            DoubleVal: 1.5,
            StringVal: "abc"
        );
        var a2 = new AllPrimitives(
            BoolVal: false,
            ByteVal: 100,
            ShortVal: -50,
            UshortVal: 30000,
            LongVal: -1000000000,
            UlongVal: 2000000000,
            LonglongVal: -5000000000000000000,
            UlonglongVal: 10000000000000000000,
            FloatVal: 2.5f,
            DoubleVal: 1.5,
            StringVal: "xyz"
        );

        Assert.True(a1.CompareTo(a2) < 0);
    }

    [Fact]
    public void Union_CompareTo()
    {
        var u1 = new IntOrString();
        u1.IntVal = 10;

        var u2 = new IntOrString();
        u2.IntVal = 20;

        var u3 = new IntOrString();
        u3.IntVal = 10;

        Assert.True(u1.CompareTo(u2) < 0);
        Assert.True(u2.CompareTo(u1) > 0);
        Assert.Equal(0, u1.CompareTo(u3));
    }

    [Fact]
    public void Struct_CompareTo_Reflexive()
    {
        var p1 = new Point(10, 20);
        var p2 = new Point(5, 15);
        var p3 = new Point(-100, 200);

        Assert.Equal(0, p1.CompareTo(p1));
        Assert.Equal(0, p2.CompareTo(p2));
        Assert.Equal(0, p3.CompareTo(p3));
    }

    [Fact]
    public void AllPrimitives_CompareTo_Reflexive()
    {
        var a = new AllPrimitives(
            BoolVal: true,
            ByteVal: 255,
            ShortVal: -100,
            UshortVal: 65000,
            LongVal: -2000000000,
            UlongVal: 4000000000,
            LonglongVal: -9000000000000000000,
            UlonglongVal: 18000000000000000000,
            FloatVal: 3.14f,
            DoubleVal: 2.71828,
            StringVal: "test"
        );

        Assert.Equal(0, a.CompareTo(a));
    }

    [Fact]
    public void Union_CompareTo_Reflexive()
    {
        var u1 = new IntOrString();
        u1.IntVal = 42;

        var u2 = new IntOrString();
        u2.StrVal = "hello";

        Assert.Equal(0, u1.CompareTo(u1));
        Assert.Equal(0, u2.CompareTo(u2));
    }
}
