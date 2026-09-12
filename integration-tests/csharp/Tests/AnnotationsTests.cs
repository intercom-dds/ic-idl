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

using System.Reflection;
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
        Assert.Null(o.OptionalInt);
        Assert.Null(o.OptionalString);
        Assert.Null(o.OptionalSeq);
    }

    [Fact]
    public void OptionalStruct_CanSetFields()
    {
        var o = new OptionalStruct();
        o.RequiredField = 100;
        o.OptionalInt = 42;
        o.OptionalString = "hello";
        o.OptionalSeq = new List<int> { 1, 2 };

        Assert.Equal(100, o.RequiredField);
        Assert.Equal(42, o.OptionalInt);
        Assert.Equal("hello", o.OptionalString);
        Assert.Equal(new[] { 1, 2 }, o.OptionalSeq);
    }

    [Theory]
    [InlineData(typeof(OptionalStruct), nameof(OptionalStruct.OptionalString))]
    [InlineData(typeof(OptionalStruct), nameof(OptionalStruct.OptionalSeq))]
    [InlineData(typeof(OptionalDerived), nameof(OptionalDerived.OptionalString))]
    [InlineData(typeof(OptionalDerived), nameof(OptionalDerived.OptionalSeq))]
    [InlineData(typeof(OptionalTypes), nameof(OptionalTypes.MaybeMap))]
    [InlineData(typeof(OptionalTypes), nameof(OptionalTypes.MaybeArray))]
    [InlineData(typeof(OptionalTypes), nameof(OptionalTypes.MaybeMatrix))]
    [InlineData(typeof(OptionalError), nameof(OptionalError.Reason))]
    [InlineData(typeof(OptionalValue), nameof(OptionalValue.Text))]
    [InlineData(typeof(OptionalUnion), nameof(OptionalUnion.Text))]
    public void OptionalReferences_Nullability(Type type, string name)
    {
        var context = new NullabilityInfoContext();
        var property = type.GetProperty(name)!;
        Assert.Equal(NullabilityState.Nullable, context.Create(property).ReadState);
        Assert.Equal(NullabilityState.Nullable, context.Create(property).WriteState);
    }

    [Theory]
    [InlineData(typeof(OptionalStruct), nameof(OptionalStruct.OptionalString))]
    [InlineData(typeof(OptionalStruct), nameof(OptionalStruct.OptionalSeq))]
    [InlineData(typeof(OptionalDerived), nameof(OptionalDerived.OptionalString))]
    [InlineData(typeof(OptionalDerived), nameof(OptionalDerived.OptionalSeq))]
    [InlineData(typeof(OptionalTypes), nameof(OptionalTypes.MaybeMap))]
    [InlineData(typeof(OptionalTypes), nameof(OptionalTypes.MaybeArray))]
    [InlineData(typeof(OptionalTypes), nameof(OptionalTypes.MaybeMatrix))]
    [InlineData(typeof(OptionalError), nameof(OptionalError.Reason))]
    public void OptionalReferences_ConstructorNullability(Type type, string name)
    {
        var context = new NullabilityInfoContext();
        var property = type.GetProperty(name)!;
        var parameter = Assert.Single(
            type.GetConstructors().SelectMany(c => c.GetParameters()),
            p => p.Name == name);

        Assert.Equal(property.PropertyType, parameter.ParameterType);
        Assert.Equal(NullabilityState.Nullable, context.Create(parameter).ReadState);
    }

    [Fact]
    public void OptionalStruct_ConstructorsAndClearing()
    {
        var empty = new OptionalStruct(10, null, null, null);
        Assert.Equal(new OptionalStruct { RequiredField = 10 }, empty);
        Assert.Equal(empty, new OptionalStruct(empty));

        var present = new OptionalStruct(10, 42, "hello", new List<int> { 1, 2 });
        var copy = new OptionalStruct(present);
        Assert.Equal(42, copy.OptionalInt);
        Assert.Equal("hello", copy.OptionalString);
        Assert.Equal(new[] { 1, 2 }, copy.OptionalSeq);

        present.OptionalInt = null;
        present.OptionalString = null;
        present.OptionalSeq = null;
        Assert.Equal(empty, present);
    }

    [Fact]
    public void OptionalStruct_EqualityHashingAndOrdering()
    {
        var empty = new OptionalStruct();
        var other = new OptionalStruct();
        Assert.Equal(empty, other);
        Assert.Equal(empty.GetHashCode(), other.GetHashCode());
        Assert.Equal(0, empty.CompareTo(other));

        foreach (var present in new[]
        {
            new OptionalStruct { OptionalInt = 0 },
            new OptionalStruct { OptionalString = "" },
            new OptionalStruct { OptionalSeq = new List<int>() }
        })
        {
            Assert.NotEqual(empty, present);
            Assert.True(empty.CompareTo(present) < 0);
            Assert.True(present.CompareTo(empty) > 0);

            var copy = new OptionalStruct(present);
            Assert.Equal(present, copy);
            Assert.Equal(present.GetHashCode(), copy.GetHashCode());
            Assert.Equal(0, present.CompareTo(copy));
        }
    }

    [Theory]
    [InlineData(nameof(OptionalTypes.MaybeBool), typeof(bool?))]
    [InlineData(nameof(OptionalTypes.MaybeChar), typeof(char?))]
    [InlineData(nameof(OptionalTypes.MaybeDouble), typeof(double?))]
    [InlineData(nameof(OptionalTypes.MaybeFixed), typeof(decimal?))]
    [InlineData(nameof(OptionalTypes.MaybeKind), typeof(OptionalKind?))]
    [InlineData(nameof(OptionalTypes.MaybeFlags), typeof(OptionalFlags?))]
    [InlineData(nameof(OptionalTypes.MaybeAlias), typeof(int?))]
    [InlineData(nameof(OptionalTypes.MaybeMap), typeof(IDictionary<string, int>))]
    [InlineData(nameof(OptionalTypes.MaybeArray), typeof(int[]))]
    [InlineData(nameof(OptionalTypes.MaybeMatrix), typeof(int[,]))]
    public void OptionalTypes_MappingAndDefaults(string name, Type expectedType)
    {
        var property = typeof(OptionalTypes).GetProperty(name)!;
        var parameter = typeof(OptionalTypes).GetConstructors()
            .Single(c => c.GetParameters().Length == 10)
            .GetParameters().Single(p => p.Name == name);

        Assert.Equal(expectedType, property.PropertyType);
        Assert.Equal(expectedType, parameter.ParameterType);
        Assert.Null(property.GetValue(new OptionalTypes()));
    }

    [Fact]
    public void OptionalTypes_ArrayValidationAndCopying()
    {
        var emptyCopy = new OptionalTypes(new OptionalTypes());
        Assert.Null(emptyCopy.MaybeArray);
        Assert.Null(emptyCopy.MaybeMatrix);

        var o = new OptionalTypes();
        Assert.Throws<ArgumentOutOfRangeException>(() => o.MaybeArray = new int[2]);
        Assert.Throws<ArgumentOutOfRangeException>(() => o.MaybeMatrix = new int[3, 2]);
        Assert.Throws<ArgumentOutOfRangeException>(() => o.MaybeMatrix = new int[2, 2]);
        Assert.Null(o.MaybeArray);
        Assert.Null(o.MaybeMatrix);

        o.MaybeArray = new[] { 1, 2, 3 };
        o.MaybeMatrix = new int[,] { { 1, 2, 3 }, { 4, 5, 6 } };
        var copy = new OptionalTypes(o);
        Assert.Equal(o.MaybeArray, copy.MaybeArray);
        Assert.Equal(o.MaybeMatrix, copy.MaybeMatrix);
        Assert.NotSame(o.MaybeArray, copy.MaybeArray);
        Assert.NotSame(o.MaybeMatrix, copy.MaybeMatrix);

        o.MaybeArray = null;
        o.MaybeMatrix = null;
        Assert.Null(o.MaybeArray);
        Assert.Null(o.MaybeMatrix);
    }

    [Fact]
    public void OptionalDerived_AllValuesConstructor()
    {
        var empty = new OptionalDerived(1, null, null, null, null);
        Assert.Equal(1, empty.RequiredField);
        Assert.Null(empty.OptionalInt);
        Assert.Null(empty.OptionalString);
        Assert.Null(empty.OptionalSeq);
        Assert.Null(empty.MaybeOther);

        var present = new OptionalDerived(1, 2, "hello", new List<int> { 3 }, 4);
        Assert.Equal(2, present.OptionalInt);
        Assert.Equal("hello", present.OptionalString);
        Assert.Equal(new[] { 3 }, present.OptionalSeq);
        Assert.Equal(4, present.MaybeOther);
    }

    [Fact]
    public void NonOptionalFields_Defaults()
    {
        var o = new NonOptionalFields();
        Assert.Equal(0, o.RequiredInt);
        Assert.Equal("", o.RequiredString);
        Assert.Empty(o.RequiredSeq);
        Assert.Equal(0, o.CustomInt);

        var context = new NullabilityInfoContext();
        foreach (var property in typeof(NonOptionalFields).GetProperties())
        {
            Assert.Equal(NullabilityState.NotNull, context.Create(property).ReadState);
        }
    }

    [Fact]
    public void OptionalError_Constructors()
    {
        var empty = new OptionalError();
        Assert.Null(empty.Code);
        Assert.Null(empty.Reason);

        var absent = new OptionalError(null, null);
        Assert.Null(absent.Code);
        Assert.Null(absent.Reason);

        var present = new OptionalError(42, "reason");
        Assert.Equal(42, present.Code);
        Assert.Equal("reason", present.Reason);
    }

    [Fact]
    public void OptionalValue_DefaultsAndAssignment()
    {
        var o = new ConcreteOptionalValue();
        Assert.Null(o.Number);
        Assert.Null(o.Text);

        o.Number = 42;
        o.Text = "hello";
        Assert.Equal(42, o.Number);
        Assert.Equal("hello", o.Text);

        o.Number = null;
        o.Text = null;
        Assert.Null(o.Number);
        Assert.Null(o.Text);
    }

    [Fact]
    public void OptionalUnion_ActiveMembersCanBeAbsent()
    {
        var o = new OptionalUnion { Discriminator = 0 };
        Assert.Null(o.Number);
        Assert.Throws<InvalidOperationException>(() => o.Text);

        o.SetNumber(42, 1);
        Assert.Equal(42, o.Number);
        o.SetNumber(null, 0);
        Assert.Null(o.Number);
        Assert.Equal(o, new OptionalUnion(o));

        o.Text = "hello";
        Assert.Equal("hello", o.Text);
        o.Text = null;
        Assert.Null(o.Text);
        Assert.Throws<InvalidOperationException>(() => o.Number);
        Assert.Equal(o, new OptionalUnion(o));
    }

    private sealed class ConcreteOptionalValue : OptionalValue
    {
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
