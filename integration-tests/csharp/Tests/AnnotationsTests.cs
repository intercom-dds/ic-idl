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
using AnnotationTypes;

namespace IntegrationTests;

public class AnnotationsTests
{
    [Fact]
    public void KeyedStruct_Exists()
    {
        var k = new KeyedStruct(1, "test", 3.14);
        Assert.Equal(1, k.Id);
        Assert.Equal("test", k.Name);
        Assert.Equal(3.14, k.Value);
    }

    [Fact]
    public void KeyedStruct_HasKeyFields()
    {
        Assert.NotNull(typeof(KeyedStruct).GetProperty("Id"));
    }

    [Fact]
    public void MultiKeyStruct_Exists()
    {
        var m = new MultiKeyStruct("ns", 42, "data");
        Assert.Equal("ns", m.Namespace);
        Assert.Equal(42, m.Id);
        Assert.Equal("data", m.Data);
    }

    [Fact]
    public void MultiKeyStruct_HasMultipleKeyFields()
    {
        Assert.NotNull(typeof(MultiKeyStruct).GetProperty("Namespace"));
        Assert.NotNull(typeof(MultiKeyStruct).GetProperty("Id"));
    }

    [Fact]
    public void OptionalStruct_Defaults()
    {
        var o = new OptionalStruct();
        Assert.Equal(0, o.RequiredField);
        Assert.Equal(0, o.OptionalInt);
        Assert.Equal("", o.OptionalString);
        Assert.Empty(o.OptionalSeq);
    }

    [Fact]
    public void OptionalStruct_CanSetFields()
    {
        var o = new OptionalStruct();
        o.RequiredField = 100;
        o.OptionalInt = 42;
        o.OptionalString = "hello";
        o.OptionalSeq.Add(1);
        o.OptionalSeq.Add(2);

        Assert.Equal(100, o.RequiredField);
        Assert.Equal(42, o.OptionalInt);
        Assert.Equal("hello", o.OptionalString);
        Assert.Equal(new[] { 1, 2 }, o.OptionalSeq);
    }

    [Fact]
    public void NestedStruct_Instantiation()
    {
        var n = new NestedStruct(10, 20);
        Assert.Equal(10, n.X);
        Assert.Equal(20, n.Y);
    }

    [Fact]
    public void SharedRefs_HasSharedFields()
    {
        var nested = new NestedStruct(5, 10);
        var s = new SharedRefs("shared", nested);

        Assert.Equal("shared", s.SharedString);
        Assert.Equal(5, s.SharedStruct.X);
        Assert.Equal(10, s.SharedStruct.Y);
    }

    [Fact]
    public void CombinedAnnotations_Exists()
    {
        var c = new CombinedAnnotations(1, "shared name");
        Assert.Equal(1, c.Id);
        Assert.Equal("shared name", c.MaybeSharedName);
    }

    [Fact]
    public void AnnotatedInterface_Exists()
    {
        Assert.True(typeof(IAnnotatedInterface).IsInterface);
    }

    [Fact]
    public void AnnotatedInterface_HasMethods()
    {
        Assert.NotNull(typeof(IAnnotatedInterface).GetMethod("FireAndForget"));
        Assert.NotNull(typeof(IAnnotatedInterface).GetMethod("GetValue"));
        Assert.NotNull(typeof(IAnnotatedInterface).GetMethod("SetValue"));
    }

    [Fact]
    public void TopicMessage_Exists()
    {
        var t = new TopicMessage(1, "payload", 123456);
        Assert.Equal(1, t.MessageId);
        Assert.Equal("payload", t.Payload);
        Assert.Equal(123456, t.Timestamp);
    }

    [Fact]
    public void TopicMessage_HasFields()
    {
        Assert.NotNull(typeof(TopicMessage).GetProperty("MessageId"));
        Assert.NotNull(typeof(TopicMessage).GetProperty("Payload"));
        Assert.NotNull(typeof(TopicMessage).GetProperty("Timestamp"));
    }

    [Fact]
    public void MutableStruct_Exists()
    {
        var m = new MutableStruct(1, "data");
        Assert.Equal(1, m.Version);
        Assert.Equal("data", m.Data);
    }

    [Fact]
    public void MutableStruct_CanModify()
    {
        var m = new MutableStruct();
        m.Version = 2;
        m.Data = "updated";
        Assert.Equal(2, m.Version);
        Assert.Equal("updated", m.Data);
    }

    [Fact]
    public void FinalStruct_Exists()
    {
        var f = new FinalStruct(42);
        Assert.Equal(42, f.FixedField);
    }

    [Fact]
    public void KeyedStruct_Equality()
    {
        var k1 = new KeyedStruct(1, "test", 3.14);
        var k2 = new KeyedStruct(1, "test", 3.14);
        var k3 = new KeyedStruct(2, "test", 3.14);

        Assert.Equal(k1, k2);
        Assert.NotEqual(k1, k3);
    }

    [Fact]
    public void MultiKeyStruct_Equality()
    {
        var m1 = new MultiKeyStruct("ns", 1, "data");
        var m2 = new MultiKeyStruct("ns", 1, "data");
        var m3 = new MultiKeyStruct("other", 1, "data");

        Assert.Equal(m1, m2);
        Assert.NotEqual(m1, m3);
    }

    [Fact]
    public void TopicMessage_Equality()
    {
        var t1 = new TopicMessage(1, "msg", 100);
        var t2 = new TopicMessage(1, "msg", 100);
        var t3 = new TopicMessage(2, "msg", 100);

        Assert.Equal(t1, t2);
        Assert.NotEqual(t1, t3);
    }

    [Fact]
    public void NestedStruct_Equality()
    {
        var n1 = new NestedStruct(10, 20);
        var n2 = new NestedStruct(10, 20);
        var n3 = new NestedStruct(30, 40);

        Assert.Equal(n1, n2);
        Assert.NotEqual(n1, n3);
    }
}
