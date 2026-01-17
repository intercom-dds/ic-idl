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
using NestedModuleTypes;
using NestedModuleTypes.Level1;
using NestedModuleTypes.Level1.Level2;
using NestedModuleTypes.Level1.Level2.Level3;
using NestedModuleTypes.Sibling;

namespace IntegrationTests;

public class NestedModulesTests
{
    [Fact]
    public void TopLevelTypes_Exist()
    {
        var s = new TopLevelStruct(42);
        Assert.Equal(42, s.Value);
        Assert.True(Enum.IsDefined(typeof(TopLevelEnum), "First"));
        Assert.True(Enum.IsDefined(typeof(TopLevelEnum), "Second"));
    }

    [Fact]
    public void Level1Module_Exists()
    {
        var s = new Level1Struct();
        Assert.NotNull(s);
        Assert.True(Enum.IsDefined(typeof(Level1Enum), "A"));
        Assert.True(Enum.IsDefined(typeof(Level1Enum), "B"));
        Assert.True(Enum.IsDefined(typeof(Level1Enum), "C"));
    }

    [Fact]
    public void Level2Module_Exists()
    {
        var s = new Level2Struct();
        Assert.NotNull(s);
    }

    [Fact]
    public void Level3Module_Exists()
    {
        var s = new Level3Struct();
        Assert.NotNull(s);
        Assert.Equal(42, Constants.DeepConst);
    }

    [Fact]
    public void SiblingModule_Exists()
    {
        var s = new SiblingStruct(100);
        Assert.Equal(100, s.Id);
        var c = new CrossRef();
        Assert.NotNull(c);
    }

    [Fact]
    public void TopLevelStruct_Instantiation()
    {
        var s = new TopLevelStruct(123);
        Assert.Equal(123, s.Value);
    }

    [Fact]
    public void Level1Struct_WithParentRef()
    {
        var top = new TopLevelStruct(10);
        var l1 = new Level1Struct(20, top);
        Assert.Equal(20, l1.Data);
        Assert.Equal(10, l1.ParentRef.Value);
    }

    [Fact]
    public void Level2Struct_WithRefs()
    {
        var top = new TopLevelStruct(1);
        var l1 = new Level1Struct(2, top);
        var l2 = new Level2Struct("test", l1, top);

        Assert.Equal("test", l2.Name);
        Assert.Equal(2, l2.Level1Ref.Data);
        Assert.Equal(1, l2.TopRef.Value);
    }

    [Fact]
    public void Level3Struct_WithAllRefs()
    {
        var top = new TopLevelStruct(1);
        var l1 = new Level1Struct(2, top);
        var l2 = new Level2Struct("l2", l1, top);
        var l3 = new Level3Struct(3, l2, l1, top);

        Assert.Equal(3, l3.Id);
        Assert.Equal("l2", l3.Level2Ref.Name);
        Assert.Equal(2, l3.Level1Ref.Data);
        Assert.Equal(1, l3.TopRef.Value);
    }

    [Fact]
    public void DeepConstant()
    {
        Assert.Equal(42, NestedModuleTypes.Level1.Level2.Level3.Constants.DeepConst);
    }

    [Fact]
    public void CrossRef_ReferencesAllLevels()
    {
        var l1 = new Level1Struct();
        var l2 = new Level2Struct();
        var l3 = new Level3Struct();

        var crossRef = new CrossRef(l1, l2, l3);
        Assert.NotNull(crossRef.FromLevel1);
        Assert.NotNull(crossRef.FromLevel2);
        Assert.NotNull(crossRef.FromLevel3);
    }

    [Fact]
    public void TopUsingNested_ReferencesAllNestedTypes()
    {
        var t = new TopUsingNested();
        Assert.NotNull(t.L1);
        Assert.NotNull(t.L2);
        Assert.NotNull(t.L3);
        Assert.NotNull(t.Sib);
    }

    [Fact]
    public void Level1Enum_Values()
    {
        Assert.Equal(0, (int)Level1Enum.A);
        Assert.Equal(1, (int)Level1Enum.B);
        Assert.Equal(2, (int)Level1Enum.C);
    }

    [Fact]
    public void TopLevelEnum_Values()
    {
        Assert.Equal(0, (int)TopLevelEnum.First);
        Assert.Equal(1, (int)TopLevelEnum.Second);
    }

    [Fact]
    public void NestedNamespaces_FullyQualified()
    {
        Assert.Equal(typeof(NestedModuleTypes.Level1.Level1Struct), typeof(Level1Struct));
        Assert.Equal(typeof(NestedModuleTypes.Level1.Level2.Level2Struct), typeof(Level2Struct));
        Assert.Equal(typeof(NestedModuleTypes.Level1.Level2.Level3.Level3Struct), typeof(Level3Struct));
        Assert.Equal(typeof(NestedModuleTypes.Sibling.SiblingStruct), typeof(SiblingStruct));
    }
}
