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
using EnumTypes;

namespace IntegrationTests;

public class EnumsTests
{
    [Fact]
    public void Color_MembersExist()
    {
        Assert.True(Enum.IsDefined(typeof(Color), "Red"));
        Assert.True(Enum.IsDefined(typeof(Color), "Green"));
        Assert.True(Enum.IsDefined(typeof(Color), "Blue"));
    }

    [Fact]
    public void Color_IsEnumType()
    {
        Assert.True(typeof(Color).IsEnum);
    }

    [Fact]
    public void Color_HasCorrectValues()
    {
        Assert.Equal(0, (int)Color.Red);
        Assert.Equal(1, (int)Color.Green);
        Assert.Equal(2, (int)Color.Blue);
    }

    [Fact]
    public void Color_CanBeUsedInSwitch()
    {
        var color = Color.Green;
        var result = color switch
        {
            Color.Red => "red",
            Color.Green => "green",
            Color.Blue => "blue",
            _ => "unknown"
        };
        Assert.Equal("green", result);
    }

    [Fact]
    public void Status_HasExplicitValues()
    {
        Assert.Equal(0, (int)Status.Ok);
        Assert.Equal(100, (int)Status.Warning);
        Assert.Equal(200, (int)Status.Error);
    }

    [Fact]
    public void GappedEnum_HasCorrectGaps()
    {
        Assert.Equal(0, (int)GappedEnum.First);
        Assert.Equal(5, (int)GappedEnum.Second);
        Assert.Equal(10, (int)GappedEnum.Third);
        Assert.Equal(100, (int)GappedEnum.Fourth);
    }

    [Fact]
    public void NegativeEnum_HasNegativeValues()
    {
        Assert.Equal(-2, (int)NegativeEnum.NegTwo);
        Assert.Equal(-1, (int)NegativeEnum.NegOne);
        Assert.Equal(0, (int)NegativeEnum.Zero);
        Assert.Equal(1, (int)NegativeEnum.PosOne);
    }

    [Fact]
    public void MixedEnum_AutoAndExplicitValues()
    {
        Assert.Equal(0, (int)MixedEnum.AutoFirst);
        Assert.Equal(10, (int)MixedEnum.ExplicitTen);
        Assert.Equal(11, (int)MixedEnum.AutoEleven);
        Assert.Equal(100, (int)MixedEnum.ExplicitHundred);
        Assert.Equal(101, (int)MixedEnum.AutoHundredOne);
    }

    [Fact]
    public void Enum_CanCastToInt()
    {
        int value = (int)Status.Warning;
        Assert.Equal(100, value);
    }

    [Fact]
    public void Enum_CanCastFromInt()
    {
        var status = (Status)100;
        Assert.Equal(Status.Warning, status);
    }

    [Fact]
    public void Enum_Equality()
    {
        var a = Color.Red;
        var b = Color.Red;
        var c = Color.Blue;
        Assert.Equal(a, b);
        Assert.NotEqual(a, c);
    }

    [Fact]
    public void Color_Iteration()
    {
        var values = Enum.GetValues<Color>();
        Assert.Equal(3, values.Length);
        Assert.Contains(Color.Red, values);
        Assert.Contains(Color.Green, values);
        Assert.Contains(Color.Blue, values);
    }

    [Fact]
    public void Enum_ByName()
    {
        Assert.Equal(Color.Red, Enum.Parse<Color>("Red"));
        Assert.Equal(Status.Error, Enum.Parse<Status>("Error"));
    }

    [Fact]
    public void Enum_NameProperty()
    {
        Assert.Equal("Red", Color.Red.ToString());
        Assert.Equal("Warning", Status.Warning.ToString());
    }

    [Fact]
    public void Constants_EnumConst()
    {
        Assert.Equal(Status.Warning, Constants.EnumConst);
    }
}
