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
using DefaultTypes;

namespace IntegrationTests;

public class DefaultsTests
{
    [Fact]
    public void Constants_StringValues()
    {
        Assert.Equal("unnamed", Constants.DefaultName);
        Assert.Equal(100, Constants.DefaultCount);
        Assert.Equal(0.5, Constants.DefaultRate);
    }

    [Fact]
    public void Constants_StructInitializer()
    {
        Assert.Equal(10, Constants.DefaultInner.X);
        Assert.Equal("default", Constants.DefaultInner.Y);

        Assert.Equal(99, Constants.NestedInner.X);
        Assert.Equal("nested", Constants.NestedInner.Y);
    }

    [Fact]
    public void PrimitiveDefaults_BoolDefaults()
    {
        var p = new PrimitiveDefaults();
        Assert.False(p.BoolEmpty);
        Assert.False(p.BoolTrue);
        Assert.False(p.BoolFalse);
    }

    [Fact]
    public void PrimitiveDefaults_IntDefaults()
    {
        var p = new PrimitiveDefaults();
        Assert.Equal(0, p.IntEmpty);
        Assert.Equal(0, p.IntValue);
        Assert.Equal(0, p.IntNegative);
    }

    [Fact]
    public void PrimitiveDefaults_FloatDefaults()
    {
        var p = new PrimitiveDefaults();
        Assert.Equal(0.0, p.FloatEmpty);
        Assert.Equal(0.0, p.FloatValue);
        Assert.Equal(0.0, p.FloatNegative);
    }

    [Fact]
    public void PrimitiveDefaults_StringDefaults()
    {
        var p = new PrimitiveDefaults();
        Assert.Equal("", p.StringEmpty);
        Assert.Equal("", p.StringValue);
        Assert.Equal("", p.StringFromConst);
    }

    [Fact]
    public void ArrayDefaults_Sizes()
    {
        var a = new ArrayDefaults();
        Assert.Equal(3, a.ArrayEmpty.Length);
        Assert.Equal(3, a.ArrayValues.Length);
        Assert.Equal(2, a.ArrayPartial.Length);
        Assert.Equal(2, a.StringArrayEmpty.Length);
        Assert.Equal(2, a.StringArrayValues.Length);
    }

    [Fact]
    public void ArrayDefaults_IntArrayDefaults()
    {
        var a = new ArrayDefaults();
        Assert.All(a.ArrayEmpty, v => Assert.Equal(0, v));
    }

    [Fact]
    public void ArrayDefaults_StringArrayDefaults()
    {
        var a = new ArrayDefaults();
        Assert.All(a.StringArrayEmpty, v => Assert.Null(v));
    }

    [Fact]
    public void SequenceDefaults_EmptyByDefault()
    {
        var s = new SequenceDefaults();
        Assert.Empty(s.SeqEmpty);
        Assert.Empty(s.SeqValues);
        Assert.Empty(s.StringSeqEmpty);
        Assert.Empty(s.StringSeqValues);
    }

    [Fact]
    public void SequenceDefaults_CanAddElements()
    {
        var s = new SequenceDefaults();
        s.SeqEmpty.Add(1);
        s.SeqEmpty.Add(2);
        Assert.Equal(2, s.SeqEmpty.Count);
    }

    [Fact]
    public void MapDefaults_EmptyByDefault()
    {
        var m = new MapDefaults();
        Assert.Empty(m.MapEmpty);
        Assert.Empty(m.MapValues);
        Assert.Empty(m.ReverseMapEmpty);
        Assert.Empty(m.ReverseMapValues);
    }

    [Fact]
    public void MapDefaults_CanAddEntries()
    {
        var m = new MapDefaults();
        m.MapEmpty["key"] = 42;
        Assert.Single(m.MapEmpty);
        Assert.Equal(42, m.MapEmpty["key"]);
    }

    [Fact]
    public void Inner_Defaults()
    {
        var i = new Inner();
        Assert.Equal(0, i.X);
        Assert.Equal("", i.Y);
    }

    [Fact]
    public void OuterDefaults_HasInnerDefaults()
    {
        var o = new OuterDefaults();
        Assert.NotNull(o.InnerEmpty);
        Assert.NotNull(o.InnerLiteral);
        Assert.NotNull(o.InnerFromConst);
    }

    [Fact]
    public void Priority_EnumExists()
    {
        Assert.True(Enum.IsDefined(typeof(Priority), "Low"));
        Assert.True(Enum.IsDefined(typeof(Priority), "Medium"));
        Assert.True(Enum.IsDefined(typeof(Priority), "High"));
    }

    [Fact]
    public void Priority_Values()
    {
        Assert.Equal(0, (int)Priority.Low);
        Assert.Equal(1, (int)Priority.Medium);
        Assert.Equal(2, (int)Priority.High);
    }

    [Fact]
    public void EnumDefaults_HasPriorityFields()
    {
        var e = new EnumDefaults();
        Assert.Equal(Priority.Low, e.PriorityEmpty);
        Assert.Equal(Priority.Low, e.PriorityHigh);
    }

    [Fact]
    public void OptionalFields_Defaults()
    {
        var o = new OptionalFields();
        Assert.Equal(0, o.MaybeInt);
        Assert.Equal("", o.MaybeString);
        Assert.NotNull(o.MaybeStruct);
    }

    [Fact]
    public void OptionalFields_CanBeSet()
    {
        var o = new OptionalFields();
        o.MaybeInt = 42;
        o.MaybeString = "hello";
        o.MaybeStruct = new Inner(10, "test");

        Assert.Equal(42, o.MaybeInt);
        Assert.Equal("hello", o.MaybeString);
        Assert.Equal(10, o.MaybeStruct.X);
        Assert.Equal("test", o.MaybeStruct.Y);
    }

    [Fact]
    public void ComplexDefaults_NestedSeqEmpty()
    {
        var c = new ComplexDefaults();
        Assert.Empty(c.NestedSeq);
    }

    [Fact]
    public void ComplexDefaults_MapOfSeqEmpty()
    {
        var c = new ComplexDefaults();
        Assert.Empty(c.MapOfSeq);
    }

    [Fact]
    public void ComplexDefaults_CanAddNestedData()
    {
        var c = new ComplexDefaults();
        c.NestedSeq.Add(new List<int> { 1, 2, 3 });
        c.MapOfSeq["key"] = new List<int> { 4, 5, 6 };

        Assert.Single(c.NestedSeq);
        Assert.Equal(new[] { 1, 2, 3 }, c.NestedSeq[0]);
        Assert.Single(c.MapOfSeq);
        Assert.Equal(new[] { 4, 5, 6 }, c.MapOfSeq["key"]);
    }
}
