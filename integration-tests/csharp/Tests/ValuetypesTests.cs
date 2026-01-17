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
using ValuetypeTypes;

namespace IntegrationTests;

public class ValuetypesTests
{
    private class ConcreteSimpleValue : SimpleValue { }
    private class ConcreteDerivedValue : DerivedValue { }
    private class ConcreteEmpty : Empty { }
    private class ConcreteWithSequence : WithSequence { }
    private class ConcreteIdentifiableValue : IdentifiableValue { }
    private class ConcreteNamedValue : NamedValue { }
    private class ConcreteFullValue : FullValue { }
    private class ConcreteValueWithPrivate : ValueWithPrivate { }

    [Fact]
    public void SimpleValue_IsAbstract()
    {
        Assert.True(typeof(SimpleValue).IsAbstract);
    }

    [Fact]
    public void SimpleValue_Instantiation()
    {
        var v = new ConcreteSimpleValue();
        v.Id = 42;
        v.Name = "test";
        Assert.Equal(42, v.Id);
        Assert.Equal("test", v.Name);
    }

    [Fact]
    public void SimpleValue_Defaults()
    {
        var v = new ConcreteSimpleValue();
        Assert.Equal(0, v.Id);
        Assert.Equal("", v.Name);
    }

    [Fact]
    public void SimpleValue_HasExpectedProperties()
    {
        var idProp = typeof(SimpleValue).GetProperty("Id");
        var nameProp = typeof(SimpleValue).GetProperty("Name");

        Assert.NotNull(idProp);
        Assert.NotNull(nameProp);
        Assert.Equal(typeof(int), idProp!.PropertyType);
        Assert.Equal(typeof(string), nameProp!.PropertyType);
    }

    [Fact]
    public void DerivedValue_InheritsFromSimpleValue()
    {
        Assert.True(typeof(DerivedValue).IsSubclassOf(typeof(SimpleValue)));
    }

    [Fact]
    public void DerivedValue_AllFields()
    {
        var v = new ConcreteDerivedValue();
        v.Id = 1;
        v.Name = "base";
        v.Description = "derived";
        Assert.Equal(1, v.Id);
        Assert.Equal("base", v.Name);
        Assert.Equal("derived", v.Description);
    }

    [Fact]
    public void DerivedValue_HasOwnProperties()
    {
        var descProp = typeof(DerivedValue).GetProperty("Description");
        Assert.NotNull(descProp);
        Assert.Equal(typeof(string), descProp!.PropertyType);
    }

    [Fact]
    public void Empty_IsAbstract()
    {
        Assert.True(typeof(Empty).IsAbstract);
    }

    [Fact]
    public void Empty_CanInstantiate()
    {
        var e = new ConcreteEmpty();
        Assert.NotNull(e);
    }

    [Fact]
    public void WithSequence_HasSequenceProperties()
    {
        var numbersProp = typeof(WithSequence).GetProperty("Numbers");
        var namesProp = typeof(WithSequence).GetProperty("Names");

        Assert.NotNull(numbersProp);
        Assert.NotNull(namesProp);
        Assert.True(typeof(IList<int>).IsAssignableFrom(numbersProp!.PropertyType));
        Assert.True(typeof(IList<string>).IsAssignableFrom(namesProp!.PropertyType));
    }

    [Fact]
    public void WithSequence_DefaultsEmpty()
    {
        var w = new ConcreteWithSequence();
        Assert.Empty(w.Numbers);
        Assert.Empty(w.Names);
    }

    [Fact]
    public void WithSequence_CanAddElements()
    {
        var w = new ConcreteWithSequence();
        w.Numbers.Add(1);
        w.Numbers.Add(2);
        w.Names.Add("alice");
        Assert.Equal(2, w.Numbers.Count);
        Assert.Single(w.Names);
    }

    [Fact]
    public void IIdentifiable_InterfaceExists()
    {
        Assert.True(typeof(IIdentifiable).IsInterface);
        var idProp = typeof(IIdentifiable).GetProperty("Id");
        Assert.NotNull(idProp);
        Assert.Equal(typeof(int), idProp!.PropertyType);
    }

    [Fact]
    public void INamed_InterfaceExists()
    {
        Assert.True(typeof(INamed).IsInterface);
        var nameProp = typeof(INamed).GetProperty("Name");
        Assert.NotNull(nameProp);
        Assert.Equal(typeof(string), nameProp!.PropertyType);
    }

    [Fact]
    public void IdentifiableValue_ImplementsIIdentifiable()
    {
        Assert.True(typeof(IIdentifiable).IsAssignableFrom(typeof(IdentifiableValue)));
    }

    [Fact]
    public void IdentifiableValue_CanUseAsInterface()
    {
        var v = new ConcreteIdentifiableValue();
        v.Id = 99;
        v.Data = "test data";

        IIdentifiable ident = v;
        Assert.Equal(99, ident.Id);
    }

    [Fact]
    public void NamedValue_ImplementsINamed()
    {
        Assert.True(typeof(INamed).IsAssignableFrom(typeof(NamedValue)));
    }

    [Fact]
    public void NamedValue_CanUseAsInterface()
    {
        var v = new ConcreteNamedValue();
        v.Name = "test name";
        v.Value = 42;

        INamed named = v;
        Assert.Equal("test name", named.Name);
    }

    [Fact]
    public void FullValue_InheritsAndImplements()
    {
        Assert.True(typeof(FullValue).IsSubclassOf(typeof(SimpleValue)));
        Assert.True(typeof(IIdentifiable).IsAssignableFrom(typeof(FullValue)));
    }

    [Fact]
    public void FullValue_AllFields()
    {
        var v = new ConcreteFullValue();
        v.Id = 1;
        v.Name = "base";
        v.Extra = "extra data";

        Assert.Equal(1, v.Id);
        Assert.Equal("base", v.Name);
        Assert.Equal("extra data", v.Extra);

        IIdentifiable ident = v;
        Assert.Equal(1, ident.Id);
    }

    [Fact]
    public void ValueWithPrivate_HasProperties()
    {
        var labelProp = typeof(ValueWithPrivate).GetProperty("Label");
        var internalIdProp = typeof(ValueWithPrivate).GetProperty("InternalId");

        Assert.NotNull(labelProp);
        Assert.NotNull(internalIdProp);
        Assert.Equal(typeof(string), labelProp!.PropertyType);
        Assert.Equal(typeof(int), internalIdProp!.PropertyType);
    }

    [Fact]
    public void ValueWithPrivate_CanSetFields()
    {
        var v = new ConcreteValueWithPrivate();
        v.Label = "test label";
        v.InternalId = 123;

        Assert.Equal("test label", v.Label);
        Assert.Equal(123, v.InternalId);
    }
}
