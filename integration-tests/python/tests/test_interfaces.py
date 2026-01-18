# Copyright 2026 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

import abc
import inspect
from types import ModuleType

import pytest


def test_interface_is_abc(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    assert issubclass(iface.Reader, abc.ABC)


def test_interface_has_abstract_methods(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    assert hasattr(iface.Reader, "read")
    assert hasattr(iface.Reader, "has_more")


def test_interface_inheritance(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    assert issubclass(iface.ReadWriter, iface.Reader)
    assert issubclass(iface.ReadWriter, iface.Writer)


def test_interface_inherited_has_own_method(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    assert hasattr(iface.ReadWriter, "reset")
    assert hasattr(iface.ReadWriter, "read")
    assert hasattr(iface.ReadWriter, "write")
    assert hasattr(iface.ReadWriter, "has_more")
    assert hasattr(iface.ReadWriter, "flush")


def test_interface_method_signature_no_params(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    sig = inspect.signature(iface.Reader.read)
    params = list(sig.parameters.keys())
    assert params == ["self"]
    assert sig.return_annotation == "str"


def test_interface_method_signature_with_params(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    sig = inspect.signature(iface.Calculator.add)
    params = list(sig.parameters.keys())
    assert "self" in params
    assert "a" in params
    assert "b" in params
    assert sig.return_annotation == "int"


def test_interface_method_return_types(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    assert inspect.signature(iface.Reader.read).return_annotation == "str"
    assert inspect.signature(iface.Reader.has_more).return_annotation == "bool"
    assert inspect.signature(iface.Calculator.divide).return_annotation == "float"


def test_interface_void_return(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    sig = inspect.signature(iface.Writer.flush)
    assert sig.return_annotation == "None"


def test_empty_interface(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    assert iface.Empty is not None
    assert issubclass(iface.Empty, abc.ABC)


def test_interface_with_attributes(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    assert hasattr(iface.WithAttribute, "name")
    assert hasattr(iface.WithAttribute, "count")


def test_interface_writer_parameter_types(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    sig = inspect.signature(iface.Writer.write)
    params = sig.parameters
    assert "data" in params
    assert params["data"].annotation == "str"


def test_interface_calculator_all_signatures(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]

    add_sig = inspect.signature(iface.Calculator.add)
    assert add_sig.parameters["a"].annotation == "int"
    assert add_sig.parameters["b"].annotation == "int"
    assert add_sig.return_annotation == "int"

    subtract_sig = inspect.signature(iface.Calculator.subtract)
    assert subtract_sig.parameters["a"].annotation == "int"
    assert subtract_sig.parameters["b"].annotation == "int"
    assert subtract_sig.return_annotation == "int"

    divide_sig = inspect.signature(iface.Calculator.divide)
    assert divide_sig.parameters["a"].annotation == "float"
    assert divide_sig.parameters["b"].annotation == "float"
    assert divide_sig.return_annotation == "float"


def test_interface_attribute_types(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    name_sig = inspect.signature(iface.WithAttribute.name.fget)
    assert name_sig.return_annotation == "str"

    count_sig = inspect.signature(iface.WithAttribute.count.fget)
    assert count_sig.return_annotation == "int"


def test_operation_failed_exception(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    assert issubclass(iface.OperationFailed, Exception)
    exc = iface.OperationFailed(error_code=42, reason="test failure")
    assert exc.error_code == 42
    assert exc.reason == "test failure"


def test_invalid_input_exception(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    assert issubclass(iface.InvalidInput, Exception)
    exc = iface.InvalidInput(parameter_name="value")
    assert exc.parameter_name == "value"


def test_exception_can_be_raised(generated_modules: dict[str, ModuleType]) -> None:
    iface = generated_modules["interface_types"]
    with pytest.raises(iface.OperationFailed) as exc_info:
        raise iface.OperationFailed(error_code=1, reason="intentional")
    assert exc_info.value.error_code == 1
    assert exc_info.value.reason == "intentional"


def test_interface_with_out_params_exists(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    assert hasattr(iface, "WithOutParams")
    assert issubclass(iface.WithOutParams, abc.ABC)
    assert hasattr(iface.WithOutParams, "get_values")
    assert hasattr(iface.WithOutParams, "swap")
    assert hasattr(iface.WithOutParams, "process")
    assert hasattr(iface.WithOutParams, "mixed_params")


def test_interface_with_raises_exists(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    assert hasattr(iface, "WithRaises")
    assert issubclass(iface.WithRaises, abc.ABC)
    assert hasattr(iface.WithRaises, "safe_operation")
    assert hasattr(iface.WithRaises, "risky_operation")
    assert hasattr(iface.WithRaises, "complex_operation")
    assert hasattr(iface.WithRaises, "compute")


def test_combined_features_interface(
    generated_modules: dict[str, ModuleType],
) -> None:
    iface = generated_modules["interface_types"]
    assert hasattr(iface, "CombinedFeatures")
    assert issubclass(iface.CombinedFeatures, abc.ABC)
    assert hasattr(iface.CombinedFeatures, "do_work")
    assert hasattr(iface.CombinedFeatures, "update")
