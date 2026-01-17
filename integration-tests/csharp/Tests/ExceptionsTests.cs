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
using ExceptionTypes;

namespace IntegrationTests;

public class ExceptionsTests
{
    [Fact]
    public void SimpleError_InheritsFromException()
    {
        var e = new SimpleError(404, "Not found");
        Assert.IsAssignableFrom<Exception>(e);
    }

    [Fact]
    public void SimpleError_HasFields()
    {
        var e = new SimpleError(404, "Not found");
        Assert.Equal(404, e.ErrorCode);
        Assert.Equal("Not found", e.Message);
    }

    [Fact]
    public void SimpleError_CanBeThrown()
    {
        var ex = Assert.Throws<SimpleError>((Action)(() =>
        {
            throw new SimpleError(500, "Internal error");
        }));
        Assert.Equal(500, ex.ErrorCode);
        Assert.Equal("Internal error", ex.Message);
    }

    [Fact]
    public void SimpleError_CanBeCaughtAsException()
    {
        Exception caught = Assert.Throws<SimpleError>((Action)(() =>
        {
            throw new SimpleError(400, "Bad request");
        }));
        Assert.IsAssignableFrom<Exception>(caught);
    }

    [Fact]
    public void EmptyError_CanBeInstantiated()
    {
        var e = new EmptyError();
        Assert.IsAssignableFrom<Exception>(e);
    }

    [Fact]
    public void DetailedError_HasAllFields()
    {
        var e = new DetailedError(
            Code: 123,
            Message: "Something went wrong",
            Details: "Additional context here",
            Recoverable: true
        );
        Assert.Equal(123, e.Code);
        Assert.Equal("Something went wrong", e.Message);
        Assert.Equal("Additional context here", e.Details);
        Assert.True(e.Recoverable);
    }

    [Fact]
    public void ValidationError_HasFields()
    {
        var e = new ValidationError(
            FieldName: "email",
            ErrorMessage: "Invalid email format",
            Position: 10
        );
        Assert.Equal("email", e.FieldName);
        Assert.Equal("Invalid email format", e.ErrorMessage);
        Assert.Equal(10, e.Position);
    }
}
