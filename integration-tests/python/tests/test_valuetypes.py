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


def test_valuetype_instantiation(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.SimpleValue(id=1, name="test")
    assert v.id == 1
    assert v.name == "test"


def test_valuetype_defaults(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.SimpleValue()
    assert v.id == 0
    assert v.name == ""


def test_valuetype_inheritance(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.DerivedValue(id=1, name="base", description="derived")
    assert v.id == 1
    assert v.name == "base"
    assert v.description == "derived"
    assert isinstance(v, vt.SimpleValue)


def test_empty_valuetype(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.Empty()
    assert v is not None


def test_valuetype_with_sequence(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.WithSequence(numbers=[1, 2, 3], names=["a", "b"])
    assert v.numbers == [1, 2, 3]
    assert v.names == ["a", "b"]


def test_valuetype_equality(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v1 = vt.SimpleValue(id=1, name="test")
    v2 = vt.SimpleValue(id=1, name="test")
    v3 = vt.SimpleValue(id=2, name="other")
    assert v1 == v2
    assert v1 != v3


def test_valuetype_supports_interface(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.IdentifiableValue(id=42, data="test")
    assert v.id == 42
    assert v.data == "test"
    assert isinstance(v, vt.Identifiable)


def test_valuetype_supports_named(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.NamedValue(name="foo", value=100)
    assert v.name == "foo"
    assert v.value == 100
    assert isinstance(v, vt.Named)


def test_valuetype_inheritance_and_supports(
    generated_modules: dict[str, ModuleType],
) -> None:
    vt = generated_modules["valuetype_types"]
    v = vt.FullValue(id=1, name="base", extra="more")
    assert v.id == 1
    assert v.name == "base"
    assert v.extra == "more"
    assert isinstance(v, vt.SimpleValue)
    assert isinstance(v, vt.Identifiable)


def test_valuetype_field_types(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    annotations = vt.SimpleValue.__annotations__
    assert annotations["id"] == "int"
    assert annotations["name"] == "str"


def test_valuetype_sequence_field_types(
    generated_modules: dict[str, ModuleType],
) -> None:
    vt = generated_modules["valuetype_types"]
    annotations = vt.WithSequence.__annotations__
    assert annotations["numbers"] == "list[int]"
    assert annotations["names"] == "list[str]"


def test_valuetype_derived_field_types(
    generated_modules: dict[str, ModuleType],
) -> None:
    vt = generated_modules["valuetype_types"]
    annotations = vt.DerivedValue.__annotations__
    assert annotations["description"] == "str"


def test_valuetype_nested_alias_is_not_a_field(
    generated_modules: dict[str, ModuleType],
) -> None:
    vt = generated_modules["valuetype_types"]
    assert vt.WithNestedAlias.NestedType is vt.WithNestedAlias.Nested
    assert "NestedType" not in vt.WithNestedAlias.__dataclass_fields__
    assert vt.WithNestedAlias() == vt.WithNestedAlias()


def test_valuetype_self_alias(generated_modules: dict[str, ModuleType]) -> None:
    vt = generated_modules["valuetype_types"]
    assert vt.WithSelfAlias.SelfType is vt.WithSelfAlias
    assert "SelfType" not in vt.WithSelfAlias.__dataclass_fields__
    assert vt.WithSelfAlias() == vt.WithSelfAlias()
