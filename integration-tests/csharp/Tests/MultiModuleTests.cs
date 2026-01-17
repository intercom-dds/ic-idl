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
using ModuleA;
using ModuleB;
using ConstantsOnly;
using EnumsOnly;

namespace IntegrationTests;

public class MultiModuleTests
{
    [Fact]
    public void ModuleA_Exists()
    {
        Assert.NotNull(typeof(StructA1));
    }

    [Fact]
    public void ModuleB_Exists()
    {
        Assert.NotNull(typeof(StructB1));
    }

    [Fact]
    public void ModuleA_FirstOpening()
    {
        var s = new StructA1();
        Assert.NotNull(s);
        Assert.Equal(100, ModuleA.Constants.ConstA1);
    }

    [Fact]
    public void ModuleA_SecondOpening()
    {
        var s = new StructA2();
        Assert.NotNull(s);
        Assert.Equal(101, ModuleA.Constants.ConstA2);
    }

    [Fact]
    public void ModuleA_ThirdOpening()
    {
        var s = new StructA3();
        Assert.NotNull(s);
        Assert.Equal(102, ModuleA.Constants.ConstA3);
    }

    [Fact]
    public void ModuleB_BothOpenings()
    {
        var s1 = new StructB1();
        var s2 = new StructB2();
        Assert.NotNull(s1);
        Assert.NotNull(s2);
        Assert.Equal(200, ModuleB.Constants.ConstB1);
        Assert.Equal(201, ModuleB.Constants.ConstB2);
    }

    [Fact]
    public void ReopenedModuleTypes_CanReferenceEarlier()
    {
        var s2 = new StructA2();
        Assert.IsType<StructA1>(s2.RefToA1);
    }

    [Fact]
    public void ReopenedModuleChain()
    {
        var s3 = new StructA3();
        Assert.IsType<StructA1>(s3.A1);
        Assert.IsType<StructA2>(s3.A2);
    }

    [Fact]
    public void ConstantsOnlyModule()
    {
        Assert.Equal(1, ConstantsOnly.Constants.C1);
    }

    [Fact]
    public void EnumsOnlyModule()
    {
        Assert.True(Enum.IsDefined(typeof(Color), "Red"));
    }
}
