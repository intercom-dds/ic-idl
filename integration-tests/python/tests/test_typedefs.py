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


def test_primitive_typedef_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.Integer == "int"
    assert td.UnsignedInteger == "int"
    assert td.Real == "float"
    assert td.Text == "str"
    assert td.Flag == "bool"
    assert td.Byte == "int"


def test_sequence_typedef_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.IntList == "list[int]"
    assert td.StringList == "list[str]"
    assert td.RealList == "list[float]"


def test_nested_typedef_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.Count == "Integer"
    assert td.Label == "Text"


def test_map_typedef_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.StringIntMap == "dict[str, int]"
    assert td.StringStringMap == "dict[str, str]"


def test_array_typedef_value(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.LongArray == "list[int]"


def test_struct_with_typedef_fields(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    p = td.Point(x=1.5, y=2.5)
    assert p.x == 1.5
    assert p.y == 2.5


def test_struct_with_typedef_field_types(
    generated_modules: dict[str, ModuleType],
) -> None:
    td = generated_modules["typedef_types"]
    annotations = td.Point.__annotations__
    assert annotations["x"] == "Real"
    assert annotations["y"] == "Real"


def test_person_struct_field_types(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    annotations = td.Person.__annotations__
    assert annotations["name"] == "Text"
    assert annotations["age"] == "Integer"
    assert annotations["active"] == "Flag"


def test_person_struct_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    p = td.Person(name="Alice", age=30, active=True)
    assert p.name == "Alice"
    assert p.age == 30
    assert p.active is True


def test_container_struct_field_types(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    annotations = td.Container.__annotations__
    assert annotations["numbers"] == "IntList"
    assert annotations["labels"] == "StringList"
    assert annotations["lookup"] == "StringIntMap"


def test_container_struct_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    c = td.Container(
        numbers=[1, 2, 3],
        labels=["a", "b"],
        lookup={"one": 1, "two": 2},
    )
    assert c.numbers == [1, 2, 3]
    assert c.labels == ["a", "b"]
    assert c.lookup["one"] == 1


def test_nested_typedef_in_struct(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    annotations = td.Measurement.__annotations__
    assert annotations["name"] == "Label"
    assert annotations["value"] == "Count"


def test_nested_typedef_struct_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    m = td.Measurement(name="temperature", value=42)
    assert m.name == "temperature"
    assert m.value == 42


def test_array_typedef_in_struct(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    annotations = td.WithArrayTypedef.__annotations__
    assert annotations["values"] == "LongArray"


def test_array_typedef_struct_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    w = td.WithArrayTypedef(values=[1, 2, 3, 4, 5])
    assert w.values == [1, 2, 3, 4, 5]


def test_deep_typedef_chain_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.Level1 == "int"
    assert td.Level2 == "Level1"
    assert td.Level3 == "Level2"
    assert td.Level4 == "Level3"
    assert td.Level5 == "Level4"


def test_deep_sequence_typedef_chain(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.SeqLevel1 == "list[int]"
    assert td.SeqLevel2 == "SeqLevel1"
    assert td.SeqLevel3 == "SeqLevel2"


def test_deep_map_typedef_chain(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    assert td.MapLevel1 == "dict[str, int]"
    assert td.MapLevel2 == "MapLevel1"
    assert td.MapLevel3 == "MapLevel2"


def test_deep_chain_struct_field_types(
    generated_modules: dict[str, ModuleType],
) -> None:
    td = generated_modules["typedef_types"]
    annotations = td.DeepChainStruct.__annotations__
    assert annotations["deep_int"] == "Level5"
    assert annotations["deep_seq"] == "SeqLevel3"
    assert annotations["deep_map"] == "MapLevel3"


def test_deep_chain_struct_values(generated_modules: dict[str, ModuleType]) -> None:
    td = generated_modules["typedef_types"]
    s = td.DeepChainStruct(
        deep_int=42,
        deep_seq=[1, 2, 3],
        deep_map={"a": 1, "b": 2},
    )
    assert s.deep_int == 42
    assert s.deep_seq == [1, 2, 3]
    assert s.deep_map["a"] == 1
