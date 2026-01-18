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

from types import ModuleType

import pytest


def test_const_string_values(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    assert dt.DEFAULT_NAME == "unnamed"
    assert dt.DEFAULT_COUNT == 100
    assert abs(dt.DEFAULT_RATE - 0.5) < 0.001


def test_struct_const_initializer(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    assert dt.DEFAULT_INNER.x == 10
    assert dt.DEFAULT_INNER.y == "default"
    assert dt.NESTED_INNER.x == 99
    assert dt.NESTED_INNER.y == "nested"


def test_optional_fields_are_none(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    opt = dt.OptionalFields()
    assert opt.maybe_int is None
    assert opt.maybe_string is None
    assert opt.maybe_struct is None


def test_optional_fields_type_annotations(
    generated_modules: dict[str, ModuleType],
) -> None:
    dt = generated_modules["default_types"]
    annotations = dt.OptionalFields.__annotations__
    assert annotations["maybe_int"] == "int | None"
    assert annotations["maybe_string"] == "str | None"
    assert annotations["maybe_struct"] == "Inner | None"


def test_optional_fields_can_be_set(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    inner = dt.Inner(x=5, y="test")
    opt = dt.OptionalFields(maybe_int=42, maybe_string="hello", maybe_struct=inner)
    assert opt.maybe_int == 42
    assert opt.maybe_string == "hello"
    assert opt.maybe_struct.x == 5


def test_enum_default_literal_exists(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    assert hasattr(dt.Priority, "LOW")
    assert hasattr(dt.Priority, "MEDIUM")
    assert hasattr(dt.Priority, "HIGH")
    assert dt.Priority.MEDIUM.value == 1


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_primitive_bool_default(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    p = dt.PrimitiveDefaults()
    assert p.bool_true is True


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_primitive_int_default(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    p = dt.PrimitiveDefaults()
    assert p.int_value == 42
    assert p.int_negative == -100


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_primitive_float_default(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    p = dt.PrimitiveDefaults()
    assert abs(p.float_value - 3.14159) < 0.0001


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_primitive_string_default(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    p = dt.PrimitiveDefaults()
    assert p.string_value == "hello"
    assert p.string_from_const == "unnamed"


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_array_default_values(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    a = dt.ArrayDefaults()
    assert a.array_values == [1, 2, 3]


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_sequence_default_values(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    s = dt.SequenceDefaults()
    assert s.seq_values == [1, 2, 3, 4, 5]
    assert s.string_seq_values == ["a", "b", "c"]


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_map_default_values(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    m = dt.MapDefaults()
    assert m.map_values == {"one": 1, "two": 2}


@pytest.mark.xfail(reason="@default annotation ignored in Python codegen")
def test_enum_field_default(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    e = dt.EnumDefaults()
    assert e.priority_high == dt.Priority.HIGH
