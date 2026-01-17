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
using BitmaskTypes;

namespace IntegrationTests;

public class BitmaskTests
{
    [Fact]
    public void Permissions_IsFlagType()
    {
        var attr = typeof(Permissions).GetCustomAttributes(typeof(FlagsAttribute), false);
        Assert.NotEmpty(attr);
    }

    [Fact]
    public void Permissions_MembersExist()
    {
        Assert.True(Enum.IsDefined(typeof(Permissions), "Read"));
        Assert.True(Enum.IsDefined(typeof(Permissions), "Write"));
        Assert.True(Enum.IsDefined(typeof(Permissions), "Execute"));
        Assert.True(Enum.IsDefined(typeof(Permissions), "Delete"));
    }

    [Fact]
    public void Permissions_HasCorrectValues()
    {
        Assert.Equal(1u, (uint)Permissions.Read);
        Assert.Equal(2u, (uint)Permissions.Write);
        Assert.Equal(4u, (uint)Permissions.Execute);
        Assert.Equal(8u, (uint)Permissions.Delete);
    }

    [Fact]
    public void Permissions_CanCombine()
    {
        var readWrite = Permissions.Read | Permissions.Write;
        Assert.Equal(3u, (uint)readWrite);
    }

    [Fact]
    public void Permissions_HasFlagWorks()
    {
        var perms = Permissions.Read | Permissions.Execute;
        Assert.True(perms.HasFlag(Permissions.Read));
        Assert.True(perms.HasFlag(Permissions.Execute));
        Assert.False(perms.HasFlag(Permissions.Write));
        Assert.False(perms.HasFlag(Permissions.Delete));
    }

    [Fact]
    public void Permissions_BitwiseAnd()
    {
        var perms = Permissions.Read | Permissions.Write | Permissions.Execute;
        var masked = perms & Permissions.Write;
        Assert.Equal(Permissions.Write, masked);
    }

    [Fact]
    public void Permissions_BitwiseXor()
    {
        var perms = Permissions.Read | Permissions.Write;
        var toggled = perms ^ Permissions.Write;
        Assert.Equal(Permissions.Read, toggled);
    }

    [Fact]
    public void Permissions_AllFlags()
    {
        var all = Permissions.Read | Permissions.Write | Permissions.Execute | Permissions.Delete;
        Assert.Equal(15u, (uint)all);
    }

    [Fact]
    public void Permissions_NoneValue()
    {
        var empty = (Permissions)0;
        Assert.Equal(0u, (uint)empty);
        Assert.False(empty.HasFlag(Permissions.Read));
        Assert.False(empty.HasFlag(Permissions.Write));
    }

    [Fact]
    public void ExplicitFlags_HasCorrectValues()
    {
        Assert.Equal(2u, (uint)ExplicitFlags.FlagA);
        Assert.Equal(4u, (uint)ExplicitFlags.FlagB);
        Assert.Equal(16u, (uint)ExplicitFlags.FlagC);
        Assert.Equal(256u, (uint)ExplicitFlags.FlagD);
    }

    [Fact]
    public void FileInfo_WithPermissions()
    {
        var file = new BitmaskTypes.FileInfo("/path/to/file", Permissions.Read | Permissions.Write);
        Assert.Equal("/path/to/file", file.Path);
        Assert.True(file.Perms.HasFlag(Permissions.Read));
        Assert.True(file.Perms.HasFlag(Permissions.Write));
        Assert.False(file.Perms.HasFlag(Permissions.Execute));
    }

    [Fact]
    public void FileInfo_Equality()
    {
        var f1 = new BitmaskTypes.FileInfo("/path", Permissions.Read);
        var f2 = new BitmaskTypes.FileInfo("/path", Permissions.Read);
        var f3 = new BitmaskTypes.FileInfo("/path", Permissions.Write);
        Assert.Equal(f1, f2);
        Assert.NotEqual(f1, f3);
    }

    [Fact]
    public void GappedFlags_HasCorrectValues()
    {
        Assert.Equal(1u, (uint)GappedFlags.Low);
        Assert.Equal(128u, (uint)GappedFlags.High);
    }

    [Fact]
    public void SingleFlag_HasCorrectValue()
    {
        Assert.Equal(1u, (uint)SingleFlag.Only);
    }

    [Fact]
    public void MixedFlags_HasCorrectValues()
    {
        Assert.Equal(1u, (uint)MixedFlags.AutoFirst);
        Assert.Equal(16u, (uint)MixedFlags.ExplicitFour);
        Assert.Equal(4u, (uint)MixedFlags.AutoFive);
        Assert.Equal(8u, (uint)MixedFlags.AutoSix);
    }

    [Fact]
    public void Permissions_CanCastFromUint()
    {
        var perms = (Permissions)7u;
        Assert.True(perms.HasFlag(Permissions.Read));
        Assert.True(perms.HasFlag(Permissions.Write));
        Assert.True(perms.HasFlag(Permissions.Execute));
        Assert.False(perms.HasFlag(Permissions.Delete));
    }

    [Fact]
    public void Permissions_Complement()
    {
        var perms = Permissions.Read;
        var complement = ~perms;
        Assert.False(complement.HasFlag(Permissions.Read));
    }
}
