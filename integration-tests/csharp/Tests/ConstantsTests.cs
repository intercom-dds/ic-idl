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
using ConstantTypes;
using LargeIntegerTypes;

namespace IntegrationTests;

public class ConstantsTests
{
    [Fact]
    public void IntConst_HasCorrectValue()
    {
        Assert.Equal(42, ConstantTypes.Constants.IntConst);
    }

    [Fact]
    public void UintConst_HasCorrectValue()
    {
        Assert.Equal(100u, ConstantTypes.Constants.UintConst);
    }

    [Fact]
    public void ShortConst_HasCorrectValue()
    {
        Assert.Equal(-10, ConstantTypes.Constants.ShortConst);
    }

    [Fact]
    public void LonglongConst_HasCorrectValue()
    {
        Assert.Equal(9999999999L, ConstantTypes.Constants.LonglongConst);
    }

    [Fact]
    public void DoubleConst_HasCorrectValue()
    {
        Assert.Equal(3.14159, ConstantTypes.Constants.DoubleConst, 5);
    }

    [Fact]
    public void FloatConst_HasCorrectValue()
    {
        Assert.Equal(2.5f, ConstantTypes.Constants.FloatConst);
    }

    [Fact]
    public void StringConst_HasCorrectValue()
    {
        Assert.Equal("hello world", ConstantTypes.Constants.StringConst);
    }

    [Fact]
    public void BoolTrue_HasCorrectValue()
    {
        Assert.True(ConstantTypes.Constants.BoolTrue);
    }

    [Fact]
    public void BoolFalse_HasCorrectValue()
    {
        Assert.False(ConstantTypes.Constants.BoolFalse);
    }

    [Fact]
    public void OctetConst_HasCorrectValue()
    {
        Assert.Equal(255, ConstantTypes.Constants.OctetConst);
    }

    [Fact]
    public void ChainedConstants_HaveCorrectValues()
    {
        Assert.Equal(10, ConstantTypes.Constants.Chain1);
        Assert.Equal(ConstantTypes.Constants.Chain1, ConstantTypes.Constants.Chain2);
        Assert.Equal(ConstantTypes.Constants.Chain2, ConstantTypes.Constants.Chain3);
        Assert.Equal(ConstantTypes.Constants.Chain3, ConstantTypes.Constants.Chain4);
        Assert.Equal(ConstantTypes.Constants.Chain4, ConstantTypes.Constants.Chain5);
    }

    [Fact]
    public void ArithmeticConstants_HaveCorrectValues()
    {
        Assert.Equal(100, ConstantTypes.Constants.ArithBase);
        Assert.Equal(200, ConstantTypes.Constants.ArithDoubled);
        Assert.Equal(400, ConstantTypes.Constants.ArithQuadrupled);
        Assert.Equal(800, ConstantTypes.Constants.ArithOctupled);
    }

    [Fact]
    public void MathConstants_HaveCorrectValues()
    {
        Assert.Equal(5, ConstantTypes.Constants.Math1);
        Assert.Equal(15, ConstantTypes.Constants.Math2);
        Assert.Equal(30, ConstantTypes.Constants.Math3);
        Assert.Equal(25, ConstantTypes.Constants.Math4);
        Assert.Equal(5, ConstantTypes.Constants.Math5);
    }

    [Fact]
    public void NegationConstants_HaveCorrectValues()
    {
        Assert.Equal(-50, ConstantTypes.Constants.Negative);
        Assert.Equal(50, ConstantTypes.Constants.Negated);
        Assert.Equal(-50, ConstantTypes.Constants.DoubleNegated);
    }

    [Fact]
    public void BitwiseConstants_HaveCorrectValues()
    {
        Assert.Equal(15, ConstantTypes.Constants.BitsA);
        Assert.Equal(240, ConstantTypes.Constants.BitsB);
        Assert.Equal(255, ConstantTypes.Constants.BitsOr);
        Assert.Equal(255, ConstantTypes.Constants.BitsAnd);
        Assert.Equal(240, ConstantTypes.Constants.BitsXor);
        Assert.Equal(16, ConstantTypes.Constants.BitsShiftLeft);
        Assert.Equal(16, ConstantTypes.Constants.BitsShiftRight);
    }

    [Fact]
    public void FloatConstants_HaveCorrectValues()
    {
        Assert.Equal(1.0, ConstantTypes.Constants.FloatA);
        Assert.Equal(1.5, ConstantTypes.Constants.FloatB);
        Assert.Equal(3.0, ConstantTypes.Constants.FloatC);
        Assert.Equal(0.75, ConstantTypes.Constants.FloatD);
    }

    [Fact]
    public void PriorityValue_HasCorrectValue()
    {
        Assert.Equal(Priority.High, ConstantTypes.Constants.PriorityValue);
        Assert.Equal(100, (int)ConstantTypes.Constants.PriorityValue);
    }

    [Fact]
    public void StringConstants_HaveCorrectValues()
    {
        Assert.Equal("Hello", ConstantTypes.Constants.Greeting);
        Assert.Equal("Goodbye", ConstantTypes.Constants.Farewell);
    }

    [Fact]
    public void ParenthesizedConstants_HaveCorrectValues()
    {
        Assert.Equal(30, ConstantTypes.Constants.ParenA);
        Assert.Equal(20, ConstantTypes.Constants.ParenB);
        Assert.Equal(25, ConstantTypes.Constants.ParenC);
    }

    [Fact]
    public void ModuloConstants_HaveCorrectValues()
    {
        Assert.Equal(2, ConstantTypes.Constants.ModA);
        Assert.Equal(2, ConstantTypes.Constants.ModB);
    }

    // Large integer tests (LargeIntegerTypes namespace)
    [Fact]
    public void LargeIntegers_OctetBounds()
    {
        Assert.Equal(255, LargeIntegerTypes.Constants.OctetMax);
        Assert.Equal(0, LargeIntegerTypes.Constants.OctetMin);
    }

    [Fact]
    public void LargeIntegers_ShortBounds()
    {
        Assert.Equal(32767, LargeIntegerTypes.Constants.ShortMax);
        Assert.Equal(-32768, LargeIntegerTypes.Constants.ShortMin);
    }

    [Fact]
    public void LargeIntegers_UshortBounds()
    {
        Assert.Equal(65535, LargeIntegerTypes.Constants.UshortMax);
        Assert.Equal(0, LargeIntegerTypes.Constants.UshortMin);
    }

    [Fact]
    public void LargeIntegers_LongBounds()
    {
        Assert.Equal(2147483647, LargeIntegerTypes.Constants.LongMax);
        Assert.Equal(-2147483648, LargeIntegerTypes.Constants.LongMin);
    }

    [Fact]
    public void LargeIntegers_UlongBounds()
    {
        Assert.Equal(4294967295u, LargeIntegerTypes.Constants.UlongMax);
        Assert.Equal(0u, LargeIntegerTypes.Constants.UlongMin);
    }

    [Fact]
    public void LargeIntegers_LonglongBounds()
    {
        Assert.Equal(9223372036854775807L, LargeIntegerTypes.Constants.LonglongMax);
        Assert.Equal(-9223372036854775808L, LargeIntegerTypes.Constants.LonglongMin);
    }

    [Fact]
    public void LargeIntegers_UlonglongBounds()
    {
        Assert.Equal(18446744073709551615UL, LargeIntegerTypes.Constants.UlonglongMax);
        Assert.Equal(0UL, LargeIntegerTypes.Constants.UlonglongMin);
    }

    [Fact]
    public void LargeIntegers_HexValues()
    {
        Assert.Equal(0xDEADBEEFu, LargeIntegerTypes.Constants.HexDeadbeef);
        Assert.Equal(0xFFFFFFFFu, LargeIntegerTypes.Constants.HexFfffffff);
        Assert.Equal(0x123456789ABCDEF0L, LargeIntegerTypes.Constants.Hex64Bit);
    }

    [Fact]
    public void LargeIntegers_OctalValues()
    {
        Assert.Equal(511, LargeIntegerTypes.Constants.Octal777);
        Assert.Equal(2147483647, LargeIntegerTypes.Constants.OctalLarge);
    }

    [Fact]
    public void LargeIntegers_BoundaryArithmetic()
    {
        Assert.Equal(2147483646, LargeIntegerTypes.Constants.LongMaxMinusOne);
        Assert.Equal(9223372036854775806L, LargeIntegerTypes.Constants.LonglongMaxMinusOne);
    }

    [Fact]
    public void LargeIntFields_CanHoldLargeValues()
    {
        var fields = new LargeIntFields(
            BigSigned: long.MaxValue,
            BigUnsigned: ulong.MaxValue
        );
        Assert.Equal(long.MaxValue, fields.BigSigned);
        Assert.Equal(ulong.MaxValue, fields.BigUnsigned);
    }
}
