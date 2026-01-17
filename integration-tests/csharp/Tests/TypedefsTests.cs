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
using TypedefTypes;

namespace IntegrationTests;

public class TypedefsTests
{
    [Fact]
    public void Point_Instantiation()
    {
        var p = new Point(3.14, 2.71);
        Assert.Equal(3.14, p.X);
        Assert.Equal(2.71, p.Y);
    }

    [Fact]
    public void Point_Defaults()
    {
        var p = new Point();
        Assert.Equal(0.0, p.X);
        Assert.Equal(0.0, p.Y);
    }

    [Fact]
    public void Point_FieldTypes()
    {
        var p = new Point(1.5, 2.5);
        Assert.IsType<double>(p.X);
        Assert.IsType<double>(p.Y);
    }

    [Fact]
    public void Person_Instantiation()
    {
        var person = new Person("Alice", 30, true);
        Assert.Equal("Alice", person.Name);
        Assert.Equal(30, person.Age);
        Assert.True(person.Active);
    }

    [Fact]
    public void Person_Defaults()
    {
        var person = new Person();
        Assert.Equal("", person.Name);
        Assert.Equal(0, person.Age);
        Assert.False(person.Active);
    }

    [Fact]
    public void Person_FieldTypes()
    {
        var person = new Person("Bob", 25, false);
        Assert.IsType<string>(person.Name);
        Assert.IsType<int>(person.Age);
        Assert.IsType<bool>(person.Active);
    }

    [Fact]
    public void Container_Instantiation()
    {
        var numbers = new List<int> { 1, 2, 3 };
        var labels = new List<string> { "a", "b" };
        var lookup = new Dictionary<string, int> { { "one", 1 }, { "two", 2 } };
        var c = new Container(numbers, labels, lookup);

        Assert.Equal(new[] { 1, 2, 3 }, c.Numbers);
        Assert.Equal(new[] { "a", "b" }, c.Labels);
        Assert.Equal(1, c.Lookup["one"]);
        Assert.Equal(2, c.Lookup["two"]);
    }

    [Fact]
    public void Container_Defaults()
    {
        var c = new Container();
        Assert.Empty(c.Numbers);
        Assert.Empty(c.Labels);
        Assert.Empty(c.Lookup);
    }

    [Fact]
    public void Container_FieldTypes()
    {
        var c = new Container();
        Assert.IsAssignableFrom<IList<int>>(c.Numbers);
        Assert.IsAssignableFrom<IList<string>>(c.Labels);
        Assert.IsAssignableFrom<IDictionary<string, int>>(c.Lookup);
    }

    [Fact]
    public void Measurement_Instantiation()
    {
        var m = new Measurement("temperature", 42);
        Assert.Equal("temperature", m.Name);
        Assert.Equal(42, m.Value);
    }

    [Fact]
    public void Measurement_Defaults()
    {
        var m = new Measurement();
        Assert.Equal("", m.Name);
        Assert.Equal(0, m.Value);
    }

    [Fact]
    public void Measurement_FieldTypes()
    {
        var m = new Measurement("test", 100);
        Assert.IsType<string>(m.Name);
        Assert.IsType<int>(m.Value);
    }

    [Fact]
    public void WithArrayTypedef_Instantiation()
    {
        var arr = new int[] { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
        var w = new WithArrayTypedef(arr);
        Assert.Equal(arr, w.Values);
    }

    [Fact]
    public void WithArrayTypedef_DefaultSize()
    {
        var w = new WithArrayTypedef();
        Assert.Equal(10, w.Values.Length);
    }

    [Fact]
    public void WithArrayTypedef_RejectsWrongSize()
    {
        var w = new WithArrayTypedef();
        Assert.Throws<ArgumentOutOfRangeException>(() => w.Values = new int[5]);
        Assert.Throws<ArgumentOutOfRangeException>(() => w.Values = new int[15]);
    }

    [Fact]
    public void DeepChainStruct_Instantiation()
    {
        var seq = new List<int> { 10, 20, 30 };
        var map = new Dictionary<string, int> { { "x", 1 }, { "y", 2 } };
        var d = new DeepChainStruct(42, seq, map);

        Assert.Equal(42, d.DeepInt);
        Assert.Equal(new[] { 10, 20, 30 }, d.DeepSeq);
        Assert.Equal(1, d.DeepMap["x"]);
        Assert.Equal(2, d.DeepMap["y"]);
    }

    [Fact]
    public void DeepChainStruct_Defaults()
    {
        var d = new DeepChainStruct();
        Assert.Equal(0, d.DeepInt);
        Assert.Empty(d.DeepSeq);
        Assert.Empty(d.DeepMap);
    }

    [Fact]
    public void DeepChainStruct_FieldTypes()
    {
        var d = new DeepChainStruct();
        Assert.IsType<int>(d.DeepInt);
        Assert.IsAssignableFrom<IList<int>>(d.DeepSeq);
        Assert.IsAssignableFrom<IDictionary<string, int>>(d.DeepMap);
    }

    [Fact]
    public void Point_Equality()
    {
        var p1 = new Point(1.0, 2.0);
        var p2 = new Point(1.0, 2.0);
        var p3 = new Point(3.0, 4.0);

        Assert.Equal(p1, p2);
        Assert.NotEqual(p1, p3);
    }

    [Fact]
    public void Person_Equality()
    {
        var a1 = new Person("Alice", 30, true);
        var a2 = new Person("Alice", 30, true);
        var b = new Person("Bob", 25, false);

        Assert.Equal(a1, a2);
        Assert.NotEqual(a1, b);
    }

    [Fact]
    public void Measurement_Equality()
    {
        var m1 = new Measurement("temp", 100);
        var m2 = new Measurement("temp", 100);
        var m3 = new Measurement("pressure", 50);

        Assert.Equal(m1, m2);
        Assert.NotEqual(m1, m3);
    }

    [Fact]
    public void Container_SameInstance_Equal()
    {
        var c = new Container();
        Assert.True(c.Equals(c));
    }

    [Fact]
    public void DeepChainStruct_Equality()
    {
        var d1 = new DeepChainStruct(42, new List<int>(), new Dictionary<string, int>());
        var d2 = new DeepChainStruct(42, new List<int>(), new Dictionary<string, int>());
        Assert.Equal(d1.DeepInt, d2.DeepInt);
    }

    [Fact]
    public void Point_CopyConstructor()
    {
        var p1 = new Point(5.5, 6.6);
        var p2 = new Point(p1);
        Assert.Equal(p1.X, p2.X);
        Assert.Equal(p1.Y, p2.Y);
        Assert.NotSame(p1, p2);
    }

    [Fact]
    public void Person_CopyConstructor()
    {
        var p1 = new Person("Alice", 30, true);
        var p2 = new Person(p1);
        Assert.Equal(p1.Name, p2.Name);
        Assert.Equal(p1.Age, p2.Age);
        Assert.Equal(p1.Active, p2.Active);
        Assert.NotSame(p1, p2);
    }
}
