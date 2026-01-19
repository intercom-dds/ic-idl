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
from typing import Any


def test_any_default_is_none(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    c = at.ContainsAny()
    assert c.value is None


def test_any_accepts_int(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    c = at.ContainsAny(value=42)
    assert c.value == 42


def test_any_accepts_string(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    c = at.ContainsAny(value="hello")
    assert c.value == "hello"


def test_any_accepts_list(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    c = at.ContainsAny(value=[1, 2, 3])
    assert c.value == [1, 2, 3]


def test_any_accepts_dict(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    c = at.ContainsAny(value={"key": "value"})
    assert c.value == {"key": "value"}


def test_any_accepts_nested_struct(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    inner = at.ContainsAny(value="nested")
    outer = at.ContainsAny(value=inner)
    assert outer.value.value == "nested"


def test_multiple_any_fields(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    m = at.MultipleAny(first=1, second="two", third=[3.0])
    assert m.first == 1
    assert m.second == "two"
    assert m.third == [3.0]


def test_any_with_other_fields(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    a = at.AnyWithOtherFields(id=123, name="test", payload={"data": [1, 2, 3]})
    assert a.id == 123
    assert a.name == "test"
    assert a.payload == {"data": [1, 2, 3]}


def test_sequence_of_any(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    s = at.SequenceOfAny()
    assert s.items == []
    s.items = [1, "two", 3.0, None, {"key": "value"}]
    assert len(s.items) == 5
    assert s.items[1] == "two"
    assert s.items[4] == {"key": "value"}


def test_map_with_any(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    m = at.MapWithAny()
    assert m.properties == {}
    m.properties = {"int": 1, "str": "hello", "list": [1, 2, 3]}
    assert m.properties["int"] == 1
    assert m.properties["str"] == "hello"
    assert m.properties["list"] == [1, 2, 3]


def test_optional_any_default(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    o = at.OptionalAny()
    assert o.maybe_value is None


def test_optional_any_with_value(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    o = at.OptionalAny(maybe_value={"nested": "data"})
    assert o.maybe_value == {"nested": "data"}


def test_any_alias_typedef(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    assert hasattr(at, "AnyAlias")
    assert at.AnyAlias is Any


def test_using_any_alias(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    u = at.UsingAnyAlias(data="aliased value")
    assert u.data == "aliased value"


def test_any_can_be_reassigned(generated_modules: dict[str, ModuleType]) -> None:
    at = generated_modules["any_types"]
    c = at.ContainsAny(value=1)
    assert c.value == 1
    c.value = "now a string"
    assert c.value == "now a string"
    c.value = [1, 2, 3]
    assert c.value == [1, 2, 3]
