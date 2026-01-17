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
using UnionTypes;

namespace IntegrationTests;

public class UnionsTests
{
    [Fact]
    public void IntOrString_DefaultState()
    {
        var u = new IntOrString();
        Assert.Equal(0, u.Discriminator);
    }

    [Fact]
    public void IntOrString_SetIntVal()
    {
        var u = new IntOrString();
        u.IntVal = 42;
        Assert.Equal(1, u.Discriminator);
        Assert.Equal(42, u.IntVal);
    }

    [Fact]
    public void IntOrString_SetStrVal()
    {
        var u = new IntOrString();
        u.StrVal = "hello";
        Assert.Equal(2, u.Discriminator);
        Assert.Equal("hello", u.StrVal);
    }

    [Fact]
    public void IntOrString_AccessWrongMemberThrows()
    {
        var u = new IntOrString();
        u.IntVal = 42;
        Assert.Throws<InvalidOperationException>(() => u.StrVal);
    }

    [Fact]
    public void IntOrString_SwitchingVariantClearsOther()
    {
        var u = new IntOrString();
        u.IntVal = 42;
        Assert.Equal(1, u.Discriminator);

        u.StrVal = "hello";
        Assert.Equal(2, u.Discriminator);
        Assert.Equal("hello", u.StrVal);
        Assert.Throws<InvalidOperationException>(() => u.IntVal);
    }

    [Fact]
    public void IntOrString_CopyConstructor()
    {
        var u1 = new IntOrString();
        u1.IntVal = 99;
        var u2 = new IntOrString(u1);
        Assert.Equal(u1.Discriminator, u2.Discriminator);
        Assert.Equal(u1.IntVal, u2.IntVal);
    }

    [Fact]
    public void IntOrString_Equality()
    {
        var u1 = new IntOrString();
        var u2 = new IntOrString();
        u1.IntVal = 42;
        u2.IntVal = 42;
        Assert.Equal(u1, u2);

        u2.IntVal = 100;
        Assert.NotEqual(u1, u2);
    }

    [Fact]
    public void TypedValue_EnumDiscriminator()
    {
        var v = new TypedValue();
        v.IntValue = 123;
        Assert.Equal(ValueKind.IntKind, v.Discriminator);
        Assert.Equal(123, v.IntValue);
    }

    [Fact]
    public void TypedValue_FloatValue()
    {
        var v = new TypedValue();
        v.FloatValue = 3.14;
        Assert.Equal(ValueKind.FloatKind, v.Discriminator);
        Assert.Equal(3.14, v.FloatValue);
    }

    [Fact]
    public void TypedValue_StringValue()
    {
        var v = new TypedValue();
        v.StringValue = "test";
        Assert.Equal(ValueKind.StringKind, v.Discriminator);
        Assert.Equal("test", v.StringValue);
    }

    [Fact]
    public void BoolSwitch_TrueCase()
    {
        var b = new BoolSwitch();
        b.TrueVal = 42;
        Assert.True(b.Discriminator);
        Assert.Equal(42, b.TrueVal);
    }

    [Fact]
    public void BoolSwitch_FalseCase()
    {
        var b = new BoolSwitch();
        b.FalseVal = "hello";
        Assert.False(b.Discriminator);
        Assert.Equal("hello", b.FalseVal);
    }

    [Fact]
    public void MultiCase_SmallValWithSetMethod()
    {
        var m = new MultiCase();
        m.SetSmallVal(10, 2);
        Assert.Equal(2, m.Discriminator);
        Assert.Equal(10, m.SmallVal);
    }

    [Fact]
    public void MultiCase_SmallValMultipleDiscriminators()
    {
        var m = new MultiCase();
        m.SetSmallVal(5, 1);
        Assert.Equal(1, m.Discriminator);
        Assert.Equal(5, m.SmallVal);

        m.SetSmallVal(6, 3);
        Assert.Equal(3, m.Discriminator);
        Assert.Equal(6, m.SmallVal);
    }

    [Fact]
    public void MultiCase_InvalidDiscriminatorThrows()
    {
        var m = new MultiCase();
        Assert.Throws<ArgumentException>(() => m.SetSmallVal(10, 5));
    }

    [Fact]
    public void MultiCase_TextVal()
    {
        var m = new MultiCase();
        m.SetTextVal("test", 10);
        Assert.Equal(10, m.Discriminator);
        Assert.Equal("test", m.TextVal);

        m.SetTextVal("test2", 20);
        Assert.Equal(20, m.Discriminator);
        Assert.Equal("test2", m.TextVal);
    }

    [Fact]
    public void IntOrString_DefaultVal()
    {
        var u = new IntOrString();
        u.DefaultVal = true;
        Assert.Equal(0, u.Discriminator);
        Assert.True(u.DefaultVal);
        Assert.Throws<InvalidOperationException>(() => u.IntVal);
        Assert.Throws<InvalidOperationException>(() => u.StrVal);
    }
}
