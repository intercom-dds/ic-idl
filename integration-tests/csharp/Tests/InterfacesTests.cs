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
using InterfaceTypes;

namespace IntegrationTests;

public class InterfacesTests
{
    [Fact]
    public void Reader_IsInterface()
    {
        Assert.True(typeof(IReader).IsInterface);
    }

    [Fact]
    public void Reader_HasAbstractMethods()
    {
        var readMethod = typeof(IReader).GetMethod("Read");
        var hasMoreMethod = typeof(IReader).GetMethod("HasMore");

        Assert.NotNull(readMethod);
        Assert.NotNull(hasMoreMethod);
    }

    [Fact]
    public void Reader_ReadReturnsString()
    {
        var readMethod = typeof(IReader).GetMethod("Read");
        Assert.NotNull(readMethod);
        Assert.Equal(typeof(string), readMethod!.ReturnType);
        Assert.Empty(readMethod.GetParameters());
    }

    [Fact]
    public void Reader_HasMoreReturnsBool()
    {
        var hasMoreMethod = typeof(IReader).GetMethod("HasMore");
        Assert.NotNull(hasMoreMethod);
        Assert.Equal(typeof(bool), hasMoreMethod!.ReturnType);
    }

    [Fact]
    public void Writer_IsInterface()
    {
        Assert.True(typeof(IWriter).IsInterface);
    }

    [Fact]
    public void Writer_WriteParameterTypes()
    {
        var writeMethod = typeof(IWriter).GetMethod("Write");
        Assert.NotNull(writeMethod);
        Assert.Equal(typeof(void), writeMethod!.ReturnType);

        var parameters = writeMethod.GetParameters();
        Assert.Single(parameters);
        Assert.Equal(typeof(string), parameters[0].ParameterType);
        Assert.Equal("data", parameters[0].Name);
    }

    [Fact]
    public void Writer_FlushReturnsVoid()
    {
        var flushMethod = typeof(IWriter).GetMethod("Flush");
        Assert.NotNull(flushMethod);
        Assert.Equal(typeof(void), flushMethod!.ReturnType);
        Assert.Empty(flushMethod.GetParameters());
    }

    [Fact]
    public void ReadWriter_InheritsFromReaderAndWriter()
    {
        Assert.True(typeof(IReader).IsAssignableFrom(typeof(IReadWriter)));
        Assert.True(typeof(IWriter).IsAssignableFrom(typeof(IReadWriter)));
    }

    [Fact]
    public void ReadWriter_HasOwnMethod()
    {
        var resetMethod = typeof(IReadWriter).GetMethod("Reset");
        Assert.NotNull(resetMethod);
        Assert.Equal(typeof(void), resetMethod!.ReturnType);
    }

    [Fact]
    public void ReadWriter_HasAllInheritedMethods()
    {
        var allMethods = typeof(IReadWriter)
            .GetInterfaces()
            .SelectMany(i => i.GetMethods())
            .Concat(typeof(IReadWriter).GetMethods())
            .Select(m => m.Name)
            .ToHashSet();

        Assert.Contains("Read", allMethods);
        Assert.Contains("HasMore", allMethods);
        Assert.Contains("Write", allMethods);
        Assert.Contains("Flush", allMethods);
        Assert.Contains("Reset", allMethods);
    }

    [Fact]
    public void Calculator_AllSignatures()
    {
        var addMethod = typeof(ICalculator).GetMethod("Add");
        Assert.NotNull(addMethod);
        Assert.Equal(typeof(int), addMethod!.ReturnType);
        var addParams = addMethod.GetParameters();
        Assert.Equal(2, addParams.Length);
        Assert.Equal(typeof(int), addParams[0].ParameterType);
        Assert.Equal(typeof(int), addParams[1].ParameterType);

        var subtractMethod = typeof(ICalculator).GetMethod("Subtract");
        Assert.NotNull(subtractMethod);
        Assert.Equal(typeof(int), subtractMethod!.ReturnType);

        var divideMethod = typeof(ICalculator).GetMethod("Divide");
        Assert.NotNull(divideMethod);
        Assert.Equal(typeof(double), divideMethod!.ReturnType);
        var divideParams = divideMethod.GetParameters();
        Assert.Equal(typeof(double), divideParams[0].ParameterType);
        Assert.Equal(typeof(double), divideParams[1].ParameterType);
    }

    [Fact]
    public void Empty_IsInterface()
    {
        Assert.True(typeof(IEmpty).IsInterface);
        var methods = typeof(IEmpty).GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly);
        Assert.Empty(methods);
    }

    [Fact]
    public void WithAttribute_HasProperties()
    {
        var nameProp = typeof(IWithAttribute).GetProperty("Name");
        var countProp = typeof(IWithAttribute).GetProperty("Count");

        Assert.NotNull(nameProp);
        Assert.NotNull(countProp);
        Assert.Equal(typeof(string), nameProp!.PropertyType);
        Assert.Equal(typeof(int), countProp!.PropertyType);
    }

    [Fact]
    public void WithAttribute_NameIsReadOnly()
    {
        var nameProp = typeof(IWithAttribute).GetProperty("Name");
        Assert.NotNull(nameProp);
        Assert.NotNull(nameProp!.GetMethod);
        Assert.Null(nameProp.SetMethod);
    }

    [Fact]
    public void WithAttribute_CountIsReadWrite()
    {
        var countProp = typeof(IWithAttribute).GetProperty("Count");
        Assert.NotNull(countProp);
        Assert.NotNull(countProp!.GetMethod);
        Assert.NotNull(countProp.SetMethod);
    }

    [Fact]
    public void OperationFailed_IsException()
    {
        Assert.True(typeof(Exception).IsAssignableFrom(typeof(OperationFailed)));
    }

    [Fact]
    public void OperationFailed_HasFields()
    {
        var ex = new OperationFailed(500, "Internal error");
        Assert.Equal(500, ex.ErrorCode);
        Assert.Equal("Internal error", ex.Reason);
    }

    [Fact]
    public void OperationFailed_CanBeThrown()
    {
        Action throwAction = () => throw new OperationFailed(404, "Not found");
        var ex = Assert.Throws<OperationFailed>(throwAction);
        Assert.Equal(404, ex.ErrorCode);
        Assert.Equal("Not found", ex.Reason);
    }

    [Fact]
    public void InvalidInput_IsException()
    {
        Assert.True(typeof(Exception).IsAssignableFrom(typeof(InvalidInput)));
    }

    [Fact]
    public void InvalidInput_HasFields()
    {
        var ex = new InvalidInput("userId");
        Assert.Equal("userId", ex.ParameterName);
    }

    [Fact]
    public void WithOutParams_Exists()
    {
        Assert.True(typeof(IWithOutParams).IsInterface);
    }

    [Fact]
    public void WithOutParams_GetValuesSignature()
    {
        var method = typeof(IWithOutParams).GetMethod("GetValues");
        Assert.NotNull(method);
        var parameters = method!.GetParameters();
        Assert.Equal(2, parameters.Length);
        Assert.True(parameters[0].IsOut);
        Assert.True(parameters[1].IsOut);
    }

    [Fact]
    public void WithOutParams_SwapSignature()
    {
        var method = typeof(IWithOutParams).GetMethod("Swap");
        Assert.NotNull(method);
        var parameters = method!.GetParameters();
        Assert.Equal(2, parameters.Length);
        Assert.True(parameters[0].ParameterType.IsByRef);
        Assert.True(parameters[1].ParameterType.IsByRef);
    }

    [Fact]
    public void WithOutParams_ProcessSignature()
    {
        var method = typeof(IWithOutParams).GetMethod("Process");
        Assert.NotNull(method);
        Assert.Equal(typeof(int), method!.ReturnType);
        var parameters = method.GetParameters();
        Assert.Equal(2, parameters.Length);
        Assert.Equal(typeof(int), parameters[0].ParameterType);
        Assert.True(parameters[1].IsOut);
    }

    [Fact]
    public void WithRaises_Exists()
    {
        Assert.True(typeof(IWithRaises).IsInterface);
    }

    [Fact]
    public void WithRaises_HasMethods()
    {
        Assert.NotNull(typeof(IWithRaises).GetMethod("SafeOperation"));
        Assert.NotNull(typeof(IWithRaises).GetMethod("RiskyOperation"));
        Assert.NotNull(typeof(IWithRaises).GetMethod("ComplexOperation"));
        Assert.NotNull(typeof(IWithRaises).GetMethod("Compute"));
    }

    [Fact]
    public void CombinedFeatures_HasDoWork()
    {
        var method = typeof(ICombinedFeatures).GetMethod("DoWork");
        Assert.NotNull(method);
        var parameters = method!.GetParameters();
        Assert.Equal(2, parameters.Length);
        Assert.Equal(typeof(string), parameters[0].ParameterType);
        Assert.True(parameters[1].IsOut);
    }

    [Fact]
    public void CombinedFeatures_HasUpdate()
    {
        var method = typeof(ICombinedFeatures).GetMethod("Update");
        Assert.NotNull(method);
        var parameters = method!.GetParameters();
        Assert.Single(parameters);
        Assert.True(parameters[0].ParameterType.IsByRef);
    }
}
